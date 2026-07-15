+++
schema_version = 1
package = "parallax-server"
class = "product"
tier = 4
dependencies = ["parallax-analysis", "parallax-api", "parallax-greptime", "parallax-ingest", "parallax-metadata", "parallax-proto", "parallax-semconv", "parallax-spool", "parallax-storage", "parallax-test-support"]
facade_roots = ["lib.rs"]
+++

# parallax-server

Composes mandatory GreptimeDB and Turso engines, OTLP receivers, workers,
GraphQL hosting, engine supervision, and optional self-telemetry export.

## Owned concerns

Runtime composition of engines, ingest workers, receivers, API, UI, and lifecycle.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/serve.rs](src/serve.rs)
- [src/worker.rs](src/worker.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-server --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
