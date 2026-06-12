# Live telemetry streaming: what is live, what is refresh, and why

Research date: 2026-06-12 (operator request: live traces/logs per service,
explicit live enable, "verify my concern that live must have fewer features").

## The operator's concern, verified

The concern: refresh mode reads pre-calculated, fully-indexed data from the
engine, so it can afford any filter and any aggregation; live mode sees rows
in flight and cannot run heavy aggregation. **Industry practice confirms
this split exactly:**

- **Datadog Live Tail** streams all ingested logs in near real time but
  restricts the query language: full-text search syntax is not available,
  filters are simple facet/attribute matchers, and the output is **sampled
  uniformly at random when throughput exceeds the stream's budget**. No
  aggregation or analytics run on the live feed — inspection only.
- **Grafana Loki `tail`** (`/loki/api/v1/tail`) accepts log stream selectors
  and line filters only; **metric queries are not supported on the tail
  path**, concurrent tail requests are capped per tenant, and the
  query-frontend famously does not even proxy tail websockets.
- **kubectl logs -f** — the UX bar the operator named — has effectively no
  server-side filters at all: pick the pod (service), stream, grep on the
  client.

Pattern across all three: **live = cheap per-row predicates evaluated on the
flowing record; anything that needs more than one row (histograms,
percentiles, counts, time travel, SQL) belongs to the stored/indexed query
path.**

## Parallax mapping

| Capability | Refresh / polling mode | Live mode (SSE tail) |
| --- | --- | --- |
| Service selector | yes | yes (string equality per row) |
| Severity floor (logs) | yes | yes (integer compare per row) |
| Body/name substring | yes (LIKE, later FULLTEXT) | yes (`contains` per row) |
| Trace / run scoping | yes | yes (id equality per row) |
| Min span duration (traces) | yes | yes (numeric compare per row) |
| Errors only (traces) | yes — whole trace (`errorOnly`) | per **span** only (`errors_only`) — a trace-level verdict needs all spans |
| Time range presets (1m…30d) | yes | **no** — a tail has no past; switch to refresh for history |
| Histogram / count series | yes (`logCountSeries`, date_bin) | **no** — aggregation |
| Trace aggregates (span count, whole-trace status) | yes (`traces` query) | **no** — live shows finished spans as they arrive |
| Raw SQL | yes (`sql`, read-only) | **no** |
| Sort options | newest-first or engine sort | arrival order only |

Two structural consequences worth naming:

1. **Live traces are a span feed, not a trace feed.** A trace "completes"
   asynchronously; OTLP exports spans when they end. Streaming finished spans
   (Datadog's Live Search does the same) is honest and immediate — each row
   links to the full trace view for the aggregate picture. The live duration
   filter is exactly the operator's "show me slow ones right now" knob.
2. **Backpressure = drop, not buffer.** The ingest worker publishes batches
   to a bounded broadcast channel; a lagging consumer skips missed batches
   and keeps tailing. That is tail semantics (kubectl behaves the same under
   scroll lock) and it protects ingest — the hot path never waits on a
   browser, and never clones batches unless someone is subscribed.

## Defaults

- **Live is never the default.** It holds a subscription open and makes the
  worker clone every batch; the default is a one-shot "Latest" load, refresh
  off. The user (or agent) explicitly opts into live, narrows with cheap
  filters, observes, then drops back to refresh mode for forensic work.
- Entering live mode disables the controls that have no live meaning (range,
  histogram, SQL) rather than silently ignoring them; the status line says
  which filters still apply.

## Transport (decided earlier, summarized)

SSE over WebSocket for one-way tails: EventSource auto-reconnects, plain
HTTP/2, no proxy/upgrade concerns on the loopback profile; render batching
~250 ms on the client. WebSocket buys nothing for a one-directional feed.

## Agent parity (the point of all of this)

The same vocabulary exists on every surface, so an agent can verify a fix the
same way a human eyeballs it:

- **API:** `traces(service, fromNanos, toNanos, minDurationMs, errorOnly,
  query, limit)` GraphQL query; `/v1/logs/stream` + `/v1/traces/stream` SSE
  with the per-row filter params.
- **CLI:** `parallax traces [--service] [--min-duration 500ms] [--errors]
  [--grep] [--since 15m]` for history; `parallax logs|traces --follow` for
  the live tail; `--follow --for 30s` watches a fixed window and prints the
  match count — the mechanical answer to "after my fix, does it still
  appear?" (zero matches = verified gone).

## Sources

- Datadog Live Tail docs — no full-text syntax in Live Tail; sampling under
  load: <https://docs.datadoghq.com/logs/explorer/live_tail/>
- Datadog Live Tail announcement (design intent: observe deploys, inspect
  individual logs): <https://www.datadoghq.com/blog/live-tail-log-management/>
- Grafana Loki HTTP API — `tail` takes selectors/line filters; metric queries
  unsupported on tail: <https://grafana.com/docs/loki/latest/reference/loki-http-api/>
- Loki query-frontend lacks websocket/tail support (grafana/loki#2878):
  <https://github.com/grafana/loki/issues/2878>
- Loki per-tenant concurrent tail limits:
  <https://grafana.com/docs/loki/latest/configure/>
- SSE-vs-WebSocket comparison set (transport decision, 2026-06-12):
  <https://websocket.org/comparisons/sse/>, <https://ably.com/blog/websockets-vs-sse>
