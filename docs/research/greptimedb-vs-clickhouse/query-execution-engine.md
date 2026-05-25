# Query Execution Engine — Why ClickHouse Out-Scans DataFusion

<!-- markdownlint-disable MD013 -->

Status: pass 42 + pass 77 (read-path late-materialization mechanism source-verified:
`PruneReader::precise_filter` post-decode, no arrow `RowFilter`). The execution-engine half
of checklist #4 (the read-path note covers
planning / predicate pushdown / skip-vs-scan / joins; this is the *engine that runs
the plan*). It is the mechanism **behind the measured throughput gaps** — ClickHouse
**~2× on warm metric aggregation** at 40k series (Run 37; corrected from the ~10× of
Run 11, which was a cold/first-run GreptimeDB scan) and ~18× on full-text log search
(Run 12). The verdict has called this "a decade-tuned C++ vectorized engine"; this
note makes that concrete, against source + live settings (Run 21). *(The ~2× warm
agg gap fits this mechanism — 8× block + JIT + SIMD — far better than the old ~10×,
which the cold-cache explanation resolves.)*

Pins: GreptimeDB `v1.0.2` (`0ef5451`, **DataFusion `=52.1`**), ClickHouse
`v26.5.1.882-stable` (`5b96a8d8`), re-confirmed latest stable 2026-05-25.

## ClickHouse — a bespoke C++ vectorized pipeline

- **Block-at-a-time over columns.** Execution processes **blocks** of column vectors;
  **`max_block_size` = 65409** live (≈`DEFAULT_BLOCK_SIZE` 65536, `Core/Settings.cpp:132`)
  — ~8× DataFusion's batch and 8× the 8,192-row granule. Bigger vectors amortize
  per-batch overhead and feed SIMD better.
- **Processors pipeline.** The query is a graph of `Processors` (`src/Processors`,
  `QueryPipeline`) pulled by **`max_threads` parallel lanes** (live `auto(10)` → one
  per core); different lanes scan different granule ranges and merge at the top.
- **Runtime JIT (LLVM).** **`compile_expressions = 1`** and
  **`compile_aggregate_expressions = 1`** live (`min_count_to_compile_expression = 3`)
  → hot expression trees and aggregation are compiled to fused machine code for the
  specific query, removing interpreter dispatch.
- **Specialized aggregation.** Hand-written aggregate functions + adaptive hash tables
  (two-level for high cardinality, specialized for fixed-width keys), per-thread then
  merged; in-memory by default (`max_bytes_before_external_group_by = 0` live).
- **Late materialization** (PREWHERE, in `read-path-indexing-and-execution.md`) reads
  filter columns first, then only the surviving rows of the wide columns.

Net: a decade of micro-optimization (SIMD kernels, codegen, cache-aware blocks) — the
OLAP scan/aggregate **throughput bar**.

## GreptimeDB — Apache DataFusion over Arrow

- **Batch-at-a-time over Arrow.** The executor is **DataFusion `=52.1`** (pinned in
  `Cargo.toml`): a physical plan of `ExecutionPlan` nodes, each yielding a stream of
  Arrow `RecordBatch`es (DataFusion default batch **8,192** rows; GreptimeDB's
  `SessionConfig` sets `target_partitions` but not a larger batch). Operators use
  Arrow's general-purpose **vectorized compute kernels**.
- **Parallelism.** `SessionConfig::with_target_partitions(parallelism)`
  (`query_engine/state.rs:127`) + a **custom `ParallelizeScan` optimizer rule**
  (inserted at index 5) splits scans across partitions/cores. Distribution is the
  **`MergeScanExec`** boundary — EXPLAIN of `GROUP BY service` shows
  `CooperativeExec → MergeScanExec` fanning the `Aggregate→TableScan` sub-plan into the
  region engine (Run 21). So scan+aggregate run inside the region via DataFusion.
- **Codegen.** DataFusion's expression codegen is far younger/narrower than
  ClickHouse's LLVM JIT — execution is mostly interpreted-vectorized over Arrow arrays.
- **The deliberate win: extensibility.** DataFusion is *pluggable* — GreptimeDB adds
  custom logical/physical nodes for **PromQL**, the **metric engine**, and time-series
  functions, and gets **Arrow-native zero-copy** interop + Rust memory safety. The
  metrics/PromQL nativeness the verdict rewards is *bought* with DataFusion's
  extensibility; a bespoke C++ engine would have to reimplement all of it.

## Side by side

| Aspect | ClickHouse | GreptimeDB (DataFusion 52.1) |
| --- | --- | --- |
| Vector unit | **block ~65,409 rows** | Arrow `RecordBatch` **~8,192 rows** |
| Operator kernels | bespoke C++ SIMD, decade-tuned | general Arrow compute kernels |
| Codegen | **LLVM JIT** expressions + aggregation (on) | young/narrow; mostly interpreted-vectorized |
| Aggregation | specialized adaptive hash tables, per-thread | DataFusion grouping (general) |
| Parallelism | `max_threads` pipeline lanes (auto=cores) | `target_partitions` + `ParallelizeScan`; `MergeScanExec` fan-out |
| Late materialization | **PREWHERE** (column-staged) | row-group/page **prune** + **post-decode** row filter (`PruneReader::precise_filter`, `read/prune.rs:119`); **no arrow `RowFilter`** (pass 77) → decodes all projected cols of a surviving row-group before dropping rows |
| Extensibility | fixed C++ (fast, not pluggable) | **pluggable** → PromQL, metric engine, TS functions |
| Throughput | **higher** (scan/aggregate bar) | competitive, younger on raw kernels |

## Why ClickHouse wins scan/aggregate throughput (ties to Runs 11–12)

The ~10× metric-aggregation (Run 11) and ~18× full-text (Run 12) gaps are the sum of:
**(1)** 8× larger vectors → less overhead per row; **(2)** hand-tuned SIMD kernels vs
general Arrow kernels; **(3)** runtime JIT of expressions + aggregation vs interpreted
vectorization; **(4)** specialized aggregation hash tables; **(5)** PREWHERE late
materialization. None is an *architectural flaw* in GreptimeDB — it is the accumulated
micro-optimization lead of a C++ analytical engine over a younger Rust/Arrow one.

## Axis consequence

- **Speed (axis #1):** ClickHouse's engine is faster on **heavy scans and
  aggregations** — confirmed and now mechanism-explained. **But** Parallax's dominant
  query is *anchored* bundle assembly, which prunes to a tiny row set before the engine
  runs, so the composite Q6 was **not throughput-bound** on either (Run 16, ~33 ms vs
  ~10 ms, both ≪300 ms). The execution-engine gap bites on **ad-hoc large scans /
  high-cardinality aggregation**, not on anchored retrieval.
- **The metrics-native win is an execution-engine *consequence*:** GreptimeDB's PromQL
  + metric engine exist *because* DataFusion is extensible. So the same design choice
  that costs raw scan throughput buys the metrics capability the verdict rewards. This
  is the central tradeoff, now grounded in the engine.

Net: reinforces the standing verdict — ClickHouse wins raw analytical throughput by
concrete engine mechanisms; GreptimeDB trades that for extensibility (PromQL/metric
engine) and Arrow/Rust fit, and the trade is acceptable because Parallax's hot query
is anchored, not scan-bound.

## Honest caveats

- **DataFusion is improving fast** (vectorization, codegen, spill). The gap is "as of
  v52.1," not permanent; re-check on version bumps — this is exactly the kind of
  execution-engine improvement the version-pin rule guards against treating as static.
- **Smoke scale.** The 8×-block / JIT advantages are reasoned + live-config-confirmed,
  not isolated in a controlled micro-benchmark here; the Run 11/12 end-to-end numbers
  are the empirical anchor.
- ClickHouse JIT has a warm-up (`min_count_to_compile_expression=3`) — the first few
  executions are interpreted; irrelevant for repeated query shapes (Parallax's case).
- GreptimeDB pushes scan+aggregate into the region (`MergeScanExec`), so at multi-node
  scale the execution parallelism story is also a distribution story (owed to a
  cluster run).

## Source / evidence

- ClickHouse: `src/Core/Settings.cpp:132` (`max_block_size` `DEFAULT_BLOCK_SIZE`),
  `:307` (`max_threads`); `src/Processors` + `QueryPipeline` (pipeline graph); live
  `system.settings`: `max_block_size=65409`, `max_threads=auto(10)`,
  `compile_expressions=1`, `compile_aggregate_expressions=1`,
  `max_bytes_before_external_group_by=0`.
- GreptimeDB: `Cargo.toml` (`datafusion = "=52.1"`);
  `src/query/src/query_engine/state.rs:126-128` (`SessionConfig`,
  `with_target_partitions` only — **no `batch_size`**, pass 77), `optimizer/parallelize_scan.rs`
  (`ParallelizeScan`); live EXPLAIN `CooperativeExec → MergeScanExec`. Read path (pass 77):
  `src/mito2/src/sst/parquet/reader.rs` (`RowGroupSelection` + page index pruning, no arrow
  `RowFilter`), `src/mito2/src/read/prune.rs:119` (`PruneReader::precise_filter` = post-decode).
- Empirical anchors: `local-benchmark-results.md` Run 11 (metric agg ~10×), Run 12
  (full-text ~18×), Run 16 (anchored Q6 not throughput-bound), Run 21 (this pass).
- Cross-refs: `read-path-indexing-and-execution.md` (planning/PREWHERE/skip/joins),
  `per-signal-verdict.md`, `greptimedb-internals.md` (DataFusion/PromQL).
