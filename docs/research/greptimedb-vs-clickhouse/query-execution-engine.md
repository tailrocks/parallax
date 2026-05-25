# Query Execution Engine — Why ClickHouse Out-Scans DataFusion

<!-- markdownlint-disable MD013 -->

Status: pass 42 + pass 77 (read-path late-materialization mechanism source-verified:
`PruneReader::precise_filter` post-decode, no arrow `RowFilter`) + pass 88 (corrected the
full-text attribution: the Run-12 "~18×" was a config artifact, not the engine — the engine
gap is real only for broad-term/heavy-agg, Runs 48–49) + pass 95 (**scan gap measured
row-dependent, Run 58** — ~7× @1M → ~14× @5M scan-bound, ~3× @8M agg-bound). The
execution-engine half of checklist #4 (the read-path note covers
planning / predicate pushdown / skip-vs-scan / joins; this is the *engine that runs
the plan*). It is the mechanism **behind the measured throughput gaps** — ClickHouse
**~2–3× on warm metric aggregation** at 40k series (shape-dependent: ~3× flat group-by,
~2× compute-heavier bucketed panel — Runs 37/67/96; corrected from the ~10× of
Run 11, which was a cold/first-run GreptimeDB scan) and on **broad-term** full-text /
scan (the Run-12 "~18×" was a `matches()`-on-bloom config artifact — selective is ~2×,
Runs 48–49; the engine gap is real only for broad-term scans). The verdict has called
this "a decade-tuned C++ vectorized engine"; this
note makes that concrete, against source + live settings (Run 21). *(The ~2–3× warm
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

## GreptimeDB Flat SST — the scan-format foundation (v1.0 GA, source-read Run 134)

The **Flat SST** format (default since v1.0 GA) is the scan-side redesign behind several recent
findings. Source (`src/mito2/src/sst/parquet/flat_format.rs`, v1.0.2): the Parquet layout is

```text
primary-key (tag) columns, field columns, time index, __primary_key (encoded), __sequence, __op_type
```

— i.e. it **stores the tag/primary-key columns as RAW, individual columnar columns** (tags
dictionary-encoded, `dictionary(uint32, binary)`), *alongside* the encoded composite `__primary_key`
blob. The pre-Flat format stored tags **only** inside the encoded composite key, so any query that
filtered or grouped on a tag had to **decode the composite key per row**. Flat SST makes the tag a
first-class column.

**Why it matters (and what it grounds):**
- **Tag-keyed group-by / filter reads the raw tag column directly** — `GROUP BY service`,
  `WHERE service=…` no longer decode the composite key per row. This is the mechanism behind the v1.0
  GA "Flat SST" claim (write ~4×, high-cardinality TSBS query latency up to ~10×) and the marginal
  GT-nightly agg edge (Run 131).
- **It is what the prefilter reads** (Run 121/122): `prefilter_flat_batch_by_primary_key` decodes the
  raw PK/partition columns first → row selection → then the rest. Flat SST is the precondition for
  that late-materialization.
- **`__primary_key` (encoded) + `__sequence` + `__op_type`** remain for ordering, dedup
  (`DedupReader`, Runs 114–117), and MVCC — so the dedup cost (per-series merge) is unchanged by
  Flat SST; Flat SST helps the *scan/group* side, not the dedup side.

So GreptimeDB's scan format is now genuinely columnar on tags (like ClickHouse's columns), which is
why tag-keyed aggregations are ~2× (not catastrophic) and the prefilter works — while the *raw
vectorized-execution throughput* gap (SIMD/hash-agg, Runs 124/125) is separate and still the diffuse,
slow-closing part.

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

**Correction (Runs 37, 48–49) — these gaps are narrower than first measured.** The metric-agg
gap is **~2–3× warm** (Runs 37/67/96, shape-dependent; the ~10× was cold). The full-text "~18×" (Run 12) was **not the
engine at all** — it was a backend/function artifact (`matches()` on a `backend='bloom'` index
full-scans; with the correct pairing selective full-text is ~6–8 ms, ~2× CH — Runs 48–49). The
engine gap shows for real on **broad-term** full-text (~12×, scanning the matched set) and
heavy aggregation. Those genuine engine gaps are the sum of:
**(1)** 8× larger vectors → less overhead per row; **(2)** hand-tuned SIMD kernels vs
general Arrow kernels; **(3)** runtime JIT of expressions + aggregation vs interpreted
vectorization; **(4)** specialized aggregation hash tables; **(5)** PREWHERE late
materialization. None is an *architectural flaw* in GreptimeDB — it is the accumulated
micro-optimization lead of a C++ analytical engine over a younger Rust/Arrow one.

**Measured scaling (Run 58) — the gap is row-dependent, not a fixed ratio.** Re-verifying
unindexed full scans: a pure filtered count grows **~7× at 1M → ~14× at 5M** (CH 2→3 ms,
GreptimeDB 15→43 ms) — a per-row *throughput* difference, so it widens with scan width —
while a full `sum` over 8M is only **~3×** (CH 29 ms / GreptimeDB 91 ms) because the
aggregate work both engines do dilutes the scan-speed delta. So state the engine gap as
**"scales with scan width (~3× agg-bound up to ~14× scan-bound at 1–8M warm, larger at
GB-scale cold)"**, not a single multiplier. *(This also retired the stale Run 31 "GT 95 ms
/ ~10×" anchored-scan figure — the same 1M scan reproduces at 15 ms; the 95 ms was an
HTTP-wall/cold artifact, per the Run 40 correction.)*

**Also per-row-compute-dependent, not only scan-width (Run 96).** Re-verifying the metric
dashboard panel on `metrics_hc` (8M/40k-series): a **flat `avg by service`** (40 groups,
L1-resident hash table) is **~3.0× warm** (CH ~38 ms / GT ~116 ms — pure scan-throughput
bound, so ClickHouse's 65k-block+SIMD scan dominates), but the realistic **time-bucketed
line chart** `avg per 1-min bucket × service` (4,000 groups + a `date_bin`/`toStartOfMinute`
scalar per row) **narrows to ~2.0×** (CH ~63 ms / GT ~126 ms). Mechanism: the added per-row
bucket compute and the 100× larger hash table are work *both* engines pay comparably, diluting
ClickHouse's scan-throughput edge. So ~2–3× is the scan-bound *ceiling*; compute-heavier
aggregations trend toward ~2×. Both panels stay **sub-300 ms warm on GreptimeDB** — interactive
either way, so the gap is real but not user-perceptible on single-user dashboard refreshes.

**The full metric-panel picture (Runs 96/105/109/113) confirms the trend:** counter-rate panel
**~1.6×** (Run 113, CH 12 / GT 19 ms — smallest, most per-row compute), bucketed line **~2×**, flat
avg-by-service **~3×** (Run 96), and **last-value GreptimeDB *wins* ~2.4×** (Run 109 — time-sorted
layout beats `argMax`). Only the wide PromQL range is slow on GT (~5.6× its own SQL, Run 105 — use
SQL/Flow). Net: across real metric dashboards GreptimeDB ranges from winning to ~3× behind, **all
interactive** — the scan-throughput edge only dominates flat full-table aggregation.

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
- **The ~2–3× metric-agg gap is NEITHER batch size NOR JIT (Runs 124/125, empirically isolated):**
  ClickHouse at GreptimeDB's 8,192 block size is still ~3× faster (Run 124), and ClickHouse with
  `compile_aggregate_expressions=0` is still ~3.7× faster (Run 125 — JIT off was even *faster* on a
  heavy 5-aggregate query). So the gap is the **diffuse, cumulative maturity of the vectorized
  execution core** — bespoke SIMD aggregation kernels, cache-efficient/adaptive hash tables, tight
  C++ scan+group loops — not any single tunable. **It is the slowest-closing gap** (no single PR;
  accrues only as the DataFusion execution core matures upstream), unlike the SST-layer wins
  GreptimeDB shipped itself (prefilter/TopK/Flat-SST). Engineering, not physics — but the longest
  timeline.
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
  (full-text ~18×), Run 16/56 (anchored Q6 not throughput-bound), Run 21, **Run 58
  (unindexed-scan gap row-dependent: ~7× @1M → ~14× @5M scan-bound, ~3× @8M agg-bound;
  retired Run 31's 95 ms artifact)**.
- Cross-refs: `read-path-indexing-and-execution.md` (planning/PREWHERE/skip/joins),
  `per-signal-verdict.md`, `greptimedb-internals.md` (DataFusion/PromQL).
