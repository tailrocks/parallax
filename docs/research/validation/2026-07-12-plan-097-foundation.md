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
