# Plan 105: Replace metric overview and trend stubs with bounded real data

> **Executor instructions**: Preserve GreptimeDB native per-metric tables and
> current GraphQL/UI compatibility until a versioned contract is approved.
> Query bounded windows and prove both adapters agree before exposing counts.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 097, 099, 100
- **Category**: metrics / API / UI
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

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

## Steps

1. Specify whether `metric_point_count` is windowed, which samples count, how
   stale/NaN/histogram points behave, and the trend interval/downsampling cap.
   Decide whether V1's promised `parallax metrics --run` command is retained
   and implemented or removed from the contract. Define how encoded native
   table names map back to user metric names, collisions/errors, and how
   metric-only services enter service discovery. Add CLI/GraphQL compatibility
   snapshots before implementation.
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

- The implementation needs a custom raw metric table.
- Semantics require an unbounded scan or one query per result/metric row.
- Native tables lack a required capability and the mandated research/upstream
  consultation sequence has not occurred.
- Adapter results disagree and no explicit product decision explains why.

## Remove When

Delete this plan and index row when real bounded counts/trends replace both
stubs and adapter/API/UI conformance is green.
