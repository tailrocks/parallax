# Plan 051: traceCriticalPath + traceCompare — pure analyses over existing span rows, critical-path highlight in the waterfall

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-core crates/parallax-api ui/src/routes/traces.\$traceId.tsx ui/src/components/console/trace-waterfall.tsx`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (pairs with playground plan 049's per-attempt spans
  and 047's N+1 shapes for demos; advisor-plans/029's story uses critical
  path later)
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

"Which span actually gated this latency?" and "what changed between these
two traces?" are the brief's trace-analysis pair (section "Investigation
analytics algorithms": `traceCriticalPath`, `traceCompare`). Both are pure
computations over data Parallax already returns for every trace — no new
storage, no new ingest. The critical path also feeds the story timeline
(advisor-plans/029 keeps critical-path beats expanded), so landing the pure
function in `parallax-core` benefits two surfaces.

## Current state

Verified at commit `408be17`.

- `SpanRow` (`crates/parallax-storage/src/model.rs:7-28` region) carries
  everything the algorithms need: `trace_id`, `span_id`, `parent_span_id`,
  `service`, `name`, `kind`, `ts_nanos`, `duration_ns`, `status_code`.
- `spans_by_trace(trace_id)` returns the full start-ordered span set
  (`crates/parallax-storage/src/greptime.rs:669`; trait at
  `adapter.rs:145`).
- Name normalization primitives exist in
  `crates/parallax-core/src/normalize.rs` (used by fingerprinting) — reuse
  for compare's operation-key normalization (`<uuid>`/`<hex>`/`<n>`
  replacement — check its public functions before writing a new one).
- UI: trace detail (`ui/src/routes/traces.$traceId.tsx`) renders
  `TraceWaterfall` (`ui/src/components/console/trace-waterfall.tsx`) —
  rows built by `buildTraceTree` (`:31`), each row has
  `offsetPct`/`widthPct` and a styled button (`:124-134` region). A
  highlight = extra classname/stroke on rows whose span ids are in the
  critical path.
- Brief's algorithm sketch (inline it for the executor):
  - **Critical path**: walk from the root; at each span, child intervals
    that overlap sequentially chain; among PARALLEL siblings the
    latency-gating child is the one whose end time is latest (max end, not
    sum); gaps where the parent is running but no child → parent self-time
    on the path. Deterministic; ties break by earliest start then span id.
  - **Compare**: align two traces' spans by (normalized name, service,
    kind, depth, sibling index among same-key siblings); output added /
    removed / matched-with-duration-delta / status-changed lists.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | exit 0 |

## Scope

**In scope**:
- `crates/parallax-core/src/` — new `trace_analysis.rs` (pure functions +
  unit tests)
- `crates/parallax-api/src/lib.rs` — `traceCriticalPath(traceId)`,
  `traceCompare(traceIdA, traceIdB)` resolvers
- `ui/src/lib/api.ts`; `ui/src/components/console/trace-waterfall.tsx`
  (highlight prop); `ui/src/routes/traces.$traceId.tsx` (toggle + fetch)
- Compare UI: minimal — a "Compare with…" input on trace detail that takes
  a second trace id and renders the diff lists in a Sheet (no side-by-side
  waterfall in this plan)
- test files

**Out of scope**:
- `aggregateTrace` (many-trace structural rollup — audited as
  materially more expensive; separate spike later).
- Side-by-side dual-waterfall rendering (brief's fuller vision; the diff
  lists ship first).
- Story integration (advisor-plans/029 consumes `critical_path` when it
  lands — expose the core function publicly for it).
- Clock-skew correction (order by parent/child when timestamps lie is
  029's concern; here document the assumption).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: `parallax-core::trace_analysis` — pure functions + exhaustive tests

Implement over `&[SpanRow]`:
- `critical_path(spans) -> Vec<CriticalHop>` where `CriticalHop { span_id, self_time_ns, gated_by_child: Option<String> }`
  per the algorithm above. Handle: missing root (orphan set → pick earliest
  root-like span), multiple roots (analyze the tree of the earliest, list
  others in an `unattached` field), zero-duration spans, children exceeding
  parent bounds (clamp, flag `clock_suspect: true` on the hop).
- `compare(a: &[SpanRow], b: &[SpanRow]) -> TraceDiff` with
  `added`/`removed`/`changed` (duration delta ns + pct, status change) and
  a `match_key` exposed for debugging. Use `normalize.rs` helpers for the
  name key.

**Verify**: `rtk cargo nextest run -p parallax-core` → tests green (see
Test plan — this step is test-heavy by design).

### Step 2: Resolvers

`traceCriticalPath(traceId: String!)` → fetch `spans_by_trace`, run the
core function, return hops (+ total gated duration, `unattached` ids).
`traceCompare(traceIdA: String!, traceIdB: String!)` → both fetches + diff.
Both return a clear FieldError when a trace has zero spans. Follow the
neighboring resolver conventions in `parallax-api/src/lib.rs`.

**Verify**: `rtk cargo nextest run` → resolver tests green.

### Step 3: Waterfall highlight

1. `TraceWaterfall` accepts `highlightIds?: Set<string>` — rows in the set
   get an accent stroke (left border / bar outline using an existing accent
   token; check how `failed` rows are styled at `trace-waterfall.tsx:126`
   region and use a distinct treatment).
2. Trace detail: a "Critical path" toggle (header, next to existing
   controls); on enable, fetch `traceCriticalPath` and pass the ids; also
   show "N spans gate X ms of Y ms total" as a one-line summary.

**Verify**: (from `ui/`) `bun run typecheck && bun run test` — extend
`waterfall.test.tsx` with a highlight-prop assertion; existing tests stay
green.

### Step 4: Minimal compare UI

On trace detail: "Compare…" button → small dialog with a trace-id input
(paste a second id; a picker over recent traces is a nice-to-have ONLY if
the recent-traces query is already imported on this route — check; don't
add new queries for it). Result Sheet: three sections (Added / Removed /
Changed) as simple tables — name, service, duration delta (formatted via
`format.ts` helpers), status change badge. Empty diff → "structurally
identical".

**Verify**: `bun run typecheck && bun run lint && bun run build` → exit 0.
Manual: compare a clean checkout trace vs a `?fail=1` trace from the
playground — diff shows the changed/added error spans (record ids).

## Test plan

Core tests (`trace_analysis.rs` `#[cfg(test)]`), minimum set:
- linear chain → path = every span, self-times sum to root duration
- parallel fan-out → only the max-end child on the path
- fan-out with a longer second wave → path switches children correctly
- orphan/multi-root → `unattached` populated, no panic
- child overrunning parent → clamped + `clock_suspect`
- compare: identical traces → empty diff; renamed span with volatile id
  (`order-123` vs `order-456`) → MATCHED via normalization; added retry
  sibling → `added`; status Ok→Error → `changed`
Resolver tests: empty-trace error; happy path over the memory adapter.
UI: waterfall highlight prop test.

## Done criteria

- [ ] `rtk cargo build`, clippy zero warnings, `rtk cargo nextest run`
      green including ≥9 new core tests
- [ ] `traceCriticalPath`/`traceCompare` resolvers live
- [ ] Waterfall critical-path toggle renders highlights + summary line
- [ ] Compare dialog renders Added/Removed/Changed for two real traces
      (recorded)
- [ ] UI gates exit 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- `SpanRow` lacks a field the algorithm needs (e.g. durations missing for
  some sources) — report actual data quality with sampled rows before
  approximating.
- `normalize.rs` helpers are private/fingerprint-specific and not cleanly
  reusable — write a local normalizer in `trace_analysis.rs` and note the
  duplication for a later merge; do NOT change fingerprint behavior.
- Traces routinely exceed ~5k spans in the demo data making the resolver
  slow — report timings; capping/streaming is a design change.

## Maintenance notes

- advisor-plans/029 (story) should call `critical_path` for its
  keep-expanded logic — the function is public in parallax-core for that.
- Deferred: `aggregateTrace` spike; side-by-side compare waterfall; a
  picker UI over recent traces.
- Reviewer: determinism (ties broken stably — tests must pin it);
  compare's match must never key on raw high-cardinality names (uses
  normalization); highlight color meets contrast on both themes.
