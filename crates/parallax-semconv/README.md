+++
schema_version = 1
package = "parallax-semconv"
class = "product"
tier = 0
dependencies = []
facade_roots = ["lib.rs"]
+++

# parallax-semconv

Owns the checked-in Rust constants generated from Parallax's versioned
semantic-convention registry. Product builds consume this dependency-free leaf
crate and never invoke Weaver or the repository generator.

## Owned concerns

Generated semantic-convention attribute, event, metric, and fixed-value names
shared by Parallax producers and consumers.

## Source map

- [src/lib.rs](src/lib.rs)
- [Registry contract](../../telemetry/semconv/contract.yaml)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml). The generated constants preserve exact
wire spellings and must not be edited by hand.

## Verification

Run `cargo nextest run -p parallax-semconv --locked` for the narrow crate gate,
`cargo xtask semconv check` for registry/output drift, and
`cargo xtask facade check` for root-surface drift.
