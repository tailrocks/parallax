# Plan 103: Focused property, fuzz, and performance residual gates

> **Executor instructions**: Do not invent ratchet thresholds before scheduled
> variance samples. UI property work stays gated on plans 133/147/148 owners.

## Status

- **Priority**: P2
- **Effort**: M remaining
- **Risk**: MEDIUM
- **Depends on**: 133, 147, 148 (UI properties); scheduled-measurement samples
  (ratchets)
- **Category**: testing / fuzzing / performance
- **Status**: IN PROGRESS — Rust lanes largely landed; residual below
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

## Residual only

1. ~~**Rust properties**~~: flaky-state record JSON fixpoint
   (`parallax-model` serde_contract) and plan-099 late-retry no-replay
   properties landed.
2. **UI properties** (partial 147 owners): live merge/identity/decoder tests
   landed with 147; still open: route-search round trips; Query-key identity;
   feature state machines (waits remaining 147/148).
3. **Ratchets**: adopt relative/allocation fail-closed thresholds only after
   enough scheduled-measurement samples model variance; no auto-refresh.
4. Optional: commit minimized crash corpora as fuzz finds them.

## Done Criteria

- [x] Residual Rust properties (trace trees, serialization, retry no-replay)
      have seeded coverage with oracles.
- [ ] UI search/decoder/Query/live/state properties have bounded Bun coverage
      once 133/147/148 establish owners.
- [ ] Ratchets use measured variance and fail without auto-refreshing baselines.

## STOP / Remove When

STOP if a target has no oracle, a threshold is copied, or measurement clones
hot-path telemetry. Delete this plan when residual gates are enforced and
stable.
