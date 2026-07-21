+++
schema_version = 1
package = "parallax-storage"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto", "parallax-semconv"]
facade_roots = ["lib.rs"]
+++

# parallax-storage

Owns query-neutral telemetry and metadata capability contracts plus their pure
shared selection and aggregation rules. Concrete engines live in adapter crates.

## Owned concerns

Engine-neutral telemetry and metadata capability contracts.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/adapter.rs](src/adapter.rs)
- [src/metadata.rs](src/metadata.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo nextest run -p parallax-storage --all-features --locked` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
