# Plan 097: Extract model/test support and enforce dependency direction

> **Executor instructions**: Move ownership in vertical slices and preserve
> serialized, SQL, Arrow, GraphQL, and allocation contracts. Do not create a
> Cargo dev cycle, expose a test fallback, or clone telemetry to satisfy a new
> trait. Remove each architecture exception as soon as its edge disappears.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 096
- **Category**: architecture / storage / testing
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

`parallax-core` depends on storage-owned normalized rows, inverting domain and
infrastructure ownership. Storage combines a broad port, query DTOs, production
Greptime/Turso implementations, and a 2,500-line MemoryStore. Reusable fakes
remain in product ownership, and existing conformance does not yet run full
dual-seed parity for edge-case service names.

## Target Graph

```text
T0  parallax-model       normalized domain rows and stable value types
T0  parallax-proto       OTLP/wire definitions
T1  parallax-core        normalization, analysis, redaction, bundles
T1  parallax-storage     production ports and GreptimeDB/Turso adapters
T2  parallax-api
T3  parallax-server
T4  parallax-cli

Aux parallax-test-support
Aux parallax-xtask
Aux parallax-mcp-spike
```

Required product direction is
`proto/model -> core and storage -> api -> server -> cli`. Core and storage may
not depend on each other.

## Scope

In scope:

- New `parallax-model` and `parallax-test-support` crates.
- Normalized row/type ownership and compile-driven consumer migration.
- Capability-specific storage ports.
- MemoryStore/failure fixtures/conformance ownership.
- Full memory/Greptime conformance expansion.
- Architecture exceptions and zero-copy allocation evidence.

Out of scope:

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
valid owner. Keep an umbrella trait only at composition points that require all
capabilities.

Production Greptime/Turso implementations remain private behind capability
exports/factories. No new trait may force telemetry cloning or stringly error
conversion.

### Step 4: Create cycle-safe test support

`parallax-test-support` uses normal downward dependencies on production
traits/types it implements. Product crates consume it only through dev edges.
Because MemoryStore implements storage traits, storage itself must not
dev-depend back on test support; run shared conformance in test support or a
downstream integration owner.

Move MemoryStore, shared builders, sample rows, failure controls, and
conformance harnesses. Remove duplicate fixtures after consumers migrate.
Ensure test-support is absent from release metadata/binaries.

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

## Done Criteria

- [ ] `parallax-core` has no storage dependency.
- [ ] Cargo metadata matches the target direction through T1.
- [ ] Every current/new member is classified and no stale exception remains.
- [ ] Production normal/build graphs cannot reach test support.
- [ ] MemoryStore and reusable fakes live outside product ownership.
- [ ] Storage capabilities are cohesive and conformance-complete.
- [ ] Dual-seed/backslash real-engine cases pass or a documented engine defect
  blocks only the exact case.
- [ ] Serde/SQL/Arrow/GraphQL compatibility and zero-copy evidence are unchanged.

## STOP Conditions

- A move requires cloning decoded telemetry on the hot path.
- A normalized domain type needs database-specific fields in model.
- Test support creates any production/dev cycle.
- Conformance exposes a product semantic decision not settled in the spec.
- The work touches custom raw-signal tables.

## Remove When

Delete this plan and row when model/test-support ownership and capability ports
are enforced by the architecture gate and all compatibility oracles pass.
