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
- **Status**: BLOCKED

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

### Preliminary decision work landed (helper, 2026-07-17) — peer verify/extend

**Do not retire yet.** Plan/index status intentionally remains unchanged under
the helper objective; the peer executor owns the status transition.

- `docs/research/decisions/retention-and-prune-contract.md` records contract
  version 1 under the operator unblock directive: every current data class and
  the rule for future derived extensions have an owner/lifecycle;
  unresolved/active/saved/pinned protections are explicit; resolved issues and
  terminal invocations receive 30 days;
  prune defaults to a deterministic dry run; destructive execution requires
  confirmation; cross-store work uses a durable resumable journal.
- Physical-reclaim wording follows current GreptimeDB 1.1 documentation: TTL
  expiry and compaction are asynchronous. Success requires logical deletion;
  measured physical bytes and pending compaction are reported separately.
  Existing native metric tables are reconciled through the bounded catalog,
  while creation hints cover newly created tables.
- `retention-and-prune-contract.toml` pins the record digest and every
  destructive decision. `product.retention-decision` validates approval,
  ownership, defaults, protections, confirmation, recovery, reclaim honesty,
  native metric TTL reconciliation, and compatibility; focused positive and
  mutation tests are included.
- Peer must challenge the lifecycle matrix against live schema/current product
  intent, expand mutation coverage if needed, then implement Steps 2-5 and run
  the live Greptime/CLI proof. Do not treat this preliminary record alone as
  implementation completion.

### Preliminary deterministic plan core (helper, 2026-07-17) — peer verify/extend

- `parallax_storage::prune` defines the query-neutral Step-2 contract:
  canonical store/class ordering across every current lifecycle class, typed
  row/object/byte estimates, typed active/unresolved/pinned/not-expired
  exclusions, bounded warnings, and string-encoded nanosecond cutoffs for safe
  machine output.
- `PrunePlan::build` fails closed on item/annotation/text caps, empty snapshot
  generations, cutoff disagreement, missing estimates, empty/duplicate targets,
  and produces a stable SHA-256 plan identity independent of input item or
  annotation ordering.
- `validate_snapshot` and `authorize` bind execution to the exact plan ID and
  unchanged config/protection/catalog generations. Dry-run needs no destructive
  confirmation; execution does. Eight focused tests and strict storage clippy
  cover the preliminary contract.
- Peer must wire bounded store-owned candidate discovery, progress reporting,
  CLI human/JSON presentation, durable journal execution, and real-store
  dry-run/execution parity. Revisit caps with live cardinality evidence; this
  core alone performs no deletion.
- `MetadataPruneStore::invocation_prune_item` plus the Turso adapter now adds
  one read-only bounded aggregate for terminal invocation eligibility. It
  treats the cutoff as inclusive, counts active and not-yet-expired rows as
  typed exclusions, and emits no identifiers or unbounded row set. The focused
  temp-Turso test covers eligible-at-boundary, active, and recent terminal
  rows. Peer must add pin exclusions and the remaining metadata classes before
  claiming complete store discovery.
- Issue lifecycle discovery now uses a persisted `issues.resolved_at` timestamp,
  added forward-only at bootstrap for existing Turso databases. Resolving sets
  the timestamp, reopening clears it, and one bounded aggregate reports
  inclusive-cutoff eligible rows plus typed unresolved/not-expired exclusions.
  This remains read-only candidate discovery: no issue or cascade deletion is
  authorized by this slice, and pin protection still must land before execution.
- The same bounded Turso read now estimates `issue_buckets` and
  `issue_occurrences` reachable from eligible resolved owners. Both items state
  that execution is owner-cascade-only; unresolved/recent owners contribute no
  dependent candidates. Standalone occurrence-ledger compaction remains the
  existing ingest-maintenance concern and is not broadened by prune planning.
- Normal-prune discovery now emits zero-eligibility items for dashboards,
  investigations, and saved views instead of silently omitting user-owned
  state. Their bounded counts use a typed `retained_by_policy` exclusion;
  explicit user deletion remains the only deletion path.
  Focused metadata and prune-core tests pass. The combined strict Clippy gate
  was temporarily obstructed by a peer's uncommitted metrics-explorer helper
  (`adapter_math::increase_from_buckets`); peer verification must rerun it once
  that concurrent slice is integrated.
- Alert rules, rule states, incidents, destinations, delivery events, and
  bounded checks now receive the same explicit zero-eligibility treatment via
  one bounded aggregate. Their alert-owner/user-delete lifecycles remain
  authoritative; normal prune gains no alert deletion capability.
- `MetadataPruneStore::metadata_prune_items` now assembles all 13 current Turso
  lifecycle classes behind one bounded deterministic facade with one shared
  cutoff. Focused coverage pins complete class membership and ordering. Pin
  reachability and journal-backed execution remain deliberately unwired.
- Restart-safe journal recovery now has a validated persisted-plan seam:
  `PrunePlan::decode` rejects unknown contract versions, unknown fields,
  identity-changing mutations, and plans outside current safety bounds before
  reconstructing the private immutable plan. The Turso journal tables and
  transitions still need peer implementation/verification.
- A preliminary Turso journal now atomically persists the immutable plan plus
  ordered `planned` steps, treats repeat creation as idempotent, and validates
  plan/step bytes and current bounds on restart. It cannot transition or delete
  anything yet; peer must add `executing -> complete`, failure recording,
  resume behavior, and cross-store execution proof.
- Journal steps now enforce atomic `planned -> executing -> complete`
  transitions, preserve bounded failure evidence for retry, clear it when a
  retry begins, skip already-complete steps, and complete the parent journal
  only after every step completes. No store deletion is attached; peers must
  review crash windows around real external work and prove resume behavior.
- `Spool::prune_estimate` adds bounded read-only local-disk discovery for
  recognized active, legacy, and rotated spool files. Every directory entry
  consumes the scan cap, unrelated files are never selected, and cap overflow
  fails closed. Wiring this estimate into the unified plan and replacing the
  unsynchronized legacy CLI truncation path remain peer work.

## Historical Blocker Evidence (2026-07-14; superseded 2026-07-17)

At that date, `docs/research/decisions/retention-and-prune-contract.md` did not
exist.
The current V1 scope explicitly says `parallax prune` reclaims spool segments
only and assigns immediate physical reclaim to this plan. No approved record
names the required data-class ownership, resolved/unresolved policy, pin
protection, cross-store recovery, or truthful physical-reclaim promise.
Consequently Steps 2-5 were forbidden until the operator supplied the required
approved contract.

Fresh audit on 2026-07-15 at `691cf17`: repository search still finds no
`docs/research/decisions/retention-and-prune-contract.md`, and no operator
approval names the destructive lifecycle fields required by Step 1. The
spool-only implementation remains authoritative and no deletion work is safe.

The operator unblock directive and preliminary decision record above supersede
this blocker evidence. Peer review of that record is still required before
destructive implementation; never infer different destructive semantics from
prose or implement a partial delete path.

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
