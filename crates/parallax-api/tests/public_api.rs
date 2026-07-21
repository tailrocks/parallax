//! Compile-time regression tests for the documented public API boundary.

#[test]
fn resolver_implementation_stays_private() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/private_resolvers.rs");
}
