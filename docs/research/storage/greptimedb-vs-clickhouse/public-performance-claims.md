# Public Performance Claims — Gathered and Rated

<!-- markdownlint-disable MD013 -->

Status: pass 22, **re-verified pass 47**, full-text claim corrected passes 86-87 /
Runs 48-49 + **Run 106 (vendor-page audit, `vendor-claims-audit.md`)** — audited
`greptime.com/compare/click_house` + 15 blogs: the compare page sells GT on fit/storage/economics/
native-protocols and **never claims raw-analytical-speed superiority** (the one thing our data would
refute); the log-monitoring blog *concedes* CH is faster on keyword search; the ingestion benchmark
independently confirms our cardinality-insensitivity win on v1.0 GA. **Correction logged: OTel-Arrow
is Phase-2 / experimental, NOT GA** — only baseline OTLP (HTTP/gRPC) metrics/logs/traces is GA, so
do not cite OTel-Arrow as a shipped advantage. (Method step #4). Gathers the public
performance claims for both systems and rates each against the **source code**, the
**local Docker runs** (Runs 1–25), and a periodic web re-sweep. Ratings: *confirmed
(my runs)*, *confirmed (code)*, *workload-specific*, *vendor-reported (not re-run
here)*, *contradicted*. Claims go stale — dates/versions noted. Pins: GreptimeDB
`v1.0.2`, ClickHouse `v26.5.1.882-stable` (re-verified current 2026-05-25).

## Claims table

| # | Claim (source) | Rating | Reconciliation with this loop |
| --- | --- | --- | --- |
| 1 | **ClickHouse has the best ingestion throughput** (Greptime log benchmark, 2024–25) | **confirmed (my runs)** | Run 5/11: CH bulk ingest ~1.55–4.5× faster than GreptimeDB COPY. |
| 2 | **ClickHouse wins aggregate throughput at high volume / many group-bys** (ClickBench; independent dev.to/oneuptime) | **confirmed for scans/agg; selective log-search narrowed** | Run 37 corrected metric agg to ~2× warm; full scans remain ~4×. Runs 48-49 corrected the log-search headline: selective GreptimeDB full-text is ~6 ms with tantivy+`matches()` or ~8 ms with bloom+`matches_term()`, vs ClickHouse ~3 ms — not 18×. The large remaining gap is broad-term scan analytics. |
| 3 | **ClickHouse handles high-frequency small writes poorly; async inserts are the workaround** (oneuptime, independent) | **confirmed (my runs + code)** | Run 7 (B9): one part per INSERT, merges collapse bursts, `async_insert=1` default in 26.x mitigates; sustained-rate failure. |
| 4 | **GreptimeDB is object-storage-native (~1–2% loss vs local); ClickHouse uses S3 as a cold tier, not primary** (Greptime) | **confirmed (code + my runs)** | Run 8–9 (B10): GreptimeDB single `[storage]` block, 4 objects; ClickHouse 74 objects via S3-disk-under-policy. `distributed-and-scaling.md` (SharedMergeTree Cloud-only). |
| 5 | **GreptimeDB offers better compression / resource efficiency** (Greptime) | **workload-specific** | Run 4/10: a tuning-dependent **wash** — GreptimeDB wins out-of-the-box (ZSTD-all default) but ClickHouse ties/beats with matched per-column ZSTD. Not a blanket win. |
| 6 | **GreptimeDB ranked #1 on cold run (4th hot) in ClickHouse's official JSONBench, 1B JSON docs — beats ClickHouse/VictoriaLogs** (Greptime blog, on ClickHouse's harness, 2026) | **vendor-reported (not re-run here); plausible** | **Key counterpoint** — see below. On ClickHouse's *own* public harness, so hard to game; but vendor-selected framing and not locally reproduced. |
| 7 | **GreptimeDB: native OTLP + full PromQL + Jaeger API; ClickHouse treats time as just another column** (Greptime) | **mostly confirmed — one part corrected** | All three GreptimeDB sub-claims now verified: native GA OTLP metrics/logs/traces (pass 46, live); GA default-on PromQL (pass 44/45, live); **native GA Jaeger query API** (pass 55 — `/v1/jaeger/api/services` live HTTP 200, `http/jaeger.rs` 1750 lines: services/operations/traces + tag/attribute search). ClickHouse has **none of these natively** — OTLP via collector (pass 46), PromQL via the experimental `TimeSeries` engine (pass 44, limited to `rate`/`delta`/`increase`), Jaeger via the external `jaeger-clickhouse` storage plugin. **Correction (pass 44/47):** PromQL is **no longer absent in ClickHouse** ("absent" was wrong) — but it is early-stage/experimental. So the claim holds in *spirit* (GreptimeDB = GA-native observability protocols; ClickHouse = assembled/external/experimental). See `promql-and-metrics-query.md`. |
| 8 | **GreptimeDB "up to 50× total cost reduction for observability"** (Greptime OSS marketing) | **marketing headline** | Directionally consistent with object-store retention economics (`retention-and-ttl.md` $-framing), but "50×" is vs SaaS, not vs self-hosted ClickHouse. Treat as marketing, not a ClickHouse comparison. |
| 9 | **ClickHouse is a strong choice for observability (logs/traces/metrics) + broad ecosystem** (clickhouse.com; independent) | **confirmed (code + my runs)** | Its log scan/search + analytical maturity (Run 12) and ecosystem are real; the gap is metrics-PromQL nativeness + ingest ergonomics. |

## The JSONBench cold-run counterpoint (claim #6) — important for Parallax

My local runs (B1/B5) measured **warm, small-to-medium scale, hot-cache** queries,
where ClickHouse's vectorized engine won on scans and aggregation; the original
headline log-search gap was corrected by Runs 48-49. But
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

The public claims **mostly triangulate** with this loop's local + code findings:
ClickHouse wins hot analytical throughput and broad scans; small-write handling
needs async-insert; GreptimeDB wins object-store-native economics, PromQL/OTLP
nativeness, selective incident grep is competitive with either correct full-text
pairing, and GreptimeDB has a vendor-reported cold-run object-store JSONBench win
at 1B scale. No public claim is fully contradicted, but the old local "ClickHouse
log search is 18× faster" wording is now **over-broad**: it describes the
`matches()`/bloom mismatch and broad-term scan cases, not the exact-term or
query-syntax selective search Parallax likely needs most. The
"50× cost" headline remains marketing vs SaaS, not a ClickHouse comparison. The
JSONBench cold-run claim remains the most important public claim to reproduce.

## Version freshness + index-maturity context (pass 25 re-check)

- **Pins re-verified 2026-05-25 (pass 25): still current.** GreptimeDB latest stable
  = `v1.0.2` (`v1.1.0` exists only as nightly, not GA); ClickHouse latest stable =
  `v26.5.1.882-stable`. No bump needed.
- **ClickHouse `text` (full-text inverted) index GA'd in 26.2 (March 2026).** The
  ClickHouse `hasToken` path remains a production-GA fast path for token search. But
  Runs 48-49 corrected B1: the GreptimeDB table was bloom-backed and the query used
  `matches()`, which does not prune that backend. Use `matches_term()` for bloom exact
  terms; use `matches()` with the tantivy/fulltext backend for query-syntax search.
- **GreptimeDB fulltext has two practical modes.** Bloom + `matches_term()` is the
  exact-term incident-grep mode and is competitive in the 5M warm smoke. Tantivy +
  `matches()` is the query-syntax/phrase/relevance mode and Run 49 made it competitive
  too (~6 ms selective). Broad terms still route to scan-engine work. Re-check on each release for fulltext
  planner/backend changes.

## Re-verification sweep (pass 47)

Periodic re-check of all 9 claims against the current pins + a web re-sweep
(claims drift — PromQL just did):

- **Claim #7 corrected — the one drift.** "PromQL absent in ClickHouse" is **stale**:
  ClickHouse 26.x has experimental PromQL (`prometheusQuery[Range]` over the
  `TimeSeries` engine), **early-stage, limited to `rate`/`delta`/`increase`**, off by
  default. **Triangulated three ways** — source (pass 44), live test (pass 45), **and
  Greptime's own comparison page** ("PromQL support in the ClickHouse ecosystem is
  early-stage and limited to basic functions like rate, delta, and increase"). So even
  the *vendor* frames it as early-stage-present, not absent. Net: still a GreptimeDB
  maturity win, not a binary capability gap.
- **Claim #6 (JSONBench cold-run #1 GreptimeDB) still stands**, vendor-reported,
  **still not locally reproduced** — remains the single highest-value claim to
  reproduce (1B docs, cold object-store regime = Parallax's pattern). Greptime blog
  dates to **2025-03** (earlier than first noted); no newer public cold-run that
  reverses it surfaced in the sweep.
- **Claims #1–5, #8–9 mostly unchanged, with passes 86-87 caveat on log search** — re-scan
  found no drift on CH ingest/agg, small-write async-insert, object-store-native,
  compression wash, or "50× cost" marketing-vs-SaaS. Runs 48-49 narrowed local B1:
  selective log search is not an 18× GreptimeDB deficit when the backend/function pairing
  is correct (`matches_term()`+bloom or `matches()`+tantivy).
- **OTLP (part of claim #7) re-verified — no drift** (pass 46): ClickHouse still has
  no native OTLP receiver; GreptimeDB native GA. Notably ClickHouse's 26.x protocol
  investment went to **Prometheus** (TimeSeries/remote-write/PromQL), not OTLP.

## Sources

- [GreptimeDB as a ClickHouse alternative for time-series/observability (Greptime, 2026-04)](https://greptime.com/tech-content/2026-04-17-clickhouse-alternative-greptimedb)
- [GreptimeDB vs. ClickHouse comparison page (Greptime) — source of the "ClickHouse PromQL early-stage, limited to rate/delta/increase" framing](https://greptime.com/compare/click_house)
- [GreptimeDB vs ClickHouse vs Elasticsearch — log engine benchmark (Greptime)](https://greptime.com/blogs/2024-08-22-log-benchmark)
- [GreptimeDB takes on the billion-JSON-document challenge (JSONBench) (Greptime, 2026)](https://medium.com/@sunng87/greptimedb-takes-on-the-billion-json-document-challenge-outperforms-clickhouse-victorialogs-48214d3311dd)
- [ClickBench — benchmark for analytical DBMS (ClickHouse)](https://benchmark.clickhouse.com/)
- [ClickBench repo (ClickHouse)](https://github.com/ClickHouse/ClickBench)
- [What really matters for performance: a year of benchmarks (ClickHouse)](https://clickhouse.com/blog/what-really-matters-for-performance-lessons-from-a-year-of-benchmarks)
- [TimescaleDB vs ClickHouse vs MongoDB for observability (dev.to, independent)](https://dev.to/aws-builders/i-benchmarked-timescaledb-vs-clickhouse-vs-mongodb-for-observability-data-the-results-surprised-me-3d7d)
- [Announcing GA of ClickHouse full-text search (text index GA in 26.2)](https://clickhouse.com/blog/full-text-search-ga-release)
- [Full-text search with text indexes — ClickHouse docs](https://clickhouse.com/docs/engines/table-engines/mergetree-family/textindexes)
