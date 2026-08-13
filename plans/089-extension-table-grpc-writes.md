# Plan 089: Move derived extension-table writes to GreptimeDB's row API

> **Executor instructions**: Recheck Step 0 first. Do not add or enable rustls,
> fork the dependency, change the storage stack, or weaken native-TLS policy.
> If the upstream blocker remains, refresh the evidence and leave this plan
> BLOCKED. Update `plans/README.md` with any state change.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: upstream `greptimedb-ingester` native-TLS/plaintext feature fix
- **Category**: storage / ingest performance
- **Status**: BLOCKED on upstream release
- **Blocker**: Published `greptimedb-ingester` ≤0.18.0 hard-enables tonic
  `tls-ring` (rustls). Contribution
  [greptimedb-ingester-rust#58](https://github.com/GreptimeTeam/greptimedb-ingester-rust/pull/58)
  is OPEN, not merged (recheck 2026-07-17T17:18Z). crates.io still 0.18.0
  (updated 2026-05-13); latest GitHub release tag remains `v0.18.0`. HTML
  state label `pullOpened`; title "feat: make rustls-backed TLS an opt-in
  feature"; head branch `donbeave:feat/opt-in-tls-feature`.

## Residual only

After the upstream blocker clears:

1. Wire trusted local / explicit external gRPC endpoints; ready-banner name.
2. Route `error_events`, `run_metric_points`, `metric_exemplars` through
   bounded typed row batches (no hot-path clone).
3. Real-engine parity vs SQL; one-cycle SQL compatibility switch only.
4. Remove SQL compatibility path after one released row-path cycle.

## Done Criteria

- [ ] Latest ingester graph contains no active rustls backend.
- [ ] Derived row writes use the typed row API by default.
- [ ] Row batches bounded by count and encoded size; SQL/row parity green.
- [ ] SQL compatibility switch and text-row path removed after one cycle.
- [ ] Default Rust, strict Clippy, nextest, storage integration gates pass.

## STOP / Remove When

STOP on any rustls activation, row semantic loss, or custom raw-signal table.
Delete this plan after row transport is green and the SQL switch is gone, or
operator permanently reaffirms SQL-only.
