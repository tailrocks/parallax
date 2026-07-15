+++
schema_version = 1
package = "parallax-greptime"
class = "product"
tier = 2
dependencies = ["parallax-model", "parallax-proto", "parallax-semconv", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-greptime

Owns GreptimeDB HTTP/Arrow transport, native OTLP table SQL, migrations, row
mapping, and the concrete implementation of telemetry storage capabilities.

## Owned concerns

Concrete GreptimeDB telemetry capability implementation over native signal tables.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/greptime.rs](src/greptime.rs)
- [src/greptime/transport.rs](src/greptime/transport.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-greptime --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
