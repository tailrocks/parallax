# ClickHouse Internals — Architecture and Code-Path Teardown

<!-- markdownlint-disable MD013 -->

Status: foundational pass (pass 2). Architecture map + MergeTree storage-engine
teardown, structured to mirror `greptimedb-internals.md` for side-by-side
reading. Deeper per-subsystem dives (KeyCondition mark selection, merge selector,
text-index internals, vectorized pipeline scheduling, S3 disk cache) split into
their own notes as the loop deepens.

## Version pin (this note)

| Item | Value |
| --- | --- |
| Version | ClickHouse `v26.5.1.882-stable` (latest stable 2026-05-25) |
| Tag object | `fae722ba30c82d0975692fa2a93cbbe8f6ae3af2` |
| Source commit read | `5b96a8d8a5e2f4800b43a780911a39dc5a666e1c` |
| Language / runtime | C++ |
| LTS alternative | `v26.3.12.3-lts` (`f118ee7c3b4c1a57dde6a389e5c3e29080f38c5d`) |

Re-check for a newer stable every pass (version-freshness rule). The stable line
`v26.5.x` is newer than the LTS line `v26.3.x`; pin stable for the analysis,
note LTS as the conservative deployment option.

## One-paragraph mechanism summary

ClickHouse is a **columnar OLAP engine written in C++** whose storage is the
**MergeTree** family. An `INSERT` becomes an immutable **part** — columns stored
sorted by the table's `ORDER BY` key. A **sparse primary index** holds one entry
per **granule** (`index_granularity = 8192` rows, or adaptive at
`index_granularity_bytes = 10 MB`), so reads use `KeyCondition` to translate the
`WHERE` clause into a set of **mark ranges** and read only the granules that can
match — everything else is skipped without decompression. **Data-skipping
indexes** (minmax, set, bloom_filter, token/ngram bloom text, a native inverted
**text index**, vector similarity) skip granules further. Execution is a
**vectorized, pull-based pipeline** (`Processors`) of transforms over column
**Chunks**, hand-tuned C++ with SIMD. **Background merges** combine small parts
into larger ones and, depending on the engine variant (`Replacing`, `Summing`,
`Aggregating`, `Collapsing`, …), transform rows at merge time. Data is
**queryable the moment the part is written** (no flush barrier); **async inserts**
batch small writes server-side. The distributed story is the **Distributed**
engine for query fan-out plus **ReplicatedMergeTree + ClickHouse Keeper (Raft)**
for replication, with an **S3 disk** + storage policies for object storage and
tiering.

## Storage-engine topology

ClickHouse is a **single binary** (`clickhouse-server`); there is no separate
storage/compute/metadata process split like GreptimeDB. Roles are *table
engines* and optional coordination, not separate daemons.

| Concept | Role | Notes |
| --- | --- | --- |
| **MergeTree** (table engine) | Local storage+compute: parts, sparse index, merges, reads. | The workhorse; everything else wraps it. `src/Storages/MergeTree/` |
| **ReplicatedMergeTree** | Adds replication via Keeper-coordinated replication log. | `src/Storages/StorageReplicatedMergeTree.cpp` |
| **Distributed** (table engine) | Stateless view that fans a query out across shards and merges results. | `src/Storages/StorageDistributed.cpp` — sharding is **manual** (sharding key / cluster config). |
| **ClickHouse Keeper** | Raft-based ZooKeeper replacement; replication + DDL consensus. | `src/Coordination/Keeper*` |

Single-node Parallax (Tier-1) is just `clickhouse-server` with plain MergeTree —
no Keeper, no Distributed. That is the **simplest possible operational shape**.

## The MergeTree part → granule → mark structure

This is the core mechanism. Source: `src/Storages/MergeTree/`.

```text
Table (MergeTree)
  └─ Parts (immutable, one per INSERT-or-merge; merged in background)
       ├─ Part format: Compact (<10MB, all columns in one file)
       │                Wide (>=10MB, one file PER column)   # default_min_bytes_for_wide_part = 10 MB
       ├─ Rows sorted by ORDER BY key within the part
       ├─ Granule = index_granularity rows (default 8192; adaptive at 10 MB)
       ├─ Sparse primary index = 1 (key value -> mark) entry PER GRANULE   # primary.idx
       │     (NOT a B-tree; an in-memory sorted array, kept hot in the mark cache)
       ├─ Marks = (offset in compressed file, offset in decompressed block) per granule per column
       └─ Skip indexes (optional, per N granules): minmax / set / bloom_filter /
            token+ngram bloom text / native text (inverted) / vector_similarity
```

**Mechanism consequences:**

- **Sparse index = cheap key-range pruning, no point-lookup index.** `KeyCondition`
  (`src/Storages/MergeTree/KeyCondition.{h,cpp}`) turns a `WHERE` on `ORDER BY`
  prefix columns into mark ranges; the engine reads only those granules. There is
  **no per-row index** — a granule that matches is fully scanned (8192 rows).
  Great for range/prefix scans, weak for needle-in-haystack point lookups on a
  non-key column unless a skip index helps.
- **Granule is the skip unit (8192 rows).** Finer than GreptimeDB's Parquet row
  group (102,400 rows), so ClickHouse can skip at ~12× finer granularity by
  default → less wasted decompression on selective reads.
- **Wide vs Compact parts.** Big parts store one file per column (true on-disk
  columnar → column pruning + per-column codec); tiny parts pack into one file to
  avoid file-count explosion from frequent small inserts.
- **Freshness.** An `INSERT` writes a part and it is **immediately queryable** —
  no flush wait. This is ClickHouse's freshness mechanism; it parallels
  GreptimeDB's "queryable on memtable insert" but the unit is a durable on-disk
  part, not an in-memory memtable.

## Subsystem checklist (mechanism-level, v26.5.1.882)

| Subsystem | Implementation | Mechanism / consequence |
| --- | --- | --- |
| **On-disk layout** | Columnar parts sorted by `ORDER BY`; Wide (file/column) or Compact. `MergeTreeSettings.cpp:33-83` | Per-column codecs + column pruning. Sort-key locality decides scan cost. |
| **Write path** | `INSERT` → sort by key → write immutable part → instantly visible. Async inserts batch via `AsynchronousInsertQueue` (`src/Interpreters/AsynchronousInsertQueue.cpp`). | No flush barrier → fresh on write. Many small inserts = many small parts = merge pressure; async insert trades a bit of freshness for fewer parts. |
| **Indexing** | Sparse primary index (1/granule, `index_granularity=8192`, `MergeTreeSettings.cpp:69`); adaptive `index_granularity_bytes=10MB` (`:1650`). Skip indexes: minmax, set, bloom_filter, bloom_filter **text** (token/ngram), **native text/inverted** (`MergeTreeIndexText`, posting lists), vector_similarity. | Primary index prunes mark ranges; skip indexes prune granule blocks. **Text inverted index** (new) accelerates log substring/token search beyond the older tokenbf. No index helps an arbitrary point lookup off the sort key except a matching skip index. |
| **Read path / execution** | `Processors` pull-based vectorized pipeline (`src/Processors/`): `Chunk` = column block flows through `Transform`s; multi-stream parallelism across parts. | Hand-tuned C++ + SIMD on contiguous columns → very fast large scans/aggregations. Parallelism scales with cores and part/stream count. |
| **Compaction / merge** | Background merges combine parts; `MergingParams::Mode` (`MergeTreeData.h:442`): Ordinary, Collapsing, Summing, Aggregating, Replacing, VersionedCollapsing, Graphite. Mutations = `ALTER` rewrites. | Merges bound the number of parts a read must touch (read speed). Variant = **merge-time row transform** (dedup/sum/aggregate), not a separate engine. Write amplification from repeated merges. |
| **Compression** | Per-column codecs (`src/Compression/`): LZ4 (default), ZSTD, Delta, DoubleDelta, **Gorilla**, T64, GCD, **ALP**, FPC; `LowCardinality` dictionary wrapper. | Codec matched to column: Gorilla/DoubleDelta/ALP/FPC for float metrics, Delta/T64/GCD for ints/timestamps, ZSTD for log strings, LowCardinality for service/severity. Big lever on cost (axis 2). |
| **Caching** | Mark cache (sparse-index marks), uncompressed-block cache, primary-key cache, query cache; OS page cache underneath. | Mark cache keeps the sparse index hot → warm reads skip granules without disk. Cold cache pays mark + granule reads; cold/warm divergence is large for selective queries. |
| **Object storage** | S3 disk (`src/Disks/DiskObjectStorage/ObjectStorages/S3/S3ObjectStorage.cpp`) via storage policies; local disk cache in front. Zero-copy replication exists but **default OFF** (`allow_remote_fs_zero_copy_replication=false`, `MergeTreeSettings.cpp:1955`). | S3 is a *disk type under a storage policy*, not the default home — hot data is typically local, cold tiered to S3 (TTL move). More configuration than GreptimeDB's object-store-native default. |
| **Schema / dynamic** | `Map`, `JSON` type, `LowCardinality`, `Array`; materialized views (transform on insert), **projections** (alternate sort order inside a part), AggregatingMergeTree for rollups. | Dynamic OTLP attributes → `Map`/`JSON`; rollups/correlation via MV + AggregatingMergeTree. Projections give a second sort order without a second table. |
| **Distributed** | `Distributed` engine fan-out + `ReplicatedMergeTree` + Keeper (Raft). Sharding is **manual** (sharding key). | Scale-out was **added on top of** the single-node engine, not designed-in as the primary unit (contrast GreptimeDB regions). Powerful but more operator-driven. |

## Engine variants are merge algorithms, not engines

A critical structural fact for Parallax schema design: `Replacing`, `Summing`,
`Aggregating`, `Collapsing`, `VersionedCollapsing`, `Graphite` are all the **same
MergeTree storage** with a different **`MergingParams::Mode`** applied during
background merge (`MergeTreeData.h:178-185,442`). Consequences:

- Row-collapsing/dedup/rollup happens **eventually, at merge time** — not on
  insert. A query before merge sees un-collapsed rows unless it adds `FINAL` (slow)
  or aggregates explicitly.
- `AggregatingMergeTree` + a materialized view is the canonical metric/rollup
  pattern; `ReplacingMergeTree` is the canonical upsert/dedup pattern. Both shape
  Parallax's metric and dedup design (covered in `clickhouse-implementation.md`).

## What this means for the operator hypothesis (so far)

Hypothesis: *GreptimeDB fastest, then ClickHouse.* From architecture alone,
ClickHouse's structural strengths for Parallax:

1. **Log/trace analytical scan** (axis 1, the signal where this likely *beats*
   GreptimeDB): 8192-row granule (finer skip than GreptimeDB's 102,400-row Parquet
   row group), hand-tuned C++ vectorized pipeline, the new inverted text index,
   and string-tuned ZSTD/LowCardinality. This is the honest counter to the
   hypothesis — to be confirmed in the read-path note and a Docker run.
2. **Compression breadth** (axis 2): the widest codec set (Gorilla/DoubleDelta/
   ALP/FPC/T64/GCD + LowCardinality) → likely smallest on-disk size per signal.
3. **Operational simplicity at Tier-1**: one binary, plain MergeTree, no
   coordinator.

Structural weaknesses vs GreptimeDB to test:

- **No *GA* PromQL / metric model** — metrics rely on AggregatingMergeTree + MV.
  **(Corrected pass 44:** ClickHouse 26.x *does* have **experimental** PromQL via the
  `TimeSeries` engine + `prometheusQuery[Range]` — off by default, limited to
  `rate`/`delta`/`increase`; see `promql-and-metrics-query.md`. So "no PromQL" → "no
  *GA* PromQL".) Still a GreptimeDB advantage for metrics, on maturity/ergonomics.
- **Object storage is a disk policy, not the native home** — more config for the
  cheap-S3-retention story than GreptimeDB's OpenDAL default.
- **Scale-out is operator-driven** (manual sharding) vs GreptimeDB's region model.
- **Cross-signal join** (evidence-bundle Q1–Q6): undecided; join strategy +
  schema decide it. Top-priority unanswered question for both systems.

No "X is faster" verdict asserted without the side-by-side mechanism + scenario;
those land in `read-path-indexing-and-execution.md` and `per-signal-verdict.md`.

## Confidence

- **Confirmed by code (commit `5b96a8d8`):** part/granule/mark structure,
  `index_granularity=8192` + adaptive 10MB, Wide/Compact 10MB threshold, skip-index
  type set incl. native text index, codec set, `MergingParams::Mode` variants,
  async insert queue, S3 object storage path, zero-copy default OFF, Keeper/Raft,
  Distributed engine.
- **Architecture-level (structure, not yet line-traced):** exact `KeyCondition`
  mark-range algorithm, merge selector heuristics, `Processors` scheduling, text
  posting-list format, S3 disk cache behavior. Each gets a deepening pass.

## Source references

- Part/granule/mark: `src/Storages/MergeTree/{IMergeTreeDataPart,MarkRange,KeyCondition}.{h,cpp}`
- Settings (granularity, wide-part threshold, zero-copy): `src/Storages/MergeTree/MergeTreeSettings.cpp:69,75,1650,1955`
- Skip indexes: `src/Storages/MergeTree/MergeTreeIndex{MinMax,Set,BloomFilter,BloomFilterText,Text,VectorSimilarity}.{h,cpp}`
- Engine variants: `src/Storages/MergeTree/MergeTreeData.h:178-185,442`
- Execution: `src/Processors/` (`Chunk.h`, transforms, `Executors/`)
- Compression: `src/Compression/CompressionCodec*.cpp`
- Async insert: `src/Interpreters/AsynchronousInsertQueue.cpp`
- Object storage: `src/Disks/DiskObjectStorage/ObjectStorages/S3/S3ObjectStorage.cpp`
- Distributed/replication: `src/Storages/StorageDistributed.cpp`, `src/Storages/StorageReplicatedMergeTree.cpp`, `src/Coordination/Keeper*`
