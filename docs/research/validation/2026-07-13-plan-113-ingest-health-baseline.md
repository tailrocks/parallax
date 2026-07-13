# Plan 113 ingest health baseline and instrument contract

Validation date: 2026-07-13
Baseline: `4bff0f6`

## Current behavior

Parallax owns three independent bounded Tokio MPSC queues, one each for traces,
logs, and metrics. Capacity is `[limits].ingest_queue_batches` per signal
(default 256). Both OTLP transports decode once, append the raw protobuf to the
durable spool, await queue capacity, then acknowledge. Each signal has one
worker and preserves its FIFO; the worker retries a staged transaction three
times at 100 ms, 500 ms, and 2 s before a terminal drop. Graceful shutdown stops
listeners and allows at most five seconds for all workers to drain.

The spool has per-signal active and rotated raw-frame segments, a 64 MiB segment
default, 512 MiB total default, 72-hour default retention, and a ten-minute
reaper. `doctor` reports bytes, request counts, and rotated segments, but the
server health endpoint currently returns only `ok`. Self-telemetry exports
traces and logs, with ingest/transport targets filtered to prevent a feedback
loop; it has no metrics provider.

## Stable instrument contract

All instruments use only `signal=traces|logs|metrics` and, where named,
`outcome=accepted|unavailable|retry|dropped` or a fixed stage enum. No tenant,
service, trace, batch, route, error text, or other unbounded value is permitted.

| Instrument | Type/unit | Update owner and invariant |
| --- | --- | --- |
| `parallax.ingest.queue.depth` | gauge `{batch}` | transport enqueue + worker dequeue; exact current depth |
| `parallax.ingest.queue.capacity` | gauge `{batch}` | initialization; immutable per signal |
| `parallax.ingest.queue.high_water` | gauge `{batch}` | enqueue; monotonic maximum until process restart |
| `parallax.ingest.queue.oldest_age` | gauge `s` | preallocated timestamp ring; zero iff empty |
| `parallax.ingest.queue.age` | histogram `s` | worker dequeue; actual residence time for every observed batch |
| `parallax.ingest.enqueue.wait` | histogram `s` | transport; one sample for every capacity wait |
| `parallax.ingest.enqueue.outcomes` | counter `{batch}` | transport; exactly one accepted/unavailable outcome |
| `parallax.ingest.worker.retries` | counter `{retry}` | worker retry boundary, fixed signal label |
| `parallax.ingest.worker.drops` | counter `{batch}` | terminal exhaustion only |
| `parallax.ingest.worker.drain` | histogram `s` | graceful worker drain outcome |
| `parallax.ingest.spool.bytes` | gauge `By` | spool inventory, active plus rotated per signal |
| `parallax.ingest.spool.oldest_age` | gauge `s` | oldest retained segment per signal |
| `parallax.ingest.spool.reclaimed` | counter `By` | reaper/prune reclaim boundary |

The timestamp ring is allocated once at configured capacity. Every accepted item
carries only a monotonic enqueue instant and a self-metric observation flag
beside its already owned decoded request/raw `Bytes`; no telemetry payload is
cloned. The worker removes the oldest timestamp and records age before
processing.

## Loop and noise contract

The metrics provider exports OTLP to the same opt-in self-telemetry endpoint as
traces/logs, so normal Rotel configuration still reaches GreptimeDB native
per-metric tables. A returned metric request whose resource identifies
`service.name=parallax` is self-export traffic: it is still stored as native
telemetry, but queue/spool health instrumentation does not observe that request.
This prevents `metric export → Parallax ingest → queue metric → metric export`
recursion. Exact fixtures must prove both the exclusion and ordinary workload
metrics observation.

Normal enqueue/dequeue metric updates do not log. Existing bounded worker retry
warnings and terminal-drop errors remain the exceptional paths; no routine batch
log was added. Error text never becomes a metric label.

No research prompt changed: this packet makes an existing implementation plan
executable and does not change product research direction or evaluation
criteria.
