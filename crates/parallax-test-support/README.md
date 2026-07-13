+++
schema_version = 1
package = "parallax-test-support"
class = "test-support"
dependencies = ["parallax-model", "parallax-proto", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-test-support

Owns reusable in-memory telemetry fakes, typed fixture builders, and shared
storage conformance scenarios. Product crates may consume it only as a dev
dependency; it is unreachable from release roots.

## Owned concerns

Cycle-safe fakes, builders, and storage conformance scenarios.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/memory.rs](src/memory.rs)
- [src/conformance.rs](src/conformance.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-test-support --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
