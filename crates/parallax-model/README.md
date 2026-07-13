+++
schema_version = 1
package = "parallax-model"
class = "product"
tier = 0
dependencies = []
facade_roots = ["lib.rs"]
+++

# parallax-model

Owns normalized telemetry rows, query-neutral records, and stable value types.
It contains no protocol, database, transport, or runtime dependency.

## Owned concerns

Query-neutral telemetry records and shared value types.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/types.rs](src/types.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-model --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
