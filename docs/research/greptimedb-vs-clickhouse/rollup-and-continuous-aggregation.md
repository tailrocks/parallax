# Rollup / Continuous Aggregation — Flow vs Materialized Views

<!-- markdownlint-disable MD013 -->

Status: pass 71 (live-verified; was pass 27 source-only), re-verified pass 106 (Run 70 —
correctness reproduced + rollup freshness distinction measured). The "rollup / correlation
tooling" dissection (checklist item for ClickHouse; previously only covered on the
ClickHouse side). Compares GreptimeDB's **Flow engine** to ClickHouse's **Materialized
Views + AggregatingMergeTree** for Parallax's rollups: metric downsampling (5 m / 1 h),
issue/fingerprint counts per window, release-regression aggregates. Code-grounded **and
now Docker-verified (Run 43)** — this was the last major note with no live run.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## Mechanism, side by side

| Aspect | GreptimeDB — Flow engine | ClickHouse — MV + AggregatingMergeTree |
| --- | --- | --- |
| What it is | A dedicated **dataflow engine** (`src/flow`): substrait plan → dataflow → sink table. | A **materialized view** that runs on insert + an `AggregatingMergeTree` target storing partial aggregate states. |
| Modes | **Streaming** (continuous incremental, low-latency — laminar-flow RFC `2025-09-08`) **and Batching** (time-window-aware query re-run when new data arrives — `src/flow/src/batching_mode.rs`, RFC `2026-03-16-flow-inc-query`). | **Push MV** (runs per **insert block**) + **Refreshable MV** (periodic full re-run). |
| DDL | `CREATE FLOW … SINK TO tbl EXPIRE AFTER INTERVAL '…' EVAL INTERVAL '…' AS <agg query>` (`create_parser.rs:277-320`). | `CREATE MATERIALIZED VIEW mv TO agg AS SELECT …State(), toStartOfInterval(ts,…) GROUP BY …`. |
| Result form | Sink table holds **finalized** rows (Flow computes the full aggregate). | Target holds **partial `-State`**; queries must use `-Merge` (or `FINAL`) to finalize. |
| Windowing | Time-window-native (`date_bin`, PromQL range) + **`EXPIRE AFTER`** drops old windows. | `toStartOfInterval`; `AggregatingMergeTree` merges states across blocks at compaction. |
| Per-block semantics | None — Flow sees the stream/window, not a single insert block. | MV sees **only the inserted block**; cross-block correctness relies on `AggregatingMergeTree` merging states (a known foot-gun for JOINs/dedup in MVs). |
| Auto-population scope | **Forward-only** — Flow computes data inserted *after* creation; pre-existing rows are not pulled in (Run 43: 864k → 0 on flush). | **Forward-only** — push-MV runs only on new insert blocks; pre-existing rows not seen. |
| Historical backfill | sink is a plain table → one-off `INSERT…SELECT date_bin(…)` (Run 43, 84 rows). | target is a plain table → one-off `INSERT…SELECT …State()`. |
| Maturity | **Younger** (laminar 2025, incremental-query 2026). | **Very mature**, battle-tested at scale; the `-State`/`-Merge` ceremony is the cost. |

## Live verification (Run 43)

Built the same 1 h `avg(gauge)`-by-service rollup on both engines over `metrics_real`
(864 000 rows, ~6 h, 12 services) and measured warm:

| | GreptimeDB Flow | ClickHouse MV+AggMT |
| --- | --- | --- |
| Raw windowed-avg over 864k (warm) | ~16–25 ms | ~10–13 ms |
| Rollup-table read (warm) | ~3–4 ms | ~2 ms |
| Pre-aggregation read speedup | **~5×** | **~5–6×** |

- **Pre-aggregation pays off on both (~5–6×), confirmed live** — not just reasoned.
- **Flow auto-population is forward-only.** `CREATE FLOW` + `ADMIN FLUSH_FLOW` over the
  pre-existing 864k rows yielded **0 sink rows**; only inserts arriving *after* flow
  creation flowed through (a fresh probe insert appeared post-flush). **But the sink is a
  plain writable table**, so a one-off `INSERT INTO sink SELECT … date_bin(…)` backfills
  history (verified, 84 rows). This is **operationally parallel to ClickHouse**, where the
  push-MV maintains forward and a manual `INSERT…SELECT …State()` backfills the target.
- **Stored-form contrast confirmed live:** GT sink holds **finalized** values (read direct);
  CH target holds partial **`-State`** (read via `avgMerge`). Cleaner read model for GT.
- **Flow correctness confirmed** (sink matched raw post-dedup truth; the dedup wrinkle
  cross-confirmed `dedup-and-update-semantics.md`). Full numbers: `local-benchmark-results.md`
  Run 43.

**Re-verification (Run 70) — correctness reproduced + a freshness distinction.** Built the
same minute+service rollup (`avg`, `count`) on both from 4 source rows: **identical
results** (api min0 avg=15/n=2, web min0 5/1, api min1 30/1) — Flow and MV+AggMT both
correct. **New finding — rollup freshness:** ClickHouse's **push-MV populates the sink
synchronously within the INSERT** (the 3 sink rows were present *immediately*, no flush);
**GreptimeDB Flow is batched/async** — the sink stayed empty until `ADMIN FLUSH_FLOW`
(the laminar *streaming* mode is low-latency, but the default/batching path materializes
on an interval/flush, not inside the insert). So for **real-time rollup reads** (a
dashboard refreshing a downsample seconds after ingest) ClickHouse's MV is fresher; for
*eventually-consistent* downsamples both are fine. A freshness tilt to ClickHouse on the
rollup path specifically (distinct from raw-write freshness, which is a tie —
`write-path-and-ingestion.md`).

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
  ingest/background, making dashboard/rollup reads cheap on both — **measured ~5–6× read
  speedup on each** (Run 43), so this is real, not just reasoned. Not a differentiator —
  both have the capability and a comparable payoff.
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
- Live: `local-benchmark-results.md` Run 43 (rollup ~5–6× read speedup both; Flow forward-only auto-pop + manual sink backfill; finalized vs `-State` read model), Run 70 (correctness reproduced minute+svc; **rollup freshness: CH MV synchronous-on-insert vs GT Flow flush/interval-batched**). Cross-ref `dedup-and-update-semantics.md` (the dedup wrinkle in the correctness check).
