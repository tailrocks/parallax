use super::*;
use std::fs;

#[test]
fn missing_and_malformed_ratchets_fail_closed() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("ratchet.toml");
    assert!(Ratchet::load(&path).is_err());
    fs::write(&path, "schema_version = 'wrong'").expect("fixture write");
    assert!(Ratchet::load(&path).is_err());
}
