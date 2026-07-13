#![expect(
    clippy::excessive_nesting,
    clippy::too_many_arguments,
    reason = "measured legacy blocking spool boundary"
)]

//! The ingest spool: a length-prefixed raw-protobuf landing zone for accepted
//! OTLP export requests.
//!
//! Format (active segments, `.pspl`):
//!   magic "PSPL1" once per file, then per record: u32-LE length + raw bytes.
//!
//! Legacy NDJSON segments (`.ndjson`) remain readable for `doctor/line_count`
//! until the reaper ages them out (default 72h). New appends always write
//! `.pspl`. The spool is a diagnostic record and crash-forensics trail, NOT a
//! write-ahead log — see plan 073.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

mod append;
mod framing;
mod retention;

#[cfg(test)]
use framing::count_pspl_frames;
use framing::rotated_timestamp;

const DEFAULT_MAX_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;
const MAGIC: &[u8; 5] = b"PSPL1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Traces,
    Logs,
    Metrics,
}

impl Signal {
    const ALL: [Signal; 3] = [Signal::Traces, Signal::Logs, Signal::Metrics];

    fn index(self) -> usize {
        match self {
            Signal::Traces => 0,
            Signal::Logs => 1,
            Signal::Metrics => 2,
        }
    }

    fn file_name(self) -> &'static str {
        match self {
            Signal::Traces => "traces.pspl",
            Signal::Logs => "logs.pspl",
            Signal::Metrics => "metrics.pspl",
        }
    }

    fn legacy_file_name(self) -> &'static str {
        match self {
            Signal::Traces => "traces.ndjson",
            Signal::Logs => "logs.ndjson",
            Signal::Metrics => "metrics.ndjson",
        }
    }

    fn stem(self) -> &'static str {
        match self {
            Signal::Traces => "traces",
            Signal::Logs => "logs",
            Signal::Metrics => "metrics",
        }
    }
}

#[derive(Debug)]
struct SignalState {
    size: u64,
    file: Option<std::fs::File>,
}

#[derive(Debug, Clone, Copy)]
pub struct SpoolRetention {
    pub max_total_bytes: u64,
    pub max_age: Duration,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpoolReclaim {
    pub removed_segments: usize,
    pub reclaimed_bytes: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpoolHealth {
    pub bytes: u64,
    pub oldest_age: Duration,
}

#[derive(Debug, Clone)]
struct RotatedSegment {
    path: PathBuf,
    size: u64,
    timestamp_secs: Option<u64>,
}

impl Spool {
    pub fn health(&self, signal: Signal, now: SystemTime) -> std::io::Result<SpoolHealth> {
        let mut health = SpoolHealth::default();
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !path.is_file()
                || !(name == signal.file_name()
                    || name == signal.legacy_file_name()
                    || name.starts_with(&format!("{}.", signal.stem())))
            {
                continue;
            }
            let metadata = entry.metadata()?;
            health.bytes = health.bytes.saturating_add(metadata.len());
            let age = metadata
                .modified()
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .unwrap_or_default();
            health.oldest_age = health.oldest_age.max(age);
        }
        Ok(health)
    }
}

#[derive(Debug)]
pub struct Spool {
    dir: PathBuf,
    max_segment_bytes: u64,
    states: [Mutex<SignalState>; 3],
}

impl Spool {
    pub fn open(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::open_with_max_segment_bytes(dir, DEFAULT_MAX_SEGMENT_BYTES)
    }

    pub fn open_with_max_segment_bytes(
        dir: impl AsRef<Path>,
        max_segment_bytes: u64,
    ) -> anyhow::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        let mut states = [
            Mutex::new(SignalState {
                size: 0,
                file: None,
            }),
            Mutex::new(SignalState {
                size: 0,
                file: None,
            }),
            Mutex::new(SignalState {
                size: 0,
                file: None,
            }),
        ];
        for signal in Signal::ALL {
            let size = dir
                .join(signal.file_name())
                .metadata()
                .map_or(0, |metadata| metadata.len());
            *states[signal.index()].get_mut() = SignalState { size, file: None };
        }
        Ok(Self {
            dir,
            max_segment_bytes: max_segment_bytes.max(1),
            states,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests;
