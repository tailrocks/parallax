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
- Six `fuzz/` boundaries + PR `fuzz-bench` lane + nightly
  `scheduled-measurement.yml`; drift gate in `parallax-xtask`.
- Criterion benches + `bench-alloc/`; Step-6 advanced tools rejected for now.

## Residual only

1. **Rust properties still open**: trace parent/child trees; serialization
   compatibility contracts; plan-099 late-retry no-replay properties.
2. **UI properties** (blocked until 133/147/148 owners exist): route-search
   round trips; GraphQL/SSE runtime decoders; Query-key identity; live
   ordering/dedup; feature state machines.
3. **Ratchets**: adopt relative/allocation fail-closed thresholds only after
   enough scheduled-measurement samples model variance; no auto-refresh.
4. Optional: commit minimized crash corpora as fuzz finds them.

## Done Criteria

- [ ] Residual Rust properties (trace trees, serialization, retry no-replay)
      have seeded coverage with oracles.
- [ ] UI search/decoder/Query/live/state properties have bounded Bun coverage
      once 133/147/148 establish owners.
- [ ] Ratchets use measured variance and fail without auto-refreshing baselines.

## STOP / Remove When

STOP if a target has no oracle, a threshold is copied, or measurement clones
hot-path telemetry. Delete this plan when residual gates are enforced and
stable.
