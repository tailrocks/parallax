# Plan 096: Activate a strict measured Rust baseline

> **Executor instructions**: Activate workspace lint inheritance before adding
> categories. Fix operational findings intentionally; never discard errors,
> delete required progress output, or add broad allows just to make Clippy
> green. Resolve the latest stable toolchain at execution time and implement
> the target behavior in `ENGINEERING-STANDARDS.md`.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 095, 127
- **Category**: Rust / code health
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

The root currently declares only a minimal Clippy warning, and six of seven
workspace crates do not opt into workspace lints. Rust floats on `stable` with
no explicit `rust-version`, `rustfmt.toml`, or `clippy.toml`. Bare suppressions,
silent operational results, unsafe test environment mutation, and broad public
modules need a staged, measurable policy rather than a copied lint table.

## Scope

In scope:

- Exact latest-stable toolchain/components/targets and workspace rust-version.
- Rustfmt/Clippy configuration and workspace lint inheritance.
- Staged Rust/rustdoc/Clippy tables.
- Reason-bearing expectations and one suppression ratchet.
- Unsafe/silent-result/print-output cleanup required for strictness.
- Async blocking-method ownership and a release debug-symbol profile decision.

Out of scope:

- Typed domain errors, owned by 099.
- Module/facade decomposition, owned by 098.
- Nightly-only custom lints.
- Copying another repository's versions, thresholds, or allow list without a
  Parallax measurement.

## Steps

### Step 1: Pin the reproducible toolchain

Resolve the latest stable Rust release at execution time. Pin its exact version,
components, and supported release targets in `rust-toolchain.toml`; align mise,
CI, and docs; set workspace `rust-version`; and add edition/style-edition 2024
`rustfmt.toml`. Every member sets `rust-version.workspace = true` as well as
workspace version/edition/license/repository inheritance. A Cargo-metadata gate
fails missing metadata, a floating channel, or disagreement among tool surfaces.

### Step 2: Make inheritance real

Add `[lints] workspace = true` to every workspace crate in one focused change.
Run the existing policy and record the actual delta before adding categories.
New crates inherit in their creation commit.

### Step 3: Stage lint families

Define one root Rust, rustdoc, and Clippy table with explicit group priorities.
The target is:

1. Rust 2024 compatibility, future-incompatible, idiom, nonstandard-style,
   unused/must-use, dead/unreachable, unsafe-operation, missing-debug, and
   `unreachable_pub` families;
2. rustdoc broken/private intra-doc links, bare URLs, invalid HTML/code blocks,
   and public error/panic/safety documentation where applicable;
3. Clippy `all` plus `pedantic` under CI `-D warnings`, with selective `cargo`
   rules and explicit group priorities;
4. `dbg_macro`, `todo`, `unimplemented`, production panic/unwrap/expect,
   ignored future/must-use/`Result::ok`, await-held lock/refcell, unsafe/memory
   escape, stale expectation, wildcard dependency, and unowned print macros;
5. numeric truncation/sign/precision and indexing rules after semantic review.

Never enable `clippy::restriction` or `nursery` wholesale. Add
`unfulfilled_lint_expectations`. Test configuration may allow intentional
panic/unwrap/expect/indexing, but not discarded results/futures, `dbg!`, unsafe,
or ambient races. The full intended rule matrix and test valves are tested with
fixtures, so an upstream preset change cannot silently alter policy.

Cargo cannot combine inherited workspace lints with extra per-package manifest
lints. Stronger rules for `parallax-model`, `parallax-proto`, and later pure
ingest/analysis/evidence leaves use crate-root inner attributes. Pilot
applicable panic/index/time/side-effect rules on those named leaves, record
false-positive evidence, and ratchet every local attribute.

### Step 4: Add measured Clippy configuration

Set the new/restructured ceilings from `ENGINEERING-STANDARDS.md`: 100-line
functions, cognitive complexity 25, nesting 4, and 6 arguments. Existing
over-target functions receive exact shrink-only rows; do not import unrelated
150/58/5/7 thresholds. CLI progress/readiness output is required; use injected
writers or narrow reasoned expectations rather than suppressing narration.

### Step 5: Eliminate unexplained suppressions

Convert bare allows to narrow reason-bearing expectations where supported.
The syntax-aware gate verifies reason presence on both allow and expect,
tracks per-lint/per-crate counts in the single ratchet, and rejects stale
expectations.

### Step 6: Resolve unsafe and silent results

- Replace unsafe test env mutation with injected environment/config access, or
  document the smallest justified boundary before `unsafe_code` promotion.
- Handle shutdown/flush, ALTER/reconcile, and process-cleanup failures according
  to their runtime contracts.
- Do not turn an error into `let _ =` or logging-only success to satisfy lint.

### Step 7: Own blocking and release-debug boundaries

Inventory synchronous process/filesystem/network calls inside async product
code. Add measured `clippy.toml` disallowed methods plus syntax-aware context
checks. Runtime work uses Tokio or `spawn_blocking`; xtask, startup, tests, and
dedicated blocking threads may use only narrow reasoned expectations. Include
supervisor and doctor paths in the characterization.

Define and test the Cargo release debug strategy required by Parallax's source-
line/backtrace capture contract: either retain line tables in shipped binaries
or produce build-ID-keyed symbol companions. Record panic/backtrace fidelity
and size evidence. Plan 102 consumes this decision when freezing archive,
signature, SBOM, and attestation contents; do not copy a strip profile that
breaks capture.

## Test Plan

- Full workspace fmt/check/Clippy/tests on the pinned toolchain.
- Per-lint negative fixtures in xtask policy.
- Default plus every supported `embed-ui`/`conformance` lint selection, with
  plan 101 owning final feature-matrix orchestration.
- Suppression reason/stale expectation fixtures.
- CLI progress/readiness snapshot tests.
- Failure-path tests for every newly handled operational result.
- Async blocking-call and release backtrace/symbol fixtures.

## Done Criteria

- [ ] Every workspace crate inherits root lints.
- [ ] Every member inherits complete workspace package metadata including
  `rust-version`.
- [ ] Toolchain/rust-version/mise/CI/docs agree on latest stable.
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings` is clean.
- [ ] Root policy enables the named Rust/rustdoc/Clippy families, selective
  restrictions, and `unsafe_code = "forbid"`; rule fixtures prevent weakening.
- [ ] No unexplained broad allow exists.
- [ ] Stale expectations fail.
- [ ] Unsafe and silent operational outcomes are explicitly handled.
- [ ] Runtime blocking calls are async-safe or sit behind a narrow owned
  blocking boundary.
- [ ] Release line-table/symbol policy preserves source-line capture and is
  handed to plan 102.
- [ ] Required CLI progress/readiness remains intact.
- [ ] Suppression and unsafe ratchets cannot grow.

## STOP Conditions

- Strictness requires behavior change without characterization.
- A lint fix discards an operational error or hot-path ownership guarantee.
- A crate/module-level allow is the only proposed escape.
- A blocking-call restriction would force asynchronous work onto a runtime
  worker or ban legitimate xtask/dedicated-thread work without scoped policy.
- A release profile strips the only available source-line information.
- Toolchain pin is not the current stable release required by policy.

## Remove When

Delete this plan and row when every crate inherits a warning-free strict policy
on the exact current stable toolchain and suppression evidence is green.
