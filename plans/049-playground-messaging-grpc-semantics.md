# Plan 049: Messaging + gRPC semantics — batch fan-in links, lag metric, orphan consumer, real deadlines, per-attempt spans, stream events

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- services/orders services/checkout services/pricing scenarios`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 036 (propagation/status helpers)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

Two trace shapes Parallax must visualize are missing their data. Messaging
(brief domain D): the orders service has a real producer→consumer span link
and poison/dead-letter, but no **batch consumer linking many producers**
(the fan-in edge advisor-plans/028's linked-traces UI renders), no consumer
**lag metric** (lag is a sleep, invisible to dashboards), and no **orphan
consumer** (the deliberate missing-link evidence gap for advisor-plans/032).
gRPC (domain C): the "timeout" is a client-side `tokio::time::timeout` (no
`DEADLINE_EXCEEDED` status crosses the wire), retries are span events rather
than per-attempt child spans, there is no cancellation case, and streams
carry no per-message `rpc.message.*` events — so the streaming-explorer
story has nothing to show.

## Current state

Verified at playground commit `ed1f975`.

- `services/orders/src/main.rs` — the whole async branch:
  - real link: `consume` adds
    `tracing::Span::current().add_link(producer_cx.span().span_context().clone())`
    (`:59-62`);
  - lag is only a sleep (`:63-65`, `// B7 consumer lag`);
  - poison → 3 attempts → dead-letter log (`:80-92`);
  - transport is an in-process `mpsc::channel` (`:77`) — noted in the
    header: "An in-process channel stands in for the broker here; the full
    version uses the compose `broker` (Kafka)." One message consumed at a
    time (`:79`), no batching.
- gRPC client, checkout (`services/checkout/src/main.rs`):
  - retry: `quote_with_retry` (`:192-217`) loops attempts inside ONE span
    (`#[tracing::instrument]` on the function), each failure a
    `tracing::warn!` event; per-attempt deadline is
    `tokio::time::timeout(deadline, quote(...))` (`:204`) — tonic never sees
    a deadline, the server keeps working, no `DEADLINE_EXCEEDED` mapping;
  - stream consumption (`:253-277`) counts items with zero events.
- gRPC server, pricing (`services/pricing/src/main.rs`): `quote` (`:18-33`),
  `quote_stream` streams `quantity` items immediately (`:38-58`) — no
  per-message events, no delay knob, no cancellation window, no error
  mid-stream.
- Java/Kafka side (context, not in scope): fulfillment's real
  Kafka round-trip is agent-instrumented; batch/orphan cases don't exist
  there either — this plan builds them on the Rust side where the span
  plumbing is hand-rolled and cheap.
- Semconv targets (brief): `messaging.batch.message_count`, lag gauge;
  `rpc.grpc.status_code`, `rpc.message.type/id` events.

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Build | `rtk cargo build` | exit 0 |
| Lint | `rtk cargo clippy --all-targets -- -D warnings` | exit 0 |
| Scripts | `bash -n scenarios/<new>.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `services/orders/src/main.rs` (batch consumer, lag gauge, orphan knob)
- `services/checkout/src/main.rs` (real tonic deadline; per-attempt spans;
  cancellation knob; stream-consume events)
- `services/pricing/src/main.rs` (stream pacing/error knobs; per-message
  events; deadline-honoring slow knob)
- `scenarios/`: `a20-batch-fanin.sh`, `b21-orphan-consumer.sh`,
  `a7b-grpc-stream.sh`, `b3b-grpc-deadline.sh` + catalog rows
- `libs/playground-telemetry` only if a helper is factored

**Out of scope**:
- Kafka-side batch consumption in fulfillment (Java agent semantics —
  follow-up; note in report).
- Replacing the in-process channel with the broker for orders (the header
  documents the stand-in; keep it — the span semantics are what's compared).
- Parallax UI (advisor-plans/028/031/032 consume these).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Batch consumer with multi-producer links

In orders: add `?batch=1` to `publish` params (marks messages for the batch
path) and a second consumer loop mode: drain up to 10 queued messages (or
50ms window), then run ONE `consume_batch` span (`otel.kind=consumer`) that
`add_link`s EVERY drained message's producer context and sets
`messaging.batch.message_count = n`. Scenario `a20-batch-fanin.sh`: POST ~8
orders with `?batch=1` rapidly → one consumer span linking 8 producer spans.
"Check in Parallax: trace detail of the consumer — 8 links (advisor-plan
028's linked-traces cards); linked producers each in their own trace."

**Verify**: build + clippy; live run recorded (consumer span with N links).

### Step 2: Lag metric + orphan consumer

1. Lag gauge: measure real queue depth — wrap the channel with an
   `Arc<AtomicI64>` incremented on send, decremented on receive; a 5s gauge
   task emits `messaging.queue.depth` (custom name, documented — no stable
   OTel lag semconv for this shape; comment it) plus per-message
   `messaging.delivery.lag_ms` span attribute = now − enqueue time (add an
   `enqueued_at` field to `Msg`).
2. Orphan consumer: `?orphan=1` on publish → the message carries an EMPTY
   producer context (`Context::new()`), so the consumer span has **no link
   and no parent** — the deliberate broken-causality case. Consumer also
   sets `messaging.orphan=true` attr for findability.
3. Scenario `b21-orphan-consumer.sh`: normal order, then orphan order —
   "Check in Parallax: normal consumer shows the link; orphan consumer is a
   root span with no link — an evidence gap (advisor-plan 032 will flag
   it)."

**Verify**: build + clippy; live: queue-depth gauge visible under lag
(`?lag_ms=2000` burst); orphan trace confirmed linkless (recorded).

### Step 3: Real gRPC deadline + per-attempt spans + cancellation

In checkout:
1. Replace `tokio::time::timeout` (`:204`) with a tonic request deadline:
   `request.set_timeout(Duration::from_millis(timeout_ms))` (tonic 0.14 —
   verify the API name; it propagates `grpc-timeout` so the SERVER sees the
   deadline) and map the resulting status: on
   `Code::DeadlineExceeded`, `mark_span_error("deadline_exceeded")` and
   record `rpc.grpc.status_code = 4`.
2. Wrap each attempt in `quote_with_retry` in its own child span
   (`#[tracing::instrument]` on an inner `attempt` fn or a manual
   `info_span!("pricing.attempt", attempt)`) so retries render as sibling
   spans, keeping the per-attempt warn events.
3. Cancellation knob: `?cancel_ms=<n>` on `/quote-stream` — drop the stream
   (client-side) after n ms mid-stream; server side observes the cancel.
4. In pricing: add `?delay_ms` handling (via `QuoteRequest` — add a field to
   the proto ONLY if the proto contract dir allows additive fields; else use
   gRPC metadata read server-side) so a slow server + client deadline
   actually produces `DEADLINE_EXCEEDED` on the wire. Read `proto/` first;
   additive proto field + regenerate is acceptable if the build wires it
   (`tonic-prost-build`).

**Verify**: build + clippy; `b3b-grpc-deadline.sh` live: a
deadline-exceeded trace shows client span ERROR with
`rpc.grpc.status_code=4` and per-attempt sibling spans (recorded).

### Step 4: Per-message stream events

pricing `quote_stream`: emit a span event per sent item
(`rpc.message` with `rpc.message.type="SENT"`, `rpc.message.id=i`), pace
items ~50ms apart (so the stream span visibly spans time), and add
`?fail_at=<i>` → mid-stream `Err(Status::internal(...))`. checkout's
consumer loop (`:253-277`): event per received item
(`rpc.message.type="RECEIVED"`), and surface the mid-stream error via
`mark_span_error("stream_failed")`. Scenario `a7b-grpc-stream.sh`: clean
stream of 6 + `fail_at=4` run — "Check in Parallax: stream span with
per-message events; failed run shows 3 RECEIVED + ERROR."

**Verify**: build + clippy; live run recorded (events visible on the span in
trace detail — Parallax already renders span events).

## Test plan

- Rust unit: batch-drain logic (drain caps at 10/50ms) and the atomic
  queue-depth counter — pure-logic tests in orders (has none today; add a
  `#[cfg(test)] mod tests`).
- The four scenarios' recorded runs are the acceptance evidence; register
  all in `scenarios/run.sh` + README (plan 037 format).

## Done criteria

- [ ] Batch consumer span carries N links + `messaging.batch.message_count`
      (recorded)
- [ ] `messaging.queue.depth` gauge + `messaging.delivery.lag_ms` attr
      emitted; orphan case produces a linkless consumer root (recorded)
- [ ] Deadline case yields wire-level `DEADLINE_EXCEEDED` + ERROR client
      span; retries are per-attempt sibling spans (recorded)
- [ ] Stream spans carry SENT/RECEIVED events; mid-stream failure recorded
- [ ] 4 scenarios in the catalog; `rtk cargo build` + clippy zero warnings
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- tonic 0.14's request-deadline API differs from `set_timeout` (renamed) —
  find the current API; if deadlines genuinely can't propagate in this
  version, report before faking it client-side again.
- Proto changes ripple into the Java payment service build (shared proto) —
  additive field should be safe; if the Java codegen breaks, STOP and
  report.
- The batch consumer's link count renders as JSON-only in current Parallax
  (advisor-plan 028 not landed) — that's expected; do NOT block on it, just
  record links exist in the raw span.

## Maintenance notes

- Consumers: advisor-plans/028 (linked-traces UI), 031 (edge model), 032
  (orphan = evidence gap) — these scenarios are their demo data.
- Follow-up deferred: Kafka-side batch consumption + lag on the Java tier;
  broker-backed orders transport.
- Reviewer: orphan must be TRULY contextless (no accidental parent from the
  handler span); per-message event volume is bounded (streams ≤ ~10 items in
  scenarios).
