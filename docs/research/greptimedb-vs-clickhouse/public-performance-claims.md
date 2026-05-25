# Public Performance Claims — Gathered and Rated

<!-- markdownlint-disable MD013 -->

Status: pass 22 (Method step #4). Gathers the public performance claims for both
systems and rates each against the **source code** (passes 1–3) and the **local
Docker runs** (Runs 1–12). Ratings: *confirmed (my runs)*, *confirmed (code)*,
*workload-specific*, *vendor-reported (not re-run here)*, *contradicted*.
Claims go stale — dates/versions noted. Pins: GreptimeDB `v1.0.2`, ClickHouse
`v26.5.1.882-stable`.

## Claims table

| # | Claim (source) | Rating | Reconciliation with this loop |
| --- | --- | --- | --- |
| 1 | **ClickHouse has the best ingestion throughput** (Greptime log benchmark, 2024–25) | **confirmed (my runs)** | Run 5/11: CH bulk ingest ~1.55–4.5× faster than GreptimeDB COPY. |
| 2 | **ClickHouse wins aggregate throughput at high volume / many group-bys** (ClickBench; independent dev.to/oneuptime) | **confirmed (my runs + code)** | Run 11: CH metric agg ~10× at 8M rows; Run 12: log full-text ~18×, scan ~4×. Predicted by the vectorized C++ engine (`clickhouse-internals.md`). |
| 3 | **ClickHouse handles high-frequency small writes poorly; async inserts are the workaround** (oneuptime, independent) | **confirmed (my runs + code)** | Run 7 (B9): one part per INSERT, merges collapse bursts, `async_insert=1` default in 26.x mitigates; sustained-rate failure. |
| 4 | **GreptimeDB is object-storage-native (~1–2% loss vs local); ClickHouse uses S3 as a cold tier, not primary** (Greptime) | **confirmed (code + my runs)** | Run 8–9 (B10): GreptimeDB single `[storage]` block, 4 objects; ClickHouse 74 objects via S3-disk-under-policy. `distributed-and-scaling.md` (SharedMergeTree Cloud-only). |
| 5 | **GreptimeDB offers better compression / resource efficiency** (Greptime) | **workload-specific** | Run 4/10: a tuning-dependent **wash** — GreptimeDB wins out-of-the-box (ZSTD-all default) but ClickHouse ties/beats with matched per-column ZSTD. Not a blanket win. |
| 6 | **GreptimeDB ranked #1 on cold run (4th hot) in ClickHouse's official JSONBench, 1B JSON docs — beats ClickHouse/VictoriaLogs** (Greptime blog, on ClickHouse's harness, 2026) | **vendor-reported (not re-run here); plausible** | **Key counterpoint** — see below. On ClickHouse's *own* public harness, so hard to game; but vendor-selected framing and not locally reproduced. |
| 7 | **GreptimeDB: native OTLP + full PromQL + Jaeger API; ClickHouse treats time as just another column** (Greptime) | **confirmed (code + my runs)** | Run 3: PromQL native on GreptimeDB, absent in ClickHouse. `greptimedb-internals.md` metric engine + time index. |
| 8 | **GreptimeDB "up to 50× total cost reduction for observability"** (Greptime OSS marketing) | **marketing headline** | Directionally consistent with object-store retention economics (`../retention-cost-model.md` ~100× vs ingest-priced SaaS), but "50×" is vs SaaS, not vs self-hosted ClickHouse. Treat as marketing, not a ClickHouse comparison. |
| 9 | **ClickHouse is a strong choice for observability (logs/traces/metrics) + broad ecosystem** (clickhouse.com; independent) | **confirmed (code + my runs)** | Its log scan/search + analytical maturity (Run 12) and ecosystem are real; the gap is metrics-PromQL nativeness + ingest ergonomics. |

## The JSONBench cold-run counterpoint (claim #6) — important for Parallax

My local runs (B1/B5) measured **warm, small-to-medium scale, hot-cache** queries,
where ClickHouse's vectorized engine won (log search ~18×, metric agg ~10×). But
ClickHouse's *own* JSONBench (1 billion JSON documents) reportedly ranks
**GreptimeDB #1 on the cold run** — i.e. queries served from object storage / cold
cache at large scale, on semi-structured JSON/wide-event data.

This is the **opposite regime** from my hot small-scale runs, and it matters
because **Parallax's actual retention access pattern is closer to JSONBench
cold-run than to ClickBench hot-aggregate**: evidence bundles are re-read from
cheap object storage, often cold, often as wide/semi-structured records — not
continuous hot analytical aggregation over months of data. If the JSONBench
cold-run result holds, GreptimeDB's object-store-native Parquet layout (few large
objects, Run 9) may **win the cold-read regime that Parallax actually lives in**,
even though ClickHouse wins hot in-cache analytical scans.

**Caveat (honest):** claim #6 is vendor-reported (GreptimeDB's blog) on
ClickHouse's public harness; I have **not** re-run JSONBench locally. It is the
single most important public claim to **independently reproduce** — it could
materially strengthen the GreptimeDB verdict for Parallax's cold-object-store
pattern. Routed to `benchmarking-the-differences.md` as a new high-priority case.

## Net effect on the verdict

The public claims **triangulate cleanly** with this loop's local + code findings:
ClickHouse wins hot analytical throughput (ingest, agg, log search) and small-write
handling needs async-insert; GreptimeDB wins object-store-native economics, PromQL/
OTLP nativeness, and (vendor-reported) cold-run object-store JSON queries at 1B
scale. No public claim is **contradicted** by my runs; the only **stale/inflated**
one is the "50× cost" marketing headline (vs SaaS, not vs ClickHouse). The
JSONBench cold-run claim adds a genuine, decision-relevant counterpoint that favors
GreptimeDB in **Parallax's real (cold object-store re-read) regime** and should be
reproduced before finalizing.

## Sources

- [GreptimeDB as a ClickHouse alternative for time-series/observability (Greptime, 2026-04)](https://greptime.com/tech-content/2026-04-17-clickhouse-alternative-greptimedb)
- [GreptimeDB vs ClickHouse vs Elasticsearch — log engine benchmark (Greptime)](https://greptime.com/blogs/2024-08-22-log-benchmark)
- [GreptimeDB takes on the billion-JSON-document challenge (JSONBench) (Greptime, 2026)](https://medium.com/@sunng87/greptimedb-takes-on-the-billion-json-document-challenge-outperforms-clickhouse-victorialogs-48214d3311dd)
- [ClickBench — benchmark for analytical DBMS (ClickHouse)](https://benchmark.clickhouse.com/)
- [ClickBench repo (ClickHouse)](https://github.com/ClickHouse/ClickBench)
- [What really matters for performance: a year of benchmarks (ClickHouse)](https://clickhouse.com/blog/what-really-matters-for-performance-lessons-from-a-year-of-benchmarks)
- [TimescaleDB vs ClickHouse vs MongoDB for observability (dev.to, independent)](https://dev.to/aws-builders/i-benchmarked-timescaledb-vs-clickhouse-vs-mongodb-for-observability-data-the-results-surprised-me-3d7d)
