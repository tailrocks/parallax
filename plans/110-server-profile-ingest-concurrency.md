# Plan 110: Scale ingest concurrency from measured server saturation

> **Executor instructions**: Keep single-worker until a supported server
> profile proves worker saturation after batching/spool improvements. No
> speculative task pool.

## Status

- **Priority**: P2 when triggered
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 115 + reproducible saturation packet (099, 113 vocabulary)
- **Category**: V2 / ingest / performance
- **Status**: BLOCKED
- **Blocker**: Plan 115 live lab packet
  ([live-rehearsal-2026-07-17.md](../docs/research/validation/2026-07-plan-115-v2-server-profile/live-rehearsal-2026-07-17.md))
  records GraphQL micro-RPS and invocation wall times only — **not** a
  single-worker stage-isolation saturation packet. Plan 113 vocabulary exists
  but is not trigger evidence.

## Residual only (after trigger)

1. Reproduce saturation on the named profile; isolate stage costs.
2. Write ordering/idempotency/backpressure/ownership invariants; pick smallest
   measured fix (batch tune / pipeline / partition / bounded pool).
3. Implement behind internal composition; re-run same harness + failure
   injection; prove no hot-path clone.

## Done Criteria

- [ ] Trigger packet proves worker is the bottleneck on a supported profile.
- [ ] Material predeclared gain; ordering/durability/idempotency/bounds pass.
- [ ] No telemetry clone on hot path; truthful progress/drain/ready.

## STOP / Remove When

STOP if storage/network is the limiter or design weakens durability/ordering.
Delete after measured concurrency design ships, or product retains single-worker
for all supported profiles.
