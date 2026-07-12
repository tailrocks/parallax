use super::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn missing_and_malformed_ratchets_fail_closed() {
    let path = std::env::temp_dir().join(format!(
        "ratchet-{}.toml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    assert!(Ratchet::load(&path).is_err());
    fs::write(&path, "schema_version = 'wrong'").expect("fixture write");
    assert!(Ratchet::load(&path).is_err());
    fs::remove_file(path).expect("fixture remove");
}
