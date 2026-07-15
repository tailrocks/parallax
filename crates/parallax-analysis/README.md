+++
schema_version = 1
package = "parallax-analysis"
class = "product"
tier = 1
dependencies = ["parallax-model", "parallax-proto", "parallax-semconv"]
facade_roots = ["lib.rs"]
+++

# parallax-analysis

Owns pure error derivation, deterministic fingerprints, span-event parsing,
trace comparison, and critical-path analysis. It has no ingest, storage,
transport, API, or runtime dependency.

## Owned concerns

Pure telemetry interpretation and deterministic derived analysis.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/derive.rs](src/derive.rs)
- [src/trace_analysis.rs](src/trace_analysis.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-analysis --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
