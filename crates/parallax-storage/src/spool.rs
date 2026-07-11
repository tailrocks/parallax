//! The ingest spool: an NDJSON landing zone for raw OTLP export requests.
//!
//! Every accepted OTLP request is appended here before the ingest endpoint
//! acknowledges it. Nothing reads the spool back today: it is a diagnostic
//! record and crash-forensics trail, reaped by size/age (`reap`), NOT a
//! write-ahead log. If the worker drops an item after retries (see
//! `parallax-server::worker`), the data survives only here. Replay/WAL
//! semantics are a deferred design — do not claim durability beyond this.

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

    fn index(self) -> usize {
        match self {
            Signal::Traces => 0,
            Signal::Logs => 1,
            Signal::Metrics => 2,
        }
    }

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

/// Per-signal append state: size accounting plus a cached open handle so
/// consecutive appends skip the open syscall. Handles are closed before
/// rotation so a rename cannot leave writes on the rotated inode.
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

#[derive(Debug, Clone)]
struct RotatedSegment {
    path: PathBuf,
    size: u64,
    timestamp_secs: Option<u64>,
}

/// Bounded NDJSON spool, one active file per signal plus rotated segments.
///
/// Appends for different signals do not share a lock; each signal serializes
/// its own rotate/write path so concurrent traces/logs/metrics exporters do
/// not wait on each other's disk IO.
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
                .map(|metadata| metadata.len())
                .unwrap_or(0);
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

    /// Append one export request as a single NDJSON line.
    ///
    /// JSON serialization happens before the per-signal lock. Rotate-check and
    /// the write itself run on a blocking pool thread so Tokio workers are not
    /// stalled on disk syscalls; the async lock is held across that await so
    /// line atomicity and size accounting stay exact per signal.
    pub async fn append<T: Serialize>(&self, signal: Signal, request: &T) -> anyhow::Result<()> {
        let line = serde_json::to_string(request)?;
        let write_len = u64::try_from(line.len().saturating_add(1)).unwrap_or(u64::MAX);
        let dir = self.dir.clone();
        let max_segment_bytes = self.max_segment_bytes;

        let mut state = self.states[signal.index()].lock().await;
        let size = state.size;
        let file = state.file.take();

        let (next_file, next_size) = tokio::task::spawn_blocking(move || {
            append_blocking(dir, signal, max_segment_bytes, size, write_len, file, line)
        })
        .await
        .map_err(|e| anyhow::anyhow!("spool append join: {e}"))??;

        state.file = next_file;
        state.size = next_size;
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

/// Blocking rotate-check + write for one append. Closes any cached handle
/// before renaming the active segment so writes cannot land on the rotated
/// inode.
fn append_blocking(
    dir: PathBuf,
    signal: Signal,
    max_segment_bytes: u64,
    mut size: u64,
    write_len: u64,
    mut file: Option<std::fs::File>,
    line: String,
) -> anyhow::Result<(Option<std::fs::File>, u64)> {
    if size > 0 && size.saturating_add(write_len) > max_segment_bytes {
        // Close before rename: an open fd would keep writing the rotated file.
        drop(file.take());
        rotate_active(&dir, signal)?;
        size = 0;
    }

    let mut handle = match file {
        Some(handle) => handle,
        None => std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(signal.file_name()))?,
    };
    handle.write_all(line.as_bytes())?;
    handle.write_all(b"\n")?;
    Ok((Some(handle), size.saturating_add(write_len)))
}

fn rotate_active(dir: &Path, signal: Signal) -> anyhow::Result<()> {
    let active = dir.join(signal.file_name());
    if !active.exists() {
        return Ok(());
    }
    let rotated = next_rotated_path(dir, signal);
    std::fs::rename(active, rotated)?;
    Ok(())
}

fn next_rotated_path(dir: &Path, signal: Signal) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let first = dir.join(format!("{}.{}.ndjson", signal.stem(), timestamp));
    if !first.exists() {
        return first;
    }
    for sequence in 1u64.. {
        let candidate = dir.join(format!(
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

    #[tokio::test]
    async fn different_signals_keep_independent_sizes() {
        let tmp = TempDir::new("per-signal");
        // Tiny segment budget so logs rotate while traces stay under the cap.
        let spool = Spool::open_with_max_segment_bytes(tmp.path(), 40).expect("spool");

        spool
            .append(Signal::Logs, &json!({"body": "first log"}))
            .await
            .expect("logs first");
        spool
            .append(Signal::Traces, &json!({"body": "first trace"}))
            .await
            .expect("traces first");
        spool
            .append(
                Signal::Logs,
                &json!({"body": "second log that forces rotate"}),
            )
            .await
            .expect("logs second");

        // Logs rotated once; traces still a single active line.
        assert_eq!(rotated_files(tmp.path(), Signal::Logs).len(), 1);
        assert!(rotated_files(tmp.path(), Signal::Traces).is_empty());
        assert_eq!(spool.line_count(Signal::Logs).expect("logs count"), 1);
        assert_eq!(spool.line_count(Signal::Traces).expect("traces count"), 1);

        // In-memory size for traces still accounts for the one line (not zeroed
        // by the logs rotation). A third tiny traces append must not rotate.
        spool
            .append(Signal::Traces, &json!({"b": 1}))
            .await
            .expect("traces second");
        assert!(rotated_files(tmp.path(), Signal::Traces).is_empty());
        assert_eq!(spool.line_count(Signal::Traces).expect("traces count"), 2);
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
