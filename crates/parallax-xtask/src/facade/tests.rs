use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn captures_sorted_cfg_and_nested_reexports() {
    let path = std::env::temp_dir().join(format!(
        "facade-{}.rs",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
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
    fs::remove_file(path).expect("fixture remove");
}

#[test]
fn malformed_root_fails_closed() {
    let path = std::env::temp_dir().join("facade-malformed.rs");
    fs::write(&path, "pub mod {").expect("fixture write");
    assert!(parse_root(&path).is_err());
    fs::remove_file(path).expect("fixture remove");
}
