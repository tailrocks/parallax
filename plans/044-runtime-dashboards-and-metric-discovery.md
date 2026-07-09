# Plan 044 Remaining: Runtime Dashboards And Metric Discovery

## Audit Verdict

Implementation and bounded label-value tests are landed. Remaining issue is
contract/evidence drift: the plan expected Rust `process.*` runtime metrics,
while the current playground emits `tokio.runtime.*`.

## Remaining Work

- [ ] Reconcile and document the runtime metric contract around
  `tokio.runtime.*` versus any expected `process.*` fields.
- [ ] Run a seeded metric discovery check through native GreptimeDB metric
  tables and record autocomplete/label-value proof.
- [ ] Verify runtime dashboard presets use the finalized native metric names.

## Remove When

- Runtime metric contract and seeded UI evidence are recorded.
