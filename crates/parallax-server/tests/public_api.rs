//! Compile-time regression tests for the documented public API boundary.

use parallax_server::{Config, ServerHandle, start};

fn accepts(_: Option<ServerHandle>) {}

#[test]
fn documented_public_api_boundary() -> Result<(), &'static str> {
    // Ordinary integration-test compilation proves the supported imports
    // without launching a second Cargo graph at test runtime.
    let _boundary = (Config::default(), start);
    accepts(None);

    // The syntax-derived facade manifest is checked independently by
    // `cargo xtask facade check`. Fixed assertions here keep private module
    // paths private even if somebody deliberately refreshes a widened
    // manifest after making them public.
    let facade = include_str!("../facade.toml");
    let boundary_is_exact = !facade.contains("mod worker")
        && !facade.contains("mod self_telemetry")
        && facade.contains("Installed as InstalledSelfTelemetry");
    if boundary_is_exact {
        Ok(())
    } else {
        Err("documented facade exposed a private module or lost its public alias")
    }
}
