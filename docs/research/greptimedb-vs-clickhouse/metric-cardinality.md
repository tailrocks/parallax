# Metric Cardinality — How Each Engine Stores Many Series (Checklist Lead #6)

<!-- markdownlint-disable MD013 -->

Status: pass 48, extended passes 112–114 (Runs 76–79 — high-card storage curve: CH
`LowCardinality` wins low–mid cardinality (1k/200k) but **GreptimeDB wins at ~1M unique
series — a crossover**; `LowCardinality` cliff is *graceful*; metric-engine `__tsid` is
overhead not a saving. B13 storage curve complete — storage winner is cardinality-dependent;
GreptimeDB's clear edge is ingest ergonomics + extreme-cardinality). The
*partitioning/storage consequence* of high series cardinality —
GreptimeDB lead #6 ("logical metric tables → physical wide table, and the partitioning
consequence for high-cardinality metrics"). Pass 32 confirmed the logical→physical
layout; pass 20 (Run 11) measured high-card *aggregation speed*. This note is the
**physical organization** of many series, side by side. Source + live (Run 26).
Decision-relevant: Parallax metrics can carry high-cardinality labels (per-endpoint,
per-release, per-fingerprint).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## GreptimeDB — the metric engine is *built* for high cardinality

- **Series identity = `__tsid`** (a hash of the full label set; `metric-engine/
  src/row_modifier.rs`, with a dedicated `benches/bench_tsid_generator.rs` — the tsid
  generator is perf-critical *because* high cardinality stresses it). Each distinct
  label combination is one `__tsid`.
- **Shared physical wide table** (`data_region.rs`): many *logical* metrics map onto a
  small number of *physical* regions keyed by `(__table_id, __tsid, ts)` + the union of
  label columns (pass 32). Adding series adds **rows**, not tables/regions — so 10k or
  1M series do not create 10k tables.
- **PartitionTree memtable** (`memtable/partition_tree/`) is the high-cardinality
  ingest structure: it **dictionary-encodes the primary key** (`dict.rs` — repeated
  label strings are shared, not stored per row), **shards** series
  (`shard.rs`/`shard_builder.rs`), and **multi-partitions by primary key** when needed
  (`tree.rs` `Partition::has_multi_partitions`). The sparse primary-key codec has no
  fixed field count → variable label sets are first-class.
- **Label-filtered queries** use the inverted index on label columns
  (`indexing-internals.md`) → filter by `{job="api"}` without a full scan.
- **Growth** = region repartition splits the physical region as it grows
  (`distributed-and-scaling.md`), so cardinality growth is a topology change, not a
  rewrite.

No per-series dictionary cap; the dict is per-memtable and label strings are shared.
High cardinality is the metric engine's **design center**, not an edge case.

## ClickHouse — high cardinality needs schema care (the `LowCardinality` cliff)

ClickHouse stores one row per `(series, ts)` in a MergeTree, with labels as columns
(often `LowCardinality(String)`) in the `ORDER BY`, or in a `Map`. High cardinality
hits three limits:

- **`LowCardinality` dictionary cap = 8,192** (`low_cardinality_max_dictionary_size`,
  default — **confirmed live, Run 26**). Source doc: *"All the data that can't be
  encoded due to maximum dictionary size limitation ClickHouse writes in an **ordinary
  method**."* So a label column with **>8,192 distinct values overflows the dictionary
  and falls back to plain (un-dict-encoded) storage** — the classic high-cardinality
  cliff where `LowCardinality` stops helping.
- **Sparse primary index bloat:** high-cardinality `ORDER BY` prefixes mean more
  distinct granule boundaries → more marks, weaker locality. The mitigation is a
  low-cardinality `ORDER BY` prefix (e.g. `(metric_name, …, ts)`), a schema-design
  burden.
- **Compression degrades** when labels are high-cardinality and not dict-friendly.

The newer **experimental `TimeSeries` engine** (pass 44) stores tags in a separate
`AggregatingMergeTree` tags table keyed by a series id — structurally **closer to
GreptimeDB's `__tsid` model** — but it is experimental + off by default, so the GA path
for high-cardinality metrics today is "design the `ORDER BY` carefully and respect the
`LowCardinality` cap."

## Measured storage at 200k distinct series (Run 76 — refines the "cliff")

1M rows, **200,000 distinct series**, identical data both engines:

| Table | total on disk | `series` column |
| --- | --- | --- |
| ClickHouse `LowCardinality(String)` | **9.64 MiB** | 1.53 MiB |
| ClickHouse `String` (plain) | 10.11 MiB | 1.99 MiB |
| GreptimeDB plain mito table (`series` PK) | 11.99 MiB | — |
| **GreptimeDB metric engine** (`__tsid` physical, Run 77) | **12.63 MiB** | — |

Two findings:

- **The `LowCardinality` "cliff" is *graceful*, not a storage explosion.** At 200k
  distinct (≫ the 8,192 dict cap), `LowCardinality` is **still smaller than plain
  `String`** (col 1.53 vs 1.99 MiB; total 9.64 vs 10.11). So overflowing the dict means
  *losing the peak dict-encoding benefit*, **not** regressing below `String` — especially
  with the column sorted in `ORDER BY` (per-granule locality) + ZSTD. The cliff is a
  *don't-expect-magic* caveat, not a footgun that inflates storage.
- **High-card storage winner is CARDINALITY-DEPENDENT — there is a crossover (Run 79).**
  Fixed 1M rows, varying distinct series:

  | distinct series | ClickHouse `LowCardinality` | GreptimeDB plain | winner |
  | --- | --- | --- | --- |
  | 1,000 | 8.18 MiB | 9.18 MiB | ClickHouse ~1.12× |
  | 200,000 | 9.64 MiB | 11.99 MiB | ClickHouse ~1.24× |
  | 1,000,000 (all-unique) | **16.51 MiB** | **12.36 MiB** | **GreptimeDB ~1.34×** |

  ClickHouse `LowCardinality` wins at **low-to-mid** cardinality but **blows up at extreme
  cardinality** (all-unique → dict dead, pure overhead → 16.51 MiB), while GreptimeDB
  degrades gently (11.99 → 12.36). So **GreptimeDB wins storage past ~1M unique series** —
  the regime its metric engine targets — and ClickHouse wins the moderate-cardinality
  storage (thousands–100k). At 200k the metric engine itself is **not** smaller than the
  plain table (12.63 vs 11.99, Run 77 — `__tsid` is overhead on top of the labels, not a
  saving), so GreptimeDB's storage win at the extreme is its *general* Parquet+ZSTD
  scaling, not the metric engine specifically. **Net: storage winner depends on series
  cardinality; GreptimeDB's clear edge is ingest ergonomics (cap-free) + extreme-cardinality
  storage + multi-metric consolidation — not moderate-cardinality bytes (CH) or agg latency
  (CH, Run 67).**

## Side by side

| | GreptimeDB (metric engine) | ClickHouse (MergeTree) |
| --- | --- | --- |
| Series identity | `__tsid` = label-set hash | `ORDER BY` tuple of label columns |
| New series adds | a **row** in a shared physical table | a row; new label *values* stress the dict/index |
| Label encoding | per-memtable **dict** (shared strings), no fixed cap | `LowCardinality` dict **capped at 8,192**, then plain |
| High-card ingest | PartitionTree: dict + shard + multi-partition | one part per insert; dict overflow → ordinary encoding |
| Label filter | inverted index on label columns | granule prune via `ORDER BY`/skip index |
| Growth | region repartition (topology change) | manual `ORDER BY`/shard design |
| Built for high card? | **Yes** (design center) | **Workable with care** (or experimental TimeSeries) |

## The honest two-sided result

High cardinality splits across axes — both true, different things:

- **Ingest ergonomics (operability): GreptimeDB** — *but raw storage actually favors
  ClickHouse on a plain table (Run 76).* The metric engine
  + PartitionTree are designed so high-cardinality series are rows in a shared,
  dict-encoded, sharded structure with no `LowCardinality`-style cap and label-set
  hashing built in. **Ingest rate confirms it (Run 84): GreptimeDB ingest is
  cardinality-INSENSITIVE** — 1M rows at 1k vs 1M distinct series took 357 → 381 ms (**~1.07×,
  flat**), while **ClickHouse ingest slowed ~2.6×** (0.11 → 0.28 s) as `LowCardinality`
  overflowed + `ORDER BY` keys multiplied. **Re-verified Run 101 (no drift):** via
  `INSERT…SELECT` at 12 → 1M distinct series, GreptimeDB slowed only **1.16×** (588 → 683 ms,
  ≈ flat) vs ClickHouse **1.53×** with a plain `String` key — and the ~2.6× still stands with the
  idiomatic `LowCardinality` label (Run 84). The penalty is schema-dependent on CH (String 1.53× /
  LowCardinality 2.6×); GreptimeDB has no such knob to mis-size. *(Run 101 caveat: `INSERT…SELECT`
  absolute favours CH — it is the sensitivity ratio, not the native-path throughput, that is the
  claim.)* So GreptimeDB's clearest high-card win is the **ingest axis** (cap-free, ~flat with cardinality). On *bytes* the winner is
  **cardinality-dependent (Runs 76–79):** CH
  `LowCardinality` wins low–mid (1k ~1.12×, 200k ~1.24×) but **GreptimeDB wins at ~1M
  unique series ~1.34×** (CH `LowCardinality` blows up to 16.51 MiB all-unique vs GT 12.36)
  — the very-high-cardinality regime GreptimeDB targets. ClickHouse works but needs
  deliberate `ORDER BY` design and hits
  the 8,192 dict cliff on wild label values.
- **Aggregation *speed* at volume: ClickHouse (~2× warm, Run 37; corrected from ~10×).**
  The vectorized C++ engine (`query-execution-engine.md`) out-aggregates DataFusion
  regardless of the storage model. So "GreptimeDB handles high cardinality better" is
  about **modeling/storage**, not **aggregation latency** — the operator hypothesis
  ("GreptimeDB fastest") still does **not** hold for metric *aggregation speed*.
  **And it does not hold even via GreptimeDB's own PromQL path** (Run 44): the native
  PromQL planner is ~5× slower than GreptimeDB's *own* SQL at 40k series (≈590 vs ≈120 ms),
  because `SeriesDivide`/`SeriesNormalize` pays a near-fixed series-sort setup a streaming
  SQL hash-agg avoids. Ordering for raw metric-agg latency: **CH SQL > GT SQL > GT PromQL**.

For Parallax: if metrics carry genuinely high-cardinality labels, GreptimeDB's model
avoids the schema-tuning cliff and is the more ergonomic, cost-stable fit; if the
dominant metric workload is heavy aggregation latency over moderate cardinality,
ClickHouse's engine is faster. Parallax's metric usage (dashboards + alerting over
service/release/endpoint labels) leans toward *many series, modest per-query
aggregation*, which favors GreptimeDB's cardinality model — but this is a fit call, not
a speed win.

## Honest caveats

- **Storage-size-at-high-cardinality not measured here.** The `LowCardinality` 8,192
  cap is source-documented + the setting confirmed live, but a clean
  bytes-at-50k-distinct comparison is owed to the harness (Run 26's quick
  `system.columns` probe didn't capture the part sizes — a view/timing artifact, not a
  result). Proposed as a `benchmarking-the-differences.md` case: ingest N distinct
  series (N = 1k/10k/100k/1M), compare retained bytes + ingest rate + label-filter
  latency.
- ClickHouse's cliff is **mitigable** (raise the cap, tune `ORDER BY`, use the
  experimental TimeSeries engine) — it is a default-ergonomics gap, not an absolute
  wall, consistent with the PromQL finding (`promql-and-metrics-query.md`).
- GreptimeDB's PartitionTree dict is per-memtable; extreme cardinality still costs
  memory + flush pressure — "designed for it" ≠ "free." Magnitude owed to the harness.

## Source / evidence

- GreptimeDB: `src/metric-engine/src/{data_region,row_modifier,batch_modifier}.rs`,
  `benches/bench_tsid_generator.rs`; `src/mito2/src/memtable/partition_tree/
  {tree,dict,shard,shard_builder,partition}.rs` (`Partition::has_multi_partitions`,
  sparse primary-key codec).
- ClickHouse: `low_cardinality_max_dictionary_size = 8192` (`Core/Settings.cpp:3889`,
  "writes in an ordinary method" past the cap) — **live-confirmed Run 26**;
  `TimeSeries` engine (pass 44) as the experimental closer-to-GreptimeDB option.
- Empirical: Run 11/37 (agg speed — **~2× warm**, corrected from ~10× cold), Run 26 (LowCardinality cap live).
- Cross-refs: `greptimedb-internals.md` (metric engine physical layout, pass 32),
  `query-execution-engine.md` (why CH aggregates faster), `per-signal-verdict.md`
  (metrics rows), `compression-and-cost.md`.
