# Per-Signal Verdict — The Scenario Matrix

<!-- markdownlint-disable MD013 -->

Status: pass 7 synthesis. Converges the architecture teardowns (passes 1–3) and
the Docker runs (passes 4–6) into one matrix: **for each signal and query shape,
which engine is faster/better, by which mechanism, under what scenario, at what
confidence.** Feeds `verdict-which-to-choose.md`.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Confidence legend

- **arch** — reasoned from source mechanism (passes 1–3), not yet measured.
- **smoke** — measured locally at 1M-row cache-resident scale (passes 4–6);
  **direction only**, not a production verdict. Fixed per-query overhead dominates;
  scan throughput and cold-cache behavior are *not* exercised at this scale.
- **plan** — confirmed by reading the real EXPLAIN/query plan (scale-independent).

## The matrix

| Signal · query shape | Winner | Mechanism (the *because*) | Scenario qualifiers | Confidence |
| --- | --- | --- | --- | --- |
| **Metrics** · PromQL range/aggregation | **GreptimeDB** | Native PromQL planner + Prometheus `query_range` API over the time-series/metric model; ClickHouse has **no PromQL** — needs an external PromQL→SQL layer. Capability gap, not just speed. | Any; capability is binary | **plan+smoke** (Run 3) |
| **Metrics** · SQL range-aggregation latency | ~tie (ClickHouse edge) | Both vectorize the group-by; GreptimeDB within ~1.3× (16 vs 12 ms) — its design center, so the usual gap nearly closes. | Smoke scale, warm | smoke |
| **Metrics** · high-cardinality series ingest | GreptimeDB (likely) | Metric engine maps many logical metrics onto a shared physical wide table → avoids per-series region/table explosion; ClickHouse needs careful `ORDER BY` + low-card keys. | High series cardinality | arch |
| **Metrics** · float compression | ClickHouse (likely) | Gorilla/DoubleDelta/ALP/FPC/T64 codec breadth vs GreptimeDB Parquet defaults. **Untested** — Run 3 data was incompressible (random walk). | Flat gauges / counters | arch (inconclusive) |
| **Logs** · selective filter (service/level + time) | **ClickHouse** | 8,192-row granule (12× finer than GreptimeDB's 102,400-row Parquet row group) + **PREWHERE** late materialization + `LowCardinality` + decade-tuned vectorized scan. Run 1: 3 ms vs 9 ms. | Wide table, selective predicate | arch+smoke |
| **Logs** · full-text / substring search | **ClickHouse** | Native inverted **text index** (posting lists) + token/ngram bloom; GreptimeDB has a full-text index too but ClickHouse's columnar string scan + index maturity lead. | Substring/token over window | arch |
| **Logs** · high-volume append ingest | ~tie | Both append-friendly: ClickHouse part-per-insert (+async insert batching); GreptimeDB `append_mode` skips dedup/merge. | Write-heavy | arch |
| **Traces** · `trace_id` point lookup | schema-decided (ClickHouse on seed DDL) | Sort-key prefix locality: ClickHouse `ORDER BY (trace_id, ts)` → sparse index seeks **Granules: 1** (Run 2 plan). GreptimeDB seed PK `(service,name)` leaves `trace_id` un-keyed → scan (Run 1: 16 ms vs 2 ms). **Flips to ~tie if GreptimeDB keys/indexes `trace_id`.** | Whoever keys `trace_id` wins | plan+smoke |
| **Traces** · status/duration filter, span tree over window | ClickHouse (slight) | Vectorized columnar scan + granule skip; GreptimeDB competitive via DataFusion. | Analytical scan | arch |
| **Evidence-bundle** · anchored join (Q1/Q4, by `trace_id`) | ClickHouse at smoke; **not join-decided** | Both engines **propagate the anchor constant to both join inputs** (CH PREWHERE→1 granule; GT FilterExec→both region scans), so broadcast-vs-partitioned join is **not a differentiator** for anchored queries (Run 2 plans). Winner tracks key placement + fixed overhead. Run 2: Q1 4 vs 24 ms, Q4 3 vs 54 ms (GT penalized by HTTP + 10-way repartition at toy scale). | Anchored on `trace_id`/`fingerprint` (Parallax always anchors) | plan+smoke |
| **Evidence-bundle** · un-anchored large↔large join | GreptimeDB (tentative) | Partitioned hash join (repartition both sides) vs ClickHouse broadcast/grace-spill. **But Parallax does not run this for bundle assembly** — low priority. | No selective anchor, both sides large | arch |

## Reading the matrix against the operator hypothesis

Hypothesis: *GreptimeDB fastest, then ClickHouse.*

**On raw query latency, the hypothesis is not holding (smoke scale).** ClickHouse
is faster on logs (selective + search), trace lookups (on tuned schema), and the
anchored evidence-bundle queries. The mechanisms are concrete and code-confirmed:
finer granule, PREWHERE late materialization, a mature inverted text index,
`LowCardinality`, and a decade-tuned C++ vectorized engine with lower fixed
per-query overhead.

**Where GreptimeDB genuinely wins is not "fastest" — it is *capability and fit*:**

1. **Metrics / PromQL nativeness** (plan+smoke confirmed): native PromQL + Prom
   remote-write. ClickHouse cannot do this without a translation layer. For a
   product that ingests Prometheus metrics or exposes PromQL, this is decisive.
2. **Metric aggregation latency is competitive** (within 1.3×) — the one signal
   where GreptimeDB does not clearly lose on speed.
3. **Object-storage-native economics** (arch, untested): OpenDAL + default read
   cache vs ClickHouse's S3-disk-as-policy. Likely a cost/retention edge — must
   measure (cost axis still open).
4. **Operational fit**: single Rust binary, metrics-native, object-store-first —
   aligns with the Parallax language filter (Rust) and tiny-single-node start.

**Caveats that could move cells:**

- All latency cells are **smoke / cache-resident** — they measure fixed overhead,
  not throughput. Cold-cache + GB–TB scans are where ClickHouse's scan engine and
  GreptimeDB's object-store cache truly diverge. **Bigger/cold tier is the next
  benchmark.**
- The GreptimeDB latencies carry an HTTP-API measurement penalty vs ClickHouse's
  native client — re-measure via GreptimeDB's MySQL protocol.
- Cost (compression by signal, object-store $) and scaling (single-node ceiling,
  horizontal) cells are **not yet populated** — `compression-and-cost.md` and
  `distributed-and-scaling.md` pending.

## Axis roll-up (speed > cost > scaling)

| Axis | Current read | Confidence |
| --- | --- | --- |
| **Speed — query latency** | ClickHouse leads logs/traces/bundle; GreptimeDB leads metrics (PromQL) + ties metric agg. | smoke |
| **Speed — freshness** | Both fresh-on-write (memtable-insert vs part-write visibility). Side-by-side pending in `write-path-and-ingestion.md`. | arch |
| **Cost** | Open. ClickHouse codec breadth vs GreptimeDB object-store-native; neither measured well yet. | — |
| **Scaling** | Open. GreptimeDB region model (designed-in) vs ClickHouse manual sharding. | arch |

## Bottom line (provisional, will sharpen in the verdict)

If Parallax's storage choice were decided **only on raw query latency at smoke
scale**, ClickHouse wins most cells — the operator hypothesis would be refuted.
But the decision axes are speed **and** cost **and** scaling **and** fit, and
GreptimeDB's metrics/PromQL nativeness, object-storage economics, region-based
scaling, and Rust single-binary fit are real, mechanism-grounded advantages not
captured by a smoke-scale latency number. The honest current state: **ClickHouse
is faster for log/trace analytics; GreptimeDB is the better *metrics-native,
object-store-first* fit** — and the final call depends on the still-open cost and
scaling axes plus whether Parallax's dominant query is log/trace search (favors
ClickHouse) or metrics + cheap re-readable retention (favors GreptimeDB). Resolved
in `verdict-which-to-choose.md` once cost/scaling land.
