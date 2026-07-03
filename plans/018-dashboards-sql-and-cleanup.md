# Plan 018: Dashboards + SQL redesign, then the legacy sweep (brand tokens, lucide, dead files)

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done. This is the LAST plan of the
> redesign series — its cleanup gates assume 005-017 are DONE.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/dashboards.index.tsx ui/src/routes/dashboards.\$dashboardId.tsx ui/src/routes/sql.tsx ui/src/styles.css`

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005-017 (cleanup steps hard-require ALL screen plans DONE)
- **Category**: tech-debt
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Dashboards and SQL are the remaining screens on the old grammar. The SQL page's headline
feature is broken outright: every example query targets tables that don't exist
(`otel_spans`, `otel_logs`, `otel_metrics_points` at `ui/src/routes/sql.tsx:32-70`; the real
GreptimeDB tables are `opentelemetry_traces`/`opentelemetry_logs` with `service_name`,
`duration_nano`, `severity_number`, `timestamp`, and run ids under
`"resource_attributes.parallax.run.id"` — verified against
`crates/parallax-storage/src/greptime.rs`). Dashboards' metric picker is a plain Select
(spec asks autocomplete), the builder squats permanently on the page, and saved dashboards
are not in the sidebar (spec: listed in the sidebar). After these two screens, nothing
legitimate references the legacy layer — so this plan finishes by deleting it and installing
grep gates that keep it dead.

## Current state

- `ui/src/routes/dashboards.index.tsx`: loader `dashboards + metricNames`; permanent
  create-form Card (`WidgetPicker` exported, reused by detail); grid of `.parallax-panel`
  link tiles; mutations `dashboardSave/dashboardDelete`; errors as inline
  `<p class=text-destructive>`.
- `ui/src/routes/dashboards.$dashboardId.tsx`: loader `dashboard + metricNames` then
  per-widget `metricSeries` (`:71-79`); edit mode with draft widgets (up/down/remove/add);
  `WidgetChart` Bar/Area/Line with `--chart-{n}`; `throw notFound()`.
- `ui/src/routes/sql.tsx`: `EXAMPLES` `:29-71` (broken schema, incl. `run_id = 'jk-run-…'`
  placeholders and `severity_num`/`ts`/`duration_ns` columns); schema browser reads real
  `information_schema.columns` (`:102`) — works; textarea editor ⌘⏎ (`:282` hint as plain
  text); native `<select>` for Examples/History (`:283-320`); localStorage history; results
  `<Table>`.
- Legacy layer still present (installed by plan 005 as LEGACY-COMPAT): `--brand-*` aliases,
  `.parallax-panel/.parallax-pill/.parallax-glow-border`; deprecated files
  `ui/src/components/kpi-card.tsx`, `page-heading.tsx`; `lucide-react` dependency;
  `live-stream-panel.tsx` still brand-styled.
- Kit: full plan-008 set; `combobox` primitive NOT yet added (this plan adds it);
  Dialog/Kbd/ScrollArea available since plan 006.
- Sidebar: plan 007's nav.ts; shadcn sidebar supports `SidebarMenuSub` (nested items under
  Dashboards).

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; `rtk bun remove
lucide-react`; combobox via `bunx --bun shadcn@latest add combobox` (rule 9; port the
reference's `packages/ui/src/components/combobox.tsx` recipe on top). Leak check: plan 005.

## Scope

**In scope**: `ui/src/routes/dashboards.index.tsx`, `dashboards.$dashboardId.tsx`, `sql.tsx`
(rewrites); `ui/src/components/nav.ts` + `parallax-shell.tsx` (dashboards sub-menu);
`ui/src/components/live-stream-panel.tsx` (restyle); NEW `ui/src/components/ui/combobox.tsx`;
DELETIONS: `ui/src/components/kpi-card.tsx`, `ui/src/components/page-heading.tsx`,
LEGACY-COMPAT block in `ui/src/styles.css`, `lucide-react` dep; test files updated.
**Out of scope**: API (dashboard CRUD + metricSeries suffice); Rust.

## Git workflow

`main`; separate commits: `feat(ui): redesign dashboards`, `feat(ui): redesign sql
workbench`, `chore(ui): remove legacy design layer`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: Dashboards index

`PageHeader` (Dashboards icon; actions = "New dashboard" Button opening a **Dialog** with
the builder — kills the permanent form). Builder in dialog: name Input + widget list +
"Add widget" row = metric **Combobox** (typeahead over `metricNames` — spec's autocomplete),
aggregation Select, group-by Input, chart-type Select (bar/area/line). Grid of dashboard
Cards (name, `formatCount(widgets)` hint, updated RelativeTime, delete via AlertDialog) —
interactive → detail. Empty → EmptyState "Create your first dashboard". Errors via sonner
toast, not inline `<p>`.

**Verify**: create in dialog → navigates to new dashboard; combobox filters as you type.

### Step 2: Dashboards detail

`PageHeader back` + title, actions = RangePicker (kills hardcoded 1h: pass from/to into
`metricSeries`) + Edit toggle + Delete (AlertDialog). View mode: widget grid `grid gap-4
lg:grid-cols-2`, each Card sm h-[260px] chart (plan-006 chart primitive + ChartLegend when
grouped). Edit mode: same grid + per-card controls (up/down/remove) + "Add widget" opening
the Step-1 dialog form; Save via existing `dashboardSave`.

**Verify**: range change refetches all widgets; edit round-trip preserves layout JSON shape
(the API stores `layout` as JSON string — do not change its schema).

### Step 3: Sidebar listing

`parallax-shell.tsx`: under the Dashboards nav item render `SidebarMenuSub` with saved
dashboards (name → `/dashboards/$dashboardId`), fetched lazily client-side (same `graphql`
helper; refresh on route change into /dashboards). Cap 7 + "All dashboards" link. Hidden in
icon-collapsed mode (component handles it).

**Verify**: creating a dashboard adds it to the sidebar after navigation; spec box
"listed in the sidebar" checked.

### Step 4: SQL workbench

`PageHeader` (SQL icon; description "read-only queries over telemetry tables"). Layout
`grid-cols-[16rem_1fr]`: left = schema browser Card on ScrollArea (tables → columns,
mono, click inserts identifier at cursor); right = editor Card (textarea mono, `Kbd ⌘⏎`
chip in the footer, Run Button), Examples as a `DropdownMenu` (kills native selects),
History dropdown (localStorage, unchanged mechanics), results in plan-006 Table (sticky
header, `maxHeight`, mono cells, `formatCount(rowCount)` + elapsed).
**Fix EXAMPLES** — rewrite all five against the real schema (validated names):
- slow spans + error logs join: `FROM opentelemetry_traces s JOIN opentelemetry_logs l ON
  l.trace_id = s.trace_id WHERE s.duration_nano > 10000000 AND l.severity_number >= 17
  ORDER BY s."timestamp" DESC` (select `s."timestamp", s.service_name, s.span_name,
  s.duration_nano / 1000000 AS ms, l.severity_text, l.body`).
- error events per service (last hour): `FROM error_events WHERE ts >= now() - INTERVAL
  '1 hour' GROUP BY service, error_type` (this table's columns were already correct).
- log volume by severity: `FROM opentelemetry_logs WHERE "timestamp" >= now() - INTERVAL
  '1 hour' GROUP BY service_name, severity_text`.
- run cross-section: spans/logs by run id via
  `"resource_attributes.parallax.run.id" = '<run-id>'` on both otel tables; drop the
  metric-points leg or point it at `run_metric_points` (`run_id` column exists there).
- slowest root spans: `FROM opentelemetry_traces WHERE parent_span_id IS NULL OR
  parent_span_id = '' GROUP BY span_name, service_name ORDER BY max(duration_nano) DESC`.
**Each example must be executed against a live playground-fed engine before commit**; paste
outputs (row counts) into the report. Quote `"timestamp"` (reserved word) exactly as the
storage layer does.

**Verify**: all 5 examples run without error and return plausible rows; ⌘⏎ runs; results
table scrolls with sticky header.

### Step 5: Live-stream panel restyle

`live-stream-panel.tsx`: replace `.parallax-panel.parallax-glow-border` + brand tiles with a
plan-006 Card (emerald pulsing dot chip + label + mono counters; `EmptyState`-style dashed
box for the idle state). Keep the exported API (used by traces/logs/runs).

### Step 6: Legacy sweep (hard gates)

Pre-condition: `plans/README.md` shows 005-017 DONE. Then:
1. Delete `ui/src/components/kpi-card.tsx` + `page-heading.tsx`;
   `grep -rn "KpiCard\|PageHeading" ui/src` → no hits (fix any straggler by switching to
   kit components — small mechanical edits allowed across routes here).
2. Remove the LEGACY-COMPAT block from `ui/src/styles.css`;
   `grep -rn "parallax-panel\|parallax-pill\|parallax-glow\|--brand-" ui/src` → no hits.
3. `rtk bun remove lucide-react`; `grep -rn "lucide-react" ui` → only bun.lock history if
   anything; typecheck confirms no imports.
4. `grep -rn "\"Loading…\"\|>Loading\.\.\.<" ui/src` → none.
5. Full gate: typecheck/lint/test/build; click-through all routes both themes.

**Verify**: all greps clean; build green.

## Test plan

Unit: SQL example strings contain no banned table names
(`otel_spans|otel_logs|otel_metrics_points` regex test over the EXAMPLES export);
dashboard layout JSON round-trip (parse→edit→serialize preserves unknown fields).
Component: dashboards index renders dialog on button click; sql page renders Kbd hint and
examples menu.

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] All 5 SQL examples execute successfully against a live engine (report outputs)
- [ ] Dashboards: dialog builder, combobox picker, range picker, sidebar sub-list
- [ ] Step-6 greps all clean; `lucide-react` gone from package.json
- [ ] Leak check → no output; `plans/README.md` rows updated (this plan + series complete)

## STOP conditions

- Any plan 005-017 not DONE when starting Step 6 (do Steps 1-5, mark partial, stop).
- An SQL example fails against the live engine for schema reasons you can't resolve from
  `crates/parallax-storage/src/greptime.rs` — report the actual `information_schema` output.
- Removing `lucide-react` breaks a file outside the redesign series' scope — list it.
- The Combobox CLI add fails for the Base UI variant (same rule-9 handling as plan 008's
  calendar note).

## Maintenance notes

- After this plan, the design system is closed: new UI composes kit + primitives only. Any
  new raw hex/border-separation/one-off panel is a review-blocker.
- The sidebar dashboards sub-list is the template if Issues saved-searches or pinned traces
  ever want sidebar presence.
- Keep the SQL examples in lockstep with storage schema changes — they are now tested
  strings; extend the banned-name test when tables get renamed.
