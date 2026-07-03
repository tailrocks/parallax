# Plan 014: Logs — brushable histogram, dense severity table, column control, live tail kept

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/logs.tsx ui/src/components/logs-table.tsx`
> Plans 005-008 must be DONE.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: plans/005-008
- **Category**: tech-debt (UX redesign)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Logs is the spec's Kibana-style surface, and its histogram is the spec's canonical "every
chart is a filter" entry point (`docs/research/architecture/simple-ui-v2.md:90-100`). Today
the histogram is inert (no click/brush), filter state is non-URL `useState`, the table has
fixed columns with time-of-day-only timestamps (unreadable on 7d/30d windows), and the page
opens with a KPI strip of filter echoes. This plan makes the histogram drive the window,
moves state to the URL, applies the reference table grammar, and keeps the SSE live tail.

## Current state

- `ui/src/routes/logs.tsx` (verified): client `useState` filters; `load()` `:88-137` fetches
  `services + logs(limit: 500) + logCountSeries` (histogram window = 24h when "Latest");
  cursor `loadOlder` `:141-169`; poll `:176-180`; SSE `/v1/logs/stream` with
  service/severity_min/q params `:184-199`; below: 4 KpiCards, `.parallax-panel` toolbar,
  BarChart histogram (no onClick), shared `LogsTable`, "Load older" button, `"Loading…"`
  strings.
- `ui/src/components/logs-table.tsx`: fixed columns Time/Severity/Service/Body (`:110-144`);
  `formatTime` emits `HH:MM:SS` only (`:45-51`); row click opens a `Sheet` doc viewer with
  field search + trace/run links (`:121-197` area) — the Sheet viewer is good, keep it
  restyled.
- Kit (plan 008): table toolkit, RangePicker + `resolveRangeSearch`,
  `formatTimeInRange`, severity semantics available via Badge variants, EmptyState,
  TableSkeleton, useDelayedLoading, CopyButton.
- Reference grammar: toolbar/table/pagination as in plan 011 (same source files); chart
  styling via the plan-006 `chart.tsx` + `console/trend.tsx` helpers.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + serve + playground
(logs emitter). Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/logs.tsx` (rewrite), `ui/src/components/logs-table.tsx`
(restyle + column control + timestamp fix; keep exported API used by runs detail).
**Out of scope**: SSE server; runs detail page (it reuses LogsTable — keep its props
compatible or update the single import site mechanically, nothing more); API.

## Git workflow

`main`; `feat(ui): redesign logs explorer`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: URL state

`validateSearch` (zod): `{q, service, sev (min severity number), range, from, to, live,
cols}`. Loader-based fetch for query mode (logs + logCountSeries in one document; histogram
`stepSeconds = max(30, window/60)`); live mode client-side as today. Delete the KPI strip.

**Verify**: URL round-trip reproduces filters; back/forward works.

### Step 2: Brushable histogram (THE interaction)

Histogram Card above the table (h-[180px]): bar chart of `logCountSeries`
(`--chart-2` bars; severity≥ERROR overlay series in `--destructive` if cheap — optional).
Interactions, both required:
- **Click a bar** → window narrows to that bucket (`from=bucket`, `to=bucket+step`).
- **Drag across bars** (Recharts `onMouseDown/Move/Up` on the chart container or
  `ReferenceArea` pattern) → window = drag extent.
Both write `?from&to` (custom range) via the same navigate-patch used by filters; the
RangePicker label switches to the custom range. Add a small "reset window" chip when a
custom window is active.

**Verify**: clicking a spike narrows the table to that window; drag selects a span;
RangePicker shows the custom label; browser Back restores the previous window.

### Step 3: Table + columns

Restyle `LogsTable` on the plan-006 Table: compact density, columns:
Time (`formatTimeInRange` — date shown when window > 1 day; fixes the multi-day defect) ·
Severity (dot + Badge: TRACE/DEBUG muted, INFO secondary, WARN amber, ERROR rose, FATAL rose
bold) · Service (`text-muted-foreground`) · Body (mono `text-xs`, truncate) · Trace (link
chip when `traceId` present, `stopPropagation`).
**Column control**: a small dropdown (checkbox items) toggling optional columns
(Service, Trace, Scope) — persisted in `?cols=` (comma list) so views are shareable; Time,
Severity, Body always on. Row click keeps opening the Sheet doc viewer — restyle the Sheet
content: header (severity badge + time + CopyButton for the raw JSON), field grid
(mono values), attribute/resource sections, trace/run link buttons.

**Verify**: 7d window shows dates in Time; column toggle persists in URL; Sheet shows
timestamps + copy works.

### Step 4: Pagination + live

Query mode: keep cursor semantics but present as a "Load older" **ghost button row at table
bottom** (logs are a tail, numbered pages don't fit — deviation from plan 011 recorded
here deliberately). Live mode: same SSE mechanics, table prepends, cap 500, toolbar shows
pulsing emerald Live chip; no histogram in live mode (as today, but keep layout stable —
render the histogram card with the live notice instead of unmounting).

**Verify**: live toggle streams; return to query mode reloads window.

### Step 5: States + gate

`useDelayedLoading` + TableSkeleton; empty-first-load → EmptyState with OTLP snippet;
filtered-empty → "No matching logs" + clear. No `"Loading…"` strings, no `.parallax-panel`,
no KpiCard, no lucide. Full gate + both themes + leak check.

## Test plan

Unit: histogram window math (bucket click → [from,to], drag extent normalization — inverted
drags), `cols` param round-trip, severity→variant mapping totality. Component: LogsTable
renders date-ful time for a 7d window fixture; Sheet opens on row click (testing-library).

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] Histogram click AND drag narrow the window via URL; Back restores
- [ ] Multi-day windows show dates in the Time column
- [ ] Column toggle in URL; Sheet viewer restyled with copy
- [ ] Live tail intact; layout stable across mode switch
- [ ] No KpiCard/.parallax-panel/"Loading…"/lucide in scope files
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- Recharts version in repo lacks the drag-selection events used — report the alternative
  (e.g. `Brush` component) rather than silently shipping click-only.
- LogsTable prop changes would break `runs.$runId.tsx` beyond a mechanical prop rename.
- SSE frame shape mismatch vs current `LogDoc` fields.

## Maintenance notes

- The brush→window pattern established here is the template for Overview charts and issue
  trend (follow-up ports; see plans 013/016 maintenance notes).
- Column set is intentionally small; resist per-attribute dynamic columns until a real need —
  the Sheet viewer covers deep inspection.
- Any new severity levels map through one function (`severityVariant`) — extend there only.
