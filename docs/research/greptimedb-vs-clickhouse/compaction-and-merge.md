# Compaction / Merge — Side by Side (Subsystem #5)

<!-- markdownlint-disable MD013 -->

Status: pass 23. The background reorganization process: strategy, write
amplification, and effect on read speed + freshness. Deepens what the internals
notes touched, ties together the B9 small-write finding and the read-speed/cost
axes. Code-grounded (pinned commits).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Mechanism, side by side

| Aspect | GreptimeDB — TWCS | ClickHouse — SimpleMergeSelector |
| --- | --- | --- |
| Strategy | **Time-Window Compaction** (`src/mito2/src/compaction/twcs.rs`, `window.rs`): bucket SST files by time window; merge files within a window. | **Size-tiered continuous merge** (`src/Storages/MergeTree/MergeSelector/SimpleMergeSelector`): pick a contiguous part range minimizing write amplification. |
| Levels | 2 levels: L0 (just-flushed) → L1 (compacted) (`LEVEL_COMPACTED=1`). | Implicit size tiers; parts merged repeatedly toward larger parts. |
| Trigger | `trigger_file_num` files in a window; `DEFAULT_MAX_INPUT_FILE_NUM=32` per compaction. | Background pool continuously scores candidate ranges; merges toward `max_bytes_to_merge_at_max_space_in_pool` (**150 GB** default, `MergeTreeSettings.cpp:465`). |
| Time awareness | **Time-aware by design** — windows align with the time index. | **Not time-aware** by default (time only via `PARTITION BY`); merges by size. |
| Write-amplification control | Old windows stop being re-merged once compacted → **low amplification on aged data**. | `base` knob = merge "arity" (ratio of merged size to largest input part); amplification ∝ **log(data_size)** for a balanced tree. |
| Dedup / variant transform | Dedup (last-row) only when `found_runs ≤ 2 && !append_mode` (`twcs.rs:94`); `append_mode` skips it. | Merge applies the engine-variant row transform (Replacing/Aggregating/…) at merge time. |

## Write amplification — the key cost difference

ClickHouse's own `SimpleMergeSelector` doc states the tradeoff precisely: *"To
lower the number of parts we can merge eagerly but write amplification will
increase… if the tree is balanced, depth ∝ log(data size), total work ∝
data_size·log(data_size), write amplification ∝ log(data_size)."* So ClickHouse
**re-writes data O(log N) times** as parts grow toward 150 GB — the cost of
keeping part count low for fast scans.

GreptimeDB's TWCS bounds this differently: once a **time window** is compacted to
L1 and no new data lands in that (past) window, **it is never re-merged**. For
**time-ordered telemetry** (Parallax's exact shape — recent data hot, old data
immutable), this means **write amplification is concentrated on recent windows and
drops to ~zero for aged data**. ClickHouse, being size-tiered, may re-merge old
parts into bigger ones regardless of age.

→ **Cost axis (#2, CPU/write):** for append-only time-series where old data never
changes, GreptimeDB's TWCS does **less total merge work over the data's lifetime**
than ClickHouse's size-tiered merging. ClickHouse trades that extra write
amplification for fewer/larger parts (faster full scans). A real
mechanism-level cost difference favoring GreptimeDB on aged time-series data —
**unmeasured locally** (needs a long-running ingest + CPU-over-time sample); flagged.

## Read-speed effect

- **ClickHouse** merges toward few large parts → a full scan touches few parts →
  great for the analytical scans where it already wins (B1/B5). The sparse index +
  large sorted parts are the read-speed engine.
- **GreptimeDB** keeps per-time-window files → a **time-ranged** query prunes to the
  window and reads few files (the common observability query: "last 1h of service
  X"). For a **full-history** scan it touches more files than ClickHouse's few giant
  parts — consistent with ClickHouse winning full scans (B5/B1) and GreptimeDB being
  competitive on time-windowed/anchored reads (B1 selective-filter tie).

## Freshness effect

Both merge in the background and do not block reads (MVCC `Version` snapshot /
atomic part swap). Neither requires a merge for visibility (Run 5 freshness tie).
The B9 finding is this subsystem at work: ClickHouse's continuous merge collapses
the 300 small insert-parts (the SimpleMergeSelector eagerly merging), at the cost
of write amplification; GreptimeDB's memtable avoids the small-part problem
upstream, so TWCS only ever sees flushed SSTs.

## Connection to earlier findings

- **B9 (small writes):** ClickHouse's eager merge is why 300 insert-parts became 1
  active part — SimpleMergeSelector minimizing part count. The write-amplification
  cost is the flip side.
- **B10 (object store):** GreptimeDB's TWCS produces a few large window-aligned
  Parquet objects (4 objects, Run 9); ClickHouse's size-tiered merge + per-column
  Wide-part layout produced 74 objects. The compaction strategy directly shapes the
  object-store layout.
- **Append mode:** TWCS skipping dedup under `append_mode` (`twcs.rs:94`) is the
  read+write saving for Parallax's append-only log/event signals.

## Axis roll-up

| Sub-axis | Winner | Mechanism | Confidence |
| --- | --- | --- | --- |
| Write amplification on aged time-series | **GreptimeDB** | TWCS never re-merges sealed past windows; ClickHouse re-merges by size, O(log N). | arch (code doc) |
| Full-history scan read speed | **ClickHouse** | merges to few giant parts (≤150 GB). | arch + B1/B5 |
| Time-windowed read speed | ~tie / GreptimeDB | window-aligned files prune tightly. | arch + B1 tie |
| Freshness (merge not on read path) | tie | both background, MVCC/atomic-swap. | Run 5 |
| Small-write part pressure | GreptimeDB | memtable absorbs; CH relies on eager merge + async insert. | B9 |

## Source / evidence

- GreptimeDB TWCS: `src/mito2/src/compaction/{twcs.rs:37-107,window.rs}` (time-window picker, `trigger_file_num`, `DEFAULT_MAX_INPUT_FILE_NUM=32`, `LEVEL_COMPACTED=1`, append-mode dedup skip at `twcs.rs:94`).
- ClickHouse merge: `src/Storages/MergeTree/MergeSelector/SimpleMergeSelector*` (write-amplification doc + `base` knob), `MergeTreeDataMergerMutator`, `max_bytes_to_merge_at_max_space_in_pool=150 GB` (`MergeTreeSettings.cpp:465`).
- Ties to `local-benchmark-results.md` Runs 7/9.
