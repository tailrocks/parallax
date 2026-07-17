+++
schema_version = 1
package = "parallax-metadata"
class = "product"
tier = 2
dependencies = ["parallax-model", "parallax-redaction", "parallax-semconv", "parallax-storage"]
facade_roots = ["lib.rs"]
+++

# parallax-metadata

Owns Turso connection management, schema migrations, transactions, and row
mapping for mutable Parallax product metadata.

## Owned concerns

Concrete Turso metadata persistence, migrations, and row mapping.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/turso.rs](src/turso.rs)
- [src/turso/connection.rs](src/turso/connection.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-metadata --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
