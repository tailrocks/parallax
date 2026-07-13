+++
schema_version = 1
package = "parallax-ingest"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto"]
facade_roots = ["lib.rs"]
+++

# parallax-ingest

Owns the zero-copy OTLP-to-domain normalization boundary. It accepts decoded
wire ownership and emits `parallax-model` values without storage, API, or
evidence dependencies.

## Owned concerns

Signal-specific OTLP normalization into owned domain rows.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/traces.rs](src/traces.rs)
- [src/metrics.rs](src/metrics.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-ingest --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
