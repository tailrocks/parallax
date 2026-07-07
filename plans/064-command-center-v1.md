# Plan 064: Command center v1 — shared chart-brush primitive + deterministic "what changed" top-movers lane on Overview

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- ui/src/routes/index.tsx ui/src/routes/logs.tsx ui/src/components/console crates/parallax-api/src/lib.rs`
> On mismatch with the excerpts below, STOP. Plans 038/039 touch
> `index.tsx` — land after them (see Depends on).

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (Overview is the front page; brush extraction touches Logs)
- **Depends on**: plan 038 (URL window helper — brush writes the shared
  window), plan 039 (pivot links on Overview — this plan builds on its
  clickable cards). advisor-plans/030 (attributeCompare) enriches the lane
  later but is NOT required.
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

The research brief's flagship surface is a command center answering "what
changed, what broke, what is hot?" — today's Overview shows four KPI cards
with a single scalar delta each, two trend charts you cannot brush, and two
recent lists. The only drag-brush in the app is hand-rolled inside the logs
histogram, unusable elsewhere. The backend has no anomaly primitives at all
(no baseline comparison anywhere in `crates/`), but it doesn't need one for
v1: `serviceList`-style per-service aggregates over the current and previous
windows are enough to compute deterministic top movers (error-rate/latency/
volume deltas per service) in the loader. This plan extracts the brush into a
reusable hook, applies it to the Overview trend charts, and adds a
"What changed" lane — deterministic, inspectable, no ML prose.

## Current state

Verified at commit `ed5b10f`.

- `ui/src/routes/index.tsx:135-175` — `loadOverview` already fetches
  dual-window data: `overview` + `previousOverview`, `red` + `previousRed`
  (`previousRange(range)` at `:136`), plus `issues(limit: 6)` and
  `tracesPage(DURATION_DESC, limit: 6)`. Per-card deltas via `formatDelta`
  (`:317-364`).
- No per-service movement: the overview query has no per-service series;
  `serviceList(fromNanos, toNanos)` exists (`crates/parallax-api/src/lib.rs:933`)
  returning per-service health (span count, error count/rate, p95 — confirm
  exact fields by reading the resolver + its GraphQL object before writing
  the query).
- The one brush — `ui/src/routes/logs.tsx:501-579`: hand-rolled
  `onMouseDown/Move/Up` + `activeTooltipIndex` extraction
  (`indexFromState`, `:501-505`) + `ReferenceArea` overlay (`:572-579`) +
  `dragWindow` (`:155-167`). Trapped in `LogsHistogram`.
- Overview charts: `SignalTrendCard` / `LatencyTrendCard`
  (`index.tsx:377-391`, defined from `:406`) — Recharts `AreaChart`s with no
  mouse handlers.
- Recharts primitives in use app-wide: Area/Bar/Line charts, `ReferenceArea`
  — no `Brush` component anywhere (the hand-rolled approach is the repo
  convention; keep it).
- Anomaly primitives absent server-side: grep `baseline|anomaly|zscore`
  in `crates/` → none (audit-verified).
- Conventions: URL is the state (plan 038's `rangeLinkSearch` helper —
  read it when landed); inline errors; strict TS.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| UI (from `ui/`) | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |
| Rust (only if serviceList fields are missing) | `rtk cargo build --workspace && rtk cargo nextest run` | clean |

## Scope

**In scope**:
- `ui/src/components/console/use-chart-brush.ts` (new — extracted hook) +
  `ui/src/routes/logs.tsx` (refactor `LogsHistogram` onto the hook; zero
  behavior change)
- `ui/src/routes/index.tsx` — brush on both trend cards; "What changed"
  lane + its loader additions (dual-window `serviceList`)
- `ui/src/components/console/top-movers.tsx` (new — the lane component)
- `crates/parallax-api/src/lib.rs` — ONLY if `serviceList` lacks a needed
  field (p95 or error rate); additive field, no signature change
- Tests

**Out of scope** (do NOT touch):
- Server-side anomaly/baseline resolvers (`metricChange`, z-scores) —
  deferred; the v1 lane is client-computed from dual windows (named in
  Maintenance).
- Release/flag attribution clues — needs advisor-plans/030 + plan 041; the
  lane's design leaves a slot (Maintenance note), nothing more.
- "Open investigation for this window" — plan 052's surface; the brush
  writes the URL window, which 052 captures for free.
- KPI card redesign, RecentIssues/SlowestTraces cards — 039 owns pivots.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Extract `useChartBrush`

New `use-chart-brush.ts`: a hook encapsulating exactly the logs-histogram
mechanics — state `(dragStart, dragEnd)`, `indexFromState` extraction,
handlers `{onMouseDown, onMouseMove, onMouseUp, onClick}` parameterized by
`(series, stepSeconds, onWindow)`, and `referenceRange` for the
`ReferenceArea`. Move `dragWindow`/`bucketWindow` (`logs.tsx:141-167`) into
the module. Refactor `LogsHistogram` to consume it — rendering and behavior
byte-identical (its props stay the same).

**Verify** (from `ui/`): `bun run test` — existing logs tests green; new
hook unit test: drag 2→5 yields the bucket window, reversed drag
normalizes, click yields single-bucket window.

### Step 2: Brush the Overview trend charts

In `SignalTrendCard` + `LatencyTrendCard` (`index.tsx:406+`): wire the hook;
`onWindow(fromNanos, toNanos)` navigates to `/` with the custom range params
(plan 038's URL-window convention — reuse its helper; the RangePicker at
`:309` must reflect the custom window the way `/logs` does today). Add the
`ReferenceArea` during drag and a "Reset window" affordance consistent with
logs (`logs.tsx:518-523`).

**Verify**: `bun run test` + manual: dragging the spans chart narrows the
whole page's window (URL changes, cards re-query); reset restores. Record.

### Step 3: "What changed" lane

1. Loader: add to `loadOverview` two `serviceList` calls aliased
   `servicesNow`/`servicesPrev` over `range`/`previous`. Confirm the exact
   field names from the resolver (`lib.rs:933` region) — needed per
   service: name, span/request count, error count or rate, p95 (if p95 is
   absent from `serviceList`, STOP condition below governs).
2. `top-movers.ts(x)`: pure `computeMovers(now, prev): Mover[]` —
   per service compute deltas: error-rate Δ (percentage points), p95 Δ
   (ratio), volume Δ (ratio); a mover qualifies when
   `|error-rate Δ| ≥ 2pp` or `p95 ratio ≥ 1.5` or `volume ratio ≥ 2`
   (exported consts, doc-commented; new services — absent in prev — qualify
   as "new service"). Rank: error movers first, then latency, then volume;
   cap 6.
3. Render between the KPI section (`index.tsx:312-375`) and the trend
   section: one row per mover — direction icon, service name (link to
   `/services/$service` carrying the window — 038/039 conventions),
   human sentence built from parts:
   "checkout error rate 0.4% → 6.1%" / "pricing p95 up 3.2×" /
   "new service: fulfillment". Empty state: single muted line
   "Nothing moved more than the thresholds in this window." — the lane is
   always present (determinism visible).

**Verify**: `bun run test` — `computeMovers` unit tests: threshold edges,
new-service case, ranking, cap; component renders sentences from a fixture;
empty state renders.

### Step 4: Lane ↔ brush composition check

Brushing a spike (Step 2) re-runs the loader on the narrowed window; the
lane recomputes against the preceding window of equal length
(`previousRange` already does this). Manual check: drive the playground
deploy-regression scenario (a13), brush the error spike on Overview, the
lane names the regressed service. Record it (or record the blocked reason —
needs a live playground).

## Test plan

- Hook unit tests (Step 1); `computeMovers` table-driven tests (Step 3);
  card render tests. Model on existing co-located tests near `index.tsx`
  (grep `index.test` / nearest route test).
- Existing overview/logs tests must stay green (regression gate on the
  refactor).

## Done criteria

- [ ] `bun run typecheck && bun run lint && bun run test && bun run build` all exit 0
- [ ] `logs.tsx` histogram consumes `useChartBrush`; its behavior tests
      unchanged/green
- [ ] Both Overview trend charts brush → URL window (recorded manual check)
- [ ] "What changed" lane renders movers/empty-state deterministically
      (tests) and links carry the window
- [ ] `plans/README.md` status row updated

## STOP conditions

- `serviceList` lacks p95/error fields and adding them server-side exceeds
  an additive field on the existing resolver — report the schema gap
  instead of building a new resolver in this plan.
- Plan 038's window helper hasn't landed and Overview has no custom-window
  URL convention — STOP (this plan depends on it; don't invent a second
  convention).
- Extracting the hook changes logs-histogram behavior in any test — the
  refactor must be invisible; report otherwise.
- Dual-window `serviceList` makes the Overview loader measurably slow
  (>1.5s at lab volume) — report timings; don't silently drop the previous
  window.

## Maintenance notes

- **Deferred (named)**: server-side `metricChange`/baseline resolver and
  z-score ranking (the client dual-window fold is v1); release/flag
  attribution clues via advisor-plans/030's `attributeCompare` + plan 041's
  releases — the lane's `Mover` type should grow a `clue` slot then.
- The thresholds are product knobs — reviewers check they're exported
  consts with doc comments, not magic numbers.
- Plan 052 (investigations) captures the brushed window automatically once
  both land — that composition is the brief's "open investigation for this
  window" without extra UI here.
