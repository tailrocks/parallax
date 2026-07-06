# Plan 044: Metric label discovery + runtimeSnapshot — dashboard-builder autocomplete and runtime lanes

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- crates/parallax-storage crates/parallax-api ui/src/routes/dashboards.index.tsx ui/src/routes/dashboards.\$dashboardId.tsx ui/src/routes/services.\$service.tsx ui/src/routes/runs.\$runId.tsx`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: LOW
- **Depends on**: none (playground plan 045 provides the demo data —
  Tokio/JVM metrics; the resolvers work with whatever exists)
- **Category**: direction
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

Two rungs are missing between "metrics are stored" and "Grafana-enough
dashboards" (research brief section 8): (1) the dashboard builder can pick a
metric name but cannot discover its **label keys or values**, so
"pick metric → group by label → filter value" autocomplete is impossible;
(2) runtime metric families (`process.*`, `jvm.*`, `tokio.runtime.*`,
`container.*`) can only be fetched one hard-coded name at a time — the
service-overview panel hard-codes three names — so there is no general
runtime lane on service/run views. Both are cheap: GreptimeDB's metric
engine promotes labels to real columns, and `discover_metric_names` already
proves the `information_schema` introspection path.

## Current state

Verified at commit `408be17`.

- Existing metric surface (`crates/parallax-api/src/lib.rs`):
  `metricNames(prefix)` at `:1598-1607` (client-side filter over discovered
  names); `metricSeries(name, service?, runId?, groupBy?, ...)` at
  `:1620-1678`; `histogramQuantile` at `:1681-1703`.
- Label discovery gap: `metric_series_grouped` groups on a caller-supplied
  quoted tag column (`crates/parallax-storage/src/greptime.rs:1362` region) —
  the caller must already know the label key; nothing lists keys or values.
- Introspection precedent: `discover_metric_names`
  (`greptime.rs:1498-1532`) reads `information_schema.tables`:

  ```rust
  r#"SELECT "table_name" FROM information_schema.tables ..."#
  ```

  The same mechanism against `information_schema.columns` (or
  `DESCRIBE "<metric table>"`) yields a metric's tag columns.
- Runtime reads exist but are hard-coded: `serviceOverview` fetches
  well-known names with graceful absence (`parallax-api/src/lib.rs:705-736`,
  first name at `:710` `"process.cpu.utilization"`); the bundle path uses a
  fixed list (`:1723-1727`: `process.cpu.utilization`, …,
  `tokio.runtime.alive_tasks`). `MetricStrip`
  (`ui/src/components/metric-strip.tsx:66-70`) hard-codes the same three
  series.
- Dashboard builder UI: `ui/src/routes/dashboards.index.tsx` `WidgetPicker`
  (`:241-252` usage; component defined in-file above) offers metric names
  from the loader's `metricNames`; no label picker exists.
  `ui/src/routes/dashboards.$dashboardId.tsx` renders saved widgets.
- Reserved Greptime columns to exclude from "labels": the metric-engine
  bookkeeping columns (identify them from a live `DESCRIBE` — expect
  `greptime_timestamp`, `greptime_value`; verify exact names during Step 1).

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Build | `rtk cargo build --workspace` | exit 0 |
| Lint | `rtk cargo clippy --workspace --all-targets` | zero warnings |
| Tests | `rtk cargo nextest run` | all pass |
| UI | (from `ui/`) `bun run typecheck && bun run lint && bun run test && bun run build` | exit 0 |

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs`, `greptime.rs`, `memory.rs`:
  `metric_labels(name)`, `metric_label_values(name, label, from, to)`,
  `runtime_snapshot(scope, from, to)` (scope = service | run)
- `crates/parallax-api/src/lib.rs`: `metricLabels`, `metricLabelValues`,
  `runtimeSnapshot` resolvers
- `ui/src/lib/api.ts`; `ui/src/routes/dashboards.index.tsx` (label picker in
  WidgetPicker); `ui/src/routes/services.$service.tsx` and
  `ui/src/routes/runs.$runId.tsx` (Runtime section)
- test files

**Out of scope**:
- Exemplars (advisor-plans/033), PromQL/metric-math/SLO panels (brief's
  later items), anomaly overlays.
- High-cardinality guardrails beyond a denylist for group-by (`trace_id`,
  `run_id`, `user_id`, `session_id` — brief's cross-cutting rule; implement
  the denylist, defer anything fancier).
- Replacing MetricStrip (it stays; the Runtime section is additive).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: `metric_labels` + `metric_label_values` (storage)

Greptime impl: `metric_labels(name)` → columns of the metric's table minus
reserved/bookkeeping columns (verify reserved names against a live
`DESCRIBE` of a seeded metric; record them in a constant with a comment).
`metric_label_values(name, label, from, to)` → bounded
`SELECT DISTINCT "<label>" ... WHERE ts BETWEEN ... LIMIT 100` (identifier
quoting: reuse the exact quoting/validation used by `metric_series_grouped`
at `greptime.rs:1362` — labels must be validated against the discovered
column list before interpolation, never raw user input into SQL). Memory
impl: derive from stored points' attributes.

**Verify**: `rtk cargo nextest run` → memory tests green; clippy clean.

### Step 2: Resolvers + group-by denylist

`metricLabels(name: String!): [String!]!`,
`metricLabelValues(name: String!, label: String!, fromNanos: String!, toNanos: String!): [String!]!`.
In BOTH the resolver and the existing `metricSeries` group-by path, reject
`trace_id`/`run_id`/`user_id`/`session_id` (and dotted variants
`*.trace_id` etc.) with a clear FieldError ("high-cardinality identifier —
filter, don't group"). Check what `metricSeries` currently does with an
unknown groupBy and keep its error style.

**Verify**: resolver tests: labels listed; denylisted group-by rejected;
label values bounded. `rtk cargo nextest run` green.

### Step 3: `runtimeSnapshot` (storage + resolver)

`runtime_snapshot(scope, from, to)`: enumerate known runtime families by
prefix over discovered metric names (`process.`, `system.`, `jvm.`,
`tokio.runtime.`, `container.`, `db.client.connection.`), fetch each
matching metric's series scoped to the service or run (reuse
`metric_series`'s scoping — run scope precedent `run_metric_points`
mentioned around `greptime.rs:94-100`; verify), and return grouped
`{ family, metric, unit?, points }`. Missing families → empty (graceful
absence, same rule as `serviceOverview` `:705-736`). Resolver:
`runtimeSnapshot(service: String, runId: String, fromNanos: ..., toNanos: ..., stepSeconds: Int!)`
— exactly one of service/runId required.

**Verify**: resolver test with memory adapter seeded with `jvm.gc.time` +
`process.cpu.utilization` points → two families returned; cargo gates green.

### Step 4: Dashboard builder — label autocomplete

In `WidgetPicker` (`ui/src/routes/dashboards.index.tsx`): when a metric is
chosen, fetch `metricLabels(metric)`; render a "Group by" select of the
returned labels (plus "none"); on label choice optionally fetch
`metricLabelValues` for a filter select. Persist the choice into the widget
layout JSON (check `serializeWidgets`/`parseLayout` in the same file for the
schema — extend it back-compatibly: old layouts without the new keys must
still parse; write the migration-tolerant parse first and a test for it if
the parse helpers have tests — check `ui/src/routes/__tests__/`).
`dashboards.$dashboardId.tsx` rendering: pass the stored groupBy through to
its `metricSeries` query (check how the detail page builds queries today and
mirror).

**Verify**: (from `ui/`) `bun run typecheck && bun run test` → exit 0
including the layout-parse compatibility test.

### Step 5: Runtime section on service detail + run detail

Add a "Runtime" card section to `ui/src/routes/services.$service.tsx` and
`ui/src/routes/runs.$runId.tsx`: call `runtimeSnapshot` for the page's
scope/window; render one small chart per returned metric grouped under
family headings (reuse the exact chart shape from `MetricStrip` — same
`ChartContainer`/`LineChart` pattern, `metric-strip.tsx:131-161`). Families
absent → section renders only what exists; all absent → render nothing.

**Verify**: `bun run typecheck && bun run build` → exit 0. Manual: with the
playground running (any state), service detail shows at least the `process.*`
family for Rust services; record the check. (Rich Tokio/JVM lanes appear
after playground plan 045.)

## Test plan

- Storage memory tests: label listing, label values bounded+distinct,
  runtime families grouped by prefix.
- Resolver tests: the three new queries + denylist rejection.
- UI: layout-schema back-compat parse test; existing dashboard tests stay
  green.

## Done criteria

- [ ] cargo build/clippy/nextest green with new tests
- [ ] `metricLabels`/`metricLabelValues`/`runtimeSnapshot` queryable;
      group-by denylist enforced in metricSeries too
- [ ] Widget layouts round-trip old JSON (compat test) and persist label
      choices
- [ ] Runtime sections render on service + run detail with graceful absence
- [ ] UI gates exit 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- The metric engine's table-per-metric assumption doesn't hold for some
  names (e.g. dotted names mangled) — report actual naming before writing
  the introspection SQL.
- Label validation cannot be made allowlist-based (discovered columns) for
  some reason — do NOT interpolate unvalidated identifiers; STOP.
- Widget layout schema change breaks existing saved dashboards in a way
  back-compat parsing can't absorb — report before migrating stored rows.

## Maintenance notes

- Playground plan 045 emits Tokio/JVM/container metrics — the full demo.
  Advisor-plans/033 (exemplars) will decorate these same panels later.
- The family-prefix list is a constant — extend it when new runtimes appear
  (e.g. browser web vitals from plan 050 land as metrics or events; decide
  there).
- Reviewer: SQL identifier handling (Step 1/2) is the security-sensitive
  spot — allowlist only; check the denylist also guards the legacy
  `metricSeries(groupBy:)` path.
