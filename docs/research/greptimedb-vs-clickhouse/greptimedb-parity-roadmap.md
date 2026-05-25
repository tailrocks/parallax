# GreptimeDB Parity Roadmap — What To Implement To Make It A Clear Winner For All Cases

<!-- markdownlint-disable MD013 -->

Status: pass 76 (new) + pass 77 (gaps #2/#3 source-verified) + pass 78 (gap #1
source-corrected) + pass 79 (**expanded to detailed per-improvement what/why/how**, framed
as borrowed-concept → GreptimeDB structure → value, per operator) + pass 80 (#4 JSON
binary-jsonb + per-row `json_get` source-confirmed; #2 batch-size has no runtime knob,
live-probed) + pass 81 (**added Improvement #7** — per-column codec parity; source-grounded:
user data columns default to `PLAIN`+ZSTD, floats miss the Gorilla-class encoding) + pass 82
(#5 projections **source-confirmed absent**; **added the user-first ranking** + per-improvement
user stories + explicit "does this make GreptimeDB the clear winner?" verdicts, per operator —
the reason to add anything is solving a real Parallax user's problem). This is **the dedicated,
standalone file** answering "what can GreptimeDB improve, why, and how" — the summary table
scans, the detailed sections below carry the code-oriented specifics. Answers the operator
question: GreptimeDB wins Parallax on *fit*
(`verdict-which-to-choose.md`), but ClickHouse is genuinely ahead on a few capabilities —
**what would GreptimeDB have to implement, against its actual internals, to be the
unambiguous choice for every query shape?** For each ClickHouse advantage: the
mechanism, the concrete change in GreptimeDB's real subsystems, the effort tier, and
whether it is a *design* gap or an *integration* gap.

Pins: GreptimeDB `v1.0.2` (`0ef5451`, DataFusion `=52.1`), ClickHouse
`v26.5.1.882-stable` (`5b96a8d8`), re-verified latest stable 2026-05-25.

## The load-bearing finding (why this is mostly tractable)

GreptimeDB's index **toolkit is richer**, not poorer (FST+roaring inverted, Lucene-class
tantivy full-text, configurable-granularity bloom — `indexing-internals.md`). So its
measured losses are **execution-integration gaps, not architectural ones**
(`query-execution-engine.md`): a younger DataFusion-over-Arrow scan engine and a
multi-step index→scan path, versus ClickHouse's decade-tuned C++ pipeline. **Integration
gaps close by engineering; architecture gaps would require redesign.** Almost everything
below is the former — and because GreptimeDB and DataFusion are open-source Rust (the
project's north star), the operator can *contribute* the fixes, not just wait for them.

**Pass-77 source check strengthens this** (both load-bearing engine claims verified against
v1.0.2, not just reasoned): gap #2 — the query `SessionConfig` (`state.rs:126-128`) sets only
`with_target_partitions`, never `batch_size`, so DataFusion's 8,192 default genuinely holds (a
one-line change to raise). Gap #3 — the mito2 reader already prunes row-groups/pages (indexes +
page index) and post-decode-filters rows, but has **no arrow `RowFilter`**, so the PREWHERE fix
is *wiring an arrow primitive that already exists*, not inventing late materialization. Both are
integration, exactly as the thesis predicts.

**Pass-78 source check corrected gap #1** (drift caught in this roadmap's own first draft):
GreptimeDB **already** in-memory-caches the inverted, bloom, and vector indexes + an
index-application result cache (`cache/index/`), so "warm-cache the FST" was already done —
the residual full-text cost is **tantivy-specific** (no `FulltextIndexCache`; the tantivy
applier re-opens a Lucene dir via a file/dir cache per query). Still an integration gap, but a
narrower and more accurate one — and it surfaces a **Tier-A lever**: prefer the already-cached
*bloom* full-text variant where exact phrase isn't required. The correction is itself the
method working: source beats reasoning.

## User-first ranking — does any of this actually make GreptimeDB the clear winner?

The reason to add anything is **solving a real Parallax user's problem**, not parity for its
own sake. Parallax's users are developers / SREs / AI agents debugging an incident: the hot
flow is **anchored evidence-bundle assembly** (fetch everything for a `trace_id`/`fingerprint`),
which is *already fast on GreptimeDB* (Q6 ~33 ms, Run 16). Ranked by who-actually-feels-it:

1. **Improvement 1 (full-text log search) — the only plausible clear-winner-maker, and only
   for the *exploratory* persona.** User story: *an SRE paged at 2 a.m. greps logs for a
   request-id / `payment timeout` across a service over the last hour to find the failing
   path.* This ad-hoc substring search is the one common Parallax action where ClickHouse's
   ~18× warm edge is felt by a human waiting. Closing it makes GreptimeDB clearly win the
   "search logs during an incident" case. **But** it's the *secondary* path (anchored bundle
   assembly is primary and already fast), and the **Tier-A bloom-variant lever** mitigates it
   today — so "huge win" only if real usage shows log-search is frequent.
2. **Improvement 4 (JSON attribute queries) — matters only for unplanned attribute analytics.**
   User story: *a user groups errors by `http.status_code` or filters spans by `user.id`
   across last week.* Felt only at volume on **undeclared** attributes; the Tier-A answer
   (promote hot attributes to real columns) wins the common case. Footnote unless attribute
   exploration is core.
3. **Improvement 2 (scan/agg engine) — Tier-A Flow pre-agg already wins the user-facing case.**
   User story: *a user opens a high-cardinality metric dashboard.* Flow pre-aggregation makes
   that fast (Run 43); the raw-engine gap only bites on *unplanned* heavy aggregation, which
   Parallax users rarely run interactively. Footnote for the common flow.
4. **Improvements 3 (PREWHERE), 5 (projections), 7 (codecs) — footnotes for Parallax users.**
   PREWHERE helps wide-row selective scans the anchored path avoids; projections serve a
   second scan order Parallax rarely needs (anchors on `trace_id`/`fingerprint`); codecs are
   **invisible to users** (a storage-cost second-order lever, and compression is a near-wash).
   None makes GreptimeDB a "clear winner" of a user-felt case.

**Honest bottom line (user-first):** for Parallax's actual hot path, GreptimeDB is *already*
the clear winner — none of these is required. The single improvement that would make it
clearly win a real, common user moment is **#1 (incident log search)**, and even that is
secondary and Tier-A-mitigable. **Validate the query mix first** (what fraction of real
Parallax queries are ad-hoc log search vs anchored retrieval); invest in Tier-B engine work
only if that fraction is high. Anything else is mechanism elegance without user impact.

## The gaps and what closes each

| # | ClickHouse advantage | Mechanism (why CH wins) | What GreptimeDB implements — against its real structure | Tier | Gap kind |
| --- | --- | --- | --- | --- | --- |
| 1 | **Full-text log search ~18× (warm, Run 12/38)** | Coarse `text` posting-list prune **inside** the C++ pipeline, then vectorized `hasToken` confirm on 65k-row blocks — index lookup + confirm are one fast path. | **Cache the tantivy reader + fuse its hit-set into a row-selection.** Source-corrected (pass 78): GreptimeDB already in-memory-caches the **inverted**, **bloom**, and **vector** indexes (`cache/index/{inverted_index,bloom_filter_index,vector_index}.rs`) + an index-application result cache (`result_cache.rs`) — so "warm-cache the FST" is **already done** for the inverted path (why anchored inverted lookup is competitive, Run 6). The full-text gap is **tantivy-specific**: there is **no `FulltextIndexCache`** in `cache/index/`; the tantivy applier opens a Lucene-style directory via a **file/dir cache** (`SstPuffinDir`, `dir_cache_hit/miss`) and re-opens segment readers per query, then DataFusion scans. Implement: (a) an **in-memory tantivy reader/segment cache** (parallel to `InvertedIndexCache`) so the Lucene index stays warm; (b) feed the tantivy hit-set into the arrow `RowFilter` row-selection from gap #3 so only survivors materialize. **Tier-A lever:** for log search not needing exact phrase, prefer the **bloom full-text variant** (`INDEX_BLOB_TYPE_BLOOM`) — it already uses `BloomFilterIndexCache`. Files: `src/mito2/src/sst/index/fulltext_index/applier.rs`, `cache/index/`. | **A** (bloom variant) / **B** (tantivy cache) | integration |
| 2 | **Generic scan/aggregate throughput ~2–4×** | 65,409-row blocks (8× DataFusion's 8,192) + **LLVM-JIT** expressions/aggregation + bespoke SIMD kernels + specialized adaptive hash tables. | (a) **Raise the `RecordBatch` size** in `SessionConfig` — **source-confirmed (pass 77): `state.rs:126-128` sets only `with_target_partitions`, never `batch_size`, so DataFusion's 8,192 default holds** → raise toward 32–64k so vectors amortize overhead and feed SIMD; (b) **expression + aggregation codegen** — DataFusion's is young/narrow vs LLVM JIT; (c) **specialized SIMD aggregation** (two-level hash for high-card, fixed-width-key kernels). Mostly **upstream DataFusion** — GreptimeDB inherits as DF improves, or contributes. | **B** | integration (upstream DataFusion) |
| 3 | **Late materialization (PREWHERE)** | Decodes cheap filter columns first → row mask → decodes wide columns only for surviving rows. | **Add column-staged late materialization to the mito2 reader.** Source-confirmed (pass 77): GreptimeDB already **prunes** row-groups/pages (`RowGroupSelection` from Puffin fulltext/inverted indexes + the Parquet **page index**, `reader.rs`) and then **post-decode**-filters rows (`PruneReader::precise_filter`, `read/prune.rs:119`) — so within a surviving row-group it decodes **all projected columns before dropping rows**. There is **no arrow `RowFilter`** in the reader (grep = 0) → no column-staging. Fix: wire the pushed-down predicate into arrow's **`RowFilter`** (`ParquetRecordBatchReaderBuilder::with_row_filter`, which arrow-rs already provides) so filter columns decode first, build a selection, and wide/`Json` columns materialize only for survivors. File: `src/mito2/src/sst/parquet/reader.rs`. **Arrow ships the primitive → integration, not a new algorithm.** | **B** | integration |
| 4 | **Dynamic-attribute path queries (JSON)** | `JSON` type stores each path as a **typed columnar subcolumn** → `attributes.user` reads one subcolumn. GreptimeDB `Json` is a **binary blob** (jsonb) + `json_get_*` per-row parse (`schema-evolution-and-dynamic-columns.md`). | **Shred JSON paths into Parquet subcolumns.** Adopt the emerging Parquet **Variant/shredding** layout: at flush, write hot/declared attribute paths as their own typed Parquet columns; push `attributes.k` access down to a subcolumn scan instead of a per-row blob parse. Files: a shredded column type + mito2 SST writer + DataFusion pushdown. **Biggest storage-format change here** — borders on design, but Parquet's variant work makes it integration, not a rewrite. | **B** | integration / format |
| 5 | **Projections (alternate physical `ORDER BY`)** | A projection stores a 2nd sort order **inside each part**, optimizer-picked → fast sequential scan on an alternate key from one table (Run 28). GreptimeDB indexes give *positions*, not a 2nd physical order. | Two paths: (a) **today, Tier A workaround** — maintain a re-sorted derived table via **Flow** (`CREATE FLOW … SINK TO …` keyed on the alternate order; Run 43 proved Flow works) — extra storage, app-managed; (b) **true parity, Tier B** — mito2 writes an alternate-sorted SST copy per region with planner auto-pick. Files: `src/mito2` SST writer + region metadata + DataFusion planner rule. | **A** (workaround) / **B** (native) | integration |
| 7 | **Per-column codecs (`CODEC(Gorilla/DoubleDelta/T64)`)** | Hand-picked per-column codecs match each column's shape — `Gorilla` on float gauges (Run 4: 78×), `DoubleDelta` on monotonic counters (7.3×). | **Type-aware Parquet encodings + a column codec DDL option.** Source (pass 81): mito2's writer **already** sets per-column encodings for *internal* columns (`writer.rs:387-391`: `ts`/`seq` → `DELTA_BINARY_PACKED` ≈ DoubleDelta, `op_type` → UNCOMPRESSED) but **user data columns default to `Encoding::PLAIN` + table-wide ZSTD** (`writer.rs:433-434`) — so a `Float64` gauge gets **no Gorilla/float-split encoding**. Fix: in the writer's `customize_column_config` (`writer.rs:371`), pick **type-aware encodings** for user columns (floats → Parquet **`BYTE_STREAM_SPLIT`** ≈ Gorilla; monotonic ints → `DELTA_BINARY_PACKED`); optionally expose a per-column codec option in DDL. Parquet already ships these encodings → integration. | **B** | integration |
| 6 | **Vertical single-node ceiling + analytical/merge maturity** | A decade of single-box scan tuning; battle-tested merges. | Inherited via #2 (engine) + time/battle-testing. Not a discrete feature. | **C** | maturity |

## Detailed improvements (borrowed concept · what · why · how)

Each improvement is framed as: a **concept borrowed** from the system that does it well
(almost always ClickHouse), then **how that concept lands in GreptimeDB's real structure**
(mito2 region engine, DataFusion `=52.1`, Puffin index, OpenDAL) to provide value for
Parallax. Source read at GreptimeDB `v1.0.2` (`0ef5451`).

### Improvement 1 — In-memory tantivy full-text cache + hit-set→row-selection fusion

- **Borrowed concept (ClickHouse):** the `text` index isn't just *stored* well — its
  posting-list lookup and the `hasToken` confirmation run **in the same vectorized
  pipeline**, on warm in-memory structures, over 65k-row blocks. The portable idea is
  "keep the search index hot in memory and hand its matches straight to the scan," not
  "copy ClickHouse's index format" (GreptimeDB's tantivy index is already richer).
- **What:** add an in-memory cache for the tantivy full-text reader/segments, and feed its
  matched row-set directly into the Parquet row-selection so only survivors are decoded.
- **Why:** full-text log search is ClickHouse-faster **~18× warm** (Run 12, re-confirmed
  warm Run 38 — index-bound, not a cold artifact). Source (pass 78) localizes the cause:
  GreptimeDB **already** in-memory-caches the inverted/bloom/vector indexes
  (`cache/index/{inverted_index,bloom_filter_index,vector_index}.rs`) + an
  index-application result cache (`result_cache.rs`) — which is why anchored *inverted*
  lookup is competitive (Run 6). But there is **no `FulltextIndexCache`**: the tantivy
  applier opens a Lucene-style directory through a **file/dir cache** (`SstPuffinDir`,
  `dir_cache_hit/miss`) and re-opens segment readers **per query**, then DataFusion scans.
  So the gap is a missing warm cache for the one index type Parallax leans on for logs.
- **How (code-oriented):** (1) add a `FulltextIndexCache` member alongside the existing
  caches in `src/mito2/src/cache/index/` (mirror `inverted_index.rs`) and wire it through
  `CacheManager` in `src/mito2/src/cache.rs` (`inverted_index_cache()` → add
  `fulltext_index_cache()`); cache the opened `tantivy` `Index`/segment readers keyed by
  `RegionIndexId`, not just the Puffin dir bytes. (2) In
  `src/mito2/src/sst/index/fulltext_index/applier.rs`, return the matched `RowId` set and
  feed it into the gap-#3 arrow `RowFilter`/`RowSelection` instead of a post-decode filter.
- **Tier:** **A** (lever) + **B** (cache). **Integration**, not redesign.
- **User story & clear-winner:** *an SRE paged at 2 a.m. greps logs for a request-id /
  `payment timeout` across a service over the last hour to find the failing path* — the one
  common Parallax action where ClickHouse's ~18× warm edge is felt by a human waiting. **By
  adding this, GreptimeDB clearly wins the "search logs during an incident" case** — the only
  improvement here that flips a real, common user moment (see the user-first ranking above).
  Caveat: it is the *secondary* path (anchored bundle assembly is primary and already fast),
  so the huge win lands only if real usage shows log search is frequent.
- **Value here:** directly shrinks the only large *warm* gap (log search). **Tier-A lever
  first:** for Parallax log search that does **not** need exact-phrase ranking, declare the
  **bloom full-text variant** (`INDEX_BLOB_TYPE_BLOOM`) on `message` — it already reuses
  `BloomFilterIndexCache`, no engine change. Reserve the tantivy cache (Tier B) for when
  exact phrase/relevance matters.

### Improvement 2 — Bigger execution batch + expression/aggregation JIT + SIMD aggregation

- **Borrowed concept (ClickHouse):** throughput comes from **wide vectors** (65,409-row
  blocks ≈ 8× DataFusion's 8,192), **LLVM-JIT** of expression trees and aggregation
  (`compile_expressions`/`compile_aggregate_expressions`), bespoke **SIMD** kernels, and
  **specialized adaptive hash tables** (two-level for high cardinality). Portable ideas:
  fewer, larger batches; compile hot expressions; specialize aggregation.
- **What:** raise GreptimeDB's RecordBatch size and adopt DataFusion's codegen/SIMD
  aggregation as it matures (or contribute it).
- **Why:** generic scan/aggregate is ClickHouse-faster ~2–4× (Run 11→37: metric agg ~2×
  warm; full scans ~4×, Run 39). Source (pass 77): the query `SessionConfig`
  (`src/query/src/query_engine/state.rs:126-128`) sets only `with_target_partitions` and
  **never `batch_size`**, so DataFusion's 8,192 default holds — small vectors carry more
  per-batch overhead and feed SIMD worse than ClickHouse's 65k blocks.
- **How (code-oriented):** (1) **one-line, cheap to try:** add
  `.with_batch_size(32768)` (tune 16–64k) to the `SessionConfig` builder in
  `state.rs:126-128`; measure the agg gap before/after (a proposed `benchmarking-the-
  differences.md` case). **Pass-80 live probe:** v1.0.2 has **no runtime knob** for this —
  `SET datafusion.execution.batch_size` (and `execution.batch_size`, `batch_size`) all return
  `Not supported: Unsupported set variable`, so it requires the code change (or a
  config-plumb to expose it), not a session `SET`. (2) **Upstream/contributed:** DataFusion expression + aggregate
  codegen and specialized grouping — track the DataFusion roadmap; GreptimeDB inherits on
  the `datafusion = "=52.x"` bump (`Cargo.toml`). (3) SIMD aggregation kernels: upstream
  arrow/DataFusion.
- **Tier:** **B** (batch size is a trivial config change; JIT/SIMD ride upstream
  DataFusion). **Integration.**
- **Value here:** narrows the heavy-scan/aggregation gap; mostly matters for ad-hoc
  analytics, not the anchored hot path. The batch-size probe is the cheapest experiment in
  this whole roadmap — do it first to size the win.

### Improvement 3 — Column-staged late materialization (PREWHERE) via arrow `RowFilter`

- **Borrowed concept (ClickHouse):** **PREWHERE** — decode only the cheap filter columns
  first, build a row mask, then decode the wide columns **only for surviving rows**. The
  portable primitive is "predicate-during-decode + late column materialization," which
  Parquet/arrow-rs already supports.
- **What:** decode filter columns first in the mito2 Parquet reader and late-materialize
  wide/`Json` columns only for rows that pass.
- **Why:** source (pass 77) shows GreptimeDB already prunes **row-groups/pages**
  (`RowGroupSelection` from Puffin indexes + the Parquet **page index**) and then
  **post-decode**-filters rows (`PruneReader::precise_filter`, `src/mito2/src/read/prune.rs:119`)
  — so within a surviving row-group it decodes **all projected columns before dropping
  rows**. There is **no arrow `RowFilter`** in the reader (grep = 0). For wide rows
  (spans/logs with big `message`/`attributes`) that wastes decode on rows the filter drops.
- **How (code-oriented):** in `src/mito2/src/sst/parquet/reader.rs`, build an arrow
  `RowFilter` from the pushed-down `Predicate` (the predicate is already plumbed —
  `predicate(...)` builder at line 185) and attach it via
  `ParquetRecordBatchReaderBuilder::with_row_filter`. arrow-rs ships `RowFilter` /
  `ArrowPredicate` — so this is **wiring an existing primitive**, decode the predicate
  columns first and let arrow late-materialize the rest.
- **Tier:** **B**. **Integration** (no new algorithm).
- **Value here:** helps wide-row selective scans (log/trace filters that aren't fully
  anchored); modest for the anchored bundle (already tiny row sets). Pairs with Improvement
  1 (the tantivy hit-set becomes the `RowFilter`).

### Improvement 4 — Shredded (typed-subcolumn) JSON for OTLP attributes

- **Borrowed concept (ClickHouse):** the `JSON` type stores each discovered path as a
  **typed columnar subcolumn**, so `attributes.user` reads exactly one subcolumn with full
  codec/skip benefits. Portable idea: **shred** the dynamic document into columns instead
  of keeping one opaque blob.
- **What:** store hot/declared attribute paths as their own Parquet columns and push path
  access down to a subcolumn scan.
- **Why:** GreptimeDB's `Json` is a **binary blob** (jsonb) read with `json_get_*`
  **per-row parse** — every `attributes.k` filter parses the whole blob for every row, vs
  ClickHouse reading one pre-split subcolumn. **Source-confirmed (pass 80, v1.0.2):** the
  `Json` type is stored via `BinaryVectorBuilder` (`src/datatypes/src/types/json_type.rs`);
  `json_get_*` is a DataFusion **scalar UDF** that calls `jsonb::get_by_path(...)` element-wise
  over the binary column (`src/common/function/src/scalars/json/json_get.rs`); the
  `JsonGetRewriter` (`json_get_rewriter.rs`) is only a **logical function-canonicalization**
  (a DataFusion `FunctionRewrite`), **not** a subcolumn pushdown — there are no subcolumns to
  push to. (v1.0.2 has a `JsonNativeType`→`Struct`/`List` typed *representation* for value
  conversion, but storage stays binary jsonb.) Axis: cost + speed on dynamic-attribute
  queries (Q5-shaped).
- **How (code-oriented):** adopt the emerging **Parquet Variant/shredding** layout: at
  flush in the mito2 SST writer (`src/mito2/src/sst/parquet/`), split declared/hot paths of
  a `Json` column into typed Parquet leaf columns; in the read path, lower
  `json_get_*(attributes,'k')` to a direct subcolumn projection so only that leaf decodes.
  This is the **largest change** here — a storage-format addition — but Parquet's variant
  work makes it an integration along an established spec, not a from-scratch type.
- **Tier:** **B**. **Integration / format** (borders on design — flag if it grows).
- **Value here:** only matters if Parallax does heavy arbitrary-attribute filtering at
  volume; for declared hot attributes, promoting them to real columns in the schema (Tier-A
  today) captures most of the benefit without the format work.

### Improvement 5 — Alternate physical ordering (projection-equivalent)

- **Borrowed concept (ClickHouse):** a **projection** stores a *second* physical `ORDER BY`
  inside each part, optimizer-picked transparently → one table serves two access orders
  (e.g. `service`-time **and** `trace_id`) at sequential-scan speed. Portable idea: keep a
  second physically-sorted copy and pick it automatically.
- **What:** give GreptimeDB an alternate-ordering structure for tables queried on two keys.
- **Why:** GreptimeDB's secondary indexes give *positions*, not a second physical sort
  order (Run 28, `projections-and-access-paths.md`) — a scan on a non-primary key can't get
  ClickHouse's projection locality. **Source-confirmed absent (pass 82, v1.0.2):** no
  `PROJECTION` keyword in `src/sql/src/parsers/{create_parser,alter_parser}.rs` (grep = 0),
  and the `AlterTableOperation` enum has no projection/alternate-ordering variant (only
  Add/Drop/ModifyColumn, Rename, Set/UnsetTableOptions, Set/UnsetIndex{Fulltext,Inverted,
  Skipping}) — so there is genuinely no engine-native second physical order.
- **User story & clear-winner:** *a user wants a service's span timeline (`service`-time
  order) **and** the trace bundle (`trace_id` order) fast from the spans table.* Parallax
  anchors on `trace_id`/`fingerprint`, so the second scan order is rarely hot — **this does
  not make GreptimeDB a clear winner of a common user case**; the Flow workaround covers the
  rare instance. Footnote (see ranking above).
- **How (code-oriented):** (a) **Tier-A today:** maintain a re-sorted derived table with a
  **Flow** (`CREATE FLOW … SINK TO alt_table` keyed on the alternate order — Flow verified
  Run 43); the app/optimizer queries whichever table matches. Extra storage, no engine
  change. (b) **Tier-B native parity:** have the mito2 SST writer emit an alternate-sorted
  copy per region + region metadata + a DataFusion planner rule to auto-pick — a larger
  build.
- **Tier:** **A** (Flow workaround) / **B** (native). **Integration.**
- **Value here:** small for Parallax — anchored retrieval already keys on `trace_id`/
  `fingerprint`; matters only if a second high-volume scan order emerges. Use the Flow
  workaround if/when it does.

### Improvement 7 — Type-aware Parquet encodings (per-column codec parity)

- **Borrowed concept (ClickHouse):** per-column **`CODEC()` chains** — `Gorilla` for float
  gauges, `DoubleDelta` for monotonic counters/timestamps, `T64`, etc. The portable idea is
  "pick the encoding that matches each column's *shape*," and it is general: **Parquet already
  defines equivalent column encodings** (`BYTE_STREAM_SPLIT` for floats, `DELTA_BINARY_PACKED`
  for monotonic ints, dictionary/RLE) — so this is not ClickHouse-specific.
- **What:** select type-aware Parquet encodings for user data columns (and optionally expose a
  per-column codec option in DDL), instead of defaulting every data column to `PLAIN` + ZSTD.
- **Why:** compression is mostly a **wash** (GreptimeDB's auto Parquet+ZSTD ties or beats
  ClickHouse out-of-the-box on logs/dict-friendly columns — Run 10), **but** ClickHouse wins
  where a specialized codec matches: float gauges via `Gorilla` (Run 4: gauge 84.7 KiB from
  6.59 MiB raw = **78×**) and monotonic counters via `DoubleDelta` (7.3×). Source (pass 81):
  GreptimeDB's writer (`src/mito2/src/sst/parquet/writer.rs`) **already** customizes internal
  columns (`:387-391` — `ts`/`seq` → `DELTA_BINARY_PACKED`, the DoubleDelta-equivalent for the
  time index, which is why GreptimeDB's `ts` compression is competitive) but sets user data
  columns to **`Encoding::PLAIN` + ZSTD** (`:433-434`). A flat/slow-moving `Float64` gauge
  therefore misses the Gorilla-class win ClickHouse gets. Axis: cost (#2), and only the
  numeric-metric columns — second-order for Parallax's overall bytes.
- **How (code-oriented):** extend `customize_column_config` (`writer.rs:371`, already the place
  that sets `set_column_encoding(...)` per internal column) to choose encodings by **column
  type/semantic**: `Float64` field → Parquet **`BYTE_STREAM_SPLIT`** (splits mantissa/exponent
  byte planes → ZSTD compresses flat floats far better, the Gorilla analog); monotonic integer
  field → `DELTA_BINARY_PACKED`; low-card string → keep dictionary. The writer already imports
  `parquet::basic::Encoding` and calls `set_column_encoding`, so this is **filling in a
  per-column policy that the code is structured for** (there is even a `TODO` at `:430` to set
  proper encodings for internal columns). Optionally add a `CREATE TABLE … col Float64
  CODEC(...)`-style DDL option mapping to the same `Encoding`.
- **Tier:** **B**. **Integration** — Parquet ships the encodings; the writer already does
  per-column config; this extends the policy to user columns.
- **Value here:** low/second-order. Compression is a near-wash and a 1.3–1.9× local-disk delta
  is not the cost driver (object-store request economics dominate — `compression-and-cost.md`).
  Worth it mainly for **metric float columns** if/when retained metric volume is large;
  otherwise leave GreptimeDB's zero-tuning default, which is itself an ergonomics win.

### Improvement 6 — Vertical ceiling + analytical/merge maturity

- **Borrowed concept:** none portable — this is ClickHouse's decade of single-box scan
  tuning and battle-tested merges.
- **What / How:** inherited via Improvement 2 (engine throughput) plus time and
  production hardening; not a discrete feature to implement.
- **Tier:** **C** (accept or wait). **Maturity**, not integration.
- **Value here:** GreptimeDB's answer is horizontal scale-out + object storage (the
  verdict's scaling pillar), not chasing ClickHouse's vertical ceiling.

## The three tiers (the decision-useful summary)

- **Tier A — close it in Parallax today, no engine change.** Index `trace_id`/`fingerprint`
  (Run 45 design), **Flow pre-aggregation** for dashboards (Run 43, ~5–6× read speedup,
  neutralizes the ~2× SQL-agg gap), **SQL not PromQL** for hot aggregations (Run 44: GT SQL
  ~5× faster than GT's own PromQL), FULLTEXT-index + anchored log search, and the Flow
  alternate-ordering workaround (#5a). **This already makes GreptimeDB a clear winner for
  Parallax's *anchored* workload** — the gaps below only bite on heavy *ad-hoc* analytics.
- **Tier B — contribute upstream (Rust, open-source) to win *all* cases.** Items 1–5: batch
  size, JIT, SIMD, PREWHERE late materialization, JSON shredding, native projections,
  index↔scan fusion. These are what "clear winner even for large-scale ad-hoc log/trace
  search and heavy scans" requires. Several (#2, #3) ride the **DataFusion roadmap** and
  arrive without Parallax work; the rest are GreptimeDB-specific but **engineering, not
  redesign**, and PR-able given the Rust-first north star.
- **Tier C — accept or wait.** Distributed/analytical battle-testing; matures with time.

## The alternative to closing the gap: a hybrid

If the Tier-B work is not worth it and ad-hoc log search proves heavy, the alternative is a
**GreptimeDB (system-of-record + correlation) + ClickHouse (log-search sidecar)** split.
Honest cost: it **splits Parallax's cross-signal evidence-bundle correlation across two
engines** — exactly the hot path the whole product is built on — turning anchored bundle
assembly into a cross-engine fan-out, plus doubling the ops surface (against the
startups-first/tiny-single-node trajectory). Only justified if a benchmark shows log search
is both **heavy** and **standalone** (not joined with other signals). Never split the bundle.
See `verdict-which-to-choose.md` (DQ5) and `distributed-and-scaling.md`.

## Bottom line

For Parallax's **anchored** workload, **Tier A alone makes GreptimeDB a clear winner today**
— no engine changes needed. To make it the unambiguous choice for **every** shape including
heavy ad-hoc analytics, the remaining gaps (1–5) are **execution-integration**, mostly on
the **DataFusion roadmap** or contributable in Rust — *not* architectural limits. That is the
encouraging answer to "what must GreptimeDB implement": engineering, not redesign. Validate
first which gaps Parallax's real query mix actually hits before investing in Tier B.

## Open questions handed to the benchmark

- Quantify the post-Tier-A residual: with FULLTEXT + warm index cache, how much of the ~18×
  log-search gap remains? (Owed to a tuned re-run; `benchmarking-the-differences.md`.)
- Does raising the DataFusion batch size in `SessionConfig` measurably narrow the ~2× agg
  gap at 40k series? (A cheap local probe — propose as a new case.)
- JSON-shredding payoff: attribute-path query latency, blob-parse vs subcolumn — at volume.

## Source / evidence

- Mechanisms: `query-execution-engine.md` (batch/JIT/SIMD/PREWHERE), `indexing-internals.md`
  (Puffin/tantivy vs `text`, integration-not-format), `read-path-indexing-and-execution.md`
  (pushdown/skip), `schema-evolution-and-dynamic-columns.md` (JSON blob vs subcolumn),
  `projections-and-access-paths.md` (Run 28), `rollup-and-continuous-aggregation.md` (Flow).
- Empirical: `local-benchmark-results.md` Runs 11/12/37/38 (the gaps), 43 (Flow), 44 (PromQL
  vs SQL), 45 (GreptimeDB schema build).
- Source (pass 77, v1.0.2): `src/query/src/query_engine/state.rs:126-128` (SessionConfig =
  `with_target_partitions` only, no `batch_size`); `src/mito2/src/sst/parquet/reader.rs`
  (`RowGroupSelection` + `PageIndexPolicy` + `prune_row_groups_by_{fulltext,inverted}_index`;
  **no `RowFilter`**); `src/mito2/src/read/prune.rs:119` (`PruneReader::precise_filter` =
  post-decode row filtering).
- Source (pass 78, v1.0.2): `src/mito2/src/cache/index/` = `{inverted_index, bloom_filter_index,
  vector_index, result_cache}.rs` (**no `fulltext_index` cache**); `src/mito2/src/cache.rs`
  (`InvertedIndexCache`/`BloomFilterIndexCache`/`VectorIndexCache`/`PuffinMetadataCache`);
  `src/mito2/src/sst/index/fulltext_index/applier.rs` (`TantivyFulltextIndexSearcher` over
  `SstPuffinDir` + `dir_cache_hit/miss`; bloom variant uses `BloomFilterIndexCacheRef`).
- Source (pass 80, v1.0.2): `src/datatypes/src/types/json_type.rs` (`Json` via
  `BinaryVectorBuilder` = jsonb binary); `src/common/function/src/scalars/json/json_get.rs`
  (`jsonb::get_by_path` per-row scalar UDF) + `json_get_rewriter.rs` (`JsonGetRewriter` =
  logical `FunctionRewrite`, not subcolumn pushdown). Live: `SET …batch_size` →
  `Unsupported set variable` (no runtime knob for #2).
- Source (pass 81, v1.0.2): `src/mito2/src/sst/parquet/writer.rs` — `customize_column_config`
  (`:371`) sets internal-column encodings (`:387-391` `ts`/`seq` → `DELTA_BINARY_PACKED`,
  `op_type` → UNCOMPRESSED) but user data columns default to `Encoding::PLAIN` + ZSTD
  (`:433-434`); `TODO` at `:430`. Confirms #7: no Gorilla-class float encoding on user columns,
  no per-column codec DDL.
- Source (pass 82, v1.0.2): `src/sql/src/parsers/{create_parser,alter_parser}.rs` — **no
  `PROJECTION` keyword** (grep = 0); `AlterTableOperation` = Add/Drop/ModifyColumn, Rename,
  Set/UnsetTableOptions, Set/UnsetIndex{Fulltext,Inverted,Skipping} — no projection/alternate-
  ordering op. Confirms #5: no engine-native second physical order.
- Decision context: `verdict-which-to-choose.md`. Loop target: `prompts/greptimedb-vs-clickhouse-internals.md`
  ("Closing The Gap").
