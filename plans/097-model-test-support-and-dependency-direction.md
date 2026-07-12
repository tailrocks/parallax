# Plan 097: Extract model, port, and test-support foundations

> **Executor instructions**: Move ownership in vertical slices and preserve
> serialized, SQL, Arrow, GraphQL, and allocation contracts. Do not create a
> Cargo dev cycle, expose a test fallback, or clone telemetry to satisfy a new
> trait. Rust test bodies have already moved under plan 127. Remove each
> architecture exception as soon as its edge disappears; plan 126 completes the
> final physical adapter/business crate split.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 096, 127
- **Category**: architecture / storage / testing
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

`parallax-core` depends on storage-owned normalized rows, inverting domain and
infrastructure ownership. Storage combines a broad port, query DTOs, production
Greptime/Turso implementations, and a 2,500-line MemoryStore. Reusable fakes
remain in product ownership, and existing conformance does not yet run full
dual-seed parity for edge-case service names.

## Foundation Graph

```text
T0  parallax-model       normalized domain rows and stable value types
T0  parallax-proto       OTLP/wire definitions
T1  parallax-core        temporary normalization/analysis/evidence owner
T1  parallax-storage     capability ports plus temporary private adapters
T2  parallax-api
T3  parallax-server
T4  parallax-cli

Aux parallax-test-support
Aux parallax-xtask
Aux parallax-mcp-spike
```

Required product direction is
`proto/model -> core and storage -> api -> server -> cli`. Core and storage may
not depend on each other. This is an intermediate graph only. Plan 126 moves
core responsibilities to ingest/analysis/evidence, moves Greptime/Turso/spool
implementations to concrete crates, makes server the composition root, and
deletes `parallax-core`; `ENGINEERING-STANDARDS.md` is the final authority.

## Scope

In scope:

- New `parallax-model` and `parallax-test-support` crates.
- Normalized row/type ownership and compile-driven consumer migration.
- Capability-specific, query-neutral storage ports separated from concrete
  public API even before their physical crate extraction.
- MemoryStore/failure fixtures/conformance ownership.
- Full memory/Greptime conformance expansion.
- Architecture exceptions and zero-copy allocation evidence.

Out of scope:

- Final business/adapter crate decomposition, plan 126.
- Public facade/module decomposition beyond ownership moves, plan 098.
- Typed errors and IDs, plan 099.
- Storage engine/schema substitutions.
- Raw Greptime native-table redesign.

## Steps

### Step 1: Freeze compatibility oracles

From plan 093's baseline, add any missing snapshots for:

- normalized log/span/metric/event serde;
- OTLP-to-model golden vectors;
- Arrow/database row conversion;
- GraphQL-visible values;
- extension/native table SQL and conformance;
- ingest clone/allocation counts.

### Step 2: Create `parallax-model`

Move normalized spans, logs, metric points/histograms/exemplars, error events,
and query-neutral records from storage in small slices. Protocol input stays in
`parallax-proto`; database mapping stays in storage. Preserve field names,
serde, equality/order, timestamp precision, and ownership.

Change core normalization to emit model types, then remove
`parallax-core -> parallax-storage` and delete the stale architecture exception.

### Step 3: Split storage capabilities

Replace the broad adapter surface with cohesive traits for ingest, traces,
logs, metrics, analytics, and metadata. Shared query types live at the lowest
valid owner and contain no Arrow/HTTP/Greptime/Turso fields. Keep an umbrella
trait only at composition points that require all capabilities.

Production Greptime/Turso implementations remain private behind capability
exports/factories during this phase. Mark their concrete external dependencies
and modules in plan 126's schema-checked incoming extraction handoff, using
stable IDs, source/target owners, consumers, compatibility oracles, and status.
Private visibility is not accepted as final dependency isolation. No new trait
may force telemetry cloning or stringly error conversion, and no handoff row may
remain pending or unowned when this plan retires.

### Step 4: Create cycle-safe test support

`parallax-test-support` uses normal downward dependencies only on
`parallax-model` and the `parallax-storage` capability traits/types it
implements. Product crates consume it only through acyclic dev edges; the
architecture gate permits those exact dev edges while rejecting normal/build
reachability from release roots and mixed normal/dev cycles. Because MemoryStore
implements storage traits, storage and concrete adapters must not dev-depend
back on test support. Put the shared adapter conformance target in a downstream
owner such as `parallax-server/tests/adapter_conformance.rs`; that target
dev-depends on both `parallax-test-support` and `parallax-greptime`, while the
adapter never points back.

Move MemoryStore, shared builders, sample rows, failure controls, and generic
conformance harnesses. Remove duplicate fixtures after consumers migrate.
Prove test support is unreachable from release roots via normal/build features
and absent from release binaries/SBOM graphs; workspace metadata may truthfully
retain the package and dev edges.

Private algorithm tests stay in external child modules from plan 127.
`<crate>/tests/` targets exercise reviewed public facades only; controlled
seeding moves behind test-support rather than keeping implementation modules or
fields public.

### Step 5: Remove async-lock ambiguity

Split MemoryStore capability state so no synchronous mutex guard is held across
an actual await point. Keep the fake deterministic and simple; do not introduce
a production-grade async store or product mode.

### Step 6: Complete adapter conformance

Run identical seeded scenarios on MemoryStore and live GreptimeDB, including:

- empty and bounded windows;
- backslash/quote/unicode service names;
- logs, traces, metrics, exemplars, and derived errors;
- `histogram_count_series` parity and empty/non-empty histogram windows;
- ordering, limits, missing tables/columns, and schema widening;
- both fresh and restarted engine state.

Record any true engine divergence before changing semantics.

## Test Plan

- Compile/serde/golden tests for every moved type.
- Architecture fixture and real workspace graph checks.
- Capability conformance on MemoryStore and GreptimeDB.
- Release dependency-tree assertion excluding test support.
- Clone/allocation regression tests around OTLP normalization/ingest.
- Full workspace and ignored real-engine suites.

## Incoming Handoff From 127

Plan 127 must replace the placeholder with schema-validated rows before it
retires. Stable IDs are never reused; every row has one target owner and no
unowned status.

| Stable ID | Current owner | Consumers | Target test-support API/owner | Status |
|-----------|---------------|-----------|-------------------------------|--------|
| `127-pending` | Populate during plan 127 | Populate during plan 127 | Populate during plan 127 | PENDING |

## Done Criteria

- [ ] `parallax-core` has no storage dependency.
- [ ] Cargo metadata matches the foundation direction through T1 and records
  every remaining concrete extraction for plan 126.
- [ ] Every current/new member is classified and no stale exception remains.
- [ ] Production normal/build graphs cannot reach test support.
- [ ] Acyclic dev-only test-support edges pass while normal/build/mixed-cycle
  fixtures fail.
- [ ] MemoryStore and reusable fakes live outside product ownership.
- [ ] Storage capabilities are cohesive and conformance-complete.
- [ ] Dual-seed/backslash real-engine cases pass or a documented engine defect
  blocks only the exact case.
- [ ] Serde/SQL/Arrow/GraphQL compatibility and zero-copy evidence are unchanged.

## STOP Conditions

- A move requires cloning decoded telemetry on the hot path.
- A normalized domain type needs database-specific fields in model.
- Test support creates any production/dev cycle.
- A public integration test can compile only by exposing a concrete
  implementation module.
- Conformance exposes a product semantic decision not settled in the spec.
- The work touches custom raw-signal tables.

## Remove When

Delete this plan and row when model/test-support ownership, query-neutral ports,
and the foundation graph are enforced, all compatibility oracles pass, and the
schema-validated incoming ledger has zero pending/unowned rows and has populated
plan 126's concrete extraction handoff.
