# Plan 013: Overview — the landing dashboard (KPIs, trends, recent issues/traces)

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/index.tsx ui/src/components/nav.ts`
> Plans 005-009 must be DONE (009 provides `overview`, `signalCountSeries`,
> `serviceRed`).

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plans/005-008, plans/009
- **Category**: direction (new screen; observability-norm gap)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Parallax currently has no landing view: `/` redirects straight into the Issues table
(`ui/src/routes/index.tsx:4-6`), so there is no "is anything on fire?" glance — no
throughput, error rate, latency, or recent activity. The reference console's Overview is the
model: a 4-KPI stat row with deltas and bottom-bleed sparklines, a two-chart trend band, and
ranked/recent lists that all link onward.

## Current state

- `ui/src/routes/index.tsx` — 6 lines: `beforeLoad` throws `redirect({ to: "/issues" })`.
- Nav (after plan 007): `ui/src/components/nav.ts` has a reserved Overview slot (sky chip),
  not yet inserted.
- API (after plan 009): `overview(fromNanos,toNanos)` → counts (String) + errorRate +
  activeServices; `signalCountSeries(kind, …)` for SPANS/TRACES/LOGS/ERRORS;
  `serviceRed(service?, …)` → rate/errorRate/p50/p95/p99 series. `issues(sort: LAST_SEEN,
  limit)` and `traces`/`tracesPage` (plan 010) exist for the lists.
- Kit (plan 008): StatCard (+DeltaBadge, CardSparkline, PillMeter — StatCard ordering
  doctrine: Volume → Health → Performance → Cost/none), ChartLegend + thinTicks +
  makeEdgeTick, RangePicker + `resolveRangeSearch`, EmptyState, skeletons,
  useDelayedLoading, formatters.
- Reference blueprint: `$REF_ROOT/apps/web/src/app/(app)/overview/overview-client.tsx` —
  layout: RouteHeader(withRange) → KPI `section grid gap-4 md:grid-cols-2 xl:grid-cols-4`
  (`:690`) → chart band `grid gap-4 lg:grid-cols-2` (`:762`, each chart Card `h-[260px]`)
  → breakdown band `grid gap-4 lg:grid-cols-2 xl:grid-cols-4` (`:953`); clickable header
  legends dimming unselected series to `opacity-30` (`:130-168`); latency drawn as stacked
  bands p50 / p95−p50 / p99−p95 with absolutes in the tooltip (`:438-450`); empty charts =
  blurred sample data behind a floating "No data in this range" notice (`:237-272`);
  section skeletons match final dimensions (`:172-231`).

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + serve + playground
for manual checks. Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/index.tsx` (becomes the Overview page — keep the `/` path,
no redirect), `ui/src/components/nav.ts` (prepend Overview item, sky chip,
`IconHome`/`IconHomeFilled` or `IconLayoutDashboard(+Filled)` — typecheck-verified),
NEW `ui/src/routes/__tests__/overview.test.tsx`.
**Out of scope**: Issues/Traces routes themselves; API; shell. (Decision: Overview lives AT
`/` directly — no separate `/overview` path, no redirect hop; the nav item's `href` is `/`.)

## Git workflow

`main`; `feat(ui): overview landing page`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: Route skeleton + range

`index.tsx`: drop the redirect. `validateSearch` = range schema (`resolveRangeSearch`);
loader fetches in ONE GraphQL document: `overview(from,to)`, `signalCountSeries(kind:
SPANS…)`, `signalCountSeries(kind: ERRORS…)`, `serviceRed(from,to,stepSeconds)` (no service
= all), `issues(sort: LAST_SEEN, limit: 6){items{fingerprint title service lastSeenNanos
eventCount status}}`, and the slowest traces via the plan-010 list query (`sort:
DURATION_DESC, limit: 6`). Also fetch the **previous window** `overview` (same span shifted
back) for deltas. `stepSeconds` ≈ window/60 buckets, min 30s.
Header: `PageHeader` icon = Overview nav item, title "Overview", actions = `RangePicker`
(writes `?range/from/to`).

**Verify**: typecheck; `/` renders the page (no redirect); range change refetches.

### Step 2: KPI row (Volume → Health → Performance)

`section grid gap-4 md:grid-cols-2 xl:grid-cols-4`, four `StatCard size="sm"`:
1. **Spans** — `formatCount(overview.spanCount)`, hint "`{traceCount}` traces", delta vs
   previous window, `CardSparkline` from SPANS series (sky `text-sky-500`).
2. **Logs** — count, hint "`{activeServices}` services", delta, sparkline (blue).
3. **Error rate** — `formatPercent(errorRate)`, `deltaInverted`, hint "`{errorCount}`
   errors", rose `PillMeter` (share = errorRate).
4. **p95 latency** — latest `serviceRed.p95` point (`formatDurationNs`), `deltaInverted`,
   hint "p50 `{…}`", violet sparkline from the p95 series.
Icons: filled Tabler (`IconCirclesFilled`-family for volume, `IconAlertTriangleFilled` rose
for errors, gauge-style for latency) — verify exports compile; fall back to outline.

**Verify**: values render `tabular-nums`; deltas flip color correctly (`deltaInverted` on 3
and 4).

### Step 3: Trend band

`grid gap-4 lg:grid-cols-2`, two `Card size="sm"` with `h-[260px]` chart bodies
(`ChartContainer` from the plan-006 chart primitive):
1. **Spans & errors** — area chart: spans series (`--chart-2`), errors series
   (`--destructive`); clickable header `ChartLegend`; x ticks via `thinTicks` +
   `makeEdgeTick`.
2. **Latency** — stacked bands: p50, p95−p50, p99−p95 (compute deltas client-side; guard
   negatives to 0), absolutes in the tooltip; colors: p50 `--chart-2`-neutral, p95 sky,
   p99 orange (tokens/standard hues only — no `--brand-*`).
Empty-range behavior: blurred placeholder series + floating "No data in this range" notice
(port the reference's `MaybeEmptyOverlay` idea: `opacity-50 blur-[3px]` sample + centered
chip).

**Verify**: legends toggle series dimming; empty window shows the overlay, not a blank box.

### Step 4: Recent lists

`grid gap-4 lg:grid-cols-2`, two Cards:
1. **Recent issues** — rows: status dot (open=rose/resolved=muted), title `truncate
   font-medium`, service `text-xs text-muted-foreground`, right: `formatCount(eventCount)` +
   `RelativeTime(lastSeenNanos)`; row → `/issues/$fingerprint`; rows `divide-y
   divide-border/40` inside `ScrollFade max-h-88`.
2. **Slowest traces** — rows: root name + service, right: `formatDurationNs` (HeatCell-tinted
   vs the six shown) + spans count; row → `/traces/$traceId`.
Both lists: empty → small dashed `EmptyState`. Everything links onward (interactivity rule —
`ui/AGENTS.md` rule 17).

### Step 5: Onboarding empty state

When `overview.spanCount === "0"` for the max window (90d): replace KPI+charts with a single
onboarding card — "Send your first telemetry" + OTLP endpoint (reuse the endpoint text from
the old issues empty panel) + `parallax run` CLI hint, each with CopyButton. Lists hidden.

**Verify**: against a fresh data dir, `/` shows onboarding; after playground traffic, the
dashboard.

### Step 6: Nav + gate

Prepend Overview to `nav.ts` (sky chip recipe, `href: "/"`, exact-match active so it doesn't
highlight for every route — the plan-007 `isActive` treats `/` specially: active only on
exact `/`). Full gate: typecheck/lint/test/build; both themes; leak check.

## Test plan

`overview.test.tsx`: mock `graphql` → fixture payload; assert 4 StatCards render with
labels Spans/Logs/Error rate/p95 latency; delta badge direction for inverted metrics; recent
lists render links with correct hrefs; zero-data fixture renders onboarding card. Pure-fn
test for the p95-band delta computation (negative guard).

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] `/` renders Overview (no redirect); nav shows Overview first, active only on `/`
- [ ] KPI row + 2 trend charts + 2 linked lists render from plan-009 queries
- [ ] Empty-data onboarding path works
- [ ] No lucide/`--brand-*`/`.parallax-panel`/`KpiCard` in the route
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- Plan-009 queries missing or shaped differently (check schema first via one probe query).
- Previous-window delta fetch doubles latency unacceptably (>2s locally) — ship without
  deltas and note it, don't block the page.
- Stacked-band latency rendering fights the chart primitive — fall back to three plain lines
  with a note (deviation, not STOP).

## Maintenance notes

- When plan 015 lands, the KPI row's "services" hint can link to `/services`.
- The charts intentionally aren't brushable yet; plan 014 establishes the brush→window
  pattern on the logs histogram — port it here afterwards as a follow-up.
- METRIC_POINTS series was allowed to be a v1 gap in plan 009; if it lands later, consider a
  fifth sparkline, not a fifth KPI (keep 4).
