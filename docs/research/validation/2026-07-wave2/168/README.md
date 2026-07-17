# Plan 168 — Metrics explorer

**Status:** DONE (2026-07-17)

## Closed claims

| Claim | Evidence |
|---|---|
| `metricCatalog` GraphQL | Live QA list (gauge/sum/histogram kinds) |
| `metricQuery` shared read path | Backend tests + live rate query on `cache_hits_total` |
| `/metrics` browse uses catalog | `metrics.index.tsx` loader |
| `/metrics/$metricName` uses `metricQuery` | `metrics.$metricName.tsx` loader with legacy fallback |
| Aggregation legality helpers | `metric-aggregation.ts` unit tests |
| Plan-105 Step-0 contract | `docs/research/decisions/metric-summary-contract.md` |
| Playground `m-labels` | playground `2083a89` |

## Live GraphQL

[live-metric-query.json](./live-metric-query.json) — catalog rows +
`metricQuery(cache_hits_total, sum, rate)`.

## UI

Routes wired to the shared GraphQL path; graduation buttons and full
where-filter remain incremental polish (soft 167 handoff already available
via alert rule form `signal_type=metric`).

## Tests

- API metricCatalog / metricQuery resolver tests (prior commits)
- UI typecheck green after catalog/query wiring
