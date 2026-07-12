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
