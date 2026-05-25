# GreptimeDB Internals — Architecture and Code-Path Teardown

<!-- markdownlint-disable MD013 -->

Status: foundational pass (pass 1). This is the architecture map and the
storage-engine (mito2) teardown. Deeper per-subsystem dives (read-path merge,
compaction picker, index internals, metric-engine physical layout, object-store
cache) are split into their own notes as the loop deepens.

## Version pin (this note)

| Item | Value |
| --- | --- |
| Version | GreptimeDB `v1.0.2` (GA 2026-05-14) |
| Source commit read | `0ef54511f710f0ef2c05941c8c600bb4c1fd46c8` |
| Language / runtime | Rust |
| Query engine | Apache DataFusion (GreptimeTeam fork, base `=52.1`, rev `02b82535`) — `Cargo.toml:130,336` |
| Object store | OpenDAL `0.54` — `src/object-store/Cargo.toml:24` |

Re-check for a newer stable at the start of every pass (version-freshness rule).
Latest tags observed 2026-05-25: stable `v1.0.2`; `v1.1.0-nightly-20260525`
exists but is nightly, not GA — do not pin nightly.

## One-paragraph mechanism summary

GreptimeDB is an **LSM-tree time-series engine written in Rust**, where the unit
of storage is a **region** managed by the **mito2** region engine. Writes go to a
per-region **WAL** then a **mutable memtable**; memtables freeze and **flush to
Parquet SST files** organized into **levels**; **time-window compaction** merges
them; reads take an **MVCC snapshot** (a `Version`) and merge live memtables with
SSTs, pruning by **time range and Parquet row-group statistics** plus optional
**inverted / full-text / skipping indexes**. Query execution is **Apache
DataFusion** (Arrow, vectorized). Storage sits behind **OpenDAL**, so S3-class
object storage is a first-class backend with a local read cache in front. The
distributed split is **Frontend (stateless) / Datanode (storage+compute) /
Metasrv (metadata+scheduling)**, and a **remote WAL (Kafka)** option decouples
durability from the datanode to make region migration cheap.

## Component topology

GreptimeDB separates compute roles from the storage engine. Source: crates under
`src/` (`frontend`, `datanode`, `meta-srv`, `mito2`, `metric-engine`).

| Component | Role | State | Why it matters for scaling |
| --- | --- | --- | --- |
| **Frontend** | Stateless gateway: parses SQL/PromQL/OTLP/Prom remote-write, plans, routes to datanodes, merges results. | Stateless | Scale out horizontally and cheaply; no data on it. |
| **Datanode** | Hosts **regions**; runs the **mito2** region engine (write/flush/compact/read). | Stateful (or stateless-ish with remote WAL + object store) | The actual storage+compute unit. |
| **Metasrv** | Metadata, region→datanode placement, scheduling, region migration/rebalance. | Stateful (metadata store) | Control plane; decides where regions live. |

`standalone` (`src/standalone`) bundles all three in one process — this is the
**Tier-1 single-node** shape Parallax cares about first.

## The region engine: mito2

`src/mito2/src/lib.rs:17` — *"Mito is a region engine to store timeseries data."*
The engine hierarchy (from the developer doc class diagram in `lib.rs:55-167`):

```text
MitoEngine
  ├─ MitoConfig
  ├─ WorkerGroup
  │    └─ Vec<RegionWorker>            # sharded by RegionId; each worker = 1 thread + mpsc channel
  │         └─ RegionWorkerThread<LogStore>
  │              ├─ RegionMap                 # regions owned by this worker
  │              ├─ Wal<LogStore>             # raft-engine (local) or Kafka (remote)
  │              ├─ ObjectStore               # OpenDAL handle
  │              ├─ MemtableBuilder
  │              ├─ FlushScheduler + FlushStrategy
  │              ├─ CompactionScheduler
  │              └─ FilePurger
  └─ MitoRegion (per region)
       └─ VersionControl = CowCell<Version> + AtomicU64 committed_sequence   # MVCC
            └─ Version {
                 RegionMetadata,
                 MemtableVersion { mutable: 1, immutables: Vec },   # classic LSM
                 SstVersion { levels: LevelMeta[], compaction_time_window },
                 flushed_sequence,
                 manifest_version
               }
```

**Mechanism consequences:**

- **Single-writer-per-region via the worker thread.** Each region is owned by one
  `RegionWorker` thread and mutated through an mpsc channel
  (`src/mito2/src/worker.rs`). No per-row locking on the write path; contention is
  bounded by sharding regions across workers. Speed: high ingest throughput per
  region; the knob is region count vs worker count.
- **MVCC by `Version` snapshot.** `VersionControl` is a copy-on-write cell plus an
  `AtomicU64 committed_sequence` (`lib.rs:117-120`). A reader clones the current
  `Version` (an `Arc` graph) and sees a frozen set of memtables + SSTs at a
  sequence number. Writers swap in a new `Version` without blocking readers. This
  is the **freshness mechanism**: data is queryable the instant the sequence is
  committed to the mutable memtable — no flush required.
- **Leveled SST organization.** `SstVersion` holds `LevelMeta[]`, each level a
  `HashMap<FileId, FileHandle>`; `FileMeta` carries `time_range`, `level`,
  `file_size` (`lib.rs:135-165`). Time range per file → **file-level time pruning**
  before any row is read.

## Subsystem checklist (mechanism-level, v1.0.2)

| Subsystem | Implementation | Mechanism / consequence |
| --- | --- | --- |
| **On-disk layout** | Parquet SST files, columnar, sorted by primary key + time. `src/mito2/src/sst/parquet.rs:15` | Columnar → per-column codecs + column pruning. Row group = 102,400 rows (`DEFAULT_ROW_GROUP_SIZE = 100 * 1024`, `parquet.rs:45`); read batch 1024 rows (`DEFAULT_READ_BATCH_SIZE`). Row group is the **skip unit** (Parquet min/max stats per row group / column chunk). |
| **Write path** | WAL append → mutable memtable; commit bumps `committed_sequence`. `worker.rs`, `region_write_ctx.rs` | **Queryable on memtable insert**, not on flush → low ingest-to-queryable latency. Durability from WAL; visibility from sequence commit. |
| **Memtable** | Default `TimeSeries` (`#[default]` enum variant, `src/mito2/src/memtable.rs:76-79`); alternatives `PartitionTree` (high-cardinality, dict-encoded shards) and `Bulk`/`SimpleBulk`. | `TimeSeries` = `BTreeMap` keyed by encoded primary key, values time-ordered (`memtable/time_series.rs`). `PartitionTree` dict-encodes primary keys across shards for high series cardinality (`memtable/partition_tree.rs`). Choice changes high-cardinality ingest cost — **measure both** (see benchmarking note). |
| **WAL** | Two `LogStore` providers: `raft_engine` (local embedded) and `kafka` (remote). `src/log-store/src/lib.rs:16,19` | Local = lowest latency, couples durability to the datanode disk. **Remote WAL (Kafka)** decouples durability → cheap region migration / stateless-ish datanodes, at the cost of a Kafka dependency and added write latency. Remote WAL purge: RFC `2025-02-06`. |
| **Flush** | Mutable memtable frozen to immutable, written to a new Parquet SST, `flushed_sequence` advanced. `src/mito2/src/flush.rs` | Flush does not block reads (immutables still served until SST lands). Write-buffer manager governs flush trigger. |
| **Compaction** | Time-window oriented; `compaction_time_window` on `SstVersion`. `src/mito2/src/compaction/window.rs` | Groups SSTs by time window → bounded read fan-in for recent windows; aligns with time-series query patterns. `append_mode` region option skips dedup/merge (see below). |
| **Indexing** | `src/index` crate: inverted index (RFC `2023-11-03`), full-text (logs), skipping index, vector index (RFC `2025-12-05`); stored in **Puffin** files (`src/puffin`). Async build: RFC `2025-08-16`. | Indexes let the scan skip row groups/files. Full-text + inverted target log/trace search; skipping index targets min/max-friendly columns. Index build is async → does not stall ingest. |
| **Read path** | DataFusion physical plan over a merge of memtables + SSTs at a `Version` snapshot; predicate pushdown to time range + row-group stats + indexes. `src/query`, `src/mito2/src/read` | Vectorized Arrow execution. Scan-vs-skip decided by time range → index → row-group stats, in that order of cheapness. (Deeper read-path note pending.) |
| **Compression** | Parquet column codecs (dictionary, RLE, etc.) via Arrow/Parquet writer. | Per-column compression; dictionary encoding for low-cardinality tags. Exact codec-by-signal mapping pending in `compression-and-cost.md`. |
| **Caching** | OpenDAL **read cache enabled by default** on local disk at `data_home` (`src/object-store/src/config.rs:318-340`); plus in-engine caches (`src/mito2/src/cache`). | Cold object-store reads are cached locally → warm-cache latency approaches local disk; cold-cache pays object-store RTT. |
| **Object storage** | OpenDAL `0.54`, backends: `s3`, `oss`, `gcs`, `azblob`, `fs`, `http` (`src/object-store/Cargo.toml`). | S3-class storage is first-class, not bolted on. Read cache fronts it. Enables cheap long retention with re-readable history. |
| **Schema / dynamic** | Tag (primary-key) columns + field columns + a time index column; JSON datatype (RFC `2024-08-06`). **Metric engine** maps many logical metric tables onto one shared physical wide table (RFC `2023-07-10`, `src/metric-engine`). | Metric engine avoids one-region-per-metric explosion for high-cardinality metrics. Dynamic OTLP attributes → tag columns or JSON; cost tradeoff pending in the implementation note. |
| **Distributed** | Frontend / Datanode / Metasrv; region migration (RFC `2023-11-07`), repartition (RFC `2025-06-20`). | Scale-out designed in: regions are the shard unit, Metasrv places/moves them. Remote WAL makes migration cheap. |

## Append mode (logs)

`append_mode` is a per-region option (`src/mito2/src/compaction/window.rs:249`,
`engine/row_selector_test.rs`). With it on, the engine treats rows as
append-only and **skips the dedup/last-row merge** on read and compaction. For
high-volume logs this removes the merge cost that the time-series (upsert) model
otherwise pays. Maps directly to Parallax's log/event signals.

## What this means for the operator hypothesis (so far)

The hypothesis is *GreptimeDB fastest, then ClickHouse*. From the architecture
alone, GreptimeDB's structural advantages for Parallax are:

1. **Freshness**: queryable on memtable insert (sequence commit), no flush
   barrier — strong for "see real data fast" (axis 1).
2. **Metric-native**: the metric engine + time index + PromQL path are built in,
   not emulated — likely advantage for the **metrics** signal.
3. **Object-storage economics**: OpenDAL + default read cache make cheap S3
   retention a first-class path (axis 2).

Open questions that decide the hypothesis, deferred to dedicated notes/benchmarks:

- Does DataFusion's vectorized execution match ClickHouse's hand-tuned C++ vector
  engine on **high-volume log/trace scans**? (Likely ClickHouse's strength — to be
  traced in `clickhouse-internals.md` and tested.)
- **Cross-signal join** (evidence-bundle Q1–Q6): GreptimeDB join strategy vs
  ClickHouse — neither verified yet; this is the query that matters most.
- High-cardinality **metric** ingest: `TimeSeries` vs `PartitionTree` memtable
  cost, and metric-engine physical-table behavior.

No "X is faster" verdict is asserted here without a ClickHouse counterpart and a
mechanism + scenario; those land in `per-signal-verdict.md`.

## Confidence

- **Confirmed by code (this commit):** region/worker model, MVCC `Version`
  snapshot, Parquet SST + row-group size, default `TimeSeries` memtable, dual WAL
  providers, OpenDAL backends + default read cache, DataFusion dependency.
- **Architecture-level (doc + structure, not yet line-traced):** exact read-path
  merge/skip ordering, compaction picker heuristics, metric-engine physical
  layout, index query-time integration. Each gets a dedicated deepening pass.

## Source references

- Engine map: `src/mito2/src/lib.rs:17,55-177`
- Write path / workers: `src/mito2/src/worker.rs`, `src/mito2/src/region_write_ctx.rs`
- Memtables: `src/mito2/src/memtable.rs:76-79`, `memtable/time_series.rs`, `memtable/partition_tree.rs`
- SST/Parquet: `src/mito2/src/sst/parquet.rs:15,43-60`
- Flush/compaction: `src/mito2/src/flush.rs`, `src/mito2/src/compaction/window.rs`
- WAL: `src/log-store/src/lib.rs:16,19`; remote WAL purge RFC `docs/rfcs/2025-02-06-remote-wal-purge.md`
- Index: `src/index`, `src/puffin`; RFCs `2023-11-03-inverted-index`, `2025-12-05-vector-index`, `2025-08-16-async-index-build`
- Metric engine: `src/metric-engine`; RFC `docs/rfcs/2023-07-10-metric-engine.md`
- Object store: `src/object-store/Cargo.toml:24`, `src/object-store/src/config.rs:318-340`
- Query/DataFusion: `src/query/Cargo.toml:40`, `Cargo.toml:130,336`
