//! The ingest spool: an NDJSON landing zone for raw OTLP export requests.
//!
//! M0's durability story: every accepted OTLP request is appended to a
//! per-signal NDJSON file before the ingest endpoint acknowledges it. M1's
//! workers consume from here into the storage engine; the spool then becomes
//! the bounded WAL described in the implementation spec.

use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub const DEFAULT_MAX_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Traces,
    Logs,
    Metrics,
}

impl Signal {
    const ALL: [Signal; 3] = [Signal::Traces, Signal::Logs, Signal::Metrics];

    fn file_name(self) -> &'static str {
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

#[derive(Debug, Default)]
struct SegmentSizes {
    traces: u64,
    logs: u64,
    metrics: u64,
}

impl SegmentSizes {
    fn get(&self, signal: Signal) -> u64 {
        match signal {
            Signal::Traces => self.traces,
            Signal::Logs => self.logs,
            Signal::Metrics => self.metrics,
        }
    }

    fn set(&mut self, signal: Signal, size: u64) {
        match signal {
            Signal::Traces => self.traces = size,
            Signal::Logs => self.logs = size,
            Signal::Metrics => self.metrics = size,
        }
    }
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

#[derive(Debug, Clone)]
struct RotatedSegment {
    path: PathBuf,
    size: u64,
    timestamp_secs: Option<u64>,
}

/// Bounded NDJSON spool, one active file per signal plus rotated segments.
pub struct Spool {
    dir: PathBuf,
    max_segment_bytes: u64,
    sizes: Mutex<SegmentSizes>,
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
        let mut sizes = SegmentSizes::default();
        for signal in Signal::ALL {
            let size = dir
                .join(signal.file_name())
                .metadata()
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            sizes.set(signal, size);
        }
        Ok(Self {
            dir,
            max_segment_bytes: max_segment_bytes.max(1),
            sizes: Mutex::new(sizes),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Append one export request as a single NDJSON line.
    pub async fn append<T: Serialize>(&self, signal: Signal, request: &T) -> anyhow::Result<()> {
        let line = serde_json::to_string(request)?;
        let write_len = u64::try_from(line.len().saturating_add(1)).unwrap_or(u64::MAX);
        let path = self.dir.join(signal.file_name());
        let mut sizes = self.sizes.lock().await;
        if sizes.get(signal) > 0
            && sizes.get(signal).saturating_add(write_len) > self.max_segment_bytes
        {
            self.rotate_active(signal)?;
            sizes.set(signal, 0);
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        let next_size = sizes.get(signal).saturating_add(write_len);
        sizes.set(signal, next_size);
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

    /// Delete rotated segments that exceed retention. Active files are never removed.
    pub fn reap(&self, retention: SpoolRetention, now: SystemTime) -> anyhow::Result<SpoolReclaim> {
        let mut reclaim = SpoolReclaim::default();
        let now_secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let max_age_secs = retention.max_age.as_secs();
        let mut rotated = self.rotated_segments()?;
        let mut kept = Vec::new();

        for segment in rotated.drain(..) {
            let expired = segment
                .timestamp_secs
                .is_some_and(|timestamp| now_secs.saturating_sub(timestamp) > max_age_secs);
            if expired {
                reclaim.add_removed(&segment)?;
            } else {
                kept.push(segment);
            }
        }

        let rotated_total = kept.iter().map(|segment| segment.size).sum::<u64>();
        let mut total = self.active_total_bytes()?.saturating_add(rotated_total);
        if total > retention.max_total_bytes {
            kept.sort_by_key(|segment| segment.timestamp_secs.unwrap_or(u64::MAX));
            for segment in kept {
                if total <= retention.max_total_bytes {
                    break;
                }
                reclaim.add_removed(&segment)?;
                total = total.saturating_sub(segment.size);
            }
        }

        Ok(reclaim)
    }

    fn active_total_bytes(&self) -> anyhow::Result<u64> {
        let mut total = 0u64;
        for signal in Signal::ALL {
            let path = self.dir.join(signal.file_name());
            if let Ok(metadata) = path.metadata() {
                total = total.saturating_add(metadata.len());
            }
        }
        Ok(total)
    }

    fn rotate_active(&self, signal: Signal) -> anyhow::Result<()> {
        let active = self.dir.join(signal.file_name());
        if !active.exists() {
            return Ok(());
        }
        let rotated = self.next_rotated_path(signal);
        std::fs::rename(active, rotated)?;
        Ok(())
    }

    fn next_rotated_path(&self, signal: Signal) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let first = self
            .dir
            .join(format!("{}.{}.ndjson", signal.stem(), timestamp));
        if !first.exists() {
            return first;
        }
        for sequence in 1u64.. {
            let candidate = self.dir.join(format!(
                "{}.{}-{}.ndjson",
                signal.stem(),
                timestamp,
                sequence
            ));
            if !candidate.exists() {
                return candidate;
            }
        }
        unreachable!("unbounded sequence finds a rotated spool path")
    }

    fn rotated_segments(&self) -> anyhow::Result<Vec<RotatedSegment>> {
        let mut segments = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if let Some(timestamp_secs) = rotated_timestamp(file_name) {
                segments.push(RotatedSegment {
                    size: entry.metadata()?.len(),
                    path,
                    timestamp_secs,
                });
            }
        }
        Ok(segments)
    }
}

impl SpoolReclaim {
    fn add_removed(&mut self, segment: &RotatedSegment) -> anyhow::Result<()> {
        std::fs::remove_file(&segment.path)?;
        self.removed_segments += 1;
        self.reclaimed_bytes = self.reclaimed_bytes.saturating_add(segment.size);
        Ok(())
    }
}

fn rotated_timestamp(file_name: &str) -> Option<Option<u64>> {
    for signal in Signal::ALL {
        let prefix = format!("{}.", signal.stem());
        let suffix = ".ndjson";
        if file_name == signal.file_name()
            || !file_name.starts_with(&prefix)
            || !file_name.ends_with(suffix)
        {
            continue;
        }
        let middle = &file_name[prefix.len()..file_name.len() - suffix.len()];
        let timestamp = middle
            .split('-')
            .next()
            .and_then(|part| part.parse::<u64>().ok());
        return Some(timestamp);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "parallax-spool-{name}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp spool dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn rotated_files(dir: &Path, signal: Signal) -> Vec<PathBuf> {
        let prefix = format!("{}.", signal.stem());
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("read temp spool")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name != signal.file_name()
                            && name.starts_with(&prefix)
                            && name.ends_with(".ndjson")
                    })
            })
            .collect();
        files.sort();
        files
    }

    #[tokio::test]
    async fn rotates_without_splitting_ndjson_lines() {
        let tmp = TempDir::new("rotate");
        let spool = Spool::open_with_max_segment_bytes(tmp.path(), 40).expect("spool");

        spool
            .append(Signal::Logs, &json!({"body": "first request"}))
            .await
            .expect("first append");
        spool
            .append(Signal::Logs, &json!({"body": "second request"}))
            .await
            .expect("second append");

        let rotated = rotated_files(tmp.path(), Signal::Logs);
        assert_eq!(rotated.len(), 1);
        let rotated_lines = std::fs::read_to_string(&rotated[0]).expect("rotated");
        let active_lines = std::fs::read_to_string(tmp.path().join("logs.ndjson")).expect("active");
        assert_eq!(rotated_lines.lines().count(), 1);
        assert_eq!(active_lines.lines().count(), 1);
        for line in rotated_lines.lines().chain(active_lines.lines()) {
            serde_json::from_str::<serde_json::Value>(line).expect("valid ndjson line");
        }
    }

    #[test]
    fn reaper_removes_old_rotated_segments_but_keeps_active() {
        let tmp = TempDir::new("age");
        std::fs::write(tmp.path().join("logs.ndjson"), b"active\n").expect("active");
        std::fs::write(tmp.path().join("logs.100.ndjson"), b"old\n").expect("old");
        std::fs::write(tmp.path().join("logs.9900.ndjson"), b"fresh\n").expect("fresh");
        let spool = Spool::open(tmp.path()).expect("spool");

        let reclaimed = spool
            .reap(
                SpoolRetention {
                    max_total_bytes: u64::MAX,
                    max_age: Duration::from_secs(1_000),
                },
                UNIX_EPOCH + Duration::from_secs(10_000),
            )
            .expect("reap");

        assert_eq!(reclaimed.removed_segments, 1);
        assert!(tmp.path().join("logs.ndjson").exists());
        assert!(!tmp.path().join("logs.100.ndjson").exists());
        assert!(tmp.path().join("logs.9900.ndjson").exists());
    }

    #[test]
    fn reaper_removes_oldest_rotated_segments_to_enforce_total_cap() {
        let tmp = TempDir::new("size");
        std::fs::write(tmp.path().join("logs.ndjson"), b"active").expect("active");
        std::fs::write(tmp.path().join("logs.100.ndjson"), b"11111").expect("oldest");
        std::fs::write(tmp.path().join("logs.200.ndjson"), b"22222").expect("newest");
        let spool = Spool::open(tmp.path()).expect("spool");

        let reclaimed = spool
            .reap(
                SpoolRetention {
                    max_total_bytes: 11,
                    max_age: Duration::from_secs(u64::MAX),
                },
                UNIX_EPOCH + Duration::from_secs(300),
            )
            .expect("reap");

        assert_eq!(reclaimed.removed_segments, 1);
        assert!(tmp.path().join("logs.ndjson").exists());
        assert!(!tmp.path().join("logs.100.ndjson").exists());
        assert!(tmp.path().join("logs.200.ndjson").exists());
    }
}
