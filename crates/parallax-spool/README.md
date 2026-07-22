+++
schema_version = 1
package = "parallax-spool"
class = "product"
tier = 2
dependencies = []
facade_roots = ["lib.rs"]
+++

# parallax-spool

Owns raw OTLP frame append, framing, rotation, retention, and crash recovery.
It is an ingest durability boundary, never a fallback database.

## Owned concerns

Crash-safe raw-frame durability, rotation, recovery, and retention.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/spool.rs](src/spool.rs)
- [src/spool/framing.rs](src/spool/framing.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo nextest run -p parallax-spool --all-features --locked` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
