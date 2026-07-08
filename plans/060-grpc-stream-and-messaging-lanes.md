# Plan 060: gRPC streaming + messaging rendering in trace detail — per-message timeline, deadline/cancel surfacing, capped event lists

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- ui/src/routes/traces.\$traceId.tsx ui/src/components/console ui/src/lib`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 058 (`traceEvents` resolver — the message timeline
  reads it); pairs with playground plan 049 (emits `rpc.message.*` events,
  DEADLINE_EXCEEDED, cancellation, batch links, lag). Renders empty-safe
  without 049.
- **Category**: direction (+ one render-safety fix)
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

Playground plan 049 makes streaming and messaging telemetry real — per-message
`rpc.message` span events with `rpc.message.type`/`rpc.message.id`, real
`DEADLINE_EXCEEDED` status, mid-stream errors, consumer lag — but Parallax
renders span events as a flat uncapped `<ul>` in the inspector, so a
100-message stream span means 100 identical cards (and a real DOM hazard),
and "the stream died at message 37" is invisible. The research brief calls
for a streaming explorer (per-message sent/received timeline, sizes,
mid-stream errors, cancellation) and messaging surfacing. This plan adds a
message-timeline section for streaming RPC spans, a deadline/cancel callout,
and caps the generic inspector event/link lists (the render-safety fix that
protects every high-event span, not just RPC).

## Current state

Verified at commit `ed5b10f`.

- Uncapped inspector events — `ui/src/routes/traces.$traceId.tsx:497-529`:

  ```tsx
  {events.length > 0 ? (
    <InspectorSection title={`Events (${events.length})`}>
      <ul className="space-y-2">
        {events.map((event, index) => ( ... <KeyValueList .../> ... ))}
      </ul>
  ```

  Every event renders a bordered card with a full `KeyValueList`. Links are
  likewise unbounded (`:547-563`). The page also `flatMap`s all spans'
  events/links for the summary strip (`:162-163`) — counts only, that part
  is fine.

- No `rpc.*` awareness: `rtk grep -rn '"rpc\.' ui/src` → no hits. Span kind
  chips exist (`ui/src/components/console/span-kind.tsx:22-40` —
  CLIENT/SERVER/PRODUCER/CONSUMER coloring) but nothing reads gRPC status or
  message events.

- `traceEvents` (plan 058) provides:
  `traceEvents(traceId, namePrefix: "rpc.message", limit) → { events {
  spanId spanName service name timeUnixNano attributes } truncated
  skippedSpans }`.

- Semconv the timeline keys on (brief's table + plan 049 scope):
  events named `rpc.message` (OTel convention: event name literally
  `message`, attributes `rpc.message.type` = SENT|RECEIVED,
  `rpc.message.id`, `rpc.message.compressed_size`) — **plan 049 records the
  exact emitted names; check one live trace or 049's tests before hardcoding
  and accept both `message` and `rpc.message` event names.** Span-level:
  `rpc.system="grpc"`, `rpc.grpc.status_code` (4 = DEADLINE_EXCEEDED,
  1 = CANCELLED), `messaging.system`, `messaging.batch.message_count`.

- Domain-section precedent: plan 059 establishes the pattern (pure builder
  in `ui/src/lib/`, section component in `console/`, empty-safe mount on the
  trace page). Follow it.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck/lint/test/build | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `ui/src/lib/rpc-trace.ts` (new — classify RPC/messaging spans; shape the
  message timeline from `traceEvents` rows)
- `ui/src/components/console/rpc-stream.tsx` (new — timeline section)
- `ui/src/routes/traces.$traceId.tsx` — mount section; cap the generic
  Events/Links inspector lists with a "show all" expander; add the
  deadline/cancel callout in the inspector error block
- Loader addition: fetch `traceEvents(namePrefix: "rpc")` alongside the
  existing trace query (one extra field in the same GraphQL document)
- Tests

**Out of scope** (do NOT touch):
- Backend — plan 058 provides the resolver; no schema changes here.
- Span-link causal edges / linked-traces UI — advisor-plans/028.
- Consumer-lag dashboards — plan 044's runtime/metric surfaces own metrics.
- The waterfall — plan 061.
- Virtualizing the inspector — the cap+expander is the V1; virtualization
  only if plan 040's dependency already landed and its util fits trivially.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Cap the generic inspector lists (safety first)

In `traces.$traceId.tsx`: events and links sections render at most
`INSPECTOR_LIST_CAP = 25` items, with a
"Show all N events" / "Show all N links" toggle button beneath (local
`useState`, resets on span change via `key={selectedSpan.spanId}` or an
effect). Same-named consecutive events beyond the cap are NOT summarized
here — that's the timeline's job; this is a dumb cap.

**Verify**: `bun run test` — component/page test: a 60-event span fixture
renders 25 + toggle; toggling shows 60.

### Step 2: `rpc-trace.ts` classifier + timeline shaper

```ts
export interface RpcStreamInfo {
  spanId: string
  system: string                  // rpc.system
  method: string                  // span name
  grpcStatusCode: number | null   // rpc.grpc.status_code
  outcome: "ok" | "deadline_exceeded" | "cancelled" | "error" | null
  messages: RpcMessage[]          // from traceEvents rows for this span
  truncated: boolean
}
export interface RpcMessage {
  id: number | null               // rpc.message.id
  type: "SENT" | "RECEIVED" | "unknown"
  timeUnixNano: string
  size: number | null             // rpc.message.compressed_size
}
export function buildRpcStreams(spans, events): RpcStreamInfo[]
export function messagingSummary(spans): { producer: number; consumer: number;
  batchMax: number } | null
```

Classify: a span is a "stream candidate" when `rpc.system` is present AND it
has ≥2 message events (unary calls stay in the plain inspector). Outcome map:
grpc status 4 → deadline_exceeded, 1 → cancelled, span ERROR otherwise →
error. Accept event names `message` and `rpc.message` (see Current state).

**Verify**: `bun run test` — fixtures: 5-message stream (3 SENT/2 RECEIVED)
ordered by time; deadline span maps outcome; unary (1 message event) excluded;
non-RPC trace → `[]`.

### Step 3: Timeline component + mounts

1. `rpc-stream.tsx`: per stream — header (method, system chip, outcome
   badge: rose for deadline/cancel/error), then a compact horizontal
   timeline: one dot per message positioned by time across the span window,
   direction by shape/color (SENT vs RECEIVED per the dataviz-consistent
   palette already used by `span-kind.tsx` variants), hover = id/size/time,
   an error/cancel marker at the span end when outcome ≠ ok. Under 8
   messages also list rows (id, type, size, time). `truncated` → muted
   "showing first N messages" line.
2. Mount on the trace page (below the GraphQL card slot, same empty-safe
   pattern): fetch `traceEvents(traceId, namePrefix: "rpc", limit: 500)` in
   the existing loader document; `buildRpcStreams(spans, events)`; render
   card "RPC streams" when non-empty.
3. Inspector callout: in the existing error block
   (`traces.$traceId.tsx:476-480`), when the selected span's
   `rpc.grpc.status_code` maps to deadline/cancel, append the human word —
   "DEADLINE_EXCEEDED (gRPC 4)" / "CANCELLED (gRPC 1)".
4. Messaging strip: when `messagingSummary` is non-null, one line in the
   summary strip area — "N producer / M consumer spans · max batch K"
   linking nothing yet (advisor-plans/028 owns link edges).

**Verify**: `bun run typecheck && bun run test && bun run build` clean;
live (049 landed): quote-stream trace shows dots + outcome; deadline
scenario shows the callout. Record which check ran.

## Test plan

- `rpc-trace.test.ts` — Step 2 fixtures.
- `rpc-stream.test.tsx` — render: dots count, outcome badge, truncated line.
- Page test for the Step 1 cap.
- Pattern: same harness as plan 059's tests (or the nearest existing
  co-located UI test).

## Done criteria

- [ ] `bun run typecheck && bun run lint && bun run test && bun run build` all exit 0
- [ ] Inspector events/links capped at 25 with working expander (test)
- [ ] Stream card renders for multi-message RPC spans only (test)
- [ ] Deadline/cancel callout appears for gRPC status 4/1 (test)
- [ ] `traceEvents` fetched with `namePrefix: "rpc"` in the trace loader
- [ ] `plans/README.md` status row updated

## STOP conditions

- Plan 058 not landed (no `traceEvents` in the schema) — this plan cannot
  proceed; report.
- Real 049 telemetry names message events differently than both accepted
  spellings — report the emitted shape (don't add a third guess).
- The timeline needs >1 new dependency — it shouldn't (plain SVG/divs);
  report before adding any.

## Maintenance notes

- Advisor-plans/028 (typed span links) will add producer→consumer causal
  edges near the messaging strip — keep the strip dumb until then.
- Plan 063's A19 stress trace is the cap's regression test in the flesh —
  after both land, open a stress trace and check the inspector stays
  responsive.
- Reviewer: the message-dot timeline must position by event time within the
  span window (clamped 0-100% like `positionPct` in `trace-tree.ts:76-108`),
  not by index — otherwise bursts lie.
