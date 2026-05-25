# Continuous Aggregation / Rollups — GreptimeDB Flow vs ClickHouse Materialized Views

<!-- markdownlint-disable MD013 -->

Status: pass (Run 149) — source-grounded (`src/flow` @v1.0.2) + **live-verified** (GT `flows`
catalog present; CH `MaterializedView` engine + refreshable MV enabled). Why this note exists: the
**aggregation gap** (`query-execution-engine.md`: GreptimeDB ad-hoc aggregation ~2× slower warm than
ClickHouse) is the verdict's main read-speed concession. Continuous aggregation is the **escape hatch**
— you pre-compute *recurring* rollups so dashboards query a small pre-aggregated table instead of
raw data, sidestepping the scan/vectorization gap entirely. Both engines have it; this note compares
the mechanism, maturity, and what it means for the DQ6 closability decision.

Pins: GreptimeDB `v1.0.2`, ClickHouse `v26.5.1.882-stable`, re-confirmed latest stable 2026-05-25.

## Why it matters for Parallax

Parallax has two aggregation regimes:

1. **Recurring/known rollups** — dashboard tiles, SLO burn rates, per-service error-rate timelines,
   p99 latency over fixed windows. The query shape is *known ahead of time* and runs constantly.
2. **Ad-hoc analytics** — an engineer (or agent) slices the raw data a new way during an
   investigation. Shape unknown until asked.

Continuous aggregation closes regime (1) on *both* engines: define the rollup once, the engine
maintains a small result table incrementally, and the dashboard reads the result (sub-ms, no raw
scan). So **the agg-gap only bites regime (2)** — genuinely ad-hoc, unplanned aggregation. That
reframes the verdict: GreptimeDB's slower raw aggregation is *not* a dashboard problem (rollups fix
it), only an ad-hoc-exploration problem.

## GreptimeDB — `CREATE FLOW` (a dataflow engine)

Source: `src/flow` — *"manage dataflow in Greptime … transform substrait plan into its own plan and
execute it"* (`lib.rs`). A **Flow** is a continuous query whose output is incrementally written to a
**sink table**; defined with `CREATE FLOW … AS SELECT … GROUP BY …`. Live-verified: the `flows`
catalog table exists in `information_schema` on the running v1.0.2 standalone.

- **Runs in a dedicated `Flownode` role** (`FlownodeBuilder`/`FlownodeServer`/`FlownodeInstance`,
  `server.rs`) — a separate compute role alongside Frontend/Datanode/Metasrv, with its own heartbeat
  (`heartbeat.rs`). So continuous aggregation scales as its own tier (consistent with the
  region/topology scaling story, `distributed-and-scaling.md`).
- **Two execution modes** (`engine.rs`: *"a trait for flow engine, used by both streaming engine and
  batch engine"*):
  - **Streaming** (`compute.rs` / `StreamingEngine`) — true incremental dataflow operators; updates
    the sink as each batch arrives.
  - **Batching** (`batching_mode.rs`) — *"time-window-aware normal query triggered when new data
    arrives"*: re-runs the aggregation over the affected time window on a cadence
    (`experimental_min_refresh_duration`, `query_timeout` default 10 min, `slow_query_threshold`). A
    pragmatic "re-aggregate the touched window" rather than operator-level streaming.
- **Maturity:** GA feature (catalog present, documented `CREATE FLOW`), but the batching-mode knobs
  are still `experimental_*` (min-refresh, frontend-scan/activity timeouts, grpc retries) — younger
  than ClickHouse's MV.

## ClickHouse — Materialized Views (insert-triggered) + Refreshable MVs

Live-verified: `system.table_engines` has `MaterializedView`; `allow_experimental_refreshable_materialized_view = 1`
(**on by default** in 26.x).

- **Insert-triggered MV** (the classic, fully GA): `CREATE MATERIALIZED VIEW mv TO target AS SELECT …
  GROUP BY …`. The SELECT runs on **each inserted block** at ingest time and writes partial
  aggregates (usually `AggregatingMergeTree` + `-State`/`-Merge`) to the target. **Incremental on
  every insert** → always fresh, no polling. Decade-mature, the canonical OLAP rollup pattern.
- **Refreshable MV** (`REFRESH EVERY …`, experimental-but-default-on): periodically re-executes the
  full query and replaces the target — ClickHouse's analog to GreptimeDB's *batching* mode (for
  rollups that aren't cleanly incremental, e.g. joins or dedup). Enabled out of the box in 26.x.

## Side by side

| | GreptimeDB Flow | ClickHouse Materialized View |
| --- | --- | --- |
| DDL | `CREATE FLOW … AS SELECT … GROUP BY` | `CREATE MATERIALIZED VIEW … TO target AS SELECT …` |
| Incremental-on-ingest | **Streaming mode** (dataflow operators) | **Insert-triggered MV** (runs per inserted block) |
| Periodic re-aggregate | **Batching mode** ("re-run touched window", `min_refresh_duration`) | **Refreshable MV** (`REFRESH EVERY`, experimental-default-on) |
| Where it runs | dedicated **Flownode** role (scales separately) | inside the server, at insert time (or refresh scheduler) |
| Result storage | sink table (normal table) | target table (typically `AggregatingMergeTree`) |
| Maturity | GA; batching knobs `experimental_*` (younger) | insert-MV decade-mature; refreshable still experimental |
| State model | substrait plan → flow plan, incremental state | partial agg states (`-State`/`-Merge`) merged on read |

## Parallax implication + verdict tie-in (DQ6 closability)

- **The recurring-rollup gap is closed on both engines.** Parallax's dashboards/SLOs/timelines are
  known shapes → define them as a Flow (GreptimeDB) or MV (ClickHouse); queries hit the small
  pre-aggregated table, so the raw-scan agg-gap is irrelevant for regime (1). This **narrows the
  agg-gap concession to genuinely ad-hoc analytics** — reinforcing the verdict's "anchored + planned
  rollups → GreptimeDB fine; heavy ad-hoc analytics → ClickHouse."
- **ClickHouse's continuous-agg is more mature.** Insert-triggered MVs are battle-tested and *always
  fresh* (incremental per block); GreptimeDB's streaming mode is comparable in concept but younger,
  and its batching mode is a windowed re-query with experimental knobs. So *if* Parallax leans heavily
  on complex incremental rollups, ClickHouse's MV is the safer-today bet (a maturity edge, not a
  capability gap).
- **GreptimeDB's Flownode scales as its own tier** — the rollup compute is a separate role that the
  region/Metasrv model can place/scale independently, fitting the topology-change scaling story. (CH
  MV compute is coupled to the insert path / server.)
- **Closability verdict:** continuous aggregation is a *shipped, two-sided* feature — it does **not**
  differentiate the decision much (both close regime 1). It mainly **defuses** the agg-gap as a
  blocker: the gap is real only for ad-hoc analytics, and even ClickHouse's win there is a "build a
  rollup" away from being moot. Net: a small **maturity** edge to ClickHouse on rollup tooling;
  neither side wins decisively.

## The concrete grouped-error rollup (Sentry-style) — built live, Run 160

Making the abstract rollup concrete for Parallax's grouped-error requirement (the *aggregate* part of
"grouped errors"; the mutable workflow state lives in the relational store — `platform-fit-and-alternatives.md`).
Built the Sentry-style rollup live on ClickHouse (`error_events`, via `docker exec`):

```sql
CREATE TABLE ge_roll (
  fingerprint String,
  n          AggregateFunction(count),
  first_seen AggregateFunction(min, DateTime64(3)),
  last_seen  AggregateFunction(max, DateTime64(3)),
  latest     AggregateFunction(argMax, String, DateTime64(3))
) ENGINE=AggregatingMergeTree ORDER BY fingerprint;

CREATE MATERIALIZED VIEW ge_mv TO ge_roll AS
SELECT fingerprint, countState() n, minState(ts) first_seen,
       maxState(ts) last_seen, argMaxState(message, ts) latest
FROM error_events GROUP BY fingerprint;
-- read: countMerge(n), minMerge(first_seen), maxMerge(last_seen), argMaxMerge(latest)
```

**Computes correctly** — the `-Merge` read returned the right rollup (e.g. `fp-135` → count 21,
first/last seen, latest message), matching the query-time aggregate (Run 156). So the Sentry-style
grouped-error rollup is **cleanly expressible on ClickHouse** via `AggregatingMergeTree` partial states
(`-State` on insert via the MV → `-Merge` on read). **GreptimeDB** does the same two ways: a `Flow`
(streaming/batching, Run 149) into a sink table, **or** query-time `count`/`min`/`max`/`last_value(…
ORDER BY ts)` (Run 156, also correct). So both columnar engines build the grouped-error *aggregate*;
only the *mutable workflow state* (status/assignee) needs the relational store (Turso/Postgres).

*Honest caveat:* a quick live incremental-insert test (insert one new `fp-135` error → expect the MV to
bump count/last_seen) did **not** reflect immediately — almost certainly `async_insert=1` (default-on)
buffering the single row, and I dropped the rollup before it flushed. So the *incremental-on-insert*
freshness is the **documented** MV behavior, **not freshly re-confirmed** here; the rollup *correctness*
(via backfill + `-Merge`) is confirmed. Test objects were dropped + the test insert reverted.

## Honest caveats

- **Not benchmarked here.** This grounds the *mechanism + availability* (source + live catalog), not
  rollup ingest-overhead or freshness latency. A Flow-vs-MV throughput/freshness benchmark (define the
  same per-service-per-minute error-rate rollup on both, measure ingest cost + result staleness) is
  owed to the harness — a good server-tier case.
- **GreptimeDB batching mode is experimental-knobbed** — the `experimental_min_refresh_duration` and
  frontend-scan timeouts suggest the windowed-refresh path is still settling; streaming mode is the
  more mature Flow path.
- **ClickHouse refreshable MV is experimental** (though default-on) — for strict-freshness rollups the
  insert-triggered MV is the production choice; refreshable suits non-incremental shapes (joins/dedup).
- Both pre-aggregation approaches trade ingest-time (or refresh-time) compute for query-time speed —
  the rollup must be defined ahead of time; neither helps a *novel* ad-hoc slice.

## Source / evidence

- GreptimeDB: `src/flow/src/lib.rs` (crate doc: dataflow, substrait→flow plan), `engine.rs` (trait for
  streaming + batch engines; `CreateFlowArgs`, `FlowId`), `compute.rs`/`StreamingEngine` (streaming),
  `batching_mode.rs` (*"time-window-aware normal query triggered when new data arrives"*;
  `BatchingModeOptions` with `experimental_min_refresh_duration`, `query_timeout` 10 min),
  `server.rs` (`Flownode*` role). Live: `information_schema.flows` present on v1.0.2 standalone
  (via `docker exec`).
- ClickHouse: `system.table_engines` → `MaterializedView`; `system.settings`
  `allow_experimental_refreshable_materialized_view = 1` (live, default-on 26.x). Insert-triggered MV
  + `AggregatingMergeTree` `-State`/`-Merge` is the canonical pattern.
- Cross-refs: `query-execution-engine.md` (the agg-gap this defuses), `distributed-and-scaling.md`
  (Flownode as a scalable role; two-stage agg fan-out), `verdict-which-to-choose.md` (DQ6 closability).
- Run log: `local-benchmark-results.md` Run 149.
