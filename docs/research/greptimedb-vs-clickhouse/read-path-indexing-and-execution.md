# Read Path, Indexing, and Execution — Side by Side

<!-- markdownlint-disable MD013 -->

Status: pass 3. Query planning, predicate pushdown, the scan-vs-skip decision,
vectorized execution, and — most important for Parallax — the **join strategy**
for cross-signal evidence-bundle correlation. Builds on `greptimedb-internals.md`
and `clickhouse-internals.md`.

## Version pins

| System | Version | Commit read |
| --- | --- | --- |
| GreptimeDB | `v1.0.2` | `0ef54511f710f0ef2c05941c8c600bb4c1fd46c8` |
| ClickHouse | `v26.5.1.882-stable` | `5b96a8d8a5e2f4800b43a780911a39dc5a666e1c` |

## Read path, end to end

| Stage | GreptimeDB | ClickHouse |
| --- | --- | --- |
| **Planner** | SQL/PromQL → DataFusion `LogicalPlan` → physical plan. PromQL has a **native planner** (`src/promql`, `range_select`, `extension_plan`), not SQL emulation. | SQL → AST → `InterpreterSelectQuery` → `QueryPlan` → `Processors` pipeline. No PromQL. |
| **Predicate pushdown** | DataFusion pushes filters down to the region scan; **dynamic filter pushdown reaches the region scan** (`src/query/src/datafusion.rs:959` test `test_join_dynamic_filter_pushdown_reaches_region_scan`). | `KeyCondition` turns `WHERE` on key columns into mark ranges; **PREWHERE** moves filters to a filter-columns-first read step (`optimize_move_to_prewhere=true`, `Settings.cpp:890`). |
| **Scan/skip unit** | Parquet **row group = 102,400 rows** (`DEFAULT_ROW_GROUP_SIZE`); skip via time-range → index → row-group min/max stats → page index. | **Granule = 8,192 rows** (`index_granularity`, adaptive 10 MB); skip via sparse primary index → skip indexes → mark ranges. |
| **Execution** | **DataFusion / Arrow** vectorized operators (Rust). | **`Processors`** pull-based vectorized pipeline, hand-tuned C++ + SIMD. |
| **Parallelism** | DataFusion partitions + `RepartitionExec`; distributed via `MergeScanExec` fan-out to regions (`src/query/src/dist_plan`). | Multiple streams across parts/granules; distributed via `Distributed` engine fan-out. |

## The scan-vs-skip decision (what gets pruned, cheapest first)

**ClickHouse** (selective read on a wide table is its strength):

1. **Partition pruning** (`PARTITION BY`, e.g. by day) drops whole partitions.
2. **Sparse primary index** (`KeyCondition`) → only granules whose key range can
   match; everything else skipped with zero decompression. Skip unit 8,192 rows.
3. **Skip indexes** (minmax/set/bloom/text) drop further granule blocks.
4. **PREWHERE**: read only the *filter* columns for surviving granules, evaluate
   the predicate, then read the remaining (often wide) columns **only for rows
   that passed** (`MergeTreeSplitPrewhereIntoReadSteps`, `MergeTreeRangeReader`).
   This is the big lever for logs/traces with many columns — avoids
   materializing wide columns for filtered-out rows.

**GreptimeDB** (LSM merge + Parquet pruning):

1. **Time-range pruning** on `FileMeta.time_range` drops whole SST files (and the
   time-window compaction layout makes this tight for recent windows).
2. **Index pruning** (inverted / full-text / skipping, in Puffin) drops files /
   row groups.
3. **Row-group min/max + page index** drop row groups and pages inside a file.
4. **MVCC merge**: live memtables are merged with SSTs at the `Version` snapshot
   (the freshness cost — recent data is in-memory, not yet skip-indexed).

**Consequence (axis 1, speed):** for a **selective filter on a wide table**
(typical log/trace lookup: `service=X AND level=error` over a day), ClickHouse's
**finer 8,192-row granule + PREWHERE late materialization** should read less data
than GreptimeDB's 102,400-row Parquet row group. This is a concrete,
mechanism-level reason ClickHouse can beat GreptimeDB on log/trace selective
scans — *contradicting the naive "GreptimeDB fastest everywhere" reading of the
hypothesis*. Confidence: architecture-reasoned; flagged for a Docker run.

## Join strategy — the evidence-bundle question

Evidence-bundle correlation (Q1–Q6) joins signals by `trace_id` / `fingerprint`
over a time window. The join engine decides the winner. **Both systems are
hash-join families — neither is a join-first engine.**

| Aspect | GreptimeDB (DataFusion) | ClickHouse |
| --- | --- | --- |
| Default algorithm | `HashJoinExec`; `PartitionMode::CollectLeft` (broadcast small left) **or** `Partitioned` (repartition both sides by key) — `src/query/src/datafusion.rs:640,1056`. | `join_algorithm = "direct,parallel_hash,hash"` (`Settings.cpp:3397`) — try direct, else parallel hash, else hash. |
| Build side | Builds hash table from one side; partitioned mode splits **both** sides by hash → handles **large↔large** joins. | Builds hash table from the **right** table in memory (broadcast-style) — tuned for **star schema** (big fact + small dimension). |
| Spill to disk | DataFusion supports partitioned execution; spilling depends on operator config. | **Grace hash join** auto-converts when right side exceeds `max_bytes_ratio_before_external_join` (default 0.5) → spills (`GraceHashJoin.cpp`). |
| Selective-join help | **Dynamic filter pushdown into the region scan**: the build side's key set prunes the probe-side scan (confirmed by test). | `direct` join does point lookups against a key-value/MergeTree right side (`DirectJoin`); otherwise relies on PREWHERE + index on the probe side. |
| Structural bias | General-purpose joins (DataFusion optimizer), younger analytical maturity. | Excellent **fact×dimension**; large↔large historically the weak spot (must fit memory or grace-spill). |

**Mechanism verdict (architecture-reasoned, pre-benchmark):**

- For **large↔large** joins (e.g. join a day of spans to a day of logs by
  `trace_id`, both high-volume), GreptimeDB's **partitioned hash join + dynamic
  filter pushdown** is structurally better-suited than ClickHouse's broadcast
  default, which leans on grace-hash spilling. This is a *plausible* GreptimeDB
  advantage on the query that matters most — **must be benchmarked**, not asserted.
- But the decisive move in **both** engines is to **avoid the big join**: put the
  correlation key (`trace_id`) in the sort/primary key prefix so a point/range
  lookup reduces each signal to a tiny set *before* joining, turning a large↔large
  join into small↔small. This is a **schema-design lever**, handled in the two
  implementation notes — and it likely matters more than the engine's join
  algorithm for Parallax.

→ This is the top open question. Routed to `benchmarking-the-differences.md`
(large↔large `trace_id` join, cold/warm, single-node) and the implementation
notes (sort-key co-location to avoid the join).

**Update (Run 2 EXPLAIN, 2026-05-25 — see `local-benchmark-results.md`).** Real
planner output on Q4 confirmed the algorithms *and corrected the framing*:

- ClickHouse picks `SpillingHashJoin(ConcurrentHashJoin)` with `FillRightFirst`;
  GreptimeDB picks `HashJoinExec: mode=Partitioned` with `RepartitionExec` — both
  as predicted.
- **But both engines propagate the anchor `trace_id` constant to *both* join
  inputs** (ClickHouse via PREWHERE → spans pruned to `Granules: 1`; GreptimeDB via
  `FilterExec` on both `MergeScanExec` scans). So the earlier worry that
  ClickHouse's broadcast join "builds the whole right side" is **wrong for a
  constant-anchored join** — it prunes the right side first.
- **Therefore the join algorithm is not a differentiator for Parallax's
  evidence-bundle queries**, which are always anchored on a `trace_id`/
  `fingerprint`. Both prune to a tiny working set before joining. What matters is
  **key placement** so the anchor prunes cheaply (Run 1: ClickHouse 2 ms vs
  GreptimeDB 16 ms on the un-keyed `trace_id` lookup). The large↔large join
  scenario where partitioned-vs-broadcast would matter is one Parallax does not run
  for bundle assembly — so it drops in priority.

## Execution engine: DataFusion vs Processors

| | GreptimeDB (DataFusion, Rust) | ClickHouse (Processors, C++) |
| --- | --- | --- |
| Model | Arrow columnar; `ExecutionPlan` operators; pull via streams. | `Chunk` (column block) through `Transform`s; pull-based pipeline. |
| Maturity / tuning | Younger; rides DataFusion's improvements (GreptimeTeam fork, base 52.1). | ~Decade of hand-tuned C++ + SIMD; widely regarded as the fastest OLAP scan/aggregate engine. |
| Consequence | Competitive and improving, but ClickHouse's vectorized scan/aggregate is the bar to beat on raw throughput (axis 1). | Raw scan/group-by throughput leader; the reason ClickHouse wins generic analytical scans. |

## Per-signal read consequence (first cut)

| Signal / query | Likely winner (read path) | Mechanism | Confidence |
| --- | --- | --- | --- |
| Logs: selective filter + substring/token search | **ClickHouse** | 8,192 granule + PREWHERE + native inverted text index + LowCardinality. | arch-reasoned |
| Traces: `trace_id` point lookup | **Tie → schema-decided** | Whoever has `trace_id` in key prefix wins; both then do a small range read. | arch-reasoned |
| Metrics: PromQL range/aggregation | **GreptimeDB** | Native PromQL planner + metric engine + time-index; ClickHouse must emulate via SQL + AggregatingMergeTree. | arch-reasoned |
| Evidence-bundle: large↔large join | **GreptimeDB (tentative)** | Partitioned hash join + dynamic filter pushdown vs ClickHouse broadcast/grace-spill — **benchmark before trusting**. | low / must-test |
| Generic wide scan + group-by | **ClickHouse** | Decade-tuned C++ vectorized pipeline. | arch-reasoned |

## Confidence and open questions

- **Confirmed by code:** ClickHouse join-algorithm default + grace hash + PREWHERE
  + `KeyCondition`; GreptimeDB DataFusion `HashJoinExec` partition modes + dynamic
  filter pushdown to region scan + native PromQL planner + `MergeScanExec`.
- **Architecture-reasoned (needs benchmark):** the per-signal winners above,
  especially the evidence-bundle join — the single most decision-relevant number.
- **To verify next:** GreptimeDB's late-materialization aggressiveness inside a
  Parquet row group vs ClickHouse PREWHERE (does DataFusion's Parquet page-index +
  RowSelection approach the same data-read savings?). Trace in a deepening pass.

## Source references

- ClickHouse joins: `src/Core/Settings.cpp:3397,7807`; `src/Interpreters/{HashJoin,GraceHashJoin,DirectJoin,FullSortingMergeJoin}.{h,cpp}`; `src/Core/SettingsEnums.cpp:52-59`
- ClickHouse PREWHERE: `src/Storages/MergeTree/{MergeTreeRangeReader,MergeTreeSplitPrewhereIntoReadSteps}.{h,cpp}`; `src/Core/Settings.cpp:890`
- ClickHouse key scan: `src/Storages/MergeTree/KeyCondition.{h,cpp}`, `MarkRange.{h,cpp}`
- GreptimeDB query/joins: `src/query/src/datafusion.rs:640,959,1056`; `src/query/src/optimizer/`; `src/query/src/dist_plan` (`MergeScanExec`)
- GreptimeDB PromQL: `src/promql/src/{extension_plan,range_array}.rs`, `src/query/src/range_select.rs`
- GreptimeDB region scan: `src/query/src/region_query.rs`
