//! Compile-time regression tests for the documented public API boundary.

use parallax_server::{Config, ServerHandle, start};

fn accepts(_: Option<ServerHandle>) {}

#[test]
fn documented_public_api_boundary() {
    // Ordinary integration-test compilation proves the supported imports
    // without launching a second Cargo graph at test runtime.
    let _ = (Config::default(), start);
    accepts(None);

    // The syntax-derived facade manifest is checked independently by
    // `cargo xtask facade check`. Fixed assertions here keep private module
    // paths private even if somebody deliberately refreshes a widened
    // manifest after making them public.
    let facade = include_str!("../facade.toml");
    assert!(!facade.contains("mod worker"));
    assert!(!facade.contains("mod self_telemetry"));
    assert!(facade.contains("Installed as InstalledSelfTelemetry"));
}
