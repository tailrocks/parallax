+++
schema_version = 1
package = "parallax-api"
class = "product"
tier = 3
dependencies = ["parallax-analysis", "parallax-evidence", "parallax-metadata", "parallax-storage", "parallax-test-support"]
facade_roots = ["lib.rs"]
+++

# parallax-api

Owns the GraphQL schema, resolvers, batching boundaries, and API error
projection over domain and storage contracts.

## Owned concerns

GraphQL schema composition, request batching, and resolver orchestration.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/resolvers.rs](src/resolvers.rs)
- [src/resolvers/issues.rs](src/resolvers/issues.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo nextest run -p parallax-api --all-features --locked` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
