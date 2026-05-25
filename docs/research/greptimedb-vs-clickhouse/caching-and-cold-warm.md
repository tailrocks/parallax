# Caching and Cold-vs-Warm Divergence (Subsystem #7)

<!-- markdownlint-disable MD013 -->

Status: pass 24, extended pass 92 (**B10 measured, Run 55**). The cache hierarchy of
each engine and **why cold-cache and warm-cache performance diverge** — the dimension
my local runs (all warm, cache-resident at ≤5M rows) could not measure, and the
mechanism behind the cold-object-store regime the verdict now hinges on (JSONBench
cold-run, `public-performance-claims.md`). Code-grounded. **Pass 92 replaced the
predicted cold-re-read winner with a measured, two-sided B10 result (Run 55); pass 99
(Run 63) resolved *why* the cold selective read pulls the whole SST — scatter, not a
small-SST artifact.**

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Cache hierarchies, side by side

| Cache | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Index/marks | Inverted / bloom / vector index caches + **index result cache** (`src/mito2/src/cache/index`). | **Mark cache (5 GiB default)** — sparse-index marks, the hot path; prewarmed to 0.95 on startup. + secondary `index_mark_cache`. |
| Decompressed data | **Page cache 512 MB** (Parquet pages), **vector cache 512 MB** (Arrow arrays) (`config.rs:205-206`). | **Uncompressed cache — OFF by default (0 MiB)**; relies on the **OS page cache** for *compressed* blocks, decompresses per query. |
| Metadata | **SST-meta cache 128 MB** (Parquet footers), manifest cache. | Primary-key (index) in memory; part metadata. |
| Query results | **Selector result cache 512 MB**. | **Query cache** (1 GB max, opt-in per query). |
| **Object-store read cache** | **LRU local-disk cache in front of S3, default ON** (`object-store/src/config.rs:318-340`), `cache_capacity` bound. | S3-disk **filesystem cache** (local), configured per disk/policy — an extra config layer, not default-primary. |
| Memtable/write buffer | Global write buffer 1 GB; write cache (`write_cache.rs`). | Insert blocks; `async_insert` buffer. |

## Why cold ≠ warm — the mechanism

**ClickHouse (local-disk-tuned).** Warm = the 5 GiB **mark cache** holds the sparse
index (skip granules with zero disk I/O) + the OS page cache holds compressed
blocks. Cold (fresh start / evicted) = read marks from disk to rebuild the mark
cache, then read+decompress granules. Because the **uncompressed cache is off by
default**, ClickHouse re-decompresses on each read but keeps marks hot — so its
cold penalty is dominated by *mark loading + decompression*, mitigated by mark
prewarm (0.95). On a **local NVMe** this cold penalty is modest; the design assumes
data is on fast local disk.

**GreptimeDB (object-store-tuned).** Warm = the requested SST objects are already
in the **local read cache** (disk) + Parquet pages in the 512 MB page cache + index
data in the index caches. Cold (object not in local cache) = an **S3 GET per
needed object** (network round-trip) to pull it local, then decode. GreptimeDB's
cold penalty is dominated by **object-store request latency**, not local disk —
and its **few-large-objects layout** (4 objects for 1M spans, Run 9) means a cold
read issues *few* GETs.

**Cold-inflation magnitude ∝ bytes decoded cold (measured, Runs 37/39).** The
warm→cold gap-widening scales with how many bytes a query must decode cold, which is
why re-verified gaps differed: the **metric-agg** (8M rows × `value`+`ts`+`service` +
per-row bucketing) was **10× cold → 2× warm** (heavy cold decode); **count-by-`level`**
(5M rows × one `LowCardinality` column) was only ~94 ms cold → ~4× warm (light decode);
**full-text** — **corrected (Runs 48–49):** the old "~18× warm, index-bound, no cold
inflation" reading was wrong — the `matches()`-on-bloom query actually **full-scanned 5M
rows** (a wide decode that *would* cold-inflate). The **correctly-paired** full-text
(tantivy+`matches` or bloom+`matches_term`) **prunes** → ~2× warm with light decode = the
genuine index-bound, little-cold-inflation case. So
"cold widens the gap" is true *in proportion to scan width* — wide/heavy aggregations
inflate most cold, light single-column scans little, **pruned** index-bound queries not at all.
This is why the warm read-path numbers (Runs 37–39) are the trustworthy steady-state
and the *cold-regime* widening is a separate, scan-width-dependent effect the cold-tier
harness must quantify.

## The decisive consequence for Parallax (object-store cold re-reads) — B10 MEASURED (Run 55)

Parallax re-reads evidence bundles from cheap object storage, sometimes **cold**.
B10 (Run 55) measured the cold cost of one **anchored** `trace_id` lookup (14 of 1M
spans) by capturing MinIO requests (`mc admin trace`) after forcing each engine cold
(ClickHouse `SYSTEM DROP FILESYSTEM CACHE`; GreptimeDB `rm -rf` the local read cache
+ restart). The result is **two-sided — the earlier "GreptimeDB wins cold re-read"
prediction was too simple:**

| Cold anchored lookup | S3 GETs | egress bytes | what it reads |
| --- | --- | --- | --- |
| **ClickHouse** | **18** | **294 KiB** | only the needed **column granules** (sparse index → 1 granule × ~5 cols + marks) |
| **GreptimeDB** | **9** (4 manifest¹ + 5 SST) | **~23 MiB** | ~the **entire 21 MiB Parquet SST** (coarse ranged reads) |

¹ the 4 manifest GETs are one-time **region-open** overhead (amortized across the
process lifetime), not per-query; the per-query cold cost is the **5 SST GETs / ~23 MiB**.

- **Request count → GreptimeDB** (9 vs 18, ~2× fewer — its few-objects layout means
  even a cold read touches few objects). *But this is far less than the ~25× object-count
  ratio (Run 54), because an anchored query touches few objects on **both**.*
- **Cold egress → ClickHouse, dramatically** (294 KiB vs ~23 MiB, **~80×**). For a
  **selective** query, ClickHouse's granule-level reads fetch only what's needed;
  GreptimeDB on a cold cache pulls ~the whole SST. **On per-GB egress pricing (R2/B2/S3),
  this reverses the cost advantage for cold selective re-reads.**
- **⚠ Caveat RESOLVED (Run 63) — it is NOT a small-SST artifact, it is *scatter*.**
  `EXPLAIN ANALYZE` showed the recommended `spans_idx` (`PRIMARY KEY(service,name)`,
  `trace_id` inverted-indexed but **not** the sort key) scans an anchored `trace_id`
  lookup at **scan_cost 39 ms** vs **14 ms** for a `PRIMARY KEY(trace_id)`-clustered
  copy of the same data. So a trace's rows **scatter across all row groups** under the
  recommended design → an anchored read touches ~every row group → cold = ~whole SST,
  and this **persists/grows at larger SST** (more row groups, all touched). The
  structural cause: ClickHouse `ORDER BY (trace_id, ts)` clusters by the high-card
  anchor **at zero cardinality cost**, so its read is ~1 granule; GreptimeDB's **PK is
  also its series identity**, so clustering by `trace_id` (which *would* prune — proven,
  39→14 ms) **explodes series cardinality**, which the design avoids. Cluster-vs-cardinality
  is a tradeoff ClickHouse does not face. **The exact cold-egress byte count at large SST
  is still owed to the harness; the mechanism is settled (Run 63).**
  **Partial mitigation (Run 86):** GreptimeDB's native `opentelemetry_traces` table is
  **`PARTITION ON COLUMNS (trace_id)` (16-way)** — partitioning by the anchor **without**
  making it the PK (no series-cardinality cost), so an anchored cold trace read prunes to
  **~1/16 of the data**, not the whole table. A coarse (16-bucket, not granule-level)
  anchor-locality lever GreptimeDB *does* have — it can *partition* (cheaply), just not
  *sort*, by the anchor. Mitigates, doesn't erase, the gap vs ClickHouse's granule locality.

**The warm/repeat path is the real Parallax economics, and it favours GreptimeDB.**
GreptimeDB **write-through populates the whole SST into a persistent local read cache
on flush** (`/greptimedb_data/cache` held the full 21 MiB; **it survived a `docker
restart`** — only an explicit `rm` made the query cold). So after the first touch,
*every* re-read (anchored or scan) is served locally at **~0 S3 requests / 0 egress**
— Run 53's earlier warm anchored ~immune numbers are this path. For Parallax's
dominant pattern — repeatedly re-reading **recent** bundles — GreptimeDB pays the cold
SST egress **once**, then amortizes to zero. ClickHouse's S3-disk filesystem cache is
similar but off-the-primary-path and was the thing I had to *drop* to force cold.

**Net (corrects the prediction):** "cold object-store re-read favours GreptimeDB" is
**only true for request count and for the warm-after-first-touch steady state**. For a
genuinely **cold selective** read (evicted cold-tier history, e.g. re-opening an old
incident), **ClickHouse transfers far less data** (granules vs whole SST) — an egress-cost
win. The JSONBench cold-run claim (GreptimeDB #1 at 1B docs) is a **wide-scan** workload
(reads most columns anyway), where few-large-objects + object-store-native read cache
help; it does **not** generalize to *selective* cold reads, where granular fetch wins.
The two regimes (wide cold scan vs selective cold lookup) split the verdict — name which
one a given Parallax query is.

## Why my local runs couldn't show this (honest limitation)

All Runs 1–12 were **warm and cache-resident** (≤5M rows, <1 GB — fits in RAM/OS
page cache regardless of app caches). At that scale the cold/warm gap is muted: even
"cold" data sits in the host OS page cache. Demonstrating the divergence requires
either **data > RAM** (drop OS page cache) or **true object-store cold reads**
(evict the local read cache, force S3 GETs) — both need the larger tier / the
object-store harness (B1 cold, B10 request-counts, B12 JSONBench). The *mechanism*
above predicts the divergence direction; the *magnitude* is owed to those runs.

## Axis roll-up

| Sub-axis | Winner | Mechanism | Confidence |
| --- | --- | --- | --- |
| Warm local hot-path query | **ClickHouse** | 5 GiB mark cache + vectorized decompress; data in RAM. | arch + Runs 1–12 |
| Cold object-store re-read — **request count** | **GreptimeDB** | few-objects layout → 9 vs 18 GETs for an anchored lookup (Run 55/B10). | **measured** |
| Cold object-store re-read — **egress (selective)** | **ClickHouse** | granule reads (294 KiB) vs GreptimeDB whole-SST (~23 MiB), ~80×. **Scatter-driven, persists at scale (Run 63):** recommended design keys on `service` so `trace_id` scatters across all row groups (anchored scan 39 ms vs 14 ms clustered); CH `ORDER BY(trace_id,ts)` clusters the anchor free, GreptimeDB PK=series-identity can't without cardinality blowup. | **measured** (Run 55/B10) + mechanism (Run 63) |
| Cold object-store re-read — **wide scan** | **GreptimeDB (predicted)** | few large objects + object-store-native cache; JSONBench cold-run #1 @1B. | arch + vendor claim |
| Warm/repeat re-read (Parallax norm) | **GreptimeDB** | write-through persistent local read cache (survives restart) → ~0 S3 after first touch (Run 55). | **measured** |
| Cache memory footprint / tuning | ~tie, different shape | CH one big mark cache (5 GB) vs GreptimeDB several smaller purpose caches. | arch |
| Index-lookup caching | both | CH index_mark_cache; GreptimeDB inverted/bloom/vector + result cache. | arch |

## Query-result cache — the one layer ClickHouse has, GreptimeDB doesn't (pass 60, footnote)

A distinction worth pinning: everything above is **data/index caching** (blocks, marks,
index pages) — both engines have rich stacks there. ClickHouse *also* has a true
**whole-query-result cache** (`use_query_cache`, **off by default**; `query_cache_ttl=60`
s; `enable_reads_from_query_cache=1` — Run 35): a repeated **identical** `SELECT` returns
the cached *result* and **skips execution entirely** (no re-scan, no re-aggregate, no
re-plan). **Correction (pass 61, from the v1.0.2 changelog review):** GreptimeDB is *not*
"no result cache" as first stated — it has a **partition-range scan-result cache**
(`src/mito2/src/read/range_cache.rs` — "partition range scan result cache", keyed by a
fingerprint of the scan-request fields, reused across queries that scan the same range)
**plus** the index-probe `index/result_cache.rs`. The difference is **granularity**:
ClickHouse caches the **whole query's result** (full execution skip on a hit); GreptimeDB
caches **scan-range results** (skips the scan I/O+decode for matching ranges, but still
re-plans + re-aggregates). So GreptimeDB accelerates repeated scans of the same ranges,
not a whole-query skip. (v1.0.2 fixed a correctness bug — PR #8105 — where the range
cache could reuse a stale result under `merge_mode` + an `OR` filter on the time index;
the pinned version has the fix.)

Consequence (speed, axis #1): for **repeated identical** queries — a Grafana panel
refreshing the same metric/SQL every N seconds — ClickHouse's result cache can skip the
re-aggregation CPU; GreptimeDB re-aggregates each time on warm data (pass 20's agg cost
applies per refresh). **But a footnote for Parallax:** the dominant pattern is *anchored*
evidence-bundle queries on **unique** `trace_id`/`fingerprint` anchors → near-zero
result-cache hit rate, and dashboard clients usually cache at the panel level anyway. A
modest ClickHouse edge for repeated-identical-query dashboards, off-by-default, not a
hot-path differentiator. Does **not** move the verdict.

## Source / evidence

- GreptimeDB caches: `src/mito2/src/cache/{write_cache,file_cache,manifest_cache,index}.rs` (incl. index-probe `index/result_cache.rs`) + **partition-range scan-result cache `src/mito2/src/read/range_cache.rs`** (scan-range results, fingerprint-keyed; v1.0.2 PR #8105 fixed a merge_mode+OR-time-filter correctness bug); defaults `src/mito2/src/config.rs:204-207` (sst_meta 128 MB, vector 512 MB, page 512 MB, selector-result 512 MB); object-store read cache `src/object-store/src/config.rs:318-340` (default on); **no whole-query-result cache** (vs ClickHouse `query_cache`) — Runs 35–36.
- ClickHouse caches: `src/Core/ServerSettings.cpp:496-588,1574` (mark/uncompressed/index/query cache + prewarm); defaults `src/Core/Defines.h:85,88` (mark 5 GiB, uncompressed 0 MiB = off).
- Ties to `local-benchmark-results.md` Runs 8–9/54 (object layout/count), **Run 55 (B10 cold-read request count + egress, measured)**, **Run 63 (`EXPLAIN ANALYZE` scatter-vs-cluster: anchored scan 39 ms scattered vs 14 ms trace_id-clustered → whole-SST cold read is scatter-driven, persists at scale)**, `public-performance-claims.md` (JSONBench cold-run), `benchmarking-the-differences.md` B1/B10/B12.
- GreptimeDB persistent local read cache: `/greptimedb_data/cache` (write-through on flush; survived `docker restart`, cleared only by explicit `rm`) — `src/object-store/src/config.rs:318-340` (default on).
