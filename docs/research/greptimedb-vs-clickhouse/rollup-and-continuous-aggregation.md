# Rollup / Continuous Aggregation — Flow vs Materialized Views

<!-- markdownlint-disable MD013 -->

Status: pass 27. The "rollup / correlation tooling" dissection (checklist item for
ClickHouse; previously only covered on the ClickHouse side). Compares GreptimeDB's
**Flow engine** to ClickHouse's **Materialized Views + AggregatingMergeTree** for
Parallax's rollups: metric downsampling (5 m / 1 h), issue/fingerprint counts per
window, release-regression aggregates. Code-grounded.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Mechanism, side by side

| Aspect | GreptimeDB — Flow engine | ClickHouse — MV + AggregatingMergeTree |
| --- | --- | --- |
| What it is | A dedicated **dataflow engine** (`src/flow`): substrait plan → dataflow → sink table. | A **materialized view** that runs on insert + an `AggregatingMergeTree` target storing partial aggregate states. |
| Modes | **Streaming** (continuous incremental, low-latency — laminar-flow RFC `2025-09-08`) **and Batching** (time-window-aware query re-run when new data arrives — `src/flow/src/batching_mode.rs`, RFC `2026-03-16-flow-inc-query`). | **Push MV** (runs per **insert block**) + **Refreshable MV** (periodic full re-run). |
| DDL | `CREATE FLOW … SINK TO tbl EXPIRE AFTER INTERVAL '…' EVAL INTERVAL '…' AS <agg query>` (`create_parser.rs:277-320`). | `CREATE MATERIALIZED VIEW mv TO agg AS SELECT …State(), toStartOfInterval(ts,…) GROUP BY …`. |
| Result form | Sink table holds **finalized** rows (Flow computes the full aggregate). | Target holds **partial `-State`**; queries must use `-Merge` (or `FINAL`) to finalize. |
| Windowing | Time-window-native (`date_bin`, PromQL range) + **`EXPIRE AFTER`** drops old windows. | `toStartOfInterval`; `AggregatingMergeTree` merges states across blocks at compaction. |
| Per-block semantics | None — Flow sees the stream/window, not a single insert block. | MV sees **only the inserted block**; cross-block correctness relies on `AggregatingMergeTree` merging states (a known foot-gun for JOINs/dedup in MVs). |
| Maturity | **Younger** (laminar 2025, incremental-query 2026). | **Very mature**, battle-tested at scale; the `-State`/`-Merge` ceremony is the cost. |

## For Parallax's rollups

- **Metric downsampling (5 m / 1 h):** GreptimeDB Flow is a clean fit — a
  time-window-native streaming or batching aggregation that sinks downsampled
  series, with `EXPIRE AFTER` for raw-data windows. ClickHouse does it canonically
  with an MV writing `avgState`/`quantileState` into an `AggregatingMergeTree`,
  queried via `-Merge`. Both work; GreptimeDB's is less ceremony and aligns with the
  metric/PromQL model, ClickHouse's is more proven.
- **Issue / fingerprint rollups (count + first/last-seen per window):** both express
  it directly (`count`, `min(ts)`, `max(ts)` grouped by window). ClickHouse uses
  `SummingMergeTree`/`AggregatingMergeTree`; GreptimeDB a Flow sinking to a rollup
  table.
- **Release-regression aggregates:** periodic windowed recompute — GreptimeDB Flow
  **batching mode** (`EVAL INTERVAL`) or ClickHouse **refreshable MV** are the
  natural analogs.

## Consequence (axes)

- **Cost (#2) / speed (#1):** pre-aggregation moves compute from query time to
  ingest/background, making dashboard/rollup reads cheap on both. Not a
  differentiator — both have the capability.
- **Fit:** GreptimeDB Flow's **streaming + batching + time-window-native + EXPIRE**
  is a slightly cleaner model for Parallax's metric/issue rollups (continues the
  metric-native theme), and avoids the MV per-block + `-State`/`-Merge` ceremony.
  **But** ClickHouse's MV/AggregatingMergeTree is far more **mature** and
  battle-tested — a real maturity edge for ClickHouse here.
- **Net:** a **wash with opposite tilts** — GreptimeDB cleaner/metric-native model,
  ClickHouse more mature/proven. Neither moves the verdict; both give Parallax the
  rollup tooling it needs. Folds into the implementation notes (metric-downsampling
  design): GreptimeDB → `CREATE FLOW`; ClickHouse → MV + `AggregatingMergeTree`.

## Source / evidence

- GreptimeDB Flow: `src/flow/src/{lib.rs,adapter.rs (StreamingEngine),batching_mode.rs}`; DDL `src/sql/src/parsers/create_parser.rs:277-320,1496-1526` (`CREATE FLOW … SINK TO … EXPIRE AFTER … EVAL INTERVAL`); RFCs `2024-01-17-dataflow-framework`, `2025-09-08-laminar-flow`, `2026-03-16-flow-inc-query`.
- ClickHouse: MV + `AggregatingMergeTree`/`SummingMergeTree` (`clickhouse-internals.md`, `clickhouse-implementation.md`); refreshable MV.
