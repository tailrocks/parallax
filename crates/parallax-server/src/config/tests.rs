use super::Config;

#[test]
fn rejects_removed_none_storage_mode() {
    let config: Config = toml::from_str("[storage]\nmode = 'none'\n").expect("parse");
    let error = config.validate().expect_err("none must be rejected");
    assert_eq!(
        error.to_string(),
        "unsupported storage.mode \"none\"; supported values are \"managed\" and \"external\""
    );
}

#[test]
fn external_storage_requires_url() {
    let mut config = Config::default();
    config.storage.mode = "external".to_string();
    let error = config.validate().expect_err("URL required");
    assert_eq!(
        error.to_string(),
        "storage.mode=external requires greptime_url"
    );
}
