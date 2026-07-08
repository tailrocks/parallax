# Plan 028: Expose span links as typed GraphQL objects and resolve linked traces, then render them as causal edges

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/adapter.rs crates/parallax-storage/src/memory.rs ui/src/routes/traces.\$traceId.tsx`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (independent; benefits from plan 024's depth guard but
  does not require it)
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

Span links are the only way async reality (producer→consumer, batch, fan-in,
retry-as-new-trace, trust-boundary trace restarts) is represented, and
Parallax already stores them. But the API exposes each span's links as an
opaque JSON **string** and there is no way to resolve a linked trace, so the
UI can only print a flat list of trace-id links with no target context. The
research brief calls this the smallest high-value trace improvement: the data
is already there; the UI just needs the links resolved and rendered as causal
edges. This plan makes links a typed object and adds one resolver that returns
the linked traces' summaries so the UI can show *what* is on the other end.

## Current state

- `crates/parallax-api/src/lib.rs:255-259` — links are stringified:

  ```rust
  /// OTel span links as JSON: `[{traceId, spanId, attributes}]` …
  fn links(&self) -> String {
      self.0.links.to_string()
  }
  ```

- `crates/parallax-storage/src/model.rs` — `SpanRow.links: serde_json::Value`
  (JSON array), stored in the native `span_links` column
  (`greptime.rs:355`, read as `cols.json("span_links", row)`).
- `crates/parallax-storage/src/adapter.rs:144-145` — `spans_by_trace` is the
  only per-trace span read; there is no "resolve these linked trace ids" call.
- `crates/parallax-storage/src/memory.rs` — the memory store sets
  `links: Null` on its rows, so link features can't be exercised against it
  (relevant for tests — see Step 5).
- UI: `ui/src/routes/traces.$traceId.tsx:102-114` parses the JSON string
  (`parseLinks`) and `:545-563` renders a flat `<ul>` of `traceId` deep-links.
  The trace loader already fetches `trace { spans { links events attributes
  resource } }`.
- API layer is a single 2181-line `crates/parallax-api/src/lib.rs` (Juniper).
- UI conventions (`ui/AGENTS.md`): one data path via
  `ui/src/lib/api.ts` → `/graphql`; nanos are strings; Base UI composition
  uses `render={<El/>}`; `@tabler/icons-react`; charts via Recharts in
  `ChartContainer`. Tests: vitest, co-located `__tests__/`, exemplar
  `ui/src/components/console/__tests__/waterfall.test.tsx`.

## Commands you will need

| Purpose        | Command                                                              | Expected |
|----------------|----------------------------------------------------------------------|----------|
| Rust format    | `rtk cargo fmt --all` (repo root)                                    | exit 0   |
| Rust lint      | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0   |
| Rust tests     | `rtk cargo nextest run --workspace`                                 | all pass |
| UI typecheck   | `rtk bun run typecheck` (from `ui/`)                                | exit 0   |
| UI lint        | `rtk bun run lint` (from `ui/`)                                     | exit 0   |
| UI tests       | `rtk bun run test` (from `ui/`)                                     | all pass |
| UI build       | `rtk bun run build` (from `ui/`)                                    | exit 0   |

## Scope

**In scope**:
- `crates/parallax-api/src/lib.rs` (typed `SpanLink` object + `linkedTraces`
  resolver)
- `crates/parallax-storage/src/adapter.rs` (new trait method)
- `crates/parallax-storage/src/greptime.rs` (impl)
- `crates/parallax-storage/src/memory.rs` (impl)
- `ui/src/lib/api.ts` (types + query)
- `ui/src/routes/traces.$traceId.tsx` (render edges)
- test files for each layer

**Out of scope**:
- A full node-link **graph** visualization (that is the ecosystem plan 031);
  here, "edges" means resolved link cards, not a force-directed graph.
- Reverse link lookup (find spans that link *to* this trace) — the storage
  audit shows that needs an index; defer and note in README.
- Removing the existing JSON `links` field (keep it for back-compat; add the
  typed field alongside).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
- Two commits (Rust, then UI) is fine. Push when done.

## Steps

### Step 1: Add a typed `SpanLink` GraphQL object and a `links` typed field

In `lib.rs`, add a `SpanLink` object with fields `traceId: String`,
`spanId: String`, `attributes: String` (attributes stay JSON string — typing
them is out of scope). Add a resolver on `Span` that parses `self.0.links`
into `Vec<SpanLink>` (fall back to empty on parse failure). Name it
`typedLinks` (camelCased) to sit beside the existing `links` string field
without breaking it. Do this parse in Rust from the stored
`serde_json::Value` array; each element is `{traceId, spanId, attributes}`
(match the JSON shape the storage layer writes; confirm keys by reading how
`span_links` JSON is produced — the doc comment at lib.rs:255 states the
shape).

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Add a `linked_traces` storage method

In `adapter.rs`, add to the `TelemetryStore` trait:

```rust
/// Resolve summaries for a set of linked trace ids (span-link targets).
/// Returns at most one summary per id, in the input order where possible.
async fn traces_by_ids(&self, trace_ids: &[String]) -> anyhow::Result<Vec<TraceSummary>>;
```

Implement in `greptime.rs` by reusing the `traces_search` root-span selection
constrained to `trace_id IN (...)` (build the IN-list with the existing
`escape()` for each id — these are hex ids, but escape anyway). Implement in
`memory.rs` by filtering the in-memory trace set. Bound the input to
`MAX_ROWS`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Add a `linkedTraces(traceId)` resolver

In `lib.rs`, add a query resolver `linked_traces(traceId: String)` that:
1. fetches `spans_by_trace(traceId)`,
2. collects the distinct target `traceId`s from every span's parsed links
   (excluding the anchor trace itself),
3. calls `store.traces_by_ids(&ids)`,
4. returns `Vec<TraceSummary>` (the existing `TraceSummary` object already has
   root name, service, span count, error flag).

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Render linked traces as edge cards in trace detail

In `ui/src/lib/api.ts`, add a `SpanLink` interface and a `linkedTraces` query
(reuse the existing `TraceSummary` interface). In `traces.$traceId.tsx`,
replace the flat `<ul>` of trace-id links (`:545-563`) with cards that show,
per link target: root service + name, span count, error state (reuse
`SpanKindChip`/heat conventions), and a `<Link to="/traces/$traceId">`. Fetch
`linkedTraces` in the route loader (add to the existing GraphQL call, or a
second call — match how the route already issues its two queries). Keep the
raw span-level link list available but secondary.

Match existing patterns: `Link` typed navigation, `relativeTime`, the
Inspector section layout already in the file.

**Verify (all from `ui/`)**: `rtk bun run typecheck` → exit 0;
`rtk bun run lint` → exit 0; `rtk bun run build` → exit 0.

### Step 5: Tests

- Rust: unit test `SpanLink` parsing from a sample `links` JSON; integration
  test for `traces_by_ids` and `linkedTraces` against the **memory** store —
  but the memory store sets `links: Null`, so first extend the memory store's
  test fixtures (or the memory ingest path) to carry a non-null `links` value
  for the test. If wiring links into the memory store is more than a couple of
  lines, instead unit-test the link-id extraction as a pure function
  (extract it) and cover `traces_by_ids` directly with seeded traces. Choose
  the smaller path and note it.
- UI: a vitest case rendering the trace-detail `Content` with a fixture whose
  spans carry links + a `linkedTraces` result; assert the edge cards show the
  target service and are links. Model on
  `ui/src/components/console/__tests__/waterfall.test.tsx`.

**Verify**: `rtk cargo nextest run --workspace` → all pass;
`rtk bun run test` (from `ui/`) → all pass.

## Test plan

- Rust: `SpanLink` parse, `traces_by_ids`, `linkedTraces` (Step 5).
- UI: linked-trace edge rendering (Step 5).
- Pattern: server integration tests over memory store; `waterfall.test.tsx`
  for UI.

## Done criteria

- [ ] Rust: `fmt` no diff, `clippy -D warnings` exit 0,
      `nextest run --workspace` exit 0 with new tests
- [ ] UI: `typecheck`, `lint`, `build`, `test` all exit 0 (from `ui/`)
- [ ] `linkedTraces(traceId:)` exists in the schema and returns
      `[TraceSummary!]!` (grep the resolver)
- [ ] Trace detail renders resolved link targets with service + name (asserted
      by UI test), not a bare id list
- [ ] No out-of-scope files modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- The stored `span_links` JSON key names differ from `{traceId, spanId,
  attributes}` — report the actual shape before typing it.
- Wiring links into the memory store for tests balloons beyond a small change
  — fall back to the pure-function test path (Step 5) and report.
- `linkedTraces` requires a reverse (link-source) lookup to be useful for the
  target screen — it does not for this plan (forward resolution only); if a
  requirement pushes you toward reverse lookup, STOP (that needs an index,
  out of scope).

## Maintenance notes

- **Deferred:** reverse link lookup (which traces link *to* this one) needs a
  new index on link target ids — the storage audit flagged it as an unindexed
  full scan today. Track in README "considered".
- If plan 031 (ecosystem graph) lands, its edge model can consume the same
  `SpanLink` type — keep the type reusable.
- Reviewer: confirm the typed field is added *beside* the JSON `links` field,
  not replacing it, so nothing depending on the string breaks.
