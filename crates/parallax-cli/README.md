+++
schema_version = 1
package = "parallax-cli"
class = "product"
tier = 5
dependencies = ["parallax-model", "parallax-server"]
facade_roots = ["main.rs"]
+++

# parallax-cli

Owns command parsing, client output, configuration, and the installed
`parallax` binary composition edge.

## Owned concerns

Installed command model, API client behavior, output rendering, and server composition.

## Source map

- [src/main.rs](src/main.rs)
- [src/commands.rs](src/commands.rs)
- [src/runtime.rs](src/runtime.rs)
- [Reviewed facade manifest](facade.toml)

## Public surface

The supported `main.rs` paths are the exports recorded in the
[reviewed facade manifest](facade.toml); implementation modules are not a
compatibility surface.

## Verification

Run `cargo test -p parallax-cli --all-features` for the narrow crate gate and `cargo xtask facade check` for
root-surface drift.
