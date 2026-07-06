# Plan 029: Add a `story(anchor)` resolver and a Story timeline tab that reprojects a trace/run into ordered, layered beats

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-core/src/bundle.rs ui/src/routes/traces.\$traceId.tsx ui/src/routes/runs.\$runId.tsx ui/src/components/nav.ts`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (reuses the existing bundle-input assembly path)
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The research brief's signature surface is the **Story**: a chronological,
human-readable sequence of what happened, grouped by layer (browser →
service → span/log/event), so a developer or agent reads the execution as a
narrative instead of reconstructing it from a waterfall. The audit found this
is disproportionately cheap here because the `bundle` resolver already
assembles exactly the right inputs (trace spans + span events + correlated
logs + metric windows + issues) into one artifact — `story` is a *re-projection
of the same inputs into ordered timeline rows*. This plan adds a deterministic
`story` resolver over those inputs and a Story tab on trace and run detail.

## Current state

- `crates/parallax-core/src/bundle.rs:253-263` — `BundleInputs { anchor,
  events, trace_spans, trace_logs, metric_windows }` is the assembled input
  set; `assemble` (bundle.rs:278) is pure and deterministic over it.
- `crates/parallax-api/src/lib.rs:1435-1596` — the `bundle` resolver fetches
  these inputs from storage (spans_by_trace / spans_by_run / logs_by_trace /
  logs_by_run / metric windows) for an anchor of issue|run|trace. This fetch
  logic is the template for `story`.
- Span events are available on each `SpanRow` (`events` JSON;
  `lib.rs:261`), and typed `SpanLink` may exist if plan 028 landed.
- UI trace detail `ui/src/routes/traces.$traceId.tsx` already fetches
  `trace { spans { … events } }` + `logsByTrace`; run detail
  `ui/src/routes/runs.$runId.tsx` fetches `tracesByRun`/`logsByRun`.
- Nav config: `ui/src/components/nav.ts`; shell tabs pattern in
  `ui/src/components/parallax-shell.tsx`. UI conventions per `ui/AGENTS.md`.
- Time nanos are strings end-to-end.

## Commands you will need

(Same table as plan 028 — Rust fmt/clippy/nextest at repo root; UI
typecheck/lint/test/build from `ui/`.)

## Scope

**In scope**:
- `crates/parallax-api/src/lib.rs` (a `StoryBeat` object + `story` resolver)
- optionally `crates/parallax-core/src/` (a pure `story` projection function
  if you extract the ordering logic — recommended for testability)
- `ui/src/lib/api.ts` (types + query)
- `ui/src/routes/traces.$traceId.tsx` and `ui/src/routes/runs.$runId.tsx`
  (Story tab), and a shared `ui/src/components/console/story-timeline.tsx`
- test files

**Out of scope**:
- Any LLM/prose summarization — the story is deterministic data projection
  only (design principle 6 in the brief). No `gen_ai` calls.
- Normalized/materialized `story_events` tables — read-time projection only.
- Agent/TUI event lanes — those need telemetry that the playground doesn't
  emit yet (plan 034). Model only spans, span events, and logs here.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Define the beat projection as a pure function

Add a pure function (in `parallax-core`, e.g. a new `story.rs`, so it is unit
testable without a server) that takes the same inputs as a bundle
(`trace_spans: &[SpanRow]`, `trace_logs: &[LogRow]`, optional
`metric_windows`) and returns an ordered `Vec<StoryBeat>` where a beat is:

```rust
pub struct StoryBeat {
    pub ts_nanos: u128,
    pub lane: String,      // service.name (or "browser"/"cli" from resource attrs)
    pub kind: String,      // "span.start" | "span.end" | "event" | "log" | "error"
    pub title: String,     // low-cardinality: span name, event name, or severity+first line
    pub trace_id: String,
    pub span_id: Option<String>,
    pub severity: Option<String>,
    pub duration_ns: Option<u128>,
}
```

Ordering (deterministic, per the brief's story-assembly algorithm): sort by
`ts_nanos`, then by parent/child causality to avoid clock-skew reordering
(fall back to span start order within equal timestamps). Group/lane = the
emitting service. Prefer span-event names and log severities as titles; keep
them low-cardinality (do not inline raw user text). Redact nothing here — the
story is a UI surface, not the exported bundle; but do **not** include full
log bodies verbatim if they are large (truncate to a first line for the beat
title, keep the row linkable to the full log).

Keep it deterministic: no `Date::now`, no rng.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Add the `story(traceId | runId)` resolver

In `lib.rs`, add a `StoryBeat` GraphQL object and a `story` resolver that
accepts an anchor (`traceId: Option<String>, runId: Option<String>`, exactly
one — mirror the `bundle` resolver's exactly-one enforcement at
`lib.rs:1444-1449`). Fetch the same inputs the `bundle` resolver fetches for
that anchor kind, call the Step 1 projection, return `[StoryBeat!]!`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Shared Story timeline component

Create `ui/src/components/console/story-timeline.tsx`:
`StoryTimeline({ beats })` renders lane-grouped, time-ordered rows: a left
gutter with relative time (`formatTimeInRange`/`relativeTime`), a lane label,
and the beat title with a kind/severity chip. Error beats get the rose accent
idiom already used in the app. Each beat with a `spanId` links to the span in
the waterfall (or the trace); each log beat links to logs. Reuse
`span-kind.tsx` chips, `heat-cell` conventions, and `cn()`. No new chart
library. Keep rows virtualization-ready but a plain map is acceptable for V1
(note the cap).

**Verify (from `ui/`)**: `rtk bun run typecheck` → exit 0;
`rtk bun run lint` → exit 0.

### Step 4: Mount the Story tab on trace and run detail

Add a "Story" tab to `traces.$traceId.tsx` (beside the existing waterfall/
inspector) and to `runs.$runId.tsx`. Fetch `story` in the route loader for the
current anchor and render `StoryTimeline`. Use the shell's existing tab/
section pattern (do not invent a new routing scheme — a local tab state or a
`?tab=story` search param via the route's `validateSearch` is fine; prefer the
URL param to honor the "URL state everywhere" convention). Keep the waterfall
as the default tab.

**Verify (from `ui/`)**: `rtk bun run build` → exit 0.

### Step 5: Tests

- Rust: unit tests on the Step 1 projection — ordering under equal timestamps,
  lane grouping by service, error beats present for error spans/ERROR logs,
  determinism (same input → identical output twice). Cover in
  `parallax-core`.
- UI: a vitest case rendering `StoryTimeline` with a fixture of mixed
  span/log/error beats; assert time order, lane labels, and that an error beat
  carries the error styling and a link. Model on `waterfall.test.tsx`.

**Verify**: `rtk cargo nextest run --workspace` → all pass;
`rtk bun run test` (from `ui/`) → all pass.

## Test plan

- Rust: projection ordering/grouping/determinism (Step 5).
- UI: timeline rendering (Step 5).
- Pattern: `parallax-core` unit tests; `waterfall.test.tsx` for UI.

## Done criteria

- [ ] Rust: `fmt` no diff, `clippy -D warnings` exit 0, `nextest` exit 0 with
      new tests
- [ ] UI: `typecheck`/`lint`/`build`/`test` all exit 0 (from `ui/`)
- [ ] `story` resolver returns `[StoryBeat!]!` for a traceId and for a runId
      (grep the resolver; enforces exactly-one anchor)
- [ ] Story tab renders on both trace and run detail; error beats are visually
      distinct and linked (asserted by UI test)
- [ ] Projection is deterministic (same input twice → equal output, asserted)
- [ ] No out-of-scope files modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- The `bundle` resolver's input-fetch is not reusable for a trace/run anchor
  without also pulling issue-anchor logic — if so, factor out just the
  trace/run fetch and note the refactor; do not entangle issue-bundle logic.
- Span events' JSON shape can't be parsed into `{name, timeUnixNano,
  attributes}` — report the actual shape.
- The run anchor returns spans but no logs because of the logs-bridge run-id
  quirk (storage audit: spans-only runs are invisible via `spans_by_run`
  which bridges through logs) — if a run's story is empty despite spans
  existing, STOP and report; that is a storage-layer limitation to fix
  separately, not to work around here.

## Maintenance notes

- **Deferred:** normalized `story_events` materialization (for latency at
  scale) and agent/TUI/browser event lanes (need playground telemetry from
  plan 034). Track in README.
- The projection is the seam a future summarizer would sit behind — keep
  ordering/severity/links deterministic so prose can be layered on top without
  changing the underlying beats (brief principle 6).
- Reviewer: confirm no raw high-cardinality user text lands in beat titles
  (low-cardinality names only), and that the projection has no clock/rng.
