use super::*;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "parallax-doctor-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp doctor dir");
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

#[test]
fn spool_stats_include_rotated_segments() {
    let tmp = TempDir::new("stats");
    let mut pspl = b"PSPL1".to_vec();
    pspl.extend_from_slice(&0u32.to_le_bytes());
    pspl.extend_from_slice(&0u32.to_le_bytes());
    std::fs::write(tmp.path().join("logs.pspl"), &pspl).expect("active");
    std::fs::write(tmp.path().join("logs.100.pspl"), b"old").expect("old");
    std::fs::write(tmp.path().join("logs.200-1.ndjson"), b"newer").expect("newer");
    std::fs::write(tmp.path().join("traces.100.pspl"), b"trace").expect("trace");

    let stats = spool_stats(tmp.path(), "logs", "logs.pspl");

    assert_eq!(
        stats,
        SignalSpoolStats {
            active_lines: 2,
            active_bytes: pspl.len() as u64,
            rotated_segments: 2,
            rotated_bytes: 8,
        }
    );
}

#[test]
fn prune_dir_truncates_active_and_removes_rotated_segments() {
    let tmp = TempDir::new("prune");
    std::fs::write(tmp.path().join("logs.pspl"), b"active").expect("active");
    std::fs::write(tmp.path().join("logs.100.pspl"), b"old").expect("old");

    let reclaimed = prune_dir(tmp.path()).expect("prune");

    assert!(reclaimed > 0);
    assert_eq!(
        std::fs::read_to_string(tmp.path().join("logs.pspl")).expect("active remains"),
        ""
    );
    assert!(!tmp.path().join("logs.100.pspl").exists());
}
