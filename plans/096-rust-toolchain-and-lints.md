# Plan 096: Activate a strict measured Rust baseline

> **Executor instructions**: Activate workspace lint inheritance before adding
> categories. Fix operational findings intentionally; never discard errors,
> delete required progress output, or add broad allows just to make Clippy
> green. Resolve the latest stable toolchain at execution time.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 095
- **Category**: Rust / code health
- **Planned at**: `eefa4617`, 2026-07-12
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

Out of scope:

- Typed domain errors, owned by 099.
- Module/facade decomposition, owned by 098.
- Nightly-only custom lints.
- Copying Jackin's version, thresholds, or allow list.

## Steps

### Step 1: Pin the reproducible toolchain

Resolve the latest stable Rust release at execution time. Pin its exact version,
components, and supported release targets in `rust-toolchain.toml`; align mise,
CI, and docs; set workspace `rust-version`; and add edition/style-edition 2024
`rustfmt.toml`.

### Step 2: Make inheritance real

Add `[lints] workspace = true` to every workspace crate in one focused change.
Run the existing policy and record the actual delta before adding categories.
New crates inherit in their creation commit.

### Step 3: Stage lint families

Define Rust, rustdoc, and Clippy tables. Promote only clean/understood families
in this order:

1. correctness, suspicious, future-incompatible, unused/must-use;
2. silent-result and future misuse;
3. panic/unwrap/expect APIs in production;
4. maintainability/pedantic and public-surface lints;
5. numeric casts/conversions after semantic review.

Add `unreachable_pub` and `unfulfilled_lint_expectations`. Stronger per-crate
denies belong only on stable pure leaves.

### Step 4: Add measured Clippy configuration

Set thresholds from plan 093's census. Test valves may permit indexing/debug or
unwraps in tests where intentional. CLI progress/readiness output is required;
use explicit writers or narrow reasoned expectations rather than suppressing
narration.

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

## Test Plan

- Full workspace fmt/check/Clippy/tests on the pinned toolchain.
- Per-lint negative fixtures in xtask policy.
- Suppression reason/stale expectation fixtures.
- CLI progress/readiness snapshot tests.
- Failure-path tests for every newly handled operational result.

## Done Criteria

- [ ] Every workspace crate inherits root lints.
- [ ] Toolchain/rust-version/mise/CI/docs agree on latest stable.
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings` is clean.
- [ ] No unexplained broad allow exists.
- [ ] Stale expectations fail.
- [ ] Unsafe and silent operational outcomes are explicitly handled.
- [ ] Required CLI progress/readiness remains intact.
- [ ] Suppression and unsafe ratchets cannot grow.

## STOP Conditions

- Strictness requires behavior change without characterization.
- A lint fix discards an operational error or hot-path ownership guarantee.
- A crate/module-level allow is the only proposed escape.
- Toolchain pin is not the current stable release required by policy.

## Remove When

Delete this plan and row when every crate inherits a warning-free strict policy
on the exact current stable toolchain and suppression evidence is green.
