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
    let src = include_str!("../spool.rs");
    // Split the needle so this assertion does not match itself.
    let needle = format!("{}::{}", "serde_json", "to_string");
    assert!(!src.contains(&needle));
}
