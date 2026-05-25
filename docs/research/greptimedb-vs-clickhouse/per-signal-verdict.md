# Per-Signal Verdict — The Scenario Matrix

<!-- markdownlint-disable MD013 -->

Status: pass 7 synthesis; continually corrected (latest **pass 88**: the Logs·full-text
row updated for Runs 48–49 — the ~18× was a backend/function artifact; selective full-text
is ~2×, only broad-term is ~12×). Converges the architecture teardowns (passes 1–3) and
the Docker runs into one matrix: **for each signal and query shape, which engine is
faster/better, by which mechanism, under what scenario, at what confidence.** Feeds
`verdict-which-to-choose.md`.

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
| **Metrics** · PromQL range/aggregation | **GreptimeDB** (maturity/ergonomics, no longer binary) | Native PromQL planner (custom DataFusion nodes) + Prom `query_range` API, **GA + default-on**. **Correction (pass 44):** ClickHouse 26.x **does** have PromQL (`prometheusQuery[Range]` over the experimental `TimeSeries` engine) — but **experimental, off by default, setup-heavy**. So the win is GA-ergonomic vs experimental, not present-vs-absent. | Any; gap narrowed to maturity | **plan+live** (Run 3, Run 23) |
| **Metrics** · SQL range-aggregation latency | **ClickHouse, ~2× warm** (was misread as ~10×) | **Corrected (Run 37):** warm steady-state at 40k series / 8M rows is **CH 50 ms vs GT 107 ms (~2×)** — vectorized C++ group-by is the throughput bar (pass 42), but the gap is ~2×, not 10×. Run 11's GT 638 ms (~10×) was a **cold/first-run** scan (cold caches right after ingest), not the warm gap. Run-3's near-tie (1.3×) was a 1,200-series tiny-scale artifact. Cold-regime gap is larger (`caching-and-cold-warm.md`). | 40k series; warm ~2×, cold larger | smoke→volume (Runs 11, 37) |
| **Metrics** · high-cardinality series ingest | GreptimeDB (likely) | Metric engine maps many logical metrics onto a shared physical wide table → avoids per-series region/table explosion; ClickHouse needs careful `ORDER BY` + low-card keys. | High series cardinality | arch |
| **Metrics** · float compression | ClickHouse (likely) | Gorilla/DoubleDelta/ALP/FPC/T64 codec breadth vs GreptimeDB Parquet defaults. **Untested** — Run 3 data was incompressible (random walk). | Flat gauges / counters | arch (inconclusive) |
| **Logs** · selective filter (service/level + time) | **tie if keyed; ClickHouse ~2× if broad/un-keyed** | **Refined (Run 80):** Run 1's "3 ms vs 9 ms" was a *key-placement* effect (un-keyed GreptimeDB filter → scan). On `logs_b1` where **both key `service`(+`level`)**: a *highly selective* keyed filter is a **near-tie** (CH 4 ms / GT 5 ms, CH prunes 51/611 granules); a *broader* filter (level-only, 600k rows) is **CH ~2×** (6 vs 12 ms — vectorized scan over the matched set, ties Run 58). CH still has the finer granule (8,192) + PREWHERE + `LowCardinality`. | Wide table, selective predicate | arch+smoke (Runs 1, 80) |
| **Logs** · full-text / substring search (**selective**) | **~2× ClickHouse (corrected — was reported ~18×)** | **Runs 48–49 corrected it**: the ~18× (Run 12) was a **backend/function mismatch** — `matches()` (tantivy query-syntax fn) on a `backend='bloom'` index doesn't push to the index → **full-scans 5M** (~150 ms, EXPLAIN `output_rows: 5000000`). With the **correct pairing** GreptimeDB prunes (`output_rows: 1`): tantivy+`matches()` **~6 ms** or bloom+`matches_term()` **~8 ms**, vs CH `hasToken`/`text` **~3 ms** = **~2× warm, both sub-perceptible**. Not an index-maturity gap. **Re-verified Run 78 (no drift): CH ~3 ms vs GT bloom+`matches_term` ~8 ms (~2.7×); the wrong pairing `matches()`-on-bloom still full-scans ~157 ms — the 18× artifact reproduces.** | selective token search | **measured warm (Runs 48–49, 78)** |
| **Logs** · full-text **broad term** (matches many rows) | **ClickHouse (~12×)** | The residual real gap: a term matching ~100k+ rows scans the matched set, where CH's vectorized `hasToken`-on-65k-blocks wins (scan engine, `query-execution-engine.md` / parity-roadmap #2). Analytics, not interactive grep. | broad-term scan at 5M+ | measured warm (Run 48) |
| **Logs** · high-volume append ingest | ~tie | Both append-friendly: ClickHouse part-per-insert (+async insert batching); GreptimeDB `append_mode` skips dedup/merge. | Write-heavy | arch |
| **Traces** · `trace_id` point lookup | schema-decided; gap small in absolute ms | Sort-key prefix locality: ClickHouse `ORDER BY (trace_id, ts)` → sparse index seeks **Granules: 1** (Run 2), **2 ms**. GreptimeDB seed PK `(service,name)` leaves `trace_id` un-keyed → scan; **fair server-time = 14 ms** (Run 40 — *not* the 54 ms HTTP-wall; the ~40 ms was HTTP floor), **~8 ms with the `trace_id INVERTED INDEX` Parallax's design adds** (Run 6). So CH ~4–7× by locality, but **both ≪ 300 ms gate** — not latency-bound. **Flips toward ~tie when GreptimeDB keys/indexes `trace_id`** (its design does). | Whoever keys `trace_id` wins; absolute ms tiny | plan+smoke (Runs 2/6/40) |
| **Traces** · status/duration filter, span tree over window | ClickHouse (slight) | Vectorized columnar scan + granule skip; GreptimeDB competitive via DataFusion. | Analytical scan | arch |
| **Evidence-bundle** · anchored composite Q6 (Q1+Q2+Q3) | **not latency-bound** (both fast); CH ~3× at smoke | Full bundle measured end-to-end (Run 16, **re-verified Run 56 — no drift**): CH ~10 ms vs GT ~30 ms total, **both far under the 300 ms gate**; parity PASS (14 spans+3 logs+1 error). **Run 56 isolated the gap to Q1's `trace_id` retrieval floor** (CH sort-key seek 5 ms vs GT inverted-index ~21 ms); Q2 issue-history **tie** (2–3 ms) + Q3 release-regression anti-join **~tie** (3–6 ms) — i.e. the *correlation/assembly* is not the differentiator, only anchor retrieval is. Both propagate the anchor + prune before joining (Run 2 plans). | Anchored on `trace_id`/`fingerprint` (Parallax always anchors) | plan+smoke (Runs 2,16,56) |
| **Evidence-bundle** · Q4 cross-tier `spans`⋈`error_events` (anchored) | **ClickHouse ~13× on a direct in-DB join** (CH 4 ms / GT 54 ms); fixable | **Corrected (Run 81 — supersedes Run 30):** ClickHouse pushes `trace_id='X'` into the scan and prunes (`Granules 1` + PREWHERE) → 4 ms. **GreptimeDB does NOT push the left-side filter through the `LEFT JOIN`** — `EXPLAIN ANALYZE` shows the `spans_idx` scan `output_rows: 1,000,000` (full scan) → ~54 ms. A predicate-pushdown-into-join optimizer limitation, **not** the HTTP/repartition artifact Run 30 claimed. **Workarounds:** subquery pre-filter the left table → prunes (`output_rows: 14`, ~21 ms), or do the correlation **app-side** (anchored fetch each signal + join in app) — Parallax's pattern, avoids the in-DB join. **Re-verified live Run 154 (`EXPLAIN ANALYZE`, via `docker exec`): still reproduces** — CH `spans` (`ORDER BY (trace_id,ts)`) prunes the join to **`Granules 1/123` (~10,418 rows read)**; GreptimeDB `spans_idx` scans **1,000,000**. **Sharper isolation:** a *plain* `WHERE trace_id='X'` on `spans_idx` does **NOT** read 1M — the trace_id inverted index **prunes it fine**; the 1M full-scan happens **only through the `LEFT JOIN`**, so it is specifically *predicate-through-join propagation* the optimizer misses (consistent with Run 148 `Join=NonCommutative`), not a broken index. Subquery-prefilter restores pruning (no 1M). Parallax's anchored per-signal fetch prunes on **both** engines → the gap is sidestepped by app-side correlation, not a blocker. | Anchored cross-tier (Parallax's frontend↔backend correlation) | plan+smoke (Runs 30, 81, **154**) |
| **Evidence-bundle** · Q5 high-cardinality filter | **ClickHouse** if unindexed (scan); ~tie if indexed | **Re-verified + re-characterized (Run 58):** unindexed `span_id` → **full scan both** (CH `Granules 123/123`). Gap is **row-dependent throughput, not a fixed 10×**: CH 2 ms/GT 15 ms (~7×) at 1M, CH 3 ms/GT 43 ms (~14×) at 5M, CH 29 ms/GT 91 ms (~3×) on an 8M `sum` (agg dilutes the scan gap). CH's vectorized 65k-block+SIMD engine wins all (`query-execution-engine.md`). **Run 31's "GT 95 ms" did NOT reproduce (now 15 ms — HTTP-wall/cold artifact retired).** **Indexed** high-card filter = the anchored `trace_id` lookup (Runs 2/6, both fast); JSON-attribute filter → CH columnar subcolumn > GT blob-parse (Run 18). Parallax should index the attrs it filters on (both can). | Unindexed → scan (CH wins, grows with scan width); indexed → anchored (tie) | smoke (Runs 31,58) |
| **Evidence-bundle** · un-anchored large↔large join | GreptimeDB (tentative) | Partitioned hash join (repartition both sides) vs ClickHouse broadcast/grace-spill. **But Parallax does not run this for bundle assembly** — low priority. | No selective anchor, both sides large | arch |

## Reading the matrix against the operator hypothesis

Hypothesis: *GreptimeDB fastest, then ClickHouse.*

**On raw query latency, the hypothesis is not holding (smoke scale) — but more
narrowly than first measured.** ClickHouse is faster on selective log filters, trace
lookups (on tuned schema), broad-term log scans, and the anchored evidence-bundle
queries, by concrete code-confirmed mechanisms: finer granule, PREWHERE late
materialization, `LowCardinality`, and a decade-tuned C++ vectorized engine with lower
fixed per-query overhead. **Correction (Runs 48–49): full-text *search* is no longer a
ClickHouse blowout.** The earlier ~18× was a backend/function mismatch (`matches()` on a
bloom index full-scans); with the correct pairing (tantivy+`matches()` ~6 ms or
bloom+`matches_term()` ~8 ms vs CH ~3 ms) **selective full-text is ~2×, both
sub-perceptible** — the text-index advantage now only shows on **broad-term** scans
(~12×, scan-engine), not interactive search.

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
| **Speed — freshness** | **Tie** (measured Run 5): both visible-on-write, sub-second, no flush barrier. GreptimeDB write-path *edge*: LSM absorbs small high-frequency writes (no ClickHouse part-explosion / "too many parts") + native OTLP/Prom ingest. | smoke+arch |
| **Cost** | Measured (passes 8,17–19,29): local compression a **pattern-dependent wash**. Object storage is **query-shape-dependent**: GreptimeDB has far fewer *total* objects (4 vs 74 — wins full-scan cold reads + management), but cold S3 GET cost **splits by query shape (measured both ways)**: anchored keyed lookup → **ClickHouse fewer** (5 vs 22, Run 14, sort-key locality); full scan → **GreptimeDB fewer** (26 vs 57, Run 15, few-large-objects → confirms JSONBench mechanism). Parallax's anchored pattern → ClickHouse edge on this sub-axis; read cache makes warm re-reads local (0 GETs) on both. | compression smoke; object layout + cold GET counts (both shapes) measured |
| **Scaling** | **Split** (pass 10): ClickHouse wins vertical single-node ceiling; **GreptimeDB wins horizontal** (operator's primary) — region model + Metasrv rebalance + repartition + compute/storage separation vs ClickHouse OSS manual sharding (SharedMergeTree Cloud-only). | arch (multi-node run owed) |

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
