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

## Pedantic and measured thresholds

Clippy `pedantic` is active at workspace scope. The explicit non-target
allowlist is limited to opinionated API/documentation/style rules that do not
represent correctness, ownership, or bounded-complexity requirements; public
error documentation remains owned by Plan 099. Numeric conversion findings
were either replaced with checked or lossless conversions or retained as
reasoned expectations at the narrow semantic boundary.

`clippy.toml` now enforces the repository ceilings directly: 100 function
lines, cognitive complexity 25, nesting 4, and 6 arguments. Existing findings
are represented by exact per-crate/per-lint suppression ceilings, while the
structural gate continues to reject file/function growth. The oversized CLI
command implementation moved behind a small lint-owning facade so activating
the policy did not raise its existing line or complexity ceilings.

Validation after this stage passed:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-targets` (230 passed, 6 ignored real-engine
  tests); and
- `cargo xtask policy --output json` with `[]`.

## Async blocking ownership

The Rust policy now parses product source and rejects fully qualified blocking
filesystem, process, socket-bind, thread-sleep, and blocking HTTP calls when
they are directly reachable from an async function. Test support is excluded;
an operation deliberately moved into `tokio::task::spawn_blocking` is an owned
boundary. Positive and negative syntax fixtures prevent the context rule from
becoming a text search or banning synchronous startup/tool code.

The measured findings were in managed-engine startup, server assembly, and the
doctor command. Engine filesystem, subprocess, and port-probe operations now
use Tokio APIs; the spool's synchronous constructor runs as one complete
`spawn_blocking` operation. Extraction of engine I/O removed the supervisor's
oversized-file ratchet, lowered its `ensure_binary` function ceiling from 131
to 122, and lowered the server assembly file/function ceilings from 448/148 to
446/146. `clippy.toml` additionally bans runtime thread sleep and blocking
Reqwest entry points in all contexts.

Validation passed the 36 xtask tests, the server unit/integration suite (27
passed and 6 intentionally ignored real-engine tests), full-feature workspace
Clippy with `-D warnings`, and the repository policy with `[]`.

## Release source-line contract

The release profile retains line tables in the shipped binary:

```toml
[profile.release]
debug = "line-tables-only"
strip = "none"
```

Cargo documents `line-tables-only` as a supported debug level and `none` as the
unstripped release setting. The executable product policy requires this exact
pair, and its negative fixture rejects `debug = false` with
`strip = "debuginfo"`. No separate symbol companion is part of the Parallax
artifact contract; Plan 102 now names retained line tables as its input.

Measured on Linux x86_64 at commit `87b98b9` with
`cargo build --release -p parallax-cli`:

| Evidence | Result |
|---|---|
| Final binary | `target/release/parallax`, 267,468,392 bytes (256 MiB) |
| ELF sections | `.debug_line`, `.debug_info`, and `.symtab` present |
| GNU build ID | `fe5229fd5074f11946f3883d43699db29481b0cb` |
| Optimized entry mapping | `parallax::main` → `crates/parallax-cli/src/main.rs:217` |
| Async entry mapping | `parallax::main::{closure#0}` → `crates/parallax-cli/src/main.rs:216` |

The mapping was read from the final optimized binary with `readelf`, `nm`, and
`addr2line`. Plan 102 must repeat the check on every final archive target,
remap build-host paths for reproducibility, and track size; it must not strip
the only source-line information. Reference:
[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html).

## Matrix protection and feature selections

The executable product policy now checks the required Rust, rustdoc, and Clippy
groups and high-signal lints, `unsafe_code = "forbid"`, all four measured
thresholds, and the disallowed-method set. A negative fixture weakens
`pedantic` to `allow` and proves the policy fails closed. The blocking Reqwest
entry is intentionally marked `allow-invalid` because the repository disables
Reqwest's blocking feature; the ban becomes reachable automatically if that
feature is ever introduced.

All required selections passed with locked dependencies and `-D warnings`:

- default workspace, all targets;
- `parallax-cli` with `embed-ui`, all targets; and
- `parallax-storage` with `conformance`, all targets.

## Retirement gate

`cargo xtask ci --full` passed on `main` at `cdbe7c1`. The aggregate included
workspace formatting and strict Clippy, Bun lockfile install, formatting,
typecheck, lint, 175 UI tests, client and SSR production builds, 231 nextest
tests with 6 intentional real-engine skips, the compile-fail doctest, and the
RustSec audit. Plan 096 is complete; remaining dependency orchestration was
handed to Plan 101 and final artifact verification to Plan 102 (both later
closed — historical handoff only, not present-tense ownership).
