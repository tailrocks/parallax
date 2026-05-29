# Storage Benchmark Plan, Prototype, and Artifact Interpretation

> The database-only benchmark for observability storage compares two primary candidates — GreptimeDB and ClickHouse — against Parallax-shaped evidence-bundle workloads, ranked speed > cost > scaling, with GreptimeDB becoming the preferred backend only if it meets the numeric speed gates and wins or ties on cost/operability. The runnable Rust `parallax-bench` prototype (a `StorageAdapter` trait, a seeded deterministic dataset generator, per-candidate DDL, exact Q1–Q6 evidence-bundle/correlation queries, a measurement protocol, and numeric decision gates) realizes that plan and holds **veto power** over the default storage choice: no winner is declared until it runs against the latest stable versions (pinned for the first run as GreptimeDB `v1.0.2`, ClickHouse feature-stable `v26.5.1.882-stable`, and ClickHouse LTS `v26.3.12.3-lts`, plus MinIO). The separate benchmark agent's current `bench/four-way/` artifacts (Runs 140-158) make the four-build benchmark reproducible, validate a local (`N=100000`) vs server (`N=5000000+`) tier split, tighten schema guidance, and source-ground GreptimeDB TWCS/TTL, raft-engine WAL durability, and PartitionTree high-cardinality ingest mechanisms. Those runs prove GreptimeDB's anchored/keyed hot path stays interactive while its heavy analytics cross the 300 ms gate at 5M — confirming ClickHouse as the analytics-heavy fallback — but they do not measure mixed native ingest, Q6 p95/p99, stale-bundle rate, crash/restart loss, high-cardinality native metric memory/flush behavior, object-store economics, ClickHouse LTS, or end-to-end A5 integration. They therefore count as `smoke_only` storage and schema evidence, not a `greptime_prototype_default`, `clickhouse_storage_default`, `dual_storage_open`, or `phase1_stack_pass` result. The open gate remains: run the full mixed-load Q6 freshness gate and the object-store cost gate before turning any number into a storage default.

This note consolidates the following previously-separate research files, each preserved in full below:

- `observability-storage-benchmark-plan.md`
- `storage-benchmark-prototype.md`
- `storage-benchmark-artifact-interpretation.md`

## Observability Storage Benchmark Plan

_Provenance: merged verbatim from `observability-storage-benchmark-plan.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-11

### Purpose

Parallax needs a storage benchmark that answers a narrow technical question:

> Which self-hostable database is the best fit for storing and querying metrics,
> logs, traces, and Parallax failure-context events together?

This is not a platform comparison. Observability platforms combine storage,
collectors, pipelines, dashboards, alerts, user management, and product UX. That
is a different category. For Parallax, the hard dependency is the database or
storage engine underneath the platform.

> **Runnable prototype:** this document is the rationale, scope, axes, and
> decision criteria. The concrete, runnable harness that realizes it — Rust
> `parallax-bench`, the `StorageAdapter` trait, the seeded dataset generator,
> per-candidate DDL, the exact query SQL, the measurement protocol, and numeric
> decision gates — lives in
> [Storage benchmark prototype (runnable)](storage-benchmark-prototype.md). The
> benchmark prototype has veto power over the default storage choice. The first
> mixed-load speed gate is specified in
> [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
> The first retained-size and object-cost gate is specified in
> [Storage size and object cost gate](storage-size-and-object-cost-gate.md).

### Scope Rule

Benchmark databases, not platforms.

Included:

- databases and storage engines that Parallax can self-host or ask users to
  self-host;
- systems where we can reason about data layout, query path, ingestion path,
  retention, compression, indexing, and operational cost;
- systems that can plausibly store metrics, logs, traces, and Parallax events
  in one backend.

Excluded:

- full observability platforms as benchmark targets;
- SaaS-only systems;
- products whose value is mostly UI, alerting, dashboards, or incident workflow;
- systems that require separate databases for metrics, logs, and traces.

Platform stacks may still be useful as references. For example, ClickStack can
inform the ClickHouse schema and OpenTelemetry ingestion path, but the benchmark
target is ClickHouse, not ClickStack.

### Language and Runtime Filter

Candidates are constrained by language and runtime before anything else. Only
high-performance, low-resource systems languages are in scope: Rust, Go, Zig,
C++, and C. These are compiled and lean on memory and startup, which is the
operational profile Parallax needs for cheap, predictable self-hosting.

Heavyweight managed or interpreted runtimes are excluded outright — Java/JVM,
Python, Ruby, PHP, and similar. This is why systems such as Elasticsearch,
OpenSearch, QuestDB, Apache Doris, Apache Pinot, Kafka, and Pulsar are removed as
candidates: the JVM profile (heap tuning, GC pauses, fat runtime) and
interpreted-runtime overhead are the operational weight Parallax is trying to
escape — the same weight that makes self-hosted Sentry painful. Prefer Rust; when
two candidates are close, Rust wins.

### Priority Axes

Rank candidates against Parallax's purpose, not a general leaderboard, in this
order:

Version freshness rule: compare the latest reasonably available stable/public
version of every candidate as of the research date. Do not use stale benchmarks,
old major releases, or outdated feature matrices as current evidence unless they
are explicitly labeled historical. When versions materially affect the result,
record the exact version or release date in the benchmark notes.

1. **Speed — time to see real data.** When a production error fires, how fast can
   we pull everything correlated to that moment (metrics, traces, spans, logs)?
   Measure ingest-to-queryable latency (freshness) and evidence-bundle query
   latency under concurrent ingest plus query. Lower delay means richer AI
   context.
2. **Cost — storage size and money.** Retained size per unit telemetry,
   compression by signal, object-vs-block cost, compute, and retention math. Flag
   and quantify any candidate that is fast but storage-heavy or expensive.
3. **Scaling — horizontal first.** Single-node ceiling, scale-out difficulty, and
   whether performance holds when adding parallel servers. Horizontal scaling
   matters most; vertical scaling is secondary; vertical-only is a flagged
   limitation.

### Candidate Classes

#### Primary Candidates

| Candidate | Why included | Main hypothesis |
| --- | --- | --- |
| GreptimeDB | Purpose-built open-source observability database for metrics, logs, traces, events, PromQL, SQL, OTLP, and object-storage-oriented deployment. | Best conceptual fit for Parallax because observability semantics live in the database. |
| ClickHouse | Mature open-source columnar OLAP database with excellent event/log/trace performance and a large observability ecosystem. | Strongest mature baseline; may win raw analytical performance and ecosystem reliability. |

#### Watch List

| Candidate | Why not primary |
| --- | --- |
| Parseable | Interesting object-store-first observability datalake, but currently more platform-shaped than database-shaped for Parallax's storage dependency. Revisit only if it exposes a clean database-level interface and proves maturity. |
| OpenSearch / Elasticsearch | Excluded by the language/runtime filter (Java/JVM). Also slow for high-volume observability and not metrics-native. Keep only as a UI/UX reference for showing a log as a structured object (Kibana), never as a storage candidate. |
| VictoriaMetrics family | Strong metrics/logs/traces projects, but they are separate specialized databases rather than one unified storage engine. Useful as per-signal reference, not as Parallax's unified backend. |
| Grafana LGTM | Mature observability stack, but intentionally split across Loki, Mimir, Tempo, and other components. Useful as an ecosystem reference, not a database candidate. |
| InfluxDB / TimescaleDB / M3DB | Strong time-series or metrics heritage, but weaker fit for unified logs/traces/metrics evidence bundles. |

### What We Need To Prove

The benchmark must test the claims that matter for Parallax:

1. GreptimeDB is observability-native enough to reduce schema and operational
   complexity versus ClickHouse.
2. ClickHouse's mature columnar engine is or is not better for Parallax's actual
   query patterns.
3. GreptimeDB's object-storage-first architecture materially improves retention
   cost without hurting evidence-bundle query latency too much.
4. PromQL and metrics semantics work better in GreptimeDB than in ClickHouse for
   real Parallax usage.
5. Logs and traces are fast enough in GreptimeDB despite ClickHouse's strong
   track record on event analytics.
6. Either database can be self-hosted by a small team without turning Parallax
   into an observability-operations project.

### Candidate Dataset

The benchmark should use synthetic-but-realistic data, then later replay real
open-source project CI data when available.

#### Telemetry Signals

- OpenTelemetry traces:
  - trace ID, span ID, parent span ID;
  - service name, route, operation, span kind, status;
  - duration, start/end timestamp;
  - exception attributes;
  - high-cardinality attributes such as user, tenant, commit, branch, job ID.
- Structured logs:
  - timestamp, severity, service, message;
  - trace ID and span ID when available;
  - error class, stack trace fingerprint, test/run metadata;
  - JSON attributes.
- Prometheus-style metrics:
  - request count, latency histogram, error count;
  - CPU, memory, disk, queue depth;
  - CI worker health;
  - test duration and retry counters.
- Parallax events:
  - repository, commit SHA, branch;
  - CI provider, run ID, job ID;
  - test suite, test name, status, duration;
  - retry count, failure signature, log excerpt pointer;
  - first-seen and previous occurrence windows.
- CLI invocation traces:
  - command, subcommand, sanitized args/env, cwd, repo, branch, commit;
  - stdout/stderr excerpt refs, exit code/signal, panic/error chain;
  - spawned process spans and test/build/deploy phase markers.
- Coding-agent traces:
  - agent product/version, model/provider, context bundle ID;
  - tool calls, shell commands, files read/written, patch refs;
  - validation commands, PR/proposal refs, human review and outcome state.

### Query Workload

The benchmark should prioritize evidence-bundle queries, not generic leaderboard
queries.

1. Given a failing test, fetch related logs in the same CI run and time window.
2. Given a failing test, find previous failures with the same fingerprint.
3. Given a CI run, group related failures by signature and service/test owner.
4. Given a trace ID, fetch spans, correlated logs, and metric deltas.
5. Given a service and 30-minute window, find error-rate and latency changes,
   then fetch matching error logs and traces.
6. Given a commit SHA, compare failure fingerprints before and after the commit.
7. Given a high-cardinality attribute, measure query degradation.
8. Given a time range, build an agent-ready context bundle with bounded raw
   evidence excerpts.
9. Given an agent session, fetch context bundle refs, tool calls, command spans,
   file changes, validation commands, patch refs, and outcome.
10. Given a CLI invocation, fetch stdout/stderr excerpts, child process spans,
    repo state, related CI run, and linked runtime issue.
11. Given a runtime issue, find agent sessions and CLI commands that attempted a
    fix and whether the issue recurred after deploy.

### Benchmark Dimensions

#### Performance

- ingest throughput;
- p50/p95/p99 query latency;
- concurrent ingest plus query behavior;
- cold-cache query latency;
- hot-cache query latency;
- high-cardinality query behavior;
- batch backfill performance.

#### Cost Shape

- retained size on disk or object storage;
- compression ratio by signal type;
- CPU and memory per ingested GB;
- CPU and memory per query class;
- local SSD requirement;
- object storage request cost and egress sensitivity;
- compaction/indexing overhead.

#### Operability

- number of required services;
- local single-node setup time;
- Kubernetes setup complexity;
- backup/restore story;
- schema evolution behavior;
- retention and downsampling;
- authentication and multi-tenancy available in OSS;
- observability of the database itself;
- failure recovery under node restart or disk pressure.

#### Product Fit

- OpenTelemetry ingestion quality;
- Prometheus remote write/read quality;
- PromQL completeness for Parallax needs;
- SQL ergonomics for cross-signal joins;
- support for dynamic attributes;
- support for trace/log correlation;
- ease of exporting portable evidence bundles.

### Test Matrix

| Tier | Purpose | Approximate data size | Notes |
| --- | --- | --- | --- |
| Local smoke | Validate setup and query correctness. | 1-5 GB | Must run on a developer laptop. |
| Small realistic | Early product workload. | 25-50 GB | One repository or service over several weeks. |
| Medium | Serious self-hosted team. | 250-500 GB | Enough to expose retention, indexing, and cardinality behavior. |
| Stress | Architecture pressure test. | 1 TB+ | Run only after schema stabilizes. |

Each tier should run with:

- warm cache;
- cold cache;
- write-only ingest;
- query-only;
- mixed ingest/query;
- local disk where supported;
- object storage or MinIO where supported.

### Candidate-Specific Notes

#### GreptimeDB

Validate:

- direct OTLP ingestion;
- Prometheus remote write and PromQL;
- dynamic schema from OTLP attributes;
- trace support maturity;
- object storage performance;
- whether OSS features are enough for a credible self-hosted deployment.

Risk:

- younger ecosystem;
- traces are still young;
- public benchmarks are mostly vendor-published.

#### ClickHouse

Validate:

- raw logs/traces/events performance;
- OTel schema design;
- materialized views for metric rollups;
- Prometheus compatibility path;
- object-storage options for self-hosting;
- operational complexity without relying on managed ClickHouse Cloud.

Risk:

- metrics semantics may require more application-layer work;
- schema and pipeline choices may dominate performance;
- ClickStack improves the story, but ClickStack is a platform wrapper, not the
  database itself.

### Decision Criteria

GreptimeDB should become the preferred backend only if it proves:

- query performance is close enough to or better than ClickHouse on
  Parallax-shaped workloads;
- metrics and PromQL workflows are materially better than ClickHouse;
- storage cost and operational simplicity are meaningfully better;
- logs and traces are fast enough for evidence-bundle workflows;
- OSS self-hosting is viable without needing enterprise-only features too early.

ClickHouse should remain preferred if:

- GreptimeDB does not materially simplify cross-signal workflows;
- ClickHouse wins clearly on logs/traces and Parallax metrics needs are modest;
- GreptimeDB trace support or operational maturity becomes a blocker;
- ClickHouse schemas plus small Parallax abstractions are enough.

### Current Benchmark Shortlist

1. GreptimeDB.
2. ClickHouse.

Everything else is watch-list or sanity-check material until it proves it is a
database-level alternative rather than an observability platform.

## Storage Benchmark Prototype (Runnable)

_Provenance: merged verbatim from `storage-benchmark-prototype.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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

The focused proof gate for mixed-load freshness and Q6 bundle latency is
[Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
Treat that gate as the first storage-speed pass before using smoke-run timings
as architectural evidence.

The focused proof gate for retained size, per-signal compression, object-store
requests, cache dependency, and provider cost projection is
[Storage size and object cost gate](storage-size-and-object-cost-gate.md).
Treat that gate as the first storage-cost pass before quoting retention numbers
externally.

The storage result does not, by itself, prove assumption A5. The
[A5 stack decision ledger](a5-stack-decision-ledger.md) decides when storage
speed/cost rows can roll up with metadata, ingest-log, setup, and integration
rows into a stack-default claim.

The current checked-in `bench/four-way/` harness is a useful preliminary local
artifact, not this full prototype. See
[Storage benchmark artifact interpretation](storage-benchmark-artifact-interpretation.md):
Runs 140-147 store the four-build benchmark as code, define and validate the
local/server tier split, tighten schema guidance, and source-read GreptimeDB
TWCS/TTL, raft-engine WAL durability, and PartitionTree high-cardinality ingest
mechanisms, but they do not measure mixed native ingest, Q6 p95/p99,
stale-bundle rate, crash/restart loss counts, high-cardinality native metric
memory/flush behavior, object-store economics, ClickHouse LTS, or end-to-end A5
integration. Local `bench/four-way` defaults to `N=100000` after Run 143 and
Run 145 validates that default as laptop-safe; `N=5000000+` belongs on a server
and only when the operator explicitly asks.

Pinned candidate versions for the first run (update at run time per the
version-freshness rule):

- GreptimeDB `v1.0.2` (GA, 2026-05-14) — standalone, then object-storage mode.
- ClickHouse feature-stable `v26.5.1.882-stable` (published 2026-05-21) as the
  newest stable feature train checked.
- ClickHouse LTS `v26.3.12.3-lts` (published 2026-05-22) as the conservative
  operations baseline checked.
- MinIO (S3-compatible) for the object-storage cost path.

ClickHouse docs recommend `stable` by default and `lts` for teams that cannot
upgrade frequently or are using simpler secondary workloads. Because Parallax is
choosing a storage default, not just a leaderboard winner, the first serious
benchmark should either run both ClickHouse tracks or explicitly justify why one
track is excluded. A result that says only "latest ClickHouse" expires as soon
as either track moves.

Current release/version sources:

- [ClickHouse v26.5.1.882-stable release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable)
- [ClickHouse v26.3.12.3-lts release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts)
- [ClickHouse production version guidance](https://clickhouse.com/docs/faq/operations/production#how-to-choose-between-clickhouse-releases)

### Harness Architecture

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

### Dataset Generator

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

### Per-Candidate Schema (DDL)

#### GreptimeDB

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

#### ClickHouse

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

### Query Workload (Exact, Both Dialects)

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

### Metrics And How They Are Measured

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

### Run Procedure

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

### Decision Gates (Initial Targets — Calibrate On Smoke)

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

### Results Template

```text
candidate | tier  | fresh_p95 | Q1_p95 | Q3_p95 | Q6_p95 | size_GB | obj_cost_$/mo | cpu_ingest | verdict
----------|-------|-----------|--------|--------|--------|---------|---------------|------------|--------
greptime  | small |           |        |        |        |         |               |            |
greptime-s3|small |           |        |        |        |         |               |            |
clickhouse| small |           |        |        |        |         |               |            |
```

### Reproducibility

- Pin candidate versions, seed, and hardware in every results file.
- Same generated dataset for all candidates (generate once, load into each).
- Run smoke for correctness (row counts and query results must match across
  candidates within tolerance) before trusting any latency number.
- Re-run when a candidate ships a new stable version; label older results
  historical per the version-freshness rule.

### Relationship To Other Research

- [Observability storage benchmark plan](observability-storage-benchmark-plan.md)
  — the rationale, scope rule, axes, and decision criteria this prototype runs.
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md) — candidate
  architecture detail.
- [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md) — the Turso
  vs Postgres harness, to be made runnable the same way.
- [Storage benchmark artifact interpretation](storage-benchmark-artifact-interpretation.md)
  — how to read the current `bench/four-way` local artifacts without promoting
  them to an A5 storage-default result.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  — Q6 assembles this object; the query workload exists to serve it.

### Bottom Line

This prototype turns the storage question from opinion into measurement: one
seeded dataset, one query set that mirrors real evidence-bundle assembly, one
trait so candidates swap cleanly, and numeric gates that decide the default. Build
the smoke tier first, prove correctness, then let the numbers — not incumbency or
architecture aesthetics — choose the storage engine.

## Storage Benchmark Artifact Interpretation

_Provenance: merged verbatim from `storage-benchmark-artifact-interpretation.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Pass Target

Consume the separate benchmark agent's new artifacts without running another
storage benchmark. The goal is to decide what Runs 140-158 prove, what they
falsify, which source-read mechanism claims they strengthen, and which
product/storage claims must stay unproven until the full storage gates run.

### Artifacts Checked

| Artifact | Evidence class | What was inspected |
| --- | --- | --- |
| Commit `19e9604` | Reproducible local benchmark code + Run 140 docs | `bench/four-way/`, 1M-row default, `N >= 50000` enforcement, four builds, 20-query matrix. |
| Commit `1728da7` | Local 5M scale run interpretation | Run 141 docs showing anchored hot path holds while GreptimeDB heavy analytics cross the 300 ms gate. |
| Commit `ead9482` | Local A/B isolation | Run 142 docs showing GreptimeDB dedup aggregation vs append-mode aggregation at 5M. |
| Commit `f3a4023` | Benchmark tier policy + Run 143 docs | Local laptop default lowered to `N=100000`, 5M+ runs moved to explicit server tier, and forced compaction reduced the 5M dedup aggregation penalty. |
| Commit `20140c2` | Primary-source code read + Run 144 docs | GreptimeDB `v1.0.2` TWCS picker, window picker, and compactor source: compaction is time-window scoped and expired SSTs are removed separately from successful merges. |
| Commit `a6107e3` | Consolidation note | The storage verdict's DQ6 section now carries the 5M `v1.1.0-nightly-20260525` dedup-aggregation regression instead of preserving the stale "no regressions" wording from the 1M tier. |
| Commit `5d97084` | Local 100k preliminary validation | Run 145 docs showing the new `N=100000` local default completes without freezing the laptop and keeps all 20 four-way queries interactive, but compresses gaps enough that it is directional only. |
| Commit `72c6498` | Primary-source code read + Run 146 docs | GreptimeDB `v1.0.2` raft-engine WAL source: appends entries as `LogBatch`, forwards `sync_write` into the raft-engine write call, and runs a periodic sync task over `engine.sync()`. |
| Commit `0926606` | Primary-source code read + Run 147 docs | GreptimeDB `v1.0.2` PartitionTree memtable source: primary-key dictionary, shard builder, per-shard key indexing, and memory-budgeted `fork_dictionary_bytes`; ClickHouse `LowCardinality` source setting caps shared dictionaries at 8192 rows before ordinary encoding. |
| `bench/four-way/gen.sh` | Reproducibility source | Generates six logical tables in-engine across all four builds; now defaults `N=100000`; rejects `N < 50000`; flushes GreptimeDB tables. |
| `bench/four-way/bench.sh` | Reproducibility source | Runs the 20-query matrix; `REPS` defaults to 6 and must be recorded per run when docs cite medians. |
| GreptimeDB TWCS source | Primary source | `TwcsPicker` groups files into compaction windows by max timestamp; `WindowedCompactionPicker` splits strict-window compaction by file time spans; the compactor removes expired SSTs even when merge output fails. |
| GreptimeDB raft-engine WAL source | Primary source | `RaftEngineLogStore` owns `sync_write`, `sync_period`, and the raft-engine handle; writes convert entries into a `LogBatch` and call `engine.write(&mut batch, self.sync_write)`, while `SyncWalTaskFunction` calls `engine.sync()`. |
| GreptimeDB PartitionTree source | Primary source | `PartitionTreeMemtable` uses a partition tree with `dict`, `partition`, `shard`, and `shard_builder`; `KeyDictBuilder` maps encoded primary keys to compact key indexes; default dictionary memory is capped by `min(total_memory / 8, 512 MiB)`. |
| ClickHouse `LowCardinality` setting source | Primary source | `low_cardinality_max_dictionary_size` defaults to 8192 rows; values beyond the dictionary limit are written in the ordinary method. |
| Commit `3e50523` | Local plan re-verification | Run 154 re-runs the Q4 anchored cross-tier join gap and isolates it to GreptimeDB predicate-through-`LEFT JOIN` propagation: plain `trace_id` filtering prunes, but the direct join scans 1M rows; subquery prefilter or app-side correlation restores pruning. |
| Commit `b06d519` | Local object-storage interpretation under the Parallax proxy lens | Run 155 confirms ClickHouse can use S3/object-storage tiering, narrowing the raw cold-storage gap, while GreptimeDB's remaining edge is self-hosted 1x object storage, elastic compute/storage separation, and simpler self-hosted HA assumptions. |
| Commit `f5dbb57` | Local SQL capability recheck | Run 156 verifies both engines can compute Sentry-style grouped-error rollups and evidence-window ranking with SQL, so GreptimeDB is not capability-blocked for those product views; ClickHouse's build-on-top edge is ecosystem/maturity, not SQL expressibility. |
| Commit `0d63446` | Local 5M full-text mechanism recheck | Run 157 confirms the full-text mechanism: selective terms prune on both engines, with ClickHouse reading one 8192-row granule and GreptimeDB bloom reading about five 10240-row blocks; broad terms prune on neither and become scan-bound, favoring ClickHouse. |
| Commit `f630914` | Local anchored-bundle plan recheck | Run 158 verifies the dominant anchored evidence-bundle pillar: both engines prune hard when `trace_id` is keyed/indexed on the signal table, and both full-scan when it is not. It also corrects GreptimeDB plan reading: scan `output_rows` is post-filter emission, not rows-read; use `scan_cost`, `elapsed_poll`, and `file_ranges` for scan work. |
| GitHub release redirects and tag refs | Current release-track check | `releases/latest` still resolves GreptimeDB to `v1.0.2`; `git ls-remote` confirms `v1.1.0-nightly-20260525` is a pre-release tag. ClickHouse `releases/latest` currently resolves to LTS `v26.3.12.3-lts`, while the benchmarked feature-stable `v26.5.1.882-stable` tag still exists. |

### What The Artifacts Prove

1. **The benchmark is now reproducible enough to audit.** `bench/four-way/`
   stores the version matrix, table generation, data-size floor, and query set as
   code. This is a major improvement over ad hoc local numbers.
2. **Benchmark tier policy matters.** Local laptop runs are now a small but
   meaningful preliminary tier (`N=100000` default, `N >= 50000` enforced).
   Large `N=5000000+` runs are server-tier only and should be run only when the
   operator explicitly asks. Run 145 validates the local default operationally:
   generation finished in about 10 seconds, did not freeze the laptop, and left
   all 20 query shapes interactive across four builds.
3. **At 1M warm local rows, every measured query is interactive on all four
   builds.** ClickHouse is still faster on most scans/joins/JSON/log-tail shapes;
   GreptimeDB wins or ties last-value, selective full-text, and high-cardinality
   exact distinct. This supports "fit not speed" for the anchored/local warm
   tier, not a production default.
4. **At 100k warm local rows, the benchmark is safe but too compressed for
   magnitude claims.** Run 145 shows all 20 queries at 2-52 ms and nightlies
   roughly equal to stables. It confirms direction and harness health; it does
   not replace the 1M matrix or the 5M scale findings.
5. **At 5M, the distinction matters.** GreptimeDB's anchored/keyed hot path still
   holds: anchored lookup, last-value, and time-range reads remain interactive.
   GreptimeDB's heavy analytical queries cross or approach the 300 ms gate:
   metric aggregations, dynamic JSON, in-DB cross-tier join, and high-card
   distinct. This turns the DQ5 flip trigger from theory into measured local
   evidence: analytics-heavy usage favors ClickHouse.
6. **GreptimeDB table mode, compaction state, and TWCS window count are
   load-bearing.** Run 142 isolates dedup aggregation as roughly 8x slower than
   append-mode aggregation in the less-compacted 5M state; Run 143 shows forced
   compaction drops GreptimeDB stable's dedup aggregation from about 314 ms to
   about 60 ms, while append mode stays faster at about 40 ms and avoids
   compaction dependence. Run 144 makes the mechanism more precise: forced
   compaction can collapse within-window state, but a long-retention table still
   keeps at least one SST per TWCS window, so a dedup reader can still merge
   across windows. For scrape-style metrics where `(series, ts)` is already
   unique, append mode is still the safer load-bearing default; dedup/
   `last_non_null` belongs where partial upsert or out-of-order correction is
   actually needed.
7. **GreptimeDB `v1.1.0-nightly-20260525` is not a clean upgrade signal.** It is
   better on some append/scan paths but regresses the dedup aggregation path at
   5M. Run 143 makes the compaction-state sensitivity visible; Run 144 shows why
   many-window metric tables remain structurally different from a single compacted
   window; commit `a6107e3` carries that caveat into the storage verdict. The
   defensible future claim is "re-test v1.1 GA", not "v1.1 fixes GreptimeDB
   performance."
8. **GreptimeDB's cheap-retention claim is stronger than a benchmark number.**
   The Run 144 source read shows TWCS windows are the compaction boundary and the
   compactor includes expired SSTs in removals separately from successful merge
   outputs. That makes whole-SST TTL drop a structural GreptimeDB advantage,
   while object-store request counts and cold-read cost are still unmeasured for
   the full Parallax gate.
9. **GreptimeDB's strict-durability advantage is now source-grounded.** Run 75's
   measured delta said GreptimeDB `sync_write=true` was roughly 10x cheaper than
   ClickHouse per-part fsync on the local smoke path. Run 146 explains the
   mechanism in `v1.0.2`: GreptimeDB batches WAL entries into raft-engine
   `LogBatch` records, passes `sync_write` into `engine.write`, and also exposes
   `sync_period` as a periodic sync path. This supports the architectural claim
   that strict local durability fsyncs an append-log path rather than a
   multi-file part. It does not replace crash/restart testing or a mixed native
   ingest run.
10. **GreptimeDB's cardinality-insensitive ingest claim is now
    source-grounded.** Runs 84 and 101 measured GreptimeDB ingest as nearly flat
    as distinct metric series rose, while ClickHouse slowed on high-cardinality
    labels. Run 147 explains the GreptimeDB side: the PartitionTree memtable
    dictionary-encodes primary keys/label sets and writes rows through shard
    indexes rather than storing every label string per row. It also confirms the
    ClickHouse caveat from source: `LowCardinality` has a default shared
    dictionary cap of 8192 rows before ordinary encoding. This strengthens the
    metrics-ingest ergonomics thesis; it does not prove aggregation latency,
    native Prometheus/OTLP freshness, or memory pressure at production
    cardinality.
11. **The in-database `LEFT JOIN` gap is real but avoidable for Parallax's hot
    path.** Run 154 re-verifies that GreptimeDB's `trace_id` index prunes a
    plain anchored filter, but the same predicate does not push through the
    direct `LEFT JOIN` shape and the spans side scans 1M rows. That makes
    app-side correlation and subquery pre-filtering load-bearing design choices,
    not cosmetic implementation preferences.
12. **ClickHouse's object-storage story is stronger than the old local-disk
    comparison implied.** Run 155 shows ClickHouse can use S3/object-storage
    tiering, so "GreptimeDB is cheaper because ClickHouse cannot use object
    storage" is false. The remaining GreptimeDB argument is narrower:
    self-hosted 1x object-store economics, elastic compute/storage separation,
    fewer always-on HA assumptions, and operational fit.
13. **Grouped errors are not a GreptimeDB SQL capability blocker.** Run 156
    verifies both engines can compute Sentry-style grouped-error rollups and
    evidence-window ranking. The relevant ClickHouse edge is the ecosystem that
    already builds observability platforms on it, not a unique ability to express
    Parallax's grouped-error queries.
14. **Full-text findings survived a 5M plan recheck.** Run 157 confirms the
    corrected interpretation: selective full-text is effectively a tie with a
    small ClickHouse granularity edge, while broad terms remain a
    ClickHouse-shaped scan workload.
15. **The anchored-bundle pillar is conditional on schema, not engine magic.**
    Run 158 re-verifies the dominant query shape: keyed/indexed `trace_id`
    fetches prune on both engines, while un-keyed `trace_id` filters full-scan
    on both engines. Parallax controls that condition, so the schema blueprint
    and result rows must prove `trace_id` or `fingerprint` anchoring on every
    signal table used by bundles.
16. **GreptimeDB plan reading needs a stricter evidence rule.** Run 158 shows
    GreptimeDB scan-node `output_rows` can mean rows emitted after a pushed
    filter, not rows read. Future mechanism claims must use `scan_cost`,
    `elapsed_poll`, and `file_ranges` for scan work; otherwise a full scan with
    one emitted row can be misread as a pruned scan.

### What The Artifacts Do Not Prove

These runs do **not** satisfy the storage benchmark prototype, storage freshness
gate, storage cost gate, or A5 stack decision ledger:

- no 25-50 GB small-tier dataset;
- no cold-cache/drop-cache comparison under the full Q1-Q6 bundle workload;
- no native OTLP/Prometheus ingest path comparison for queryable freshness;
- no mixed ingest+query p95/p99, stale bundle rate, or Q6 p95 result rows;
- no high-cardinality native metric ingest run that records distinct series,
  PartitionTree dictionary memory, flush pressure, compaction behavior, and
  query p95/p99 under concurrent bundle reads;
- no storage durability fault test showing acknowledged rows survive
  crash/restart under the declared GreptimeDB `sync_write`/`sync_period` mode
  or the declared ClickHouse fsync/replication mode;
- no S3/MinIO object-store request, egress, cache-size, or provider-cost rows;
- no ClickHouse LTS run, even though GitHub's latest ClickHouse release redirect
  currently points to the LTS line;
- no server-tier re-run of the Run 154 `LEFT JOIN` shape, subquery workaround,
  and app-side correlation under mixed ingest;
- no storage-cost gate that prices the Run 155 object-storage conclusions across
  retained bytes, request count, reread percentage, egress, cache size, and
  HA/replication profile;
- no product bundle proof from Run 156: SQL expressibility does not prove the
  evidence graph schema, redaction policy, bundle canonicalization, or
  outcome-loop rows;
- no full-text user-workflow proof from Run 157: it isolates search mechanisms,
  not alert triage quality, ranking, or evidence-bundle usefulness;
- no schema-conformance row proving every Parallax signal table carries the
  anchor key/index (`trace_id`, `fingerprint`, or equivalent) required by Run
  158;
- no updated Q6 mixed-load run using the fully indexed per-signal schema after
  Run 158's methodological correction;
- no metadata-store, ingest-log, setup, restart, redaction, or integration rows;
- no production hardware profile. Runs 140-143 and 145 are local Docker
  warm-cache artifacts, with four containers sharing a host and different timing
  bases by engine; Run 143 explicitly demotes large local runs and moves
  `N=5000000+` to an operator-requested server tier. Runs 144, 146, and 147 are
  source-read mechanism evidence, not new timing, cost, ingest, or
  fault-injection runs.

Therefore, these artifacts can support `smoke_only` storage evidence and schema
decisions. They cannot produce `greptime_prototype_default`,
`clickhouse_storage_default`, `dual_storage_open`, or `phase1_stack_pass` in A5.

### Claim Updates

| Prior wording risk | Corrected wording |
| --- | --- |
| "GT-nightly has no regressions." | "GT-nightly has no regressions at the 1M warm tier, but Runs 141/142 found a 5M dedup aggregation regression; re-test v1.1 GA, compaction states, and many-window metric tables." |
| "Every query is below the 300 ms gate." | "Every 100k/1M warm query is below the gate; at 5M, GreptimeDB heavy analytics cross the gate while anchored/keyed hot paths remain interactive. Server-tier runs own future large absolute numbers." |
| "Metrics should use GreptimeDB dedup/metric mode by default." | "Use append mode for scrape-style unique metrics when aggregation is load-bearing; reserve dedup/`last_non_null` for true correction/upsert semantics, and measure compaction-state plus TWCS-window-count sensitivity." |
| "Strict durability is only a smoke timing claim." | "Run 75 timing is now source-grounded in GreptimeDB's append-log sync path, but A5 still needs a version-pinned durability-mode run that records `sync_write`, `sync_period`, ClickHouse fsync/replication settings, crash/restart loss counts, and mixed-ingest p95/p99." |
| "GreptimeDB high-cardinality ingest is only a local smoke result." | "Runs 84/101 are now source-grounded by the PartitionTree primary-key dictionary, but A5 still needs native metric ingestion with series-count, dictionary-memory, flush-pressure, and mixed-query rows." |
| "GreptimeDB is blocked from Sentry-style grouped-error views." | "Run 156 falsifies that capability fear for grouped-error rollup and evidence-window ranking; the remaining ClickHouse advantage is ecosystem/maturity, not SQL expressibility for those views." |
| "ClickHouse cannot compete on object-storage economics." | "Run 155 narrows this: ClickHouse can tier to S3/object storage, so GreptimeDB's surviving cost edge must be proven on self-hosted 1x object storage, HA/server count, request/egress, and elastic compute behavior." |
| "The full-text correction might be a one-off." | "Run 157 re-verifies at 5M that selective full-text prunes on both engines while broad terms become scan-bound and favor ClickHouse." |
| "Anchored bundle retrieval is automatically fast on both engines." | "Run 158 narrows this: anchored retrieval is fast only where the queried signal table keys or indexes `trace_id`/`fingerprint`; un-keyed signal tables full-scan on both engines." |
| "GreptimeDB `output_rows` proves rows read." | "Run 158 corrects this: scan `output_rows` can be post-filter emission. Use `scan_cost`, `elapsed_poll`, and `file_ranges` to judge scan work." |
| "Benchmark confirms GreptimeDB as the default." | "Benchmark confirms GreptimeDB remains plausible for Parallax's anchored hot path, while ClickHouse is the fallback/default for analytics-heavy usage until the full gates decide." |

### Decision Impact

Keep GreptimeDB as the **prototype-fit candidate**, not as a measured stack
default. The new evidence strengthens both sides:

- GreptimeDB still fits Parallax's intended anchored evidence-bundle path.
- ClickHouse is clearly safer if Parallax drifts into ad hoc analytics, wide
  dynamic JSON, broad log search, or in-database cross-tier joins at scale.
- A storage adapter boundary remains mandatory because the measured flip trigger
  is now real, not hypothetical.
- For GreptimeDB, the hot path should fetch anchored signal slices separately
  and correlate in application code or use explicit subquery prefilters. A direct
  `LEFT JOIN` on the bundle path is a known optimizer trap until a future
  version proves predicate-through-join pushdown.
- The storage schema gate must fail if any bundle-participating signal table
  lacks the anchor key/index used by its retrieval path. Run 158 makes this
  engine-agnostic: both GreptimeDB and ClickHouse scan when the anchor is absent.
- GreptimeDB plan evidence must record `scan_cost`/`file_ranges`, not just
  `output_rows`, whenever a claim depends on pruning.
- ClickHouse's build-on-top advantage should be treated as an ecosystem and
  operating-model advantage, not as proof that GreptimeDB cannot express
  Parallax grouped-error or evidence-window queries.
- The object-storage decision is no longer "GreptimeDB object storage versus
  ClickHouse local disk." Both can use object storage; the remaining question is
  total self-hosted cost under Parallax's retention, cache, HA, and reread
  profile.
- GreptimeDB's durability fit is stronger for a no-loss-on-crash single-node
  profile because the source-backed strict path is an append-log sync, but A5
  must still pin durability settings and run crash/restart loss tests before the
  product can claim strict durability.
- GreptimeDB's metric-ingest fit is stronger for high-cardinality labels because
  the source-backed PartitionTree dictionary explains the measured flat ingest
  curve. This is a metric-ingest ergonomics win, not an aggregation-speed win;
  ClickHouse remains the safer fallback for heavy analytical metric scans.
- Future storage docs should separate local preliminary (`N=100000` default),
  historical 1M/5M local warm artifacts, operator-requested server large-tier,
  small-tier cold/object-store, and A5 stack-proof claims. Run 145 belongs in
  the preliminary bucket, not the canonical comparison bucket.

### Remaining Uncertainty

- The `bench/four-way` harness is valuable but narrower than the Rust
  `parallax-bench` prototype. It does not yet emit the JSONL result rows that A5
  expects.
- Runs 154-158 sharpen mechanisms but stay local, warm, and artifact-specific:
  they do not settle server-tier mixed-ingest p95/p99, cold/object-store reads,
  crash durability, or an end-to-end Parallax evidence-bundle workflow.
- Runs 142-144 isolate the dedup path, compaction sensitivity, and TWCS window
  mechanism, but the real native metric engine / Prometheus remote-write path
  still needs a v1.1 GA re-test on the server tier.
- Run 146 source-grounds the WAL sync path, but it does not prove the durability
  contract under process crash, OS crash, object-store mode, Kafka WAL mode, or
  mixed ingest with concurrent bundle queries.
- Run 147 source-grounds the high-cardinality ingest mechanism, but it does not
  prove native Prometheus remote-write/OTLP behavior, production dictionary
  memory limits, or the Q6 bundle latency impact of high-cardinality metrics.
- Object-store economics and cold selective reads are still the cost decision's
  highest-risk gap.

### Next Evidence Gap

Run the full mixed-load Q6 freshness gate and the object-store cost gate before
turning these numbers into a storage default. The next storage-specific
falsification target is: **does GreptimeDB keep Q6 p95 under 300 ms under mixed
native ingest with cold/object-store reads, using the app-side/subquery
correlation shape that avoids the Run 154 join trap and the fully keyed/indexed
per-signal schema required by Run 158, while preserving acknowledged rows under
the declared durability mode, and does its total self-hosted object-store/HA
cost beat ClickHouse's object-storage-capable profile?**

### Sources

- [Local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [Four-way version comparison](greptimedb-vs-clickhouse/four-way-version-comparison.md)
- [Four-way benchmark harness](../../bench/four-way/README.md)
- [A5 stack decision ledger](a5-stack-decision-ledger.md)
- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
- [GreptimeDB `v1.0.2` TWCS picker source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/twcs.rs)
- [GreptimeDB `v1.0.2` compactor source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/compactor.rs)
- [GreptimeDB `v1.0.2` raft-engine WAL log store source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/log-store/src/raft_engine/log_store.rs)
- [GreptimeDB `v1.0.2` raft-engine WAL config source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/common/wal/src/config/raft_engine.rs)
- [GreptimeDB `v1.0.2` Kafka WAL datanode config source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/common/wal/src/config/kafka/datanode.rs)
- [GreptimeDB `v1.0.2` PartitionTree memtable source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/memtable/partition_tree.rs)
- [GreptimeDB `v1.0.2` PartitionTree dictionary source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/memtable/partition_tree/dict.rs)
- [GreptimeDB `v1.0.2` PartitionTree shard-builder source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/memtable/partition_tree/shard_builder.rs)
- [ClickHouse `v26.5.1.882-stable` settings source](https://github.com/ClickHouse/ClickHouse/blob/v26.5.1.882-stable/src/Core/Settings.cpp)
- [GreptimeDB `v1.0.2` release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [ClickHouse `v26.5.1.882-stable` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable)
- [ClickHouse `v26.3.12.3-lts` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts)

### Bottom Line

Runs 140-158 made the storage evidence better and less comfortable. The anchored
Parallax hot path still supports the GreptimeDB fit thesis, but the 5M results
prove that analytics-heavy usage is a ClickHouse-shaped workload and that
GreptimeDB table mode, compaction state, and TWCS window count can dominate
version choice. Run 145 confirms the laptop-safe smoke tier, but also reinforces
why small local timings are directional only. Runs 144, 146, and 147
source-ground three GreptimeDB mechanism claims: TTL window drops, append-log
durability, and cardinality-insensitive metric ingest. Runs 154-158 add five
more guardrails: avoid direct in-DB `LEFT JOIN` correlation on GreptimeDB's hot
path, treat ClickHouse object storage as real, stop describing grouped-error
rollups as a GreptimeDB capability blocker, keep broad full-text search as a
ClickHouse flip trigger, and require anchor keys/indexes on every
bundle-participating signal table. They are still mechanism evidence. Treat the
benchmark code, local runs, and source-read evidence as strong smoke/schema
guidance, not as an A5 pass.
