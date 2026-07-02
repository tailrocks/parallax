# Plan 003: Convert Data Pages To Dense Observability Panels

> **Executor instructions**: Follow this plan step by step. Run every verification command and confirm expected result before next step. If any STOP condition occurs, stop and report. When done, update `plans/README.md`.
>
> **Drift check (run first)**: `rtk git diff --stat 8dde008..HEAD -- ui/src/routes ui/src/components ui/src/styles.css`
> If any in-scope file changed since this plan was written, compare "Current state" excerpts against live code before proceeding; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED, broad UI surface but behavior should stay unchanged
- **Depends on**: `plans/001-reference-style-theme-tokens.md`, `plans/002-dark-product-shell.md`
- **Category**: direction
- **Planned at**: commit `8dde008`, 2026-07-03
- **Completed**: 2026-07-03

## Why This Matters

visual reference looks polished because dashboard data is packaged into dense metric cards, trace/workflow panes, colored status marks, and compact controls. Parallax currently exposes useful data mostly as plain tables/forms. This plan keeps loaders, GraphQL contracts, routes, and domain behavior unchanged while replacing presentation with reference-style observability panels.

## Current State

- `ui/src/routes/issues.index.tsx:133-280` renders a basic title, filter row, empty text, and a raw `Table`.
- `ui/src/components/metric-strip.tsx:95-148` renders one `Card` with three small charts, but uses neutral chart color `var(--chart-1)` from old grayscale tokens.
- `ui/src/routes/dashboards.index.tsx` uses default card/forms and raw labels for dashboard creation.
- `ui/src/components/ui/card.tsx:15` uses default `rounded-xl bg-card py-(--card-spacing) ... shadow-xs ring-1`.

Current issue page excerpt:

```tsx
// ui/src/routes/issues.index.tsx:133
return (
  <div className="space-y-4">
    <div className="flex flex-wrap items-center gap-2">
      <h1 className="text-lg font-semibold">Issues</h1>
      <span className="text-sm text-muted-foreground">{issues.total} matching</span>
    </div>
```

reference product evidence gathered 2026-07-03:

- Dashboard overview shows page title plus subtitle, then four compact KPI cards.
- KPI cards include icon tile, label, large value, delta, tiny chart/bars, and muted secondary value.
- Left nav active row uses colored icon tiles; content cards use dark raised surfaces and subtle borders.
- Public copy says visual reference surfaces cost, latency, errors, eval scores, traces, alerts, and per-agent spend; for Parallax, analogous metrics are issues, traces, logs, services, runs, dashboards, SQL, CPU/memory/tasks, event counts, and freshness.

## Commands You Will Need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Lint | `rtk bun run lint` | exit 0 |
| Tests | `rtk bun run test` | exit 0 |
| Build | `rtk bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/routes/issues.index.tsx`
- `ui/src/routes/issues.$fingerprint.tsx`
- `ui/src/routes/runs.index.tsx`
- `ui/src/routes/runs.$runId.tsx`
- `ui/src/routes/traces.index.tsx`
- `ui/src/routes/traces.$traceId.tsx`
- `ui/src/routes/logs.tsx`
- `ui/src/routes/services.tsx`
- `ui/src/routes/dashboards.index.tsx`
- `ui/src/routes/dashboards.$dashboardId.tsx`
- `ui/src/routes/sql.tsx`
- `ui/src/components/metric-strip.tsx`
- New presentational components under `ui/src/components/` if they reduce duplication, e.g. `kpi-card.tsx`, `page-heading.tsx`, `signal-panel.tsx`
- `ui/src/styles.css` only for small utility additions

**Out of scope**:
- GraphQL query shape changes unless strictly needed for already-loaded data
- Backend/API/crate changes
- Route path changes
- Data semantics or sorting defaults
- Package installs

## Git Workflow

Work on current branch unless operator says otherwise. Commit style follows existing conventional commits. If committing, use `rtk git commit -s` and include `Co-authored-by: Codex <codex@openai.com>`.

## Steps

### Step 1: Create Shared Page And KPI Components

Add presentational components, keeping them data-agnostic:

- `PageHeading`: eyebrow/status optional, title, description, right action area.
- `KpiCard`: icon, label, value, delta/status, secondary text, optional sparkline/bars.
- `SignalPanel`: raised `.parallax-panel` wrapper using shadcn `Card` composition internally if practical.

Use `Card`, `Badge`, `Button`, `Table`, `ChartContainer`, and lucide icons. Do not make cards inside cards.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 2: Redesign Issues Index

In `issues.index.tsx`, keep loader/search/update behavior unchanged. Change layout:

- Page heading: `Issues`, subtitle like "Open failure groups across services."
- Top KPI strip: total issues, open issues, events, last seen freshness. Use values already in `issues.items` and `issues.total`; if exact open count needs all rows but only 100 loaded, label it clearly as "loaded open".
- Filter row becomes compact pill-like toolbar inside a panel, not floating controls on white.
- Table lives in a raised panel. Rows get colored severity/status indicators, mono fingerprint/tag chips, and a richer trend column using brand/chart tokens.
- Empty state becomes a panel with clear command/code, not plain paragraph.

Do not alter URL search params or query args.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 3: Redesign Detail Pages Around Inspector Layouts

For issue, run, trace detail pages:

- Use two-column desktop layout: main evidence/timeline left, inspector metadata right.
- Use sticky or visually persistent right panel only if it does not overlap content.
- Use colored badges/icons for signal type: issue, trace, log, metric, run.
- Keep existing links and data actions intact.

Apply same component vocabulary across details so the app feels one product.

**Verify**: `rtk bun run build` -> exit 0.

### Step 4: Upgrade Metrics And Charts

In `metric-strip.tsx` and dashboard pages:

- Use distinct chart colors per metric from Plan 001.
- Reduce chart chrome, matching visual reference compact sparklines.
- Put current value and unit above chart where data exists.
- Keep `MetricStrip` rendering `null` when no points, unless the parent page needs explicit empty state.

**Verify**: `rtk bun run typecheck` -> exit 0.

### Step 5: Redesign Logs, Services, Dashboards, SQL

Apply same product system:

- Logs: dense table panel + severity color rail, detail sheet uses dark panel styling.
- Services: service cards/table with latency/error/log/trace summaries if already available; no new backend fields.
- Dashboards: creation form becomes compact configuration panel; dashboard cards become raised panels with metric preview.
- SQL: workbench gets dark editor-like panel, mono output table, result metadata/status pill.

Do not change user workflows; only presentation.

**Verify**: `rtk bun run lint` -> exit 0.

### Step 6: Run Full UI Gate

Run all UI gates.

**Verify**:

- `rtk bun run typecheck` -> exit 0
- `rtk bun run lint` -> exit 0
- `rtk bun run test` -> exit 0
- `rtk bun run build` -> exit 0

## Test Plan

- Existing tests must pass.
- Add lightweight component tests only if new components have conditional behavior worth locking:
  - `KpiCard` renders label/value/status.
  - Empty-state component renders command/code without backend.
- Manual screenshot checks required:
  - Desktop 1440px `/issues`, one detail route if sample API data is available, `/dashboards`, `/sql`.
  - Mobile 390px `/issues` and shell nav.

## Done Criteria

- [x] Main data pages use dark raised panels and shared page/KPI components.
- [x] Search/filter/sort/navigation behavior unchanged.
- [x] No route paths changed.
- [x] No backend/API files changed.
- [x] Charts use non-gray brand/data colors.
- [x] Empty states look like product panels, not loose paragraphs.
- [x] `rtk bun run typecheck`, `rtk bun run lint`, `rtk bun run test`, and `rtk bun run build` exit 0.

## STOP Conditions

- Any page needs new backend fields to match the design.
- Any route has no current source or has moved since planned commit.
- Component changes require modifying shadcn generated component internals broadly; prefer wrapping/composing locally.
- Responsive layout causes text overlap at 390px width.

## Maintenance Notes

Reviewers should compare screenshots against the visual reference's structural cues, not literal copy: dark product frame, compact metric cards, colored signal accents, and dense observability panels. Keep Parallax terms and workflows intact.
