use super::{Config, is_loopback_bind, resolve_api_token_from};

#[test]
fn unknown_top_level_key_is_rejected_not_silently_ignored() {
    // Regression: `bind`/`otlp_grpc_port` written at the top level (instead of
    // under [server]) used to parse as unknown keys and fall back to defaults
    // silently, leaving the server on loopback:4317 while the operator believed
    // the config applied.
    let error = toml::from_str::<Config>("bind = '0.0.0.0'\notlp_grpc_port = 14317\n")
        .expect_err("stray top-level keys must fail the load");
    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn unknown_section_key_is_rejected() {
    let error = toml::from_str::<Config>("[server]\napi_prot = 4000\n")
        .expect_err("typo'd section key must fail the load");
    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn public_url_prefers_explicit_then_derives_from_bind() {
    let mut config = Config::default();
    assert_eq!(config.resolved_public_url(), "http://127.0.0.1:4000");

    config.server.bind = "0.0.0.0".to_string();
    config.server.api_port = 4043;
    assert_eq!(
        config.resolved_public_url(),
        "http://127.0.0.1:4043",
        "wildcard bind must not leak 0.0.0.0 into outbound links"
    );

    config.server.bind = "192.168.1.10".to_string();
    assert_eq!(config.resolved_public_url(), "http://192.168.1.10:4043");

    config.server.public_url = "https://parallax.example.com/".to_string();
    assert_eq!(
        config.resolved_public_url(),
        "https://parallax.example.com",
        "explicit public_url wins and loses its trailing slash"
    );
}

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
fn login_defaults_off_and_requires_token_plus_username() {
    let mut config = Config::default();
    assert!(!config.resolved_login_enabled());
    config.server.login_enabled = true;
    let error = config.validate().expect_err("login needs token");
    assert!(
        error
            .to_string()
            .contains("login_enabled requires an API token")
    );
    config.server.api_token = "a".repeat(16);
    let error = config.validate().expect_err("login needs username");
    assert!(error.to_string().contains("login_username"));
    config.server.login_username = "operator@example.com".to_string();
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

#[test]
fn plan_115_example_config_loads_and_validates() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/research/validation/2026-07-plan-115-v2-server-profile/example-config.toml"
    );
    let config = Config::load(Some(std::path::Path::new(path))).expect("example config");
    assert_eq!(config.storage.mode, "managed");
    assert!(is_loopback_bind(&config.server.bind));
    assert!(!config.sentry.enabled);
    assert!(!config.github_deploy.enabled);
    assert!(!config.github_actions.enabled);
    assert_eq!(config.server.api_port, 4000);
    assert_eq!(config.server.otlp_grpc_port, 4317);
}
