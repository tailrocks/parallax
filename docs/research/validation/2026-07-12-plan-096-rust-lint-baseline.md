# Plan 096 Rust lint baseline

Date: 2026-07-12

## Toolchain and inheritance

The repository pins Rust 1.97.0, released 2026-07-09, with rustfmt, Clippy,
and all four release targets. Cargo metadata reports Rust 1.97.0 and edition
2024 for every workspace member; mise and `rust-toolchain.toml` agree.

Before inheritance, only `parallax-mcp-spike` and `parallax-xtask` opted into
the root lint table. Enabling `[lints] workspace = true` on the other six crates
without changing any lint category produced exactly one diagnostic class:

| Diagnostic | Count | Initial owner |
|---|---:|---|
| `clippy::unwrap_used` | 129 | Test-only code; explicit test valves own the accepted uses |

The measurement came from structured Cargo JSON for
`cargo clippy --workspace --all-targets --locked --message-format=json`; it is
not a stderr grep. Strict-family activation is measured separately after this
inheritance-only checkpoint.
