use super::*;
fn config_with(endpoint: &str) -> Config {
    let mut config = Config::default();
    config.telemetry.self_otlp_endpoint = endpoint.to_string();
    config
}

#[test]
fn endpoint_off_and_empty_disable() {
    assert_eq!(
        resolve_endpoint_from(&config_with(""), Err(std::env::VarError::NotPresent)),
        None
    );
    assert_eq!(
        resolve_endpoint_from(&config_with("off"), Err(std::env::VarError::NotPresent)),
        None
    );
    assert_eq!(
        resolve_endpoint_from(
            &config_with("http://localhost:4317"),
            Err(std::env::VarError::NotPresent),
        )
        .as_deref(),
        Some("http://localhost:4317"),
    );
}

#[test]
fn env_overrides_config_including_off() {
    assert_eq!(
        resolve_endpoint_from(&config_with("http://localhost:4317"), Ok("off".into())),
        None
    );
    assert_eq!(
        resolve_endpoint_from(&config_with(""), Ok("http://rotel:4317".into())).as_deref(),
        Some("http://rotel:4317"),
    );
}

// The ingest-path suppression (the self → sink → self loop guard) is
// verified live against a running serve in the validation note — exporting
// to Parallax's own receiver and asserting only non-ingest spans return.
