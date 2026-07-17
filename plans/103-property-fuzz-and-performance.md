# Plan 103: Focused property, fuzz, and performance residual gates

> **Executor instructions**: Do not invent ratchet thresholds before scheduled
> variance samples. No auto-refresh of baselines.

## Status

- **Priority**: P2
- **Effort**: S remaining
- **Risk**: MEDIUM
- **Depends on**: scheduled-measurement samples (ratchets only)
- **Category**: testing / fuzzing / performance
- **Status**: BLOCKED — fail-closed performance ratchets need measured variance
- **Blocker**: Nightly `scheduled-measurement.yml` uploads artifacts only;
  no durable multi-run variance model is committed yet (recheck
  2026-07-17T14:45Z UTC). Inventing thresholds is forbidden.
- **Evidence**:
  [`docs/research/testing/property-invariants.md`](../docs/research/testing/property-invariants.md)

## Landed (do not replay)

- Named Rust invariants: math/semconv/SQL, redaction/canonical-JSON/fingerprint
  fixpoints, OTLP normalize determinism proptest.
- Trace parent-graph traversal is cycle-safe and permutation-stable under a
  bounded seeded proptest; unattached identifiers are canonicalized.
- Worker late-retry no-replay is covered across every effect checkpoint and
  generated retry depths by the real live/storage/metadata effect oracle.
- Six `fuzz/` boundaries + PR `fuzz-bench` lane + nightly
  `scheduled-measurement.yml`; drift gate in `parallax-xtask`.
- Criterion benches + `bench-alloc/`; Step-6 advanced tools rejected for now.
- UI properties: live merge/identity/decoder (plan 147), route-search
  round-trips across features, Query-key identity
  (`ui/src/platform/query/tests/graphql-query-key.test.ts`).

## Residual only

1. **Ratchets**: adopt relative/allocation fail-closed thresholds only after
   enough scheduled-measurement samples model variance; no auto-refresh.
2. Optional: commit minimized crash corpora as fuzz finds them.

## Done Criteria

- [x] Residual Rust properties (trace trees, serialization, retry no-replay)
      have seeded coverage with oracles.
- [x] UI search/decoder/Query/live/state properties have bounded Bun coverage
      (owners: plans 133/147/148 + query-key identity tests).
- [ ] Ratchets use measured variance and fail without auto-refreshing baselines.

## STOP / Remove When

STOP if a target has no oracle, a threshold is copied, or measurement clones
hot-path telemetry. Delete this plan when residual ratchets are enforced from
measured samples, or operator permanently rejects fail-closed performance
ratchets for this program phase.
