# Plan 094: Repair CI and repository security foundations

> **Executor instructions**: Preserve the stable `ci-required` contract and
> full-SHA action pins. Add tests for path/permission decisions before changing
> the workflow DAG. Do not introduce Node, a foreign package manager, rustls,
> an extra branch, or release-publication behavior owned by plan 102.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 093
- **Category**: CI / security / DX
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

Parallax already has strong SHA pinning, path routing, caches, and aggregate
checks. The remaining gaps are specific: check/clippy serialization, incomplete
path classification, UI format not enforced, no required advisory gate,
contradictory public/private security prose, and no recorded proof that GitHub
rules actually require the aggregate check.

## Current Evidence

- `clippy` waits for `check` despite no artifact handoff.
- Shell/release-script validation is not triggered by every script input.
- Preview archive routing omits `mise.toml` and shared release actions.
- `ui/package.json` has `check`, but PR CI does not run it.
- No required cargo-audit lane exists.
- `SECURITY.md` calls the repository private while contribution/protection docs
  call it public.
- Current lockfile advisories are recorded in the Jackin reference note.

## Scope

In scope:

- CI classifiers, DAG, timeouts, Bun install/format step, source hygiene.
- A mise-pinned required cargo-audit gate.
- Security/contribution/protection policy reconciliation.
- Workflow permissions and GitHub ruleset evidence.
- Actionlint and classifier/aggregate fixtures.

Out of scope:

- Full cargo-deny/shear/hack policy and nextest telemetry, owned by plan 101.
- Deterministic archive/signing implementation, owned by plan 102.
- Automatic updater branches.
- Replacing existing cache backends before measurement.

## Steps

### Step 1: Remove current advisory debt

Update `anyhow` to at least 1.0.103 and `crossbeam-epoch` to at least 0.9.20
when current dependency constraints permit. Run a fresh RustSec audit. If Turso
pins an affected transitive version, record reachability, upstream issue,
owner, expiry, and scheduled recheck.

Pin `cargo-audit` in mise and add a lockfile-sensitive required CI job. An
exception requires a reason and expiry; a warning/soundness category is not
silently ignored.

### Step 2: Test and centralize path classification

Move path classification to one reusable, fixture-tested implementation.
Ensure every command runs when its true inputs change, including:

- `scripts/**`;
- `mise.toml` and tool locks;
- shared `.github/actions/**`;
- release/preview workflows;
- Cargo/Bun manifests and lockfiles;
- policy/config/ratchet files as they appear.

Tests cover Rust-only, UI-only, shared, release-only, deletion, rename, and
mixed changes plus skipped-as-success aggregation.

### Step 3: Shorten the required DAG safely

Make check and Clippy siblings after common prerequisites. Keep tests, UI,
embed, actionlint, advisory, and policy inputs explicit in `ci-required`.
Preserve current job name, concurrency, SHA pins, cache fallbacks, and timeouts.

### Step 4: Enforce the Bun/source contract

- Use `bun ci` after verifying current trusted dependency behavior.
- Add `bun run check` before lint/typecheck/test/build.
- Reject npm/pnpm/yarn lock/config files and stale commands in active metadata.
- In CI, run `git diff --check` over validated event-specific base/head SHAs
  (pull-request base to head; push before to after) and fixture zero/missing-base
  behavior. Local xtask runs both `git diff --check` and
  `git diff --cached --check` so unstaged and indexed changes are covered. A
  clean checkout with no explicit range is not accepted as CI evidence.

### Step 5: Reconcile security and GitHub policy

Make `SECURITY.md`, `CONTRIBUTING.md`, and `REPOSITORY_PROTECTION.md` agree on
public visibility, vulnerability reporting, DCO, required checks, reviews, and
administrator bypass. If the reporting channel cannot be derived, STOP for the
operator rather than inventing an address.

Record sanitized evidence that the live GitHub ruleset requires `ci-required`
and DCO. Remove unused OIDC permission from jobs that do not sign or attest;
plan 102 handles release-specific permissions during its rewrite.

## Test Plan

- Table-driven classifier fixtures.
- Aggregate result fixtures for success/failure/cancel/skipped combinations.
- Source-hygiene range fixtures for pull request, push, initial/zero base, and
  local staged/unstaged changes.
- Actionlint.
- `bun ci` and all UI scripts.
- Cargo audit clean or exact approved expiring exception.
- Workflow permission assertions.

## Done Criteria

- [ ] Cargo audit is required on lock/dependency changes.
- [ ] Check and Clippy are parallel siblings.
- [ ] Every command/archive input routes to its validator.
- [ ] UI format, lint, types, tests, and build are required where applicable.
- [ ] Source hygiene checks the committed event range in CI and both local diff
  surfaces instead of passing vacuously on a clean checkout.
- [ ] No mutable third-party Action tag exists.
- [ ] `ci-required` and DCO are verified in the live ruleset.
- [ ] Security/contribution/protection prose agrees.
- [ ] Actionlint and classifier/aggregate tests pass.

## STOP Conditions

- A change weakens or renames `ci-required` without a ruleset migration.
- Advisory success requires a permanent unreasoned ignore.
- Path routing cannot distinguish a required archive/security input.
- A proposed action needs broader permissions without a specific operation.
- Security reporting details need an operator decision.

## Remove When

Delete this plan and row after the required workflow is green on `main`, live
ruleset evidence is recorded, and security docs are coherent.
