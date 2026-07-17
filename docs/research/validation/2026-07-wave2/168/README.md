# Plan 168 — Metrics explorer

**Status:** DONE (2026-07-17, completed with full graduation/where evidence)

## Closed claims

| Claim | Evidence |
|---|---|
| `metricCatalog` GraphQL (kind/unit/services/count/freshness) | Live: 378-metric 7-day catalog (after the UNION-ALL batching fix); browse screenshot below |
| `metricQuery` shared read path, full legality table | gauge `avg\|min\|max\|last`, sum `sum\|rate\|increase`, histogram `p50\|p95\|p99\|avg`; resolver tests + live assertions below |
| Where-filters (`attributeFilters`) | Backend threaded through metric_series/grouped/histogram paths; live `region = eu` narrows to one series; UI where-clause editor on the detail page |
| Breakdown click-to-filter | Clicking a group badge pins `groupBy = value` as a where filter (`detail-click-filter-eu.png`) |
| Sum-bucket zero-fill; honest gauge/histogram gaps | `metric_query` zero-fills sum-family windows (resolver test); live `increase` shows `[0,…,400]` |
| Dashed incomplete tail | Newest bucket rendered as dashed continuation series (detail screenshots) |
| Graduation → alert | `signal_type=metric` URL params open the pre-filled plan-167 dialog; rule `metric > 50 over 5m` created live (`graduate-alert-*.png`) |
| Graduation → dashboard | `widget_metric/widget_agg/widget_group_by` params open the create dialog pre-filled; dashboard created and widget renders the grouped query (`graduate-dashboard-*.png`) |
| Permalink reproduction | Re-opening the exact URL reproduces the filtered 1-series chart (`permalink-reload.png`) |
| Plan-105 Step-0 contract + reconciliation | `docs/research/decisions/metric-summary-contract.md`; note added to plan 105 |
| Playground `m-labels` / `m-shapes` | playground `2083a89`; both scenarios re-run live on 2026-07-17 |

## Live GraphQL assertions (operator host, 2026-07-17)

Against `parallax serve` (managed GreptimeDB) with fresh `m-labels` +
`m-shapes` scenario emissions:

- `metricQuery(shapes.region.load, gauge, last, groupBy: region)` →
  three series at the seeded 6/3/1 magnitudes (eu 60 / us 30 / ap 10).
- `attributeFilters: [region = eu]` → single series, value 60 only.
- `metricQuery(shapes.region.requests_total, sum, increase)` → zero-filled
  window with the emitted growth (400) in its bucket.
- `metricQuery(shapes.requests_total, sum, rate)` over the m-shapes counter
  reset → all values ≥ 0 (reset clamps, never negative).
- `metricQuery(shapes_request_duration, histogram, p95)` → 0.5 plateau
  from the seeded explicit buckets; `avg` returns an empty series because
  repeated single-export runs have Δcount = 0 (contract: no fabricated
  samples).
- 7-day, 500-limit `metricCatalog` returns 378 rows with the engine healthy.

[live-metric-query.json](./live-metric-query.json) — earlier preliminary
catalog + rate sample.

## Browser walk (agent-browser, vite dev + live API)

1. `browse-shapes.png` — /metrics?q=shapes: kinds, unit, services,
   datapoints, freshness columns.
2. `detail-histogram-p95.png` — histogram detail; group-by hidden for
   histograms (illegal combos unrepresentable).
3. `detail-gauge-groupby-last.png` — gauge `last` grouped by region,
   3 series.
4. `detail-click-filter-eu.png` — breakdown badge click pins
   `where=region = eu`, chart narrows to 1 series.
5. `permalink-reload.png` — same URL reloaded fresh reproduces the chart.
6. `graduate-alert-prefilled.png` / `graduate-alert-created.png` —
   create-alert graduation round-trip (rule `metric > 50 over 5m`).
7. `graduate-dashboard-prefilled.png` / `graduate-dashboard-created.png` —
   add-to-dashboard graduation round-trip (widget renders grouped query).

## Test lanes

- `cargo nextest run --locked -p parallax-greptime -p parallax-api
  -p parallax-storage -p parallax-test-support` — green.
- Full ignored live-engine lane `--run-ignored all -E 'binary(/greptime/)'`
  — 7/7 green (after the JSON-attribute-path and facet-column fixes).
- UI gates (typecheck, lint, check, test:ci) — green.

## Defects found and fixed by this verification

- Giant catalog UNION ALL killed standalone GreptimeDB (batched 24 arms
  per query).
- `reap_stale_child` SIGTERMed a live engine owned by another serve
  (now orphan-only).
- `resource_json_path` emitted backslash-escaped member quotes: every log
  attribute where-filter/JSON facet silently matched nothing on the live
  engine (unit tests had locked in the wrong SQL).
- Trace facets failed outright when an auto-widened attribute column was
  absent from the corpus (now degrade to an empty facet).
