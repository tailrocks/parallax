# Plan 015: Services — health index table + per-service RED detail

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/services.tsx`
> Plans 005-009 must be DONE (009 provides `serviceList` + `serviceRed`).

## Status

- **Priority**: P1
- **Effort**: M/L
- **Risk**: MED
- **Depends on**: plans/005-008, plans/009
- **Category**: tech-debt (UX redesign) + spec gap (request/error rate required by spec)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Today Services is a single-service chart page: one `Select` picks a service, the page fans
out N per-metric queries client-side (`services.tsx:74-92` pattern), shows CPU/memory and
HTTP/gRPC duration percentiles — but **no request rate, no error rate** (two of the three RED
signals the spec requires at `docs/research/architecture/simple-ui-v2.md:58-61`), no list of
all services, a hardcoded 1h window, and zero links onward (dead end; `ui/AGENTS.md` rule
17). The redesign: a services **index** (one row per service: RED at a glance + last seen +
sparkline, linking everywhere) and a service **detail** page on the reference's detail
grammar (stat cards → trend band → recent traces table).

## Current state

- `ui/src/routes/services.tsx`: loader loads `services` then per-metric
  `metricSeries`(CPU/memory via well-known names) + `histogramQuantile`(http/grpc
  p50/p95/p99); service chosen via header Select; window hardcoded 1h (`:70` area); charts in
  `Panel` grid; empty state at `:272` when no metrics. **Does not call `serviceOverview`**
  (client-side reimplementation).
- After plan 009: `serviceList(from,to) → [ServiceSummary{name,lastSeenNanos,spanCount,
  errorCount,p95Ms}]` (one query) and `serviceRed(service,from,to,stepSeconds) →
  {rate,errorRate,p50,p95,p99}` (span-derived — works for trace-only services);
  `signalCountSeries` for sparklines. Existing `metricSeries`/`histogramQuantile` stay for
  CPU/memory and app-histogram latency.
- Kit: StatCard, ChartLegend/thinTicks, HeatCell, RelativeTime, RangePicker +
  `resolveRangeSearch`, data-table toolkit, EmptyState, skeletons, formatters.
- Reference detail grammar: `$REF_ROOT/apps/web/src/app/(app)/agents/[name]/
  agent-detail-client.tsx` — PageHeader (back crumb + titleLeading icon + RangePicker
  action) → 4 stat cards (`:320-353`) → trend band `grid lg:grid-cols-2` with
  `h-[220px]` charts (`:356-465`; latency = stacked bands) → entity sub-table
  (`:514-753`, rows drive/expand detail). Reference list grammar: plan 011's table columns
  + heat cells.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + serve +
multi-service playground traffic. Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/services.tsx` (becomes the index), NEW
`ui/src/routes/services.$service.tsx` (detail), `ui/src/components/nav.ts` untouched (nav
already points at `/services`).
**Out of scope**: API; other routes. If plan 011 hasn't landed yet, skip adding the
service-filter links INTO traces (leave TODO comments referencing plan 011) — do not edit
`traces.index.tsx` here.

## Git workflow

`main`; `feat(ui): services health index + detail`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: Services index (`services.tsx` rewrite)

`validateSearch`: `{q, range, from, to, sort}`. One loader query: `serviceList(from,to)`.
Layout: `PageHeader` (Services icon, description, actions=RangePicker) → `Toolbar`
(SearchInput client-filter by name, count right) → `Table` (interactive rows →
`/services/$service`):

| Column | Treatment |
|---|---|
| Service | name `font-medium` + emerald `SpanKindChip`-style dot; error>0 rows get rose left-accent bar |
| Spans | right `w-28` sortable `formatCount` |
| Errors | right `w-28` sortable; >0 → rose text; 0 → `text-muted-foreground/40` |
| Error rate | right `w-28`; errorCount/spanCount `formatPercent`, HeatCell-tinted |
| p95 | right `w-28` sortable; `p95Ms` formatted; null → muted "—" |
| Last seen | right `w-32` sortable `RelativeTime` |

Sorting client-side (`sortRows` — the list is small). Empty → EmptyState ("No services yet"
+ OTLP snippet). Skeleton via useDelayedLoading.

**Verify**: two+ playground services listed; row click navigates; sort works; URL holds
q/sort/range.

### Step 2: Service detail (`services.$service.tsx`)

Param = service name (URL-encode on link; decode via `Route.useParams`). `validateSearch` =
range schema (default 24h — kills the hardcoded 1h). One loader document: `serviceRed(
service, from, to, stepSeconds)`, CPU/memory `metricSeries` (existing well-known-name
approach — copy the metric-name fallback lists from the current file), `histogramQuantile`
for http/grpc when present, and recent traces for the service via the plan-010 list query
(`service, sort: START_DESC, limit: 10`) — if plan 010 is not yet DONE use the legacy
`traces(service, limit: 10)` and note it.

Layout (reference detail grammar):
1. `PageHeader back={navItem("/services")}` title=service, actions=RangePicker.
2. Stat row (4 × StatCard sm; Volume→Health→Performance): Requests (sum of rate series;
   sparkline), Error rate (`deltaInverted`, rose PillMeter), p95 (latest point, hint p50),
   Last seen (RelativeTime value, no chart).
3. Trend band `grid gap-4 lg:grid-cols-2`, `h-[220px]`: "Requests & errors" (rate +
   errorRate×rate overlay, legend-toggled) · "Latency" (stacked p50/p95−p50/p99−p95 bands —
   prefer span-derived `serviceRed`; if the service has app histograms, add a legend toggle
   "app histogram" using `histogramQuantile` series).
4. Infra band (conditional): CPU + Memory charts when those metrics exist (`--chart-1/2`);
   whole band hidden when empty (no dead cards).
5. Recent traces table (compact): root name / duration HeatCell / when RelativeTime /
   error accent — rows → `/traces/$traceId`.

**Verify**: a trace-only service (no OTel metrics) shows Requests/Error rate/p95 populated
from spans — the old blank-panel failure is gone; a metrics-emitting service also shows
CPU/memory.

### Step 3: States + gate

Not-found service (empty serviceRed + no spans) → EmptyState with back link. Full gate:
typecheck/lint/test/build; both themes; leak check.

## Test plan

Unit: error-rate derivation + percent formatting; URL param encode/decode for service names
with spaces/slashes. Component: index table renders fixture rows with correct link hrefs;
detail renders 4 stat cards and hides the infra band on empty CPU series.

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] `/services` = index table (all services, RED columns, links); detail at
      `/services/$service` with range picker (no hardcoded 1h anywhere:
      `grep -n "3_600" ui/src/routes/services*.tsx` → none or justified)
- [ ] Trace-only service shows non-empty RED (manual check recorded in report)
- [ ] Recent-traces table links to trace detail (no dead end)
- [ ] No KpiCard/.parallax-panel/lucide in scope files
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- Plan-009 `serviceList`/`serviceRed` missing or renamed (check README notes + schema).
- Service names containing `/` break TanStack param routing even encoded — report; consider
  `?service=` query-param detail page as the fallback decision, don't pick silently.
- CPU/memory well-known-name lists in the current file turn out to live server-side only —
  copy the exact lists from `crates/parallax-api/src/lib.rs` (`REQUEST_DURATION_METRICS`
  area) into the plan report and proceed.

## Maintenance notes

- When plan 011 is live, add cross-links: index row's Spans/Errors cells → traces list
  pre-filtered (`/traces?service=X&errors=1`).
- The infra band's metric-name fallback lists must stay in sync with the server's
  `ServiceOverview` lists; consider exposing them via GraphQL later instead of duplicating.
- If service count grows large, switch index sorting to server-side (plan-009
  `service_summaries` already orders deterministically).
