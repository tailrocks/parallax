//! Compile-time regression tests for the documented public API boundary.

#[test]
fn resolver_implementation_stays_private() -> Result<(), &'static str> {
    // `cargo xtask facade check` independently derives the public facade from
    // syntax. This source invariant proves the implementation module itself
    // remains private without launching a nested Cargo compiler graph.
    let root = include_str!("../src/lib.rs");
    let resolver_is_private = root.contains("\nmod resolvers;\n")
        && !root.contains("\npub mod resolvers;\n")
        && !root.contains("\npub(crate) mod resolvers;\n");
    if resolver_is_private {
        Ok(())
    } else {
        Err("resolver implementation module became visible")
    }
}
