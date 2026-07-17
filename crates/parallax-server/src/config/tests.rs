use super::{Config, is_loopback_bind, resolve_api_token_from};

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

#[test]
fn loopback_without_token_is_valid() {
    let config = Config::default();
    config.validate().unwrap();
    assert_eq!(config.auth_status_label(), "off");
    assert!(is_loopback_bind("127.0.0.1"));
    assert!(is_loopback_bind("::1"));
}

#[test]
fn non_loopback_requires_api_token() {
    let mut config = Config::default();
    config.server.bind = "0.0.0.0".to_string();
    let error = config.validate().expect_err("token required");
    assert!(
        error.to_string().contains("non-loopback"),
        "unexpected error: {error}"
    );
}

#[test]
fn api_token_length_bounds() {
    let mut config = Config::default();
    config.server.api_token = "short".to_string();
    assert!(config.validate().is_err());
    config.server.api_token = "a".repeat(16);
    config.validate().unwrap();
}

#[test]
fn env_off_disables_config_token() {
    assert_eq!(
        resolve_api_token_from(Some("off".to_string()), "configured-token-value"),
        None
    );
    assert_eq!(
        resolve_api_token_from(None, "configured-token-value").as_deref(),
        Some("configured-token-value")
    );
}
