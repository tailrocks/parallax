use super::*;

#[test]
fn captures_sorted_cfg_and_nested_reexports() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("facade.rs");
    fs::write(
        &path,
        "pub use crate::{b::{C, D}, a};\n#[cfg(feature = \"x\")] pub mod gated;\nmod private;",
    )
    .expect("fixture write");
    let entries = parse_root(&path).expect("fixture parse");
    assert_eq!(entries.len(), 2);
    assert!(entries[0].contains("cfg"));
    assert!(
        entries[1].contains("b::{C") && entries[1].contains("D}"),
        "{entries:?}"
    );
}

#[test]
fn malformed_root_fails_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("facade-malformed.rs");
    fs::write(&path, "pub mod {").expect("fixture write");
    parse_root(&path).unwrap_err();
}
