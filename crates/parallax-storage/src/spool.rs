//! The ingest spool: an NDJSON landing zone for raw OTLP export requests.
//!
//! M0's durability story: every accepted OTLP request is appended to a
//! per-signal NDJSON file before the ingest endpoint acknowledges it. M1's
//! workers consume from here into the storage engine; the spool then becomes
//! the bounded WAL described in the implementation spec.

use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Traces,
    Logs,
    Metrics,
}

impl Signal {
    fn file_name(self) -> &'static str {
        match self {
            Signal::Traces => "traces.ndjson",
            Signal::Logs => "logs.ndjson",
            Signal::Metrics => "metrics.ndjson",
        }
    }
}

/// Append-only NDJSON spool, one file per signal.
pub struct Spool {
    dir: PathBuf,
    write_lock: Mutex<()>,
}

impl Spool {
    pub fn open(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            write_lock: Mutex::new(()),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Append one export request as a single NDJSON line.
    pub async fn append<T: Serialize>(&self, signal: Signal, request: &T) -> anyhow::Result<()> {
        let line = serde_json::to_string(request)?;
        let path = self.dir.join(signal.file_name());
        let _guard = self.write_lock.lock().await;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Count spooled lines for a signal (used by tests and `doctor`).
    pub fn line_count(&self, signal: Signal) -> anyhow::Result<usize> {
        let path = self.dir.join(signal.file_name());
        if !path.exists() {
            return Ok(0);
        }
        Ok(std::fs::read_to_string(path)?.lines().count())
    }
}
