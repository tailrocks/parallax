# Plan 058: Trace-events backbone — typed `traceEvents(traceId)` resolver + span-read hygiene (parse once server-side, fix O(n²) dedup)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-core/src crates/parallax-storage/src/greptime.rs`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P1 (unblocks plans 059 and 060)
- **Effort**: M
- **Risk**: LOW (additive resolver + internal refactor with tests)
- **Depends on**: none
- **Category**: direction (+ a perf fix)
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

Span events are the carrier for the research brief's richest trace detail —
gRPC per-message `rpc.message` events, retries, feature-flag evaluations,
exceptions — but Parallax exposes them only as an **opaque JSON string per
span** (`Span.events`), so every consumer must fetch all spans and JSON-parse
each string in the browser. Plans 059 (GraphQL explorer) and 060 (stream/
messaging lanes) both need a trace-wide, typed, filterable event list; without
this backbone each would re-implement client-side parsing. Alongside, two
small server hygiene issues bite as traces grow: the run→trace fan-out dedups
trace ids with `Vec::contains` inside a loop (O(n²)), and nothing bounds how
many parsed events one request can return. This plan adds one typed resolver
with explicit bounds and fixes the dedup.

## Current state

Verified at commit `ed5b10f`.

- `crates/parallax-storage/src/model.rs:20-22` — events are a raw string on
  the row:

  ```rust
  /// Raw OTel span events JSON (`[{name, time_unix_nano, attributes}]`) when
  /// the backing source projects it; absent sources default to no events.
  pub events: Option<String>,
  ```

  Populated only on the GreptimeDB read path (`greptime.rs:336-339` reads the
  native `span_events` column as a JSON value and stringifies it); the
  normalize path sets `events: None` (`normalize.rs:150`), so the in-memory
  store has no span events today.

- `crates/parallax-api/src/lib.rs:260-263` — the only exposure:

  ```rust
  /// OTel span events as JSON: `[{name, timeUnixNano, attributes}]`.
  fn events(&self) -> String {
      self.0.events.clone().unwrap_or_else(|| "[]".to_string())
  }
  ```

- `crates/parallax-api/src/lib.rs:1054-1064` — `trace(traceId)` fetches via
  `spans_by_trace`, which is **uncapped** (`greptime.rs:669-676`, no LIMIT
  clause) — fine for the backbone, but the parsed-events resolver must bound
  its own output.

- `crates/parallax-api/src/lib.rs:1088-1094` — the O(n²) dedup (one of
  several — grep `iter_mut().find` and `contains(` over trace-id
  accumulations in the file; the audit found the same shape near `:461-465`
  and `:1496-1512`):

  ```rust
  let mut by_trace: Vec<(String, Vec<model::SpanRow>)> = Vec::new();
  for span in spans {
      match by_trace.iter_mut().find(|(t, _)| *t == span.trace_id) {
          Some((_, group)) => group.push(span),
          None => by_trace.push((span.trace_id.clone(), vec![span])),
      }
  }
  ```

- UI parses events client-side today: `parseEvents` in
  `ui/src/routes/traces.$traceId.tsx` (used at `:163` for the summary count
  and `:442` for the inspector). This plan does NOT remove that — the
  inspector keeps per-span parsing; the new resolver serves trace-wide,
  filtered questions.

- Repo conventions: pure analysis lives in `parallax-core`
  (plan 051 puts `trace_analysis.rs` there; advisor-plans/032 puts
  `gaps.rs` there); resolvers stay thin; resolver-level caps via `MAX_ROWS`
  (`lib.rs:44`).

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build/lint/test | `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets && rtk cargo nextest run` | clean |

## Scope

**In scope**:
- `crates/parallax-core/src/span_events.rs` (new — pure parse/filter over
  `&[SpanRow]`)
- `crates/parallax-core/src/lib.rs` (module export)
- `crates/parallax-api/src/lib.rs` (`TraceEvent` GraphQL object +
  `traceEvents` resolver; `HashSet`/`HashMap` dedup fixes)
- Tests in both crates

**Out of scope** (do NOT touch):
- UI rendering — plans 059/060 consume this.
- GreptimeDB-side JSON pushdown/filtering — read-time parse in Rust is the
  V1; note it as the scale fallback (Maintenance).
- `Span.events` raw field — keep for back-compat.
- Populating `events` on the normalize/in-memory path from OTLP protos —
  DO change tests to construct rows with `events: Some(json)` directly, but
  wiring `normalize_traces` to serialize proto events is a separate ingest
  decision (note it; the GreptimeDB path already carries real events).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Pure core — `span_events.rs`

New `crates/parallax-core/src/span_events.rs`:

```rust
/// One parsed span event, joined to its carrying span.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEvent {
    pub span_id: String,
    pub span_name: String,
    pub service: String,
    pub name: String,
    pub time_unix_nano: u128,
    /// Flat string map; non-string values JSON-stringified.
    pub attributes: Vec<(String, String)>,
}

/// Parse every span's `events` JSON, filter, sort by time ascending, cap.
/// Malformed JSON on one span is skipped (counted), never an error.
pub fn trace_events(
    spans: &[SpanRow],
    name_prefix: Option<&str>,
    limit: usize,
) -> (Vec<TraceEvent>, usize /* skipped_spans */)
```

Parsing rules (match what GreptimeDB emits, verified shape at
`greptime.rs:336-339` + the UI's existing `parseEvents` — read that function
in `ui/src/routes/traces.$traceId.tsx` before coding to mirror its field
names): array of objects with `name`, `time_unix_nano` OR `timeUnixNano`
(accept both spellings), `attributes` as object. Sort ascending by time,
truncate to `limit`, count spans whose JSON failed to parse.

**Verify**: `rtk cargo nextest run -p parallax-core` — tests: both timestamp
spellings; prefix filter (`rpc.message` matches `rpc.message`); malformed
JSON skipped + counted; cap enforced; deterministic order (time, then
span_id) for equal timestamps.

### Step 2: `traceEvents` resolver

In `lib.rs`:

```rust
/// Parsed span events across one trace, time-ascending. `namePrefix`
/// filters by event name (e.g. "rpc.message", "exception",
/// "feature_flag"). Capped; `truncated`/`skippedSpans` are honest flags.
async fn trace_events(
    context: &ApiContext,
    trace_id: String,
    name_prefix: Option<String>,
    limit: Option<i32>,          // clamp_limit(limit, 500)
) -> FieldResult<TraceEventsOut>
```

`TraceEventsOut { events: Vec<TraceEventGql>, truncated: bool,
skipped_spans: i32 }` — a `#[graphql_object]` pair mirroring the core
struct (attributes as a JSON-string map, matching the existing convention of
`Span.attributes` at `lib.rs:267-269`). Implementation: `spans_by_trace`
→ `parallax_core::span_events::trace_events(...)`. `truncated` = pre-cap
count exceeded the cap (have the core function also return the pre-cap
count, or request `limit + 1` and compare — pick one, test it).

**Verify**: `rtk cargo nextest run -p parallax-api` — in-memory store test:
seed two spans with `events: Some(...)` JSON (three events across them, one
`rpc.message`-prefixed), assert filter + order + `skippedSpans: 0`; a
malformed-events span bumps `skippedSpans`.

### Step 3: Dedup hygiene

Replace the O(n²) accumulations with keyed collections, preserving
first-seen order where the output is order-sensitive:
1. `lib.rs:1088-1094` (`traces_by_run`): `IndexMap`-style behavior without a
   new dependency — keep a `Vec<(String, Vec<SpanRow>)>` for order but add a
   `HashMap<String, usize>` index for lookup.
2. Apply the same shape to the other sites found by
   `rtk grep -n "iter_mut().find\|\.contains(&" crates/parallax-api/src/lib.rs`
   that accumulate trace/span ids in loops (audit saw ~3 sites, e.g. near
   `:461-465` and `:1496-1512`) — read each; only convert genuine
   per-element linear scans. Do not touch semantics.

**Verify**: `rtk cargo nextest run` all green (existing tests cover these
resolvers' outputs; identical results expected). `rtk cargo clippy` clean.

## Test plan

Listed per step above. Structural pattern: model API tests on the existing
in-memory-store resolver tests (grep `MemoryStore` in
`crates/parallax-api`). New core tests live in
`crates/parallax-core/src/span_events.rs` `#[cfg(test)]`.

## Done criteria

- [ ] `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets` → zero warnings
- [ ] `rtk cargo nextest run` → all pass incl. new core + api tests
- [ ] `traceEvents(traceId:, namePrefix:, limit:)` in the schema with
      `truncated` + `skippedSpans`
- [ ] `rtk grep -n "iter_mut().find" crates/parallax-api/src/lib.rs` → no
      trace-id-accumulation hits remain
- [ ] `plans/README.md` status row updated

## STOP conditions

- The real GreptimeDB `span_events` JSON shape differs from both accepted
  spellings (check one live row first if a stack is available; the UI's
  `parseEvents` is the second witness) — report the actual shape.
- Converting a dedup site would change output order in a way an existing
  test asserts — report instead of changing the assertion.
- `spans_by_trace` on a stress trace (plan 063's A19) makes read-time parse
  visibly slow (>2s) — note it and finish; pushdown is the recorded
  follow-up, not an improvised change.

## Maintenance notes

- Plans 059/060 consume `traceEvents`; keep the `namePrefix` contract stable.
- Scale fallback (recorded): if parse-in-Rust over big traces becomes the
  bottleneck, push filtering into GreptimeDB JSON functions over
  `span_events`, or materialize an events side-table at ingest — both are
  additive behind this resolver's signature.
- Deferred (named): populating `SpanRow.events` in `normalize_traces` so the
  in-memory/tee path carries real events — an ingest-path change under the
  zero-copy rule; today only tests construct them.
- Reviewer: `skippedSpans`/`truncated` must be honest — no silent drops.
