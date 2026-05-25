# Indexing Internals — Index File Formats and Why Richer ≠ Faster

<!-- markdownlint-disable MD013 -->

Status: pass 43 + pass 78 (index-cache asymmetry source-checked) + pass 86
(Run 48 corrected the full-text result: `matches()` on a bloom-backed index full-scans;
`matches_term()` prunes, so exact-term incident grep is competitive) + pass 87 (Run 49:
tantivy backend `matches()` **also prunes** ~6 ms — query-syntax path fast too; the ~18×
was fully a backend/function pairing artifact, not an index-maturity gap). The storage half of checklist #3: **how each engine stores its
secondary/skip indexes on disk, what they can skip, and the honest paradox** that
GreptimeDB's *richer* index toolkit appeared to lose full-text search ~18× until
Runs 48-49 traced that result to a backend/function mismatch. The
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

## The paradox corrected: richer index, query-form matters (Run 12/48 reconciled)

Despite the richer toolkit, **ClickHouse full-text search appeared ~18× faster** in Run 12
(5M logs, both indexed) and its anchored `trace_id` lookup was faster even after GreptimeDB
added an inverted index (Run 6: 14→8 ms, still not ClickHouse's 2 ms). Run 48 corrected the
full-text part: `logs_b1` used the bloom fulltext backend, but the test used `matches()`, the
tantivy query-syntax function. That pairing full-scans. With `matches_term()` on the bloom
backend, GreptimeDB prunes and selective exact-term search is ~8 ms warm, ~2-3× ClickHouse's
~3 ms, not 18×. The mechanism becomes more precise: **index format richness does not decide
query speed; the function/backend pairing and end-to-end execution integration do** (ties to
`query-execution-engine.md`):

- **ClickHouse** applies the coarse `text`/skip index to prune granules, then confirms
  with its decade-tuned vectorized `hasToken` scan — index lookup and confirmation are
  both inside the fast C++ pipeline, on 65k-row blocks.
- **GreptimeDB** has two relevant fulltext paths. The **bloom** backend is the exact-term
  token path; paired with `matches_term()`, it reuses `BloomFilterIndexCache` and prunes. The
  **tantivy** backend is the query-syntax/phrase path; paired with `matches()`, it **also prunes**
  (Run 49: selective ~6 ms warm, EXPLAIN `output_rows: 1`) — query-syntax search is fast on the
  tantivy backend, confirming the gap was purely the bad bloom+`matches()` pairing. **Cache asymmetry
  (pass 78 source):** the inverted, bloom, and vector indexes have in-memory caches plus an
  index-application result cache, while there is no dedicated `FulltextIndexCache` for tantivy.
  **Run 47/48 consequence:** missing tantivy cache is not the measured bottleneck for the
  bloom-backed `logs_b1` workload; the bad `matches()`/bloom pairing full-scanned, while the
  correct `matches_term()` path pruned. Broad terms that match many rows still become scan-engine
  work (`query-execution-engine.md`), not index lookup work.
- For the **anchored point lookup**, ClickHouse's `ORDER BY` **sort-key locality**
  (primary.cidx) beats *any* secondary index: the rows are physically contiguous, so it
  reads one granule and needs no separate index load. GreptimeDB's inverted index helps
  but still pays the load+apply+scan path (Run 6).

So GreptimeDB's index *capability* is a genuine strength (precise secondary indexing,
true full-text, configurable fine granularity), and Runs 48/49 show **both** selective
full-text paths are competitive with the correct pairing: exact-term (bloom + `matches_term`)
~8 ms and query-syntax (tantivy + `matches`) ~6 ms, vs ClickHouse ~3 ms — all sub-perceptible.
The remaining speed caveat is narrow: only **broad terms** (matching many rows) still stress
scan execution (`query-execution-engine.md`), which is analytics, not interactive search.

## Axis consequence

- **Speed (axis #1):** ClickHouse still wins anchored lookups through sort-key locality, but
  Runs 48/49 essentially dissolve the log-search gap for *interactive* search: selective
  full-text is ~8 ms (bloom + `matches_term`) and ~6 ms (tantivy + `matches`) on GreptimeDB
  vs ~3 ms ClickHouse — both sub-perceptible, not 18×. The remaining gap is only **broad-term
  scans** (many matches → scan integration + vectorized confirm), i.e. analytics, not the
  interactive incident grep.
- **Capability:** GreptimeDB's FST/roaring inverted + tantivy full-text are the more
  capable *toolkit* — relevant if Parallax later needs precise high-cardinality
  secondary indexing or real BM25-style search; today ClickHouse's coarser-but-faster
  path wins the measured workloads.
- **Cost:** Puffin keeps one sidecar per SST (fewer files, fits the object-store
  few-objects story); ClickHouse adds one `.idx` per skip index per part (more small
  files) — a minor footnote on the object-count axis already covered in
  `caching-and-cold-warm.md`.

Net: this **strengthens** the GreptimeDB verdict for Parallax's likely incident-grep path.
It corrects two tempting wrong inferences: richer indexes alone do not guarantee speed, and
the old ~18× result should not be generalized to selective log search. The next fair test is
larger/cold broad-term search, especially `SELECT *` shapes where the matched row set must be
materialized rather than counted.

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
- Empirical: `local-benchmark-results.md` Run 6 (trace_id index), Run 12 (original full-text
  ~18×), Run 22 (file formats), Run 47 (index apply sub-ms), Run 48 (`matches_term()` + bloom
  prunes and selective exact-term search is ~8 ms warm), Run 49 (tantivy + `matches()` prunes
  and selective query-syntax search is ~6 ms warm).
- Cross-refs: `read-path-indexing-and-execution.md` (what each skips),
  `query-execution-engine.md` (why integration beats index richness),
  `clickhouse-internals.md` / `greptimedb-internals.md`.
