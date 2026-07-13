# Plan 113 ingest backpressure observability validation

Validation date: 2026-07-13
Implementation: `a6bf431`, `918ccf2`, `d849ba3`
Baseline and instrument contract:
[2026-07-13-plan-113-ingest-health-baseline.md](2026-07-13-plan-113-ingest-health-baseline.md)

## Result

Parallax now exports bounded OpenTelemetry metrics for the three per-signal
queues, worker retry/drop/drain outcomes, and durable spool inventory/reclaim.
Both OTLP transports reserve bounded queue capacity before recording acceptance
and move the decoded request and raw `Bytes` into the worker unchanged. The
instrumentation adds only an enqueue `Instant` and observation flag to that
owned item; it does not clone telemetry.

The `/health` endpoint returns `503` with the exact full queues while any
observed signal queue is saturated and returns the existing exact `200 ok`
response after recovery. Queue depth, capacity, high-water, retry, and drop
mirrors make overload fixtures compare exported state with the internal state
that owns each transition.

## Export and feedback-loop boundary

`SelfTelemetry` now owns an OTLP metric exporter and flushes it during shutdown.
The standard OTLP metrics path feeds GreptimeDB's native per-metric tables; it
does not introduce a raw-signal table or Parallax metric schema. The repository's
managed-engine inventory acceptance test remains the executable guard that OTLP
metrics create native per-metric tables rather than legacy/custom metric tables.

Every health series uses only fixed `signal=traces|logs|metrics` attributes;
enqueue outcomes add only `outcome=accepted|unavailable`, and drain adds only
`outcome=completed|timeout`. No tenant, service, trace, route, batch, or error
text becomes a label.

Metric requests carrying the self-telemetry resource
`service.name=parallax` remain durably stored but bypass queue, retry, and drop
observation. Therefore an exported queue metric cannot generate another queue
metric when a collector sends Parallax telemetry back to Parallax. A fixture
asserts self requests are excluded and ordinary workload metric requests remain
observed.

## State and failure evidence

| Boundary | Enforced evidence |
| --- | --- |
| Queue overload/recovery | Capacity-two state fixture reaches `depth=2`, reports `traces=2/2`, drains to zero, preserves high-water two, and clears degradation. A live capacity-one HTTP fixture blocks trace storage, fills the real queue, observes exact `503 degraded: ingest queue full (traces=1/1)`, releases storage, and observes exact `200 ok`. |
| Queue residence | A preallocated per-signal timestamp ring supplies exact oldest age; dequeue records actual residence time. |
| Slow storage isolation | Existing gated traces storage fixture proves logs continue through their independent signal worker. |
| Retry exhaustion | The real worker loop is injected with four consecutive failures and reports exactly three retries, one terminal drop, and depth zero. |
| Spool inventory | Per-signal fixture proves byte/oldest-age scans do not mix logs and traces; the reaper reports reclaimed bytes. |
| Shutdown | The existing five-second bounded worker drain now records completed/timeout duration before the established operator outcome is narrated. |
| Hot-path ownership/noise | Transport and worker source move one owned item; no instrumentation payload clone or per-routine-batch log was added. Existing bounded retry warnings and terminal-drop errors remain exceptional. |

The spool is a retained diagnostic/crash-forensics record, not a replay queue,
so there is no replay cursor whose lag could be reported. Its byte count and
oldest retained age expose the applicable backlog/retention pressure without
inventing replay semantics.

## Verification

- `cargo fmt --all -- --check`
- `cargo nextest run -p parallax-server -p parallax-spool --all-targets --profile ci --no-tests=fail` — 40 passed, 6 skipped
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo xtask policy`
- `cargo xtask dependencies --all`
- `cargo xtask facade check`
- `cargo xtask docs links` — 278 tracked Markdown files
- GitHub CI for `a6bf431`: [run 29224475414](https://github.com/tailrocks/parallax/actions/runs/29224475414) — passed
- GitHub CI for `d849ba3`: [run 29225015755](https://github.com/tailrocks/parallax/actions/runs/29225015755) — passed, including native macOS

## Plan 110 handoff

This packet supplies stable signal names, bounded labels, internal-state
fixtures, and failure semantics for a future supported-profile saturation
harness. It does not claim saturation. Plan 110 remains blocked until Plan 115
defines an approved profile and measurements on it isolate the single worker as
the bottleneck rather than GreptimeDB, Turso, storage, or network.

No research prompt changed: this work executes an existing implementation plan
without changing research direction, evaluation criteria, or target systems.
