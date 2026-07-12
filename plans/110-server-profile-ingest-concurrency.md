# Plan 110: Scale ingest concurrency from measured server saturation

> **Executor instructions**: Keep the current single-worker design until a
> supported server profile demonstrates saturation after existing batching and
> spool improvements. Preserve per-signal ordering, bounded memory, durability,
> backpressure, and progress visibility; do not add a speculative task pool.

## Status

- **Priority**: P2 when triggered
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 099, 113, 115; measured saturation
- **Category**: V2 / ingest / performance
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: BLOCKED
- **Blocker**: No supported server profile plus reproducible evidence shows the
  current single worker is the bottleneck.

## Trigger Evidence Required

- A named supported CPU/memory/storage/network profile and workload envelope.
- Four-signal input mix, payload distributions, cardinality, burst/steady rates,
  Greptime/Turso mode, spool state, and current versions.
- CPU, allocation, queue depth/age, spool lag, storage latency, throughput,
  error/drop/retry rates, shutdown drain, and ordering/idempotency evidence.
- Proof that downstream storage/network is not the primary limiter and that
  current batching/configuration cannot meet the target.

## Scope

In scope after the trigger exists:

- Reproducible single-worker saturation characterization on a supported
  profile, invariant-driven design comparison, and the smallest measured fix.
- Bounded per-signal/stage concurrency, failure injection, ordering,
  idempotency, backpressure, allocation, and shutdown evidence.

Out of scope:

- A speculative pool without trigger measurements.
- Greptime table partition tuning, storage-engine substitution, unbounded
  queues, or weakened durability/ordering.

## Steps After Trigger

1. Reproduce saturation on the supported profile with a deterministic harness.
   Profile worker stages and isolate decode, normalize, issue derivation,
   storage writes, metadata, broadcast, and spool acknowledgment costs.
2. Write invariants: bounded queue/memory, per-signal and key ordering where
   required, at-least-once/idempotency boundary, no early spool ack, backpressure,
   fair signal progress, shutdown drain, and no hot-path telemetry clone.
3. Compare the smallest options: tune batch/flush limits, pipeline independent
   stages, partition by stable signal/key, or bounded worker pool. Model skew,
   head-of-line blocking, duplicate side effects, and downstream connection
   limits. Select only an option that materially improves the measured limit.
4. Implement behind internal composition, retaining one canonical queue and
   explicit ownership transfer. Use plan 099's retry/idempotency contract.
   Keep long-running startup/drain progress and ready banners truthful.
5. Re-run the same harness plus overload, skew, slow storage, partial failure,
   restart/replay, and shutdown. Compare throughput/tail latency/resource use
   and verify all invariants.

## Test Plan

- Deterministic supported-profile before/after benchmark packet.
- Queue/order/fairness/backpressure/ownership model tests.
- Failure injection at every side-effect and acknowledgment boundary.
- Burst, sustained, high-cardinality skew, slow Greptime/Turso, restart, and
  shutdown drain scenarios.
- Allocation/copy evidence proving zero-copy ingest ownership remains intact.
- Four-build protocol for any Greptime-vs-ClickHouse performance claim.

## Done Criteria

- [ ] Trigger evidence identifies the worker as the supported-profile bottleneck.
- [ ] Invariants and selected design are reviewed before implementation.
- [ ] The same workload shows a material predeclared throughput/latency gain.
- [ ] Ordering, durability, idempotency, fairness, and bounded memory pass failures.
- [ ] No telemetry clone is added to the hot path.
- [ ] Startup, overload, drain, and ready progress remain truthful.
- [ ] Downstream engines remain within supported connection/load limits.

## STOP Conditions

- No supported profile or reproducible saturation packet exists.
- Storage/network, not the worker, is the measured bottleneck.
- The design needs unbounded queues, cloned telemetry, early acknowledgment, or
  weaker ordering/idempotency semantics.
- Improvement is within measurement variance or shifts failure downstream.

## Remove When

Delete this plan and index row when a qualifying trigger leads to a measured,
verified concurrency design, or the product explicitly retains single-worker
operation for all supported profiles.
