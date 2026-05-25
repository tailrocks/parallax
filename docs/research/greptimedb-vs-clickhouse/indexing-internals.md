# Indexing Internals — Index File Formats and Why Richer ≠ Faster

<!-- markdownlint-disable MD013 -->

Status: pass 43 + pass 78 (index-cache asymmetry source-checked: inverted/bloom/vector
cached in-memory, **tantivy full-text not** — part of the ~18× gap). The storage half of checklist #3: **how each engine stores its
secondary/skip indexes on disk, what they can skip, and the honest paradox** that
GreptimeDB's *richer* index toolkit still lost full-text search ~18× (Run 12). The
read-path note covers *what each index lets the engine skip*; this is the **on-disk
index format + build mechanism** and the reconciliation with the measured results.
Source + live file-format check (Run 22).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## GreptimeDB — one Puffin sidecar per SST, a rich toolkit

Every indexed SST gets a **`.puffin` sidecar file** (Puffin = the Iceberg blob/stats
container format) with the **same UUID** as its `.parquet` — confirmed live (Run 22:
`6e4627ae….parquet` + `6e4627ae….puffin`). All of an SST's indexes live as **named
blobs** inside that one Puffin file (`src/mito2/src/sst/index/puffin_manager.rs`):

| Index | Blob type | Implementation (`src/index` Cargo) | What it does |
| --- | --- | --- | --- |
| **Inverted** | `greptime-inverted-index-v1` | **`fst`** term dictionary → **`roaring`** bitmap posting lists | A *true secondary index*: term → exact segment/row bitmap. Pinpoints matches, not just prunes. |
| **Full-text** | `greptime-fulltext-index-v1` / `-bloom` | **`tantivy` 0.24** (Lucene-class: tokenize + posting lists, `tantivy-jieba`) **or** a **`fastbloom`** per-segment token-presence variant | Token/phrase search; tantivy is a real search engine, the bloom variant is the cheap option. |
| **Skipping (bloom)** | `greptime-bloom-filter-v1` | **`fastbloom`** | Segment-granular "is this value possibly here" prune. |
| Vector | — | HNSW-ish | Similarity (not Parallax-relevant). |

Granularity is **configurable** (`SKIPPING INDEX WITH(granularity=…)`), segment-based —
can be finer than a ClickHouse granule. The index is built at flush/compaction and
written into the Puffin blob; on read the applier loads the blob, evaluates the
predicate, and yields the row/segment set to scan.

## ClickHouse — sparse primary index + per-skip-index `.idx` files

ClickHouse has two different things:

1. **Sparse primary index** `primary.cidx` (per part) over the `ORDER BY` key — the
   *main* pruning structure (one mark per 8,192-row granule). This is what makes
   anchored `trace_id` lookups cheap when `trace_id` is the sort prefix (Run 2/6).
2. **Data-skipping indexes** — per part, each is its own file pair
   **`skp_idx_<name>.idx`** (+ `.cmrk4` marks), confirmed live (Run 22:
   `skp_idx_i_tid.idx`, `skp_idx_i_msg.idx`, `skp_idx_i_lvl.idx`). Types:
   `minmax`, `set(N)`, `bloom_filter`, `tokenbf_v1`, `ngrambf_v1`, and the GA-26.2
   **`text`** (posting-list). `GRANULARITY N` = one index entry per **N table-granules**
   (8,192 rows each) → **coarse**: it prunes *granule ranges*, then the vectorized scan
   confirms inside survivors. It does **not** pinpoint rows.

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Container | **one `.puffin` per SST** (all indexes, named blobs) | `primary.cidx` + **one `skp_idx_*.idx` per skip index** per part |
| Inverted/secondary | **FST + roaring** (term → exact rows) | none — relies on `ORDER BY` sparse primary index for locality |
| Full-text | **tantivy** (Lucene-class) or bloom | `text` posting-list (GA 26.2), `tokenbf`/`ngrambf` |
| Skip granularity | configurable segment (can be fine) | `GRANULARITY × 8,192` rows (coarse) |
| Precision | can **pinpoint** matching rows | **prunes granule ranges**, scan confirms |

On *format*, GreptimeDB's toolkit is the **richer and more precise** one: a real
FST+roaring inverted index and a Lucene-class full-text engine, versus ClickHouse's
coarse granule-pruning skip indexes.

## The paradox: richer index, slower search (Run 12 reconciled)

Despite the richer toolkit, **ClickHouse full-text search was ~18× faster** (Run 12,
5M logs, both indexed) and its anchored `trace_id` lookup was faster even after
GreptimeDB added an inverted index (Run 6: 14→8 ms, still not ClickHouse's 2 ms). The
mechanism — **index format richness does not decide query speed; end-to-end execution
integration does** (ties to `query-execution-engine.md`):

- **ClickHouse** applies the coarse `text`/skip index to prune granules, then confirms
  with its decade-tuned vectorized `hasToken` scan — index lookup and confirmation are
  both inside the fast C++ pipeline, on 65k-row blocks.
- **GreptimeDB** loads the Puffin tantivy/inverted blob, evaluates it, maps the hits
  back to row groups, then DataFusion scans/materializes — a *more precise* index but a
  *younger, multi-step* end-to-end path with more per-query overhead. **Cache asymmetry
  (pass 78 source):** the **inverted**, **bloom**, and **vector** indexes have **in-memory
  caches** (`cache/index/{inverted_index,bloom_filter_index,vector_index}.rs`) + an
  index-application result cache (`result_cache.rs`) — so the inverted path stays warm (why
  anchored lookup is competitive). But there is **no `FulltextIndexCache`**: the **tantivy**
  full-text variant re-opens a Lucene-style directory through a file/dir cache
  (`SstPuffinDir`, `dir_cache_hit/miss`) per query (`fulltext_index/applier.rs`), heavier than
  the cached in-memory FST path. The cheaper **bloom** full-text variant (`INDEX_BLOB_TYPE_BLOOM`)
  *does* reuse `BloomFilterIndexCache`. **Refinement (Run 47):** but the missing tantivy cache
  is *not* the dominant cost — live metric isolation shows the fulltext index **apply is
  ~0.15 ms (~0.1 % of a ~150 ms `matches()` query)**, so the ~18× warm gap is the **post-index
  scan/count over the matched rows**, i.e. scan-engine maturity (`query-execution-engine.md`),
  not the index lookup or its cache (`greptimedb-parity-roadmap.md` #1, reordered accordingly).
- For the **anchored point lookup**, ClickHouse's `ORDER BY` **sort-key locality**
  (primary.cidx) beats *any* secondary index: the rows are physically contiguous, so it
  reads one granule and needs no separate index load. GreptimeDB's inverted index helps
  but still pays the load+apply+scan path (Run 6).

So GreptimeDB's index *capability* is a genuine strength (precise secondary indexing,
true full-text, configurable fine granularity) but **not a speed win at current
execution maturity** — the bottleneck is the scan/apply engine (pass 42), not the
index structure.

## Axis consequence

- **Speed (axis #1):** ClickHouse wins log-search and anchored lookups not because of
  better *indexes* but because of better *index↔scan integration* (sort-key locality +
  vectorized confirm). Richer indexes don't rescue GreptimeDB here. **But** Parallax's
  anchored bundle queries already prune to a tiny row set, where neither index richness
  nor scan speed dominates (Q6 not throughput-bound, Run 16).
- **Capability:** GreptimeDB's FST/roaring inverted + tantivy full-text are the more
  capable *toolkit* — relevant if Parallax later needs precise high-cardinality
  secondary indexing or real BM25-style search; today ClickHouse's coarser-but-faster
  path wins the measured workloads.
- **Cost:** Puffin keeps one sidecar per SST (fewer files, fits the object-store
  few-objects story); ClickHouse adds one `.idx` per skip index per part (more small
  files) — a minor footnote on the object-count axis already covered in
  `caching-and-cold-warm.md`.

Net: this **does not flip** the verdict. It corrects a tempting wrong inference ("the
engine with the richer indexes searches faster") — the opposite held, and the reason is
execution integration, not index design. Honest mechanism win for the analysis.

## Honest caveats

- Smoke scale; the index-build cost and the search latency at GB-scale cold cache are
  not measured here (Run 12 was warm 5M) — owed to the harness.
- ClickHouse's `text` index is young (GA 26.2); GreptimeDB's bloom full-text variant is
  also recent — both index stacks are moving, re-check on version bumps.
- GreptimeDB inverted-index precision *can* win where ClickHouse has no sort-key
  locality and the skip index is weak (very high-cardinality equality on a non-sort
  column) — not yet isolated in a benchmark; a candidate case for
  `benchmarking-the-differences.md`.

## Source / evidence

- GreptimeDB: `src/mito2/src/sst/index/{puffin_manager,inverted_index,fulltext_index,bloom_filter}.rs`
  (blob types `greptime-inverted-index-v1`, `greptime-fulltext-index-v1`/`-bloom`,
  `greptime-bloom-filter-v1`); `src/index` Cargo (`fst`, `roaring 0.10`, `tantivy 0.24`,
  `fastbloom 0.8`). Live (Run 22): `<uuid>.puffin` beside `<uuid>.parquet`. Cache (pass 78):
  `src/mito2/src/cache/index/{inverted_index,bloom_filter_index,vector_index,result_cache}.rs`
  (no `fulltext_index`); `src/mito2/src/sst/index/fulltext_index/applier.rs` (tantivy via
  `SstPuffinDir` dir-cache; bloom variant via `BloomFilterIndexCache`).
- ClickHouse: `src/Storages/MergeTree/MergeTreeIndex{BloomFilter,BloomFilterText,
  ConditionText,Granularity}.*`; `GRANULARITY × index_granularity(8192)`. Live (Run 22):
  `primary.cidx` + `skp_idx_<name>.idx`/`.cmrk4` per part.
- Empirical: `local-benchmark-results.md` Run 6 (trace_id index), Run 12 (full-text
  ~18×), Run 22 (this pass, file formats).
- Cross-refs: `read-path-indexing-and-execution.md` (what each skips),
  `query-execution-engine.md` (why integration beats index richness),
  `clickhouse-internals.md` / `greptimedb-internals.md`.
