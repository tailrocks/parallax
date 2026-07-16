# Plan 105: Replace metric overview and trend stubs with bounded real data

> **Executor instructions**: Preserve GreptimeDB native per-metric tables and
> current GraphQL/UI compatibility until a versioned contract is approved.
> Query bounded windows and prove both adapters agree before exposing counts.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 097, 099, 133, 151
- **Category**: metrics / API / UI
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED — Plans 133 and 151 are incomplete

## Contract reconciliation (2026-07-17)

Plan 156 replaces `parallax.run.id` with `cli.invocation.id` and renames
`run_metric_points` to `invocation_metric_points` (`invocation_id` column).
Author the still-undecided CLI metric contract as `parallax metrics
--invocation`, never `--run`; run-scoped wording below reads as
invocation-scoped. See plans/156-unified-cli-observability-contract.md and
the Unified CLI Observability note in plans/README.md.

## Why

`metric_point_count` is a documented zero stub and metric trend data is empty,
so overview/API surfaces can imply that real ingested metrics do not exist.
The fix must use GreptimeDB's native metric tables, remain bounded, and avoid a
catalog/query fan-out per field or UI row.

## Scope

- Define exact point-count and trend window/aggregation semantics.
- Decide the promised `parallax metrics --run` CLI contract and native metric
  name normalization instead of leaving stale/missing surfaces.
- Add narrow storage capability methods for MemoryStore and GreptimeDB.
- Use native per-metric tables and existing table metadata/cache mechanisms.
- Include services seen only in native metric tables in bounded service/catalog
  discovery.
- Batch/memoize API access and render honest loading/empty/error/data UI states.
- Add real-engine conformance and query-count/bound tests.

Out of scope:

- Hand-rolled raw metric tables or a fallback engine.
- Unbounded lifetime counts, high-cardinality label downloads, or per-row N+1.
- General dashboard redesign or changing metric retention.

## Decision Gate

Until an operator-approved contract exists, preserve the current GraphQL/CLI
surface and do not claim a meaning for the stubbed values. Step 1 must produce
`docs/research/decisions/metric-summary-contract.md` with the exact query window,
eligible metric kinds/samples, NaN/stale/histogram treatment, trend buckets and
cap, `parallax metrics --run` disposition, native-name/collision mapping, metric-
only service discovery rule, compatibility promise, and approval date. Add a
decision-policy fixture that fails missing, draft, rejected, or incomplete
approval.

Steps 2-5 are forbidden until that record is operator-approved. If approval is
unavailable, rejects every proposal, or changes backend/API/UI scope, mark this
plan `BLOCKED` with the exact open decision and stop instead of selecting product
semantics during implementation.

## Steps

1. Specify whether `metric_point_count` is windowed, which samples count, how
   stale/NaN/histogram points behave, and the trend interval/downsampling cap.
   Decide whether V1's promised `parallax metrics --run` command is retained
   and implemented or removed from the contract. Define how encoded native
   table names map back to user metric names, collisions/errors, and how
   metric-only services enter service discovery. Add CLI/GraphQL compatibility
   snapshots before implementation.
   Verify the checked-in decision-policy gate reports one approved complete
   contract; otherwise mark `BLOCKED` and do not begin step 2.
2. Add capability-level storage methods returning typed, bounded summaries and
   a bounded metric-only service catalog.
   Implement MemoryStore first as executable semantics, then GreptimeDB using
   native per-metric tables and SQL-side aggregation. Reuse catalog caching and
   issue a bounded number of round trips independent of result rows.
3. Seed identical representative gauges, sums, histograms, empty windows,
   multiple services, late points, and retention edges into both adapters.
   Extend the real-engine conformance suite and capture query counts/plans.
4. Batch or request-memoize resolver access. Map typed errors centrally and
   retain old fields while filling them with real data. Implement the approved
   CLI surface through the canonical API, not direct storage.
5. Update the owning UI feature with stable query keys and honest empty/error
   states. Bound polling and preserve series identity when values do not change.

## Test Plan

- Unit tests for window/bucket/count semantics and timestamp boundaries.
- MemoryStore/GreptimeDB dual-seed conformance across metric kinds.
- Real-engine SQL/query-plan and bounded round-trip assertions.
- GraphQL/CLI schema/result snapshots and request batching tests.
- Native name round-trip/collision and metric-only service discovery fixtures.
- UI loading/empty/error/data, polling identity, and responsive smoke tests.

## Done Criteria

- [ ] Point-count and trend semantics are written and compatibility-pinned.
- [ ] Both adapters return identical bounded results from representative seeds.
- [ ] Greptime reads only native metric tables with SQL-side aggregation.
- [ ] Native metric names round-trip and metric-only services are discoverable.
- [ ] The CLI metrics promise is implemented through the API or removed from
  every authoritative contract by an explicit decision.
- [ ] Query count is bounded and no active resolver/client N+1 exists.
- [ ] Stubbed zero/empty values are removed without fabricating missing data.
- [ ] UI states and stable polling identity are tested.
- [ ] Full Rust, real-engine, GraphQL, and Bun gates pass.

## STOP Conditions

- The Step-1 operator approval is missing, draft, rejected, ambiguous, or changes
  implementation scope.
- The implementation needs a custom raw metric table.
- Semantics require an unbounded scan or one query per result/metric row.
- Native tables lack a required capability and the mandated research/upstream
  consultation sequence has not occurred.
- Adapter results disagree and no explicit product decision explains why.

## Remove When

Delete this plan and index row when real bounded counts/trends replace both
stubs and adapter/API/UI conformance is green.
