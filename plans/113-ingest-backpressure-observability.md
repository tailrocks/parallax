# Plan 113: Make ingest backpressure and retry health observable

> **Executor instructions**: Instrument existing bounded queues and spool/retry
> contracts without logging per batch, cloning telemetry, or creating a
> self-export loop. Emit bounded-cardinality OTel metrics into GreptimeDB native
> metric tables and narrate only meaningful long-running state transitions.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 095, 099
- **Category**: ingest / operability / performance evidence
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: IN PROGRESS

## Why

Per-signal workers and bounded queues landed, but exporter-visible queue age,
spool lag, backpressure, retry, and drop health remain incomplete. Plan 110
requires these signals to prove server-profile saturation rather than guessing.

## Scope

- Queue depth/capacity/high-water, oldest-item age, enqueue wait/rejection,
  spool bytes/oldest age/replay lag, retries, terminal drops, and drain time.
- Stable low-cardinality signal/stage/outcome labels and explicit units.
- Self-telemetry loop prevention and quiet storage ingest paths.
- Doctor/health and CI fixtures for overload/recovery.

Out of scope:

- A new worker pool, per-tenant/cardinality labels, per-batch INFO logs, custom
  raw metric tables, or relaxing backpressure/durability.

## Steps

1. Specify each instrument, unit, type, labels, update owner, and invariant.
   Baseline current normal/overload/replay/drain behavior before adding code.
2. Add counters/gauges/histograms at queue/spool/retry/ack boundaries with no
   telemetry-batch clone. Ensure the export filter cannot feed storage ingest
   diagnostics back into the same pipeline.
3. Surface actionable degraded/recovered states in health/doctor output. Keep
   routine batches log-quiet; long waits and drain/start transitions follow the
   repository progress-visibility contract.
4. Inject full queue, slow Greptime/Turso, replay, retry exhaustion, shutdown,
   and recovery. Assert metrics match actual state and return to baseline.
5. Feed the stable measurement packet into plan 110's trigger harness.

## Test Plan

- Instrument registration/name/unit/label-cardinality fixtures.
- Queue/spool/failure-injection metric-value assertions.
- Self-export loop and log-volume negative tests.
- Doctor/health degraded/recovered snapshots.
- Allocation/copy proof on representative ingest batches.

## Done Criteria

- [ ] Queue, spool, retry, drop, and drain health have specified instruments.
- [ ] Labels are bounded and native metric tables receive the signals.
- [ ] Overload/recovery tests match real internal state.
- [ ] Storage self-telemetry cannot create an ingest loop.
- [ ] No per-batch log noise or hot-path telemetry clone is introduced.
- [ ] Plan 110 can consume a reproducible saturation evidence packet.

## STOP Conditions

- A metric needs unbounded IDs/attributes or a custom raw table.
- Instrumentation changes acknowledgment, ordering, or backpressure behavior.
- Self-export cannot be isolated from storage ingest.
- Measurement overhead is material and cannot be reduced.

## Remove When

Delete this plan and index row when bounded ingest/backpressure telemetry and
overload/recovery evidence are enforced and usable by supported-profile tests.
