# Plan 105 — Metric overview and trends

**Status:** DONE (2026-07-17)

## Closed claims

| Claim | Evidence |
|---|---|
| `metric_point_count` stub replaced | GreptimeDB overview totals count windowed finite samples (one per histogram export) via one schema scan + chunked UNION ALL; live overview returned 44,821 (1h) / 532k (24h) |
| `MetricPoints` trend stub replaced | `signalCountSeries(kind: METRIC_POINTS)` buckets the same counts; live 1h window returned four non-zero buckets |
| Metric-only service discovery | `service_names` and overview active-services include services seen only in native metric tables (m10 live assertion) |
| Canonical invocation projection | `canonical_name` persisted at ingest via the shared `parallax_semconv::native_metric_table_base`; legacy rows normalize client-side with the same deterministic function — never a catalog scan |
| `invocationMetrics` GraphQL | Bounded typed projection (finite-only, canonical grouping, name-ascending); resolver tests incl. unknown-vs-known-empty |
| `parallax metrics --invocation` CLI | Live snapshot table + `--json` (effective window included); unknown invocation errors; retired `--run` always rejected |
| Adapter conformance | `m10_metric_summaries_greptime` (live engine, 1/1 green): finite counting, NaN exclusion, trend parity, canonical naming, MemoryStore parity over identical seeds |
| UI honest state | Overview "Metric points" tile with previous-window delta (screenshot); consumes the real GraphQL field |

## Defect found and fixed by conformance

A single non-finite sample serialized as a bare `NaN` literal, failing the
whole `invocation_metric_points` INSERT batch — the worker retried three
times and then dropped every sibling point. Ingest now filters non-finite
samples per the metric-summary contract.

## Reconciliation

Plan-168's `metricQuery` remains the single shared metric read path; this
plan added only capability-level summaries (`overview_totals`,
`signal_count_series`, `invocation_metric_summaries`) on the same native
tables, plus the contract-mandated CLI surface.
