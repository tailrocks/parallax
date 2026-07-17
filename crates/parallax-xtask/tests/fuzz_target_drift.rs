//! Plan-103 drift gate: every declared fuzz target has a source file and
//! every fuzz source file is declared — silent drift fails in both
//! directions.

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use std::collections::BTreeSet;
use std::path::PathBuf;

fn fuzz_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzz")
}

#[test]
fn fuzz_targets_and_manifest_do_not_drift() {
    let manifest = std::fs::read_to_string(fuzz_dir().join("Cargo.toml")).expect("fuzz manifest");
    let declared: BTreeSet<String> = manifest
        .lines()
        .filter_map(|line| line.trim().strip_prefix("path = \"fuzz_targets/"))
        .filter_map(|rest| rest.strip_suffix(".rs\""))
        .map(str::to_string)
        .collect();
    let on_disk: BTreeSet<String> = std::fs::read_dir(fuzz_dir().join("fuzz_targets"))
        .expect("fuzz_targets dir")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()?
                .strip_suffix(".rs")
                .map(str::to_string)
        })
        .collect();
    assert!(!declared.is_empty(), "fuzz targets declared");
    assert_eq!(
        declared, on_disk,
        "fuzz/Cargo.toml [[bin]] entries and fuzz/fuzz_targets/*.rs must match"
    );
}
