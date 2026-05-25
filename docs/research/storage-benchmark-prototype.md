# Storage Benchmark Prototype (Runnable)

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This is the runnable realization of
[Observability storage benchmark plan](observability-storage-benchmark-plan.md).
The plan says *what* to measure and *why*; this document specifies a concrete
harness someone can build and run to compare storage candidates against the
Parallax goal, and it has **veto power** over the default storage choice: no
storage winner is declared until this prototype runs against the latest stable
versions.

It is opinionated and concrete on purpose — Rust harness, a `StorageAdapter`
trait so candidates swap behind one interface, a deterministic dataset generator,
per-candidate DDL, the exact evidence-bundle/correlation queries, and the
measurement protocol for each metric.

Pinned candidate versions for the first run (update at run time per the
version-freshness rule):

- GreptimeDB `v1.0.2` (GA, 2026-05-14) — standalone, then object-storage mode.
- ClickHouse latest stable (pin the exact version in results; e.g. `25.x`).
- MinIO (S3-compatible) for the object-storage cost path.

## Harness Architecture

One Rust binary, `parallax-bench`, with four parts behind a storage abstraction:

```text
parallax-bench
  ├── gen        deterministic dataset generator (seeded)
  ├── load       ingest driver (write-only / mixed)
  ├── query      workload runner (evidence-bundle + correlation queries)
  └── report     metrics recorder -> results.json + comparison table
StorageAdapter (trait)
  ├── GreptimeAdapter   (SQL over MySQL/PG wire or HTTP; OTLP ingest)
  └── ClickHouseAdapter (clickhouse-rs / HTTP)
```

The candidate sits behind one trait so the generator, workload, and metrics are
identical across candidates. Only the adapter and the DDL differ.

```rust
/// One row of generated telemetry, signal-tagged.
pub enum Signal {
    ErrorEvent(ErrorEvent),
    Span(Span),
    Log(LogRecord),
    Metric(MetricPoint),
    Deploy(DeployMarker),
    CliInvocation(CliInvocation),
    AgentAction(AgentAction),
    FrontendEvent(FrontendEvent), // session/breadcrumb/frontend error
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    async fn create_schema(&self) -> Result<()>;            // run candidate DDL
    async fn ingest(&self, batch: &[Signal]) -> Result<IngestAck>; // batched write
    async fn run_query(&self, q: &BenchQuery) -> Result<QueryResult>;
    async fn flush(&self) -> Result<()>;                    // force visibility/compaction where applicable
    async fn retained_bytes(&self) -> Result<StorageFootprint>; // on-disk + object-store
}

/// Named, dialect-specific query the runner times.
pub struct BenchQuery { pub id: &'static str, pub class: QueryClass, pub sql: String, pub params: Params }
pub enum QueryClass { TraceContext, IssueContext, ReleaseRegression, CrossTier, HighCardinality, Bundle }
```

`run_query` returns rows + server-reported timing where available; the harness
also measures wall-clock client-side. Each adapter ships the dialect SQL for the
shared `QueryClass` set below.

## Dataset Generator

Deterministic (seeded) so runs are reproducible and candidates see identical
data. Config (TOML):

```toml
[dataset]
seed = 42
tier = "small"                 # smoke | small | medium | stress
duration_hours = 720           # wall-clock window the data spans (e.g. 30 days)
services = 12
frontend_apps = 3

[mix]                          # relative volume per signal
spans_per_trace      = 14
traces_per_min       = 900
logs_per_span        = 3
metrics_series       = 40000   # cardinality of metric series
error_rate           = 0.012   # fraction of traces that emit an error_event
deploys_per_day      = 6
ci_runs_per_day      = 400
cli_invocations_per_day = 5000
agent_sessions_per_day  = 300
frontend_sessions_per_min = 600

[cardinality]
users        = 250000          # high-cardinality attribute
tenants      = 1200
commits      = 8000

[linkage]                      # makes correlation queries meaningful
frontend_to_backend_trace = 0.85  # frontend events that propagate traceparent into backend
error_in_span             = 0.97  # error events that carry trace_id/span_id
regression_releases       = 0.10  # releases that introduce a new error fingerprint
```

Tiers map to the plan's test matrix: `smoke` 1–5 GB (laptop), `small` 25–50 GB,
`medium` 250–500 GB, `stress` 1 TB+. The generator must produce **joinable**
data: a frontend error shares a `trace_id` that continues into backend spans;
errors carry `span_id`; a fraction of releases introduce a brand-new fingerprint
so the release-regression query has signal.

Generation is streamed (not held in RAM) so `stress` is feasible; the same
stream feeds `load` live for the mixed ingest+query phase.

## Per-Candidate Schema (DDL)

### GreptimeDB

```sql
-- error events (time-series table; ts is the time index, tags are indexed)
CREATE TABLE error_events (
  ts            TIMESTAMP TIME INDEX,
  project       STRING,
  environment   STRING,
  release       STRING,
  fingerprint   STRING,
  error_type    STRING,
  message       STRING,
  trace_id      STRING,
  span_id       STRING,
  panic_location STRING,
  handled       BOOLEAN,
  PRIMARY KEY (project, fingerprint)   -- tag columns
);

CREATE TABLE spans (
  ts          TIMESTAMP TIME INDEX,
  trace_id    STRING,
  span_id     STRING,
  parent_span_id STRING,
  service     STRING,
  name        STRING,
  duration_ms DOUBLE,
  status      STRING,
  attributes  JSON,
  PRIMARY KEY (service, name)
);

CREATE TABLE logs (
  ts        TIMESTAMP TIME INDEX,
  service   STRING, level STRING, message STRING,
  trace_id  STRING, span_id STRING, attributes JSON,
  PRIMARY KEY (service, level)
);

-- metrics via OTLP/Prometheus remote write (GreptimeDB auto-creates per-metric tables);
-- deploys, cli_invocations, agent_actions, frontend_events follow the same shape.
```

Run GreptimeDB twice: local-disk standalone, then with object storage
(`[storage] type = "S3"` against MinIO) to capture the cost/freshness tradeoff.

### ClickHouse

```sql
CREATE TABLE error_events (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  project LowCardinality(String),
  environment LowCardinality(String),
  release LowCardinality(String),
  fingerprint String,
  error_type LowCardinality(String),
  message String CODEC(ZSTD),
  trace_id String, span_id String,
  panic_location String, handled UInt8
) ENGINE = MergeTree
ORDER BY (project, fingerprint, ts)
TTL toDateTime(ts) + INTERVAL 90 DAY;

CREATE TABLE spans (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  trace_id String, span_id String, parent_span_id String,
  service LowCardinality(String), name LowCardinality(String),
  duration_ms Float64, status LowCardinality(String),
  attributes Map(String, String)
) ENGINE = MergeTree ORDER BY (trace_id, ts);

CREATE TABLE logs (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  service LowCardinality(String), level LowCardinality(String),
  message String CODEC(ZSTD), trace_id String, span_id String,
  attributes Map(String,String)
) ENGINE = MergeTree ORDER BY (service, ts);
-- metrics: one table keyed (metric, labels-hash, ts); object storage via S3 disk + storage_policy.
```

ClickHouse object-storage run uses an `s3` disk + `storage_policy` against MinIO,
to compare retention cost on equal footing with GreptimeDB's S3 mode.

## Query Workload (Exact, Both Dialects)

The runner times these named queries (the plan's evidence-bundle list, made
concrete). Each has a GreptimeDB and a ClickHouse form returning the same rows.

**Q1 `trace_context` (TraceContext):** given `trace_id`, fetch spans + same-trace
logs + the error event.

```sql
-- both dialects (ANSI-ish); times the join that builds a trace bundle
SELECT 'span' AS kind, span_id, name, duration_ms, status, NULL AS message
  FROM spans WHERE trace_id = :tid
UNION ALL
SELECT 'log', span_id, NULL, NULL, level, message
  FROM logs WHERE trace_id = :tid
UNION ALL
SELECT 'error', span_id, error_type, NULL, NULL, message
  FROM error_events WHERE trace_id = :tid;
```

**Q2 `issue_context` (IssueContext):** given `fingerprint`, last N events + first/
last seen + count.

```sql
SELECT min(ts) first_seen, max(ts) last_seen, count(*) n
  FROM error_events WHERE project = :proj AND fingerprint = :fp;
```

**Q3 `release_regression` (ReleaseRegression):** fingerprints present in release R
but absent in the prior release window (the core "what changed" query).

```sql
SELECT fingerprint FROM error_events
 WHERE project=:proj AND release=:rel
   AND fingerprint NOT IN (
     SELECT fingerprint FROM error_events
      WHERE project=:proj AND release=:prev_rel)
 GROUP BY fingerprint;
```

**Q4 `cross_tier` (CrossTier):** given a frontend `trace_id`, follow propagated
context into backend spans/errors — the frontend↔backend reconstruction.

```sql
SELECT s.service, s.name, s.duration_ms, e.error_type, e.message
  FROM spans s
  LEFT JOIN error_events e ON e.trace_id = s.trace_id AND e.span_id = s.span_id
 WHERE s.trace_id = :tid           -- trace originates in the frontend session
 ORDER BY s.ts;
```

**Q5 `high_cardinality` (HighCardinality):** filter spans by a high-cardinality
attribute (`user`/`tenant`) over a window — degradation probe.

**Q6 `bundle` (Bundle):** the composite — run Q1+Q2+Q3 for one anchor and
assemble the [evidence bundle](evidence-bundle-and-schema.md); time end-to-end.

## Metrics And How They Are Measured

Tied to the plan's priority axes (speed > cost > scaling):

| Metric | Axis | Measurement protocol |
| --- | --- | --- |
| Ingest-to-queryable freshness | Speed (#1) | At ingest, stamp `t_emit`. Immediately poll a point query for that row every 50 ms until it returns; record `t_visible - t_emit`. Report p50/p95/p99 under write-only and under mixed load. |
| Evidence-bundle / correlation query latency | Speed | Client wall-clock per `QueryClass`, p50/p95/p99, warm and cold cache. Cold = restart candidate + drop OS page cache before run. |
| Concurrent ingest+query | Speed | Run `load` at target rate while `query` runs Q1–Q6 in parallel; report query latency delta vs query-only. |
| Retained size | Cost (#2) | `retained_bytes()` after `flush()`/compaction: on-disk bytes and object-store bytes per tier. |
| Compression ratio by signal | Cost | Generated raw bytes per signal ÷ retained bytes for that signal's table. |
| Object-store request/egress | Cost | Count S3 GET/PUT/LIST against MinIO during ingest and during cold-cache queries; model $ at S3 list prices. |
| Compute per ingested GB / per query class | Cost | Sample candidate process CPU+RSS during each phase. |
| Single-node ceiling | Scaling (#3) | Increase tier until freshness p95 or query p95 breaches gate; record the breaking rate/size. |
| Scale-out behavior | Scaling | Repeat medium tier on distributed GreptimeDB / ClickHouse cluster; check whether p95 holds as nodes are added. |

All raw numbers land in `results.json`; `report` renders the comparison table.

## Run Procedure

```bash
# 0. bring up candidates + object store
docker compose -f bench/compose.yml up -d   # greptimedb, clickhouse, minio

# 1. generate (streams to disk manifest; deterministic by seed)
parallax-bench gen --config bench/small.toml --out data/small/

# 2. per candidate: schema -> load -> flush -> query phases
for c in greptime clickhouse greptime-s3 clickhouse-s3; do
  parallax-bench load  --adapter $c --data data/small/ --mode write-only
  parallax-bench query --adapter $c --classes all --cache cold
  parallax-bench query --adapter $c --classes all --cache warm
  parallax-bench load  --adapter $c --data data/small/ --mode mixed --with-query
  parallax-bench report --adapter $c --out results/$c.small.json
done

# 3. compare
parallax-bench report --compare results/*.small.json --table
```

Each tier (`smoke`/`small`/`medium`/`stress`) is a separate config; `smoke` must
run on a laptop and is the correctness gate before larger tiers.

## Decision Gates (Initial Targets — Calibrate On Smoke)

Concrete pass targets so the benchmark can actually pick a winner. These are
starting targets to refine after the smoke run, not laws:

| Gate | Target |
| --- | --- |
| Freshness p95 (mixed load, small tier) | ≤ 5 s ingest-to-queryable |
| Evidence-bundle Q6 p95 (warm, small) | ≤ 300 ms |
| Trace-context Q1 p95 (warm, small) | ≤ 100 ms |
| Retained size + object-store cost (30-day small) | GreptimeDB within ~1.2× of ClickHouse or cheaper |
| Concurrent penalty | query p95 under mixed ≤ 2× query-only p95 |
| Operability | single binary / single service, smoke setup ≤ 15 min |

GreptimeDB becomes the confirmed default if it meets the speed gates and wins or
ties on cost/operability; ClickHouse takes the default only if GreptimeDB breaches
a speed gate or loses badly on cost (the plan's decision criteria, now numeric).

## Results Template

```text
candidate | tier  | fresh_p95 | Q1_p95 | Q3_p95 | Q6_p95 | size_GB | obj_cost_$/mo | cpu_ingest | verdict
----------|-------|-----------|--------|--------|--------|---------|---------------|------------|--------
greptime  | small |           |        |        |        |         |               |            |
greptime-s3|small |           |        |        |        |         |               |            |
clickhouse| small |           |        |        |        |         |               |            |
```

## Reproducibility

- Pin candidate versions, seed, and hardware in every results file.
- Same generated dataset for all candidates (generate once, load into each).
- Run smoke for correctness (row counts and query results must match across
  candidates within tolerance) before trusting any latency number.
- Re-run when a candidate ships a new stable version; label older results
  historical per the version-freshness rule.

## Relationship To Other Research

- [Observability storage benchmark plan](observability-storage-benchmark-plan.md)
  — the rationale, scope rule, axes, and decision criteria this prototype runs.
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md) — candidate
  architecture detail.
- [Metadata store benchmark plan](metadata-store-benchmark-plan.md) — the Turso
  vs Postgres harness, to be made runnable the same way.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  — Q6 assembles this object; the query workload exists to serve it.

## Bottom Line

This prototype turns the storage question from opinion into measurement: one
seeded dataset, one query set that mirrors real evidence-bundle assembly, one
trait so candidates swap cleanly, and numeric gates that decide the default. Build
the smoke tier first, prove correctness, then let the numbers — not incumbency or
architecture aesthetics — choose the storage engine.
