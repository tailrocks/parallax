+++
schema_version = 1
package = "parallax-xtask"
class = "aux"
dependencies = []
facade_roots = ["lib.rs", "main.rs"]
+++

# parallax-xtask

Repository-only quality control plane for local and CI orchestration,
architecture/product policy, structural ratchets, and facade/doc validation.

## Owned concerns

Repository policy, architecture, facade, dependency, internal Markdown link
integrity, health, and CI orchestration.

## Source map

- [src/lib.rs](src/lib.rs)
- [src/main.rs](src/main.rs)
- [src/policy.rs](src/policy.rs)
- [src/facade.rs](src/facade.rs)
- [src/docs_links.rs](src/docs_links.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `lib.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-xtask --all-features` for the narrow crate gate,
`cargo xtask facade check` for root-surface drift, and
`cargo xtask docs links` for tracked internal Markdown link integrity.
