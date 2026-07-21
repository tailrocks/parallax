//! Compile-time regression tests for the documented public API boundary.

#[test]
fn documented_public_api_boundary() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/public_lifecycle.rs");
    cases.compile_fail("tests/ui/private_worker.rs");
    cases.compile_fail("tests/ui/private_self_telemetry.rs");
}
