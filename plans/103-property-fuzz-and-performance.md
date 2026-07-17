# Plan 103: Focused property, fuzz, and performance residual gates

> **Executor instructions**: Do not invent ratchet thresholds before scheduled
> variance samples. No auto-refresh of baselines.

## Status

- **Priority**: P2
- **Effort**: S remaining
- **Risk**: MEDIUM
- **Depends on**: ≥3 independent `scheduled-measurement.yml` jobs (ratchets)
- **Category**: testing / fuzzing / performance
- **Status**: BLOCKED — fail-closed performance ratchets need multi-run variance
- **Blocker**: Only **one** durable scheduled sample is committed
  ([run 29582532812](https://github.com/tailrocks/parallax/actions/runs/29582532812),
  packet
  [`docs/research/testing/measurement/2026-07-17-run-29582532812/`](../docs/research/testing/measurement/2026-07-17-run-29582532812/README.md)).
  Within-job criterion span is <5%; inventing relative ceilings from n=1 is
  still forbidden. Second dispatch `29589577179` started 2026-07-17 — adopt
  thresholds only after ≥3 independent ubuntu jobs.
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
- **2026-07-17 fuzz defects fixed from scheduled campaign:**
  - `sanitize_text` control-strip-before-detectors (crash `C@\u{6}2.srea@T`)
  - `canonical_json` stable number fixpoint (crash `9E294`)
  - Minimized corpora under `fuzz/corpus/{redaction_text,bundle_envelope_json}/`
- First scheduled bench/alloc samples committed under
  `docs/research/testing/measurement/2026-07-17-run-29582532812/`.

## Residual only

1. **Ratchets**: adopt relative/allocation fail-closed thresholds only after
   ≥3 scheduled-measurement jobs model variance; no auto-refresh.
2. Optional: more minimized crash corpora as fuzz finds them.

## Done Criteria

- [x] Residual Rust properties (trace trees, serialization, retry no-replay)
      have seeded coverage with oracles.
- [x] UI search/decoder/Query/live/state properties have bounded Bun coverage
      (owners: plans 133/147/148 + query-key identity tests).
- [ ] Ratchets use measured multi-run variance and fail without auto-refreshing
      baselines.

## STOP / Remove When

STOP if a target has no oracle, a threshold is copied, or measurement clones
hot-path telemetry. Delete this plan when residual ratchets are enforced from
measured samples (n≥3 scheduled jobs), or operator permanently rejects
fail-closed performance ratchets for this program phase.
