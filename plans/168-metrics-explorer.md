# Plan 168: Metrics explorer — browse catalog, detail view with group-by breakdown, graduation to dashboards and alerts

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> (browser-verification checklist applies after every step against playground
> metric scenarios). Read plan 105's decision gate first — this plan builds
> the product surface on top of whatever metric-summary contract 105
> records; if 105's decision record does not exist yet, execute 105's
> decision step as part of Step 0 here (same operator authority, recorded in
> `docs/research/decisions/metric-summary-contract.md`). STOP conditions
> binding. Update this plan's status row in `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- crates/parallax-api crates/parallax-greptime ui/src/routes ui/src/components/nav.ts plans/105-metric-overview-and-trends.md`
> `<wave2-base>` = the `main` commit closing Wave 1 (plan 159's evidence commit `0e0e794`).

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (depends on the plan-105 contract decision; native
  per-metric tables have sharp edges)
- **Depends on**: plans 162, 164 (attributeFilters + where-editor reuse),
  167 (alert graduation target — soft: graduation button hidden if 167
  absent); reconciles plan 105
- **Category**: direction / product / metrics
- **Planned at**: `2288011`, 2026-07-17

## Preliminary work landed (helper agent, 2026-07-17) — peer verify/extend

**Do not retire yet.** Pure aggregation legality + URL/graduation codec only.
Index status stays TODO.

**Already landed:**
- `/metrics` + `/metrics/$metricName` routes + Metrics nav entry (`f402da7`,
  UI gates green): browse list over `metricNames` with search + inferred-kind
  filter; detail view with agg/groupBy/step selects **restricted to what the
  current backend accepts** (`metricSeries` avg/min/max/sum/rate,
  `histogramQuantile` p50/p95/p99 — see `supportedAggregations` in the route),
  grouped line chart, URL-encoded state. Peer replaces the intersection list
  with the full legality table once `metricQuery` lands, adds catalog
  richness, breakdown click-to-filter, where-filter, dashed tail, and
  graduation buttons, then browser-verifies against `m-labels`/`m-shapes`.
- `docs/research/decisions/metric-summary-contract.md`: the operator-authorized
  Plan-105/168 Step-0 contract now records exact window/count semantics,
  non-finite and histogram treatment, trend bucket cap, fail-closed native-name
  collision behavior, metric-only service discovery, GraphQL compatibility,
  and the retained `parallax metrics --invocation` promise. Plan 105 links the
  record. `product.metric-decision` pins the Markdown SHA-256 and typed decision
  fields with positive, missing, malformed, incomplete, and per-field mutation
  fixtures. Focused xtask tests and strict xtask clippy pass. Peer must prove
  adapter/API/CLI conformance before treating Step 0 as closed.
- CLI prerequisite found during helper prototyping: do **not** implement
  `parallax metrics --invocation` through `runtimeSnapshot`. That resolver
  recognizes only runtime families and discovers native metric tables before
  per-metric reads; it can falsely report a custom-only invocation as empty and
  violates the contract's bounded/no-native-tag-scan rule. Step 1's canonical
  `metricQuery` (or a dedicated typed projection sharing its single bounded
  storage path) must expose all `invocation_metric_points` first. Peer then
  wires the CLI with typed GraphQL DTOs, exact unknown/known-empty semantics,
  effective step in JSON, retired `--run` rejection, and live snapshots.
- `ui/src/lib/metric-aggregation.ts` (typecheck fixed at `90527b4`): typed
  aggregation legality per metric kind (contract decision 2 — illegal combos
  unrepresentable), `coerceAggregation`, `inferMetricKind`, `MetricQuerySpec`
  encode/decode URL codec (`q/type/agg/where/groupBy/step`),
  `encodeGraduationParams` (dashboard/alert handoff incl. `signal_type=metric`),
  and `clampCounterDelta` for reset clamping.
- `ui/src/lib/__tests__/metric-aggregation.test.ts` — real unit tests over the
  shipped helpers (legality matrix, coerce, infer, URL round-trip, graduation
  params, counter-reset clamp). Green under `bun run test:ci` for this file.
- Playground `m-labels` scenario landed on the playground's main at
  `2083a89`: gauge `shapes.region.load` + monotonic sum
  `shapes.region.requests_total`, `region` ∈ eu/us/ap at fixed 6/3/1
  magnitudes over 4×5-minute timestamps, unit-tested; wired into
  `corner-cases.sh`, `run.sh`, and `docs/corner-case-matrix.md`. Run
  `scenarios/run.sh m-labels`; peer supplies live evidence.

**Peer owns:** verify codec against real route schemas; Step 0 plan-105
decision record; backend catalog/query; `/metrics` routes; graduation wiring;
`m-labels` / `m-shapes` scenario evidence; browser pack under
`docs/research/validation/2026-07-wave2/168/`; full Done then retire.

## Why this matters

Metrics are Parallax's weakest signal today: they surface only inside
dashboards, the service detail page, and the invocation MetricStrip — there
is no way to answer "what metrics exist, what does this one look like,
broken down by what?". The reference product's metric explorer closes
exactly that loop and adds the key retention move: a query you built while
exploring **graduates** into a dashboard widget or an alert rule in one
click, so exploration output is never thrown away.

## Reference (self-contained)

From Maple (`apps/web/src/routes/metrics`, `metric-detail.tsx`,
`metric-graduation-actions.tsx`): metrics index = searchable browse
grid/table (name, type, unit, services emitting it, datapoint freshness);
metric detail = chart + query controls (aggregation legal for the metric
type — sum/rate for monotonic sums, avg/min/max for gauges, percentile for
histograms; where-filter; group-by attribute; bucket step) + breakdown
panel (top series by the group-by; clicking a series appends a
where-filter) + metadata panel (services, first/last seen, datapoint
counts) + graduation actions (add-to-dashboard, create-alert) carrying the
current query spec. Everything URL-encoded (`q/type/agg/where/groupBy/step`).

## Current state

(verified at `2288011`)

- Backend already exposes the primitives (`crates/parallax-api/src/lib.rs`):
  `metric_names (:258)`, `metric_labels (:261)`, `metric_label_values
  (:264)`, `metric_series (:278)`, `histogram_quantile (:282)`,
  `metric_exemplars (:285)`; GreptimeDB native per-metric tables resolved by
  `crates/parallax-greptime/src/greptime_sql.rs:43 native_metric_base` /
  `:55 metric_table_candidates` (histograms split `_bucket/_count/_sum`).
- Plan 105 (`plans/105-metric-overview-and-trends.md`) is decision-gated:
  metric summary contract (eligible kinds, NaN/stale handling, trend
  buckets, native-name mapping, metric-only service discovery) +
  the `parallax metrics --invocation` CLI decision. This plan does NOT
  duplicate 105's overview/trend stubs work; it builds the explorer
  surface, and both consume the same decision record.
- UI: no `/metrics` route; nav has no Metrics entry; dashboards
  (`routes/dashboards.*`) have widget editing where graduation lands;
  plan-167 alert create form is URL-initializable.
- Playground `m-shapes` scenario (plan 161) provides counter reset, gauge
  gaps, exponential + explicit histograms, exemplars.

## Contract decisions (fixed)

1. Explorer is read-only over native tables — no new storage.
2. Aggregation legality is typed per metric kind (sum→rate/increase/sum;
   gauge→avg/min/max/last; histogram→p50/p95/p99/avg via
   `histogram_quantile`); illegal combinations are unrepresentable in the
   UI (select options filtered by kind).
3. Rate for monotonic sums = window-function delta over series identity
   (attribute-set fingerprint), computed in `parallax-greptime` SQL;
   counter resets clamp to ≥0 (test against `m-shapes`).
4. Graduation passes a serialized query spec via URL params to the
   dashboard widget editor and the alert form (`signal_type=metric`);
   no hidden state channel.
5. Generic-attributes-only: group-by/where operate on whatever label keys
   exist — no special-cased metric names beyond the semconv constants
   already used for RED charts.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Backend | `cargo nextest run --locked -p parallax-greptime -p parallax-api` | pass |
| Live engine | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | pass |
| UI gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Corpus | playground `scenarios/run.sh m-shapes` + demo load | metrics present |

## Scope

**In scope:**
- `crates/parallax-greptime` — metric catalog query (names + kind + unit +
  emitting services + last datapoint), rate/increase helpers with reset
  clamping, group-by series query (top-N by current window).
- `crates/parallax-api` — `metricCatalog(q, kind, limit)`,
  `metricQuery(spec)` (single entry point the explorer, dashboards, and
  alerts share), tests.
- `ui/src` — `/metrics` + `/metrics/$metricName` routes, nav entry
  (primaryNav, chart icon), browse table (search + kind filter), detail
  view (controls + Recharts chart with plan-162 tokens + incomplete-bucket
  dashed tail + breakdown panel + metadata + graduation buttons), URL
  schemas.
- `plans/105-metric-overview-and-trends.md` — reconciliation edit: its
  overview/trend stub work now consumes `metricQuery`; note added.

**Out of scope:** dashboard builder changes beyond accepting the
graduation params; exemplar deep-linking redesign; new metric ingestion;
SLOs/Apdex (defer — record in index rejected/deferred list).

## Git workflow

- Work directly on `main` in BOTH repositories — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 0: Decision record

If `docs/research/decisions/metric-summary-contract.md` absent, write it
per plan 105's Decision Gate (window, eligible kinds, NaN/stale, buckets,
native-name mapping, metric-only service discovery, `--invocation` CLI
verdict), stamped operator-directive 2026-07-17 (this /improve directive
authorizes the explorer; the record makes it durable).

**Verify**: decision file exists with all gate items answered.

### Step 1: Catalog + query backend

Queries + GraphQL + live-engine tests against `m-shapes`: catalog lists the
seeded metrics with correct kinds; rate clamps the counter reset; histogram
p95 sane; group-by top-N returns the seeded label split.

**Verify**: cargo lanes pass.

### Step 2: Browse + detail UI

Routes, controls (kind-filtered aggregations), breakdown click-to-filter,
metadata, incomplete-bucket dashed tail (last bucket rendered as a dashed
continuation series), URL round-trip.

**Verify**: component/route tests; UI gates green.

### Step 3: Graduation + browser closure

Add-to-dashboard (opens widget editor pre-filled) and create-alert (opens
plan-167 form pre-filled; hidden when 167 absent). Browser walk per
checklist against `m-shapes` + demo load: browse → open a histogram → p95 +
group-by service → click a series to filter → graduate to alert →
screenshots to `docs/research/validation/2026-07-wave2/168/`.

**Verify**: evidence complete; permalink reload reproduces the exact chart.

## Playground verification

`m-shapes` (plan 161) covers reset/gaps/histograms/exemplars. New scenario
(direct on the playground's main): `m-labels` — one gauge + one sum emitted with a 3-value label
(`region` ∈ eu/us/ap, fixed proportions) so group-by breakdown output is
exactly assertable.

## Done criteria

- [ ] Decision record exists; plan 105 reconciled (note added, no
  duplicated scope).
- [ ] Backend lanes green incl. reset-clamp and group-by live tests.
- [ ] UI gates green; browser evidence incl. graduation round-trip and
  permalink reproduction.
- [ ] `m-labels` scenario + matrix row landed on the playground's main.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- Native per-metric table name resolution fails for corpus metric names
  (unicode/dots edge cases) — report exact names; table-mapping changes are
  a 105-contract matter.
- Rate-over-window SQL unsupported by the pinned GreptimeDB version —
  report the exact error + version; do not emulate with client-side math.
- Graduation requires dashboard-builder changes beyond accepting params.

## Maintenance notes

- `metricQuery(spec)` is the single metric read path — dashboards (133-era
  work), alerts (167), and the explorer must not fork their own metric SQL.
- Reviewer focus: aggregation legality typing, reset clamping, URL schema.
