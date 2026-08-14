use super::*;

#[test]
fn prune_estimate_is_bounded_and_counts_only_owned_files() {
    let tmp = TempDir::new("prune-estimate");
    std::fs::write(tmp.path().join("traces.pspl"), b"active").expect("write active");
    std::fs::write(tmp.path().join("logs.ndjson"), b"legacy").expect("write legacy");
    std::fs::write(tmp.path().join("metrics.123.pspl"), b"rotated").expect("write rotated");
    std::fs::write(tmp.path().join("unrelated.txt"), b"ignored").expect("write unrelated");
    let spool = Spool::open(tmp.path()).expect("open spool");

    assert_eq!(
        spool.prune_estimate(4).expect("estimate prune"),
        SpoolPruneEstimate {
            active_files: 2,
            rotated_segments: 1,
            bytes: 19,
        }
    );
    spool
        .prune_estimate(3)
        .expect_err("entry cap must fail closed");
}

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
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            tracing::warn!(path = %self.0.display(), %error, "test directory cleanup failed");
        }
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
            UNIX_EPOCH + Duration::from_mins(5),
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

#[tokio::test]
async fn health_counts_only_the_selected_signal() -> Result<(), std::io::Error> {
    let tmp = TempDir::new("health");
    let spool =
        Spool::open_with_max_segment_bytes(tmp.path(), 40).map_err(std::io::Error::other)?;
    spool
        .append_raw(Signal::Logs, &bytes::Bytes::from_static(&[1; 32]))
        .await
        .map_err(std::io::Error::other)?;
    spool
        .append_raw(Signal::Traces, &bytes::Bytes::from_static(&[2; 8]))
        .await
        .map_err(std::io::Error::other)?;
    let now = UNIX_EPOCH + Duration::from_secs(4_000_000_000);
    let logs = spool.health(Signal::Logs, now)?;
    let traces = spool.health(Signal::Traces, now)?;
    if logs.bytes <= traces.bytes || traces.bytes == 0 {
        return Err(std::io::Error::other(format!(
            "spool health mismatch: logs={logs:?}, traces={traces:?}"
        )));
    }
    Ok(())
}

#[test]
fn corrupt_magic_is_an_error_not_empty() {
    let tmp = TempDir::new("bad-magic");
    let path = tmp.path().join("logs.pspl");
    std::fs::write(&path, b"XXXXX").expect("write");
    let err = count_pspl_frames(&path).expect_err("corrupt magic");
    assert!(format!("{err:#}").contains("corrupt pspl magic"), "{err:#}");
}

#[test]
fn truncated_final_frame_is_not_counted() {
    let tmp = TempDir::new("trunc");
    let path = tmp.path().join("logs.pspl");
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&20u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2, 3]);
    std::fs::write(&path, bytes).expect("write");
    assert_eq!(count_pspl_frames(&path).expect("trunc"), 0);
}

#[test]
fn zero_length_frame_counts_as_one() {
    let tmp = TempDir::new("zero");
    let path = tmp.path().join("logs.pspl");
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&0u32.to_le_bytes());
    std::fs::write(&path, bytes).expect("write");
    assert_eq!(count_pspl_frames(&path).expect("zero"), 1);
}

#[tokio::test]
async fn reopen_then_append_does_not_write_second_magic() {
    let tmp = TempDir::new("reopen");
    {
        let spool = Spool::open(tmp.path()).expect("spool");
        spool
            .append_raw(Signal::Logs, &bytes::Bytes::from(vec![1u8; 4]))
            .await
            .expect("first");
    }
    {
        let spool = Spool::open(tmp.path()).expect("reopen");
        spool
            .append_raw(Signal::Logs, &bytes::Bytes::from(vec![2u8; 4]))
            .await
            .expect("second");
    }
    let bytes = std::fs::read(tmp.path().join("logs.pspl")).expect("read");
    assert_eq!(&bytes[..5], MAGIC);
    assert_eq!(
        bytes[5..]
            .windows(5)
            .filter(|window| *window == MAGIC)
            .count(),
        0
    );
    assert_eq!(
        count_pspl_frames(&tmp.path().join("logs.pspl")).expect("count"),
        2
    );
}

#[test]
fn oversized_payload_errors_instead_of_clamping() {
    frame_len(u32::MAX as usize).expect("u32::MAX fits");
    let err = frame_len(usize::try_from(u32::MAX).expect("u32") + 1).expect_err("oversize");
    assert!(format!("{err:#}").contains("u32::MAX"), "{err:#}");
}

mod framing_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn append_then_count_round_trips(frames in proptest::collection::vec(proptest::collection::vec(0u8..16, 0..24), 0..8)) {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            runtime.block_on(async {
                let tmp = TempDir::new("proptest-count");
                let spool = Spool::open(tmp.path()).expect("spool");
                for frame in &frames {
                    spool
                        .append_raw(Signal::Logs, &bytes::Bytes::from(frame.clone()))
                        .await
                        .expect("append");
                }
                let counted = spool.line_count(Signal::Logs).expect("count");
                prop_assert_eq!(counted, frames.len());
                Ok(())
            })?;
        }
    }
}
