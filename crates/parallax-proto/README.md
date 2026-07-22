+++
schema_version = 1
package = "parallax-proto"
class = "product"
tier = 0
dependencies = ["parallax-semconv"]
facade_roots = ["lib.rs"]
+++

# parallax-proto

Owns OTLP wire and service types. It has no internal workspace dependency and
is the lowest current product tier.

## Owned concerns

OTLP protocol aliases, services, and semantic-convention constants.

## Source map

- [src/lib.rs](src/lib.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo nextest run -p parallax-proto --all-features --locked` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
