# Caching and Cold-vs-Warm Divergence (Subsystem #7)

<!-- markdownlint-disable MD013 -->

Status: pass 24. The cache hierarchy of each engine and **why cold-cache and
warm-cache performance diverge** — the dimension my local runs (all warm,
cache-resident at ≤5M rows) could not measure, and the mechanism behind the
cold-object-store regime the verdict now hinges on (JSONBench cold-run,
`public-performance-claims.md`). Code-grounded.

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

## The decisive consequence for Parallax (object-store cold re-reads)

Parallax re-reads evidence bundles from cheap object storage, often **cold**. In
that regime the cache + object-layout interaction (this note + B10) predicts:

- **GreptimeDB**: cold bundle re-read = a handful of S3 GETs (few large Parquet
  objects) → local read cache → warm thereafter. The whole stack (OpenDAL read
  cache + index caches + few objects) is **built for cold object-store reads**.
- **ClickHouse on an S3 disk**: cold re-read = **many S3 GETs** (one per column
  file per part — 74 objects for the same 1M spans, Run 9) → S3-disk filesystem
  cache. More cold round-trips + request cost; its mark/uncompressed caching assumes
  local disk, not S3 latency.

**This is the mechanism behind the JSONBench cold-run result** (GreptimeDB #1 cold
at 1B docs, `public-performance-claims.md`): fewer cold object-store round-trips +
an object-store-native read cache. It also explains why my **warm small-scale runs
favoured ClickHouse** (everything in RAM/mark-cache, ClickHouse's vectorized hot
path dominates) while the **cold object-store regime can favour GreptimeDB** — the
two regimes are genuinely different, and Parallax lives in the cold one for
retention re-reads.

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
| Cold object-store re-read | **GreptimeDB (predicted)** | object-store read cache + few large objects → few cold S3 GETs (B10); JSONBench cold-run. | arch + B10 + vendor claim |
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
- Ties to `local-benchmark-results.md` Runs 8–9 (object layout), `public-performance-claims.md` (JSONBench cold-run), `benchmarking-the-differences.md` B1/B10/B12.
