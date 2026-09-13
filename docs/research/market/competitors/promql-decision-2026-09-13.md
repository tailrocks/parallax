# PromQL product-surface decision — 2026-09-13

**Status:** decided. Freeze item 12 closed as a *decision*; implementation is
the existing typed `metricQuery` path plus shipped `rate`/`increase`.

**Decision:** keep Parallax's typed, kind-legal metric builder. Do **not**
embed Grafana. Do **not** expose PromQL as the product query language.

## Options considered

| Option | What it solves | Cost to Parallax |
| --- | --- | --- |
| A. PromQL subset in UI | Power-user muscle memory; `rate()`/`increase()`/`histogram_quantile()` | Parser, PromQL footguns (rate on gauges), second language beside Where-chips and SQL, Greptime PromQL is slower than SQL on wide ranges |
| B. Grafana-embed | Instant dashboard/Explore depth | Extra process, AGPL Grafana, two UIs, dead-ends out of issue/trace/bundle, kills single-binary self-host |
| C. Keep typed builder | Error-proof aggs; one canonical API | Power users who want full PromQL must use SQL or an external Grafana against Greptime |

## Why C

1. **The problem that matters** is correct counter math and kind-legal
   aggregations, not language completeness. Prometheus 3.14.0 still lets you
   write `rate()` on a gauge. Parallax already refuses illegal aggs:
   `gauge→avg|min|max|last`, `sum→sum|rate|increase`,
   `histogram→p50|p95|p99|avg` (`ui/graphql/schema.graphql` `metricQuery`).
2. **Shipped on HEAD `1cad2a50`:** reset-clamped `rate_from_buckets` /
   `increase_from_buckets` in `crates/parallax-storage/src/adapter_math.rs`;
   GraphQL tests in `parallax-api` `metric_query_supports_last_and_increase_aggregations`.
3. **Engine PromQL is not free.** Greptime PromQL is GA, but research Run 105
   measured GT PromQL ~5.6× slower than GT SQL on a 60-min `avg by(service)`
   ([promql-and-metrics-query.md](../../storage/greptimedb-vs-clickhouse/promql-and-metrics-query.md)).
   Hot panels already use SQL / typed `metricQuery`.
4. **Grafana-embed damages the product shape:** extra binary, AGPL, second
   navigation, no issue identity / evidence bundle / CLI invocation in Grafana.
   Conflicts with GOAL §13 (one canonical API, single-binary self-host).
5. **Escape hatch already exists:** read-only SQL (`sql` query + `parallax sql`)
   against native tables. Optional later: labeled Greptime `TQL EVAL` passthrough
   for operators, **not** a PromQL Explore clone.

## What this is not

- Not a claim that PromQL is a bad language. It is the industry reference for
  metrics *power users*. Parallax's audience is developers investigating
  failures; the typed builder plus exemplars plus SQL is the simpler product.
- Not a ban on Prometheus remote-write ingest later. Ingest compatibility ≠ UI
  language.

## Implementation remaining (not PromQL)

- Metric workbench already has Create alert / Add to dashboard
  (`ui/src/routes/metrics.$metricName.tsx`).
- Freeze item 5 remainder: spike → related traces (exemplars exist; auto-surface
  of traces in the spike window is still adopt).
- Do not add a PromQL text box in this goal.
