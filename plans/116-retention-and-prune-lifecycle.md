# Plan 116: Reconcile retention and make prune reclaim what the contract promises

> **Executor instructions**: Decide lifecycle semantics in the implementation
> spec before deleting data. Raw signals stay in GreptimeDB native tables;
> mutable issue/run state stays in Turso. Preserve pinned evidence owned by plan
> 106 and fail safely on partial cross-store cleanup.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: CRITICAL
- **Depends on**: 093, 097, 099; 105 soft
- **Category**: retention / storage lifecycle / CLI
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

V1 scope promises resolved issues/rollups expire after a grace period and
`prune` reclaims space immediately, but current prune handles spool data only.
Retention changes also reconcile a fixed set of tables while native per-metric
tables receive TTL only at creation, so later configuration changes do not
retroactively apply across the metric catalog.

## Scope

- Explicit lifecycle for raw native telemetry, derived extension data, Turso
  issues/occurrences/buckets/runs, dashboards/investigations, spool, and future
  pinned evidence.
- Bounded idempotent `prune` planning/execution, dry-run/reporting, cross-store
  failure recovery, and immediate-reclaim truthfulness.
- Existing native per-metric table TTL reconciliation using catalog/native
  extension points.

Out of scope:

- Custom raw-signal tables, disabling native TTL, deleting unresolved issues by
  surprise, or an object-store/engine substitution.

## Decision Gate

Before approval, the current spool-only `prune` behavior and existing engine TTLs
remain authoritative; no new metadata, native-table, extension-table, dashboard,
investigation, run, issue, or evidence deletion may land. Step 1 must produce an
operator-approved `docs/research/decisions/retention-and-prune-contract.md` that
names every data class, owner, TTL/grace rule, resolved/unresolved behavior,
pin/reachability protection, logical-versus-physical reclaim promise,
confirmation policy, compatibility/migration behavior, and approval date. Add a
decision-policy fixture that fails missing, draft, rejected, or incomplete
approval.

Steps 2-5 are forbidden until that exact record is approved. If approval is
unavailable, rejects every proposal, or changes storage/product scope, mark this
plan `BLOCKED` with the exact open decision and stop. Never infer destructive
semantics from prose or implement a partial delete path.

## Steps

### Step 1: Decide the lifecycle contract

Inventory every data class, owner, configured/default TTL, resolved/unresolved
state, legal/user expectations, pin protection, cascade/reachability, and
physical-versus-logical reclaim. Resolve whether the existing resolved+30-day
and immediate-reclaim promises are implemented or corrected before code.

**Verify**: the decision-policy gate reports one approved complete lifecycle
contract. Otherwise mark `BLOCKED`; do not run Step 2.

### Step 2: Add a deterministic prune plan

Build a typed plan with cutoff, object/row/byte estimates, store/table/class,
pin/active exclusions, and warnings. Support dry-run and machine-readable
output. Validate bounds and require explicit confirmation for destructive
scope; long scans report progress.

### Step 3: Implement store-owned deletion

Add capability methods and Turso transactions for approved metadata classes.
Use Greptime native TTL/DELETE/ALTER extension points for extension/native data.
Make retry/restart idempotent and record partial completion without claiming
success. Never coordinate by cloning raw telemetry or copying it to Turso.

### Step 4: Reconcile native metric TTLs

Enumerate actual native metric tables through the bounded catalog, distinguish
Parallax-owned metrics from unrelated tables, compare configured TTL, and apply
verified native `ALTER` behavior. Handle tables created during reconciliation
and record unsupported engine behavior for upstream rather than inventing a
schema.

### Step 5: Prove reclaim and safety

Seed unresolved/resolved issues, occurrences, buckets, runs, dashboards,
investigations, raw signals, per-metric tables, extension data, spool, and pin
placeholders. Test cutoff edges, concurrent ingest/resolve, restart, partial
store failure, repeated prune, dry-run parity, and measured disk/row reclaim.

## Test Plan

- Lifecycle decision/table and config compatibility snapshots.
- Turso transaction/cascade/restart and Greptime real-engine TTL/ALTER tests.
- Per-metric table catalog/race/exclusion fixtures.
- Dry-run versus execution parity and JSON/human output.
- Active/unresolved/pinned/not-yet-expired preservation negatives.
- Partial failure/retry and physical reclaim measurement.

## Done Criteria

- [ ] Every persisted data class has one explicit owner and retention rule.
- [ ] Scope/docs/CLI claims match actual resolved-issue and prune behavior.
- [ ] Dry-run is deterministic, bounded, and equal to executed eligibility.
- [ ] Active/unresolved/pinned data cannot be deleted by normal prune.
- [ ] Cross-store retries are idempotent and never report partial work as success.
- [ ] Existing and newly created native metric tables receive configured TTLs.
- [ ] Real row/disk reclaim and progress/output behavior are verified.

## STOP Conditions

- The Step-1 operator approval is missing, draft, rejected, ambiguous, or changes
  implementation scope.

- The lifecycle/product decision is unresolved.
- A path requires a custom raw table, fallback engine, or disabling native TTL.
- Pin ownership from plan 106 cannot be preserved by the proposed contract.
- Partial failure can orphan or delete evidence without recoverable state.
- "Immediate reclaim" cannot be measured and docs are not corrected.

## Remove When

Delete this plan and index row when retention ownership, native metric TTL
reconciliation, and truthful safe prune behavior are implemented and verified.
