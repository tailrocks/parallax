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

## High-signal activation

The Rust 2024, future-incompatible, idiom, style, unused, rustdoc, unsafe,
Clippy `all`, async correctness, panic, must-use, memory-safety, and layout
rules are active under CI `-D warnings`. The activation pass handled every
previously silent shutdown, cleanup, TTL-reconcile, worker-join, and broadcast
outcome according to its runtime contract. Exact test-only valves and opaque
runtime-type exceptions are reason-bearing and ratcheted by crate and lint.

Validation after activation: 230 nextest tests, the compile-fail doctest,
workspace Clippy with `-D warnings`, and the repository policy all passed.
