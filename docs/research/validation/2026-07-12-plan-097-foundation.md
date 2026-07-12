# Plan 097 foundation evidence

Date: 2026-07-12

## Model ownership slice

`parallax-model` is a tier-0 product crate with only Serde dependencies. It now
owns the normalized telemetry rows, error events, metadata records, and stable
query-neutral value types formerly declared by storage. `parallax-storage`
re-exports the crate as `parallax_storage::model` for compile-compatible
downstream migration, while `parallax-core` imports `parallax-model` directly.
Cargo metadata therefore has no core-to-storage edge, and the corresponding
architecture exception was deleted.

Compatibility evidence:

- the new model serde contract round-trips exact span, log, metric point,
  exemplar, histogram, and error-event JSON shapes;
- 54 core unit tests and its Plan 093 baseline test pass;
- 61 storage tests pass against the compatibility re-export;
- focused model/core Clippy passes with `-D warnings`;
- repository policy returns `[]`; and
- the syntax-derived facade check passes.

## Cycle-safe test support slice

`parallax-test-support` now owns `MemoryStore`, its failure gate, 20 focused
tests, and the reusable telemetry-store conformance scenarios. Its only normal
workspace dependencies point down to model, proto, and the storage port/types.
API and server consume it only through Cargo dev dependencies; storage has no
dependency back to test support.

The architecture evaluator now explicitly permits product-to-test-support dev
edges while rejecting normal/build reachability and retaining mixed-cycle
checks. Its fixture proves both sides. Live metadata reports:

- `parallax-api -> parallax-test-support (dev)`;
- `parallax-server -> parallax-test-support (dev)`;
- `parallax-test-support -> parallax-storage (normal)`; and
- no storage-to-test-support edge.

`cargo tree -p parallax-cli --edges normal,build` contains no
`parallax-test-support`. Full-feature workspace Clippy, repository policy, and
the syntax-derived facade check pass; API (32), server library (17), and moved
test-support (20) tests pass.

## Capability-port slice

The broad storage boundary now has 13 independently implementable object-safe
ports: ingest, base trace/log/metric reads, service analytics, metric analytics,
run reads, trace analytics, log analytics, runtime metrics, error analytics,
log counts, and raw SQL. `TelemetryStore` is an empty composition umbrella over
those ports. GreptimeStore and MemoryStore implement every capability directly;
forwarding adapters and telemetry clones were not introduced.

Concrete tests import only the capability that owns the operation they invoke,
while product composition can continue using `dyn TelemetryStore`. Extracting
shared adapter math and policy rules, Greptime SQL and metric naming rules, and
memory analytics capabilities made the boundary change shrink-only: adapter
630→595, Greptime 3184→3095, and MemoryStore 1553→1463 logical lines.

Validation passed workspace/all-target compilation, full-feature workspace
Clippy with warnings denied, all repository policies, and the complete nextest
workspace suite (232 passed, 6 skipped).

## Plan 126 extraction handoff

Plan 126's placeholder is replaced by six stable, owned extraction rows for
ingest normalization, analysis, evidence, Greptime, metadata, and spool. Each
row names its current surface, final crate/facade, consumers, and compatibility
oracles. The documentation policy now schema-validates the Plan 097 handoff in
addition to the existing Plan 127 ledgers and rejects pending, unowned,
malformed, or reused IDs.
