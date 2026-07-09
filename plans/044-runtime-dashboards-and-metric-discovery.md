# Plan 044 Remaining: Runtime Dashboards And Metric Discovery

## Audit Verdict

Implementation and bounded label-value tests are landed. The runtime metric
contract now documents `tokio.runtime.*` as the Rust playground path while
keeping `process.*` as a supported family. Remaining work is seeded evidence.

## Remaining Work

- [ ] Run a seeded metric discovery check through native GreptimeDB metric
  tables and record autocomplete/label-value proof.
- [ ] Verify runtime dashboard presets use the finalized native metric names.

## Remove When

- Seeded metric discovery and runtime-dashboard UI evidence are recorded.
