use super::{ConfigError, ConfigErrorKind, ServerError, ServerErrorKind};
use std::error::Error as _;

#[test]
fn classifications_and_sources_are_stable() {
    let config = ConfigError::Read {
        path: "missing.toml".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
    };
    assert_eq!(config.kind(), ConfigErrorKind::Read);
    assert_eq!(config.source().expect("read source").to_string(), "gone");

    let server = ServerError::Configuration(config);
    assert_eq!(server.kind(), ServerErrorKind::Configuration);
    assert!(server.source().is_some());
}
