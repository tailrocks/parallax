//! The ingest spool: a length-prefixed raw-protobuf landing zone for accepted
//! OTLP export requests.
//!
//! Format (active segments, `.pspl`):
//!   magic "PSPL1" once per file, then per record: u32-LE length + raw bytes.
//!
//! Legacy NDJSON segments (`.ndjson`) remain readable for doctor/line_count
//! until the reaper ages them out (default 72h). New appends always write
//! `.pspl`. The spool is a diagnostic record and crash-forensics trail, NOT a
//! write-ahead log — see plan 073.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub const DEFAULT_MAX_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;
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

    pub async fn append_raw(&self, signal: Signal, raw: &bytes::Bytes) -> anyhow::Result<()> {
        let payload = raw.to_vec();
        let write_len = u64::try_from(payload.len().saturating_add(4)).unwrap_or(u64::MAX);
        let dir = self.dir.clone();
        let max_segment_bytes = self.max_segment_bytes;

        let mut state = self.states[signal.index()].lock().await;
        let size = state.size;
        let file = state.file.take();

        let (next_file, next_size) = tokio::task::spawn_blocking(move || {
            append_blocking(
                dir,
                signal,
                max_segment_bytes,
                size,
                write_len,
                file,
                payload,
            )
        })
        .await
        .map_err(|e| anyhow::anyhow!("spool append join: {e}"))??;

        state.file = next_file;
        state.size = next_size;
        Ok(())
    }

    pub fn line_count(&self, signal: Signal) -> anyhow::Result<usize> {
        let mut total = 0usize;
        let pspl = self.dir.join(signal.file_name());
        if pspl.exists() {
            total = total.saturating_add(count_pspl_frames(&pspl)?);
        }
        let ndjson = self.dir.join(signal.legacy_file_name());
        if ndjson.exists() {
            total = total.saturating_add(std::fs::read_to_string(ndjson)?.lines().count());
        }
        Ok(total)
    }

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
            for name in [signal.file_name(), signal.legacy_file_name()] {
                let path = self.dir.join(name);
                if let Ok(metadata) = path.metadata() {
                    total = total.saturating_add(metadata.len());
                }
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

fn append_blocking(
    dir: PathBuf,
    signal: Signal,
    max_segment_bytes: u64,
    mut size: u64,
    write_len: u64,
    mut file: Option<std::fs::File>,
    payload: Vec<u8>,
) -> anyhow::Result<(Option<std::fs::File>, u64)> {
    let needs_magic = size == 0;
    let total_write = if needs_magic {
        write_len.saturating_add(MAGIC.len() as u64)
    } else {
        write_len
    };

    if size > 0 && size.saturating_add(total_write) > max_segment_bytes {
        drop(file.take());
        rotate_active(&dir, signal)?;
        size = 0;
    }

    let needs_magic = size == 0;
    let total_write = if needs_magic {
        write_len.saturating_add(MAGIC.len() as u64)
    } else {
        write_len
    };

    let mut handle = match file {
        Some(handle) => handle,
        None => std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(signal.file_name()))?,
    };
    if needs_magic {
        handle.write_all(MAGIC)?;
    }
    let len = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    handle.write_all(&len.to_le_bytes())?;
    handle.write_all(&payload)?;
    Ok((Some(handle), size.saturating_add(total_write)))
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
    let first = dir.join(format!("{}.{}.pspl", signal.stem(), timestamp));
    if !first.exists() {
        return first;
    }
    for sequence in 1u64.. {
        let candidate = dir.join(format!("{}.{}-{}.pspl", signal.stem(), timestamp, sequence));
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

fn count_pspl_frames(path: &Path) -> anyhow::Result<usize> {
    let mut file = std::fs::File::open(path)?;
    let mut magic = [0u8; 5];
    let n = file.read(&mut magic)?;
    if n == 0 {
        return Ok(0);
    }
    if n < 5 || &magic != MAGIC {
        return Ok(0);
    }
    let mut count = 0usize;
    loop {
        let mut len_buf = [0u8; 4];
        match file.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut skip = vec![0u8; len];
        file.read_exact(&mut skip)?;
        count += 1;
    }
    Ok(count)
}

fn rotated_timestamp(file_name: &str) -> Option<Option<u64>> {
    for signal in Signal::ALL {
        let prefix = format!("{}.", signal.stem());
        for suffix in [".pspl", ".ndjson"] {
            let active = if suffix == ".pspl" {
                signal.file_name()
            } else {
                signal.legacy_file_name()
            };
            if file_name == active
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
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
                            && name != signal.legacy_file_name()
                            && name.starts_with(&prefix)
                            && (name.ends_with(".pspl") || name.ends_with(".ndjson"))
                    })
            })
            .collect();
        files.sort();
        files
    }

    #[tokio::test]
    async fn rotates_without_splitting_frames() {
        let tmp = TempDir::new("rotate");
        let spool = Spool::open_with_max_segment_bytes(tmp.path(), 40).expect("spool");
        let a = bytes::Bytes::from(vec![1u8; 20]);
        let b = bytes::Bytes::from(vec![2u8; 20]);
        spool.append_raw(Signal::Logs, &a).await.expect("first");
        spool.append_raw(Signal::Logs, &b).await.expect("second");
        let rotated = rotated_files(tmp.path(), Signal::Logs);
        assert_eq!(rotated.len(), 1);
        assert_eq!(count_pspl_frames(&rotated[0]).expect("rot count"), 1);
        assert_eq!(
            count_pspl_frames(&tmp.path().join("logs.pspl")).expect("active"),
            1
        );
    }

    #[tokio::test]
    async fn different_signals_keep_independent_sizes() {
        let tmp = TempDir::new("per-signal");
        let spool = Spool::open_with_max_segment_bytes(tmp.path(), 40).expect("spool");
        let payload = bytes::Bytes::from(vec![9u8; 20]);
        spool
            .append_raw(Signal::Logs, &payload)
            .await
            .expect("logs first");
        spool
            .append_raw(Signal::Traces, &payload)
            .await
            .expect("traces first");
        spool
            .append_raw(Signal::Logs, &payload)
            .await
            .expect("logs second");
        assert_eq!(rotated_files(tmp.path(), Signal::Logs).len(), 1);
        assert!(rotated_files(tmp.path(), Signal::Traces).is_empty());
        assert_eq!(spool.line_count(Signal::Logs).expect("logs count"), 1);
        assert_eq!(spool.line_count(Signal::Traces).expect("traces count"), 1);
        spool
            .append_raw(Signal::Traces, &bytes::Bytes::from(vec![1u8; 4]))
            .await
            .expect("traces second");
        assert!(rotated_files(tmp.path(), Signal::Traces).is_empty());
        assert_eq!(spool.line_count(Signal::Traces).expect("traces count"), 2);
    }

    #[tokio::test]
    async fn mixed_legacy_ndjson_and_pspl_counted() {
        let tmp = TempDir::new("mixed");
        std::fs::write(tmp.path().join("logs.ndjson"), b"{\"a\":1}\n{\"b\":2}\n").expect("legacy");
        let spool = Spool::open(tmp.path()).expect("spool");
        spool
            .append_raw(Signal::Logs, &bytes::Bytes::from(vec![7u8; 8]))
            .await
            .expect("pspl frame");
        assert_eq!(spool.line_count(Signal::Logs).expect("count"), 3);
    }

    #[test]
    fn reaper_removes_old_rotated_segments_but_keeps_active() {
        let tmp = TempDir::new("age");
        std::fs::write(tmp.path().join("logs.pspl"), b"active").expect("active");
        std::fs::write(tmp.path().join("logs.100.pspl"), b"old").expect("old");
        std::fs::write(tmp.path().join("logs.9900.pspl"), b"fresh").expect("fresh");
        std::fs::write(tmp.path().join("logs.50.ndjson"), b"legacy-old").expect("legacy");
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
        assert!(reclaimed.removed_segments >= 2);
        assert!(tmp.path().join("logs.pspl").exists());
        assert!(!tmp.path().join("logs.100.pspl").exists());
        assert!(tmp.path().join("logs.9900.pspl").exists());
        assert!(!tmp.path().join("logs.50.ndjson").exists());
    }

    #[test]
    fn reaper_removes_oldest_rotated_segments_to_enforce_total_cap() {
        let tmp = TempDir::new("size");
        std::fs::write(tmp.path().join("logs.pspl"), b"active").expect("active");
        std::fs::write(tmp.path().join("logs.100.pspl"), b"11111").expect("oldest");
        std::fs::write(tmp.path().join("logs.200.pspl"), b"22222").expect("newest");
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
        assert!(tmp.path().join("logs.pspl").exists());
        assert!(!tmp.path().join("logs.100.pspl").exists());
        assert!(tmp.path().join("logs.200.pspl").exists());
    }

    #[test]
    fn no_serde_json_to_string_in_spool_source() {
        let src = include_str!("spool.rs");
        // Split the needle so this assertion does not match itself.
        let needle = format!("{}::{}", "serde_json", "to_string");
        assert!(!src.contains(&needle));
    }
}
