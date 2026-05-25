# Observability Storage Benchmark Plan

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-11

## Purpose

Parallax needs a storage benchmark that answers a narrow technical question:

> Which self-hostable database is the best fit for storing and querying metrics,
> logs, traces, and Parallax failure-context events together?

This is not a platform comparison. Observability platforms combine storage,
collectors, pipelines, dashboards, alerts, user management, and product UX. That
is a different category. For Parallax, the hard dependency is the database or
storage engine underneath the platform.

## Scope Rule

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

## Language and Runtime Filter

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

## Priority Axes

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

## Candidate Classes

### Primary Candidates

| Candidate | Why included | Main hypothesis |
| --- | --- | --- |
| GreptimeDB | Purpose-built open-source observability database for metrics, logs, traces, events, PromQL, SQL, OTLP, and object-storage-oriented deployment. | Best conceptual fit for Parallax because observability semantics live in the database. |
| ClickHouse | Mature open-source columnar OLAP database with excellent event/log/trace performance and a large observability ecosystem. | Strongest mature baseline; may win raw analytical performance and ecosystem reliability. |

### Watch List

| Candidate | Why not primary |
| --- | --- |
| Parseable | Interesting object-store-first observability datalake, but currently more platform-shaped than database-shaped for Parallax's storage dependency. Revisit only if it exposes a clean database-level interface and proves maturity. |
| OpenSearch / Elasticsearch | Excluded by the language/runtime filter (Java/JVM). Also slow for high-volume observability and not metrics-native. Keep only as a UI/UX reference for showing a log as a structured object (Kibana), never as a storage candidate. |
| VictoriaMetrics family | Strong metrics/logs/traces projects, but they are separate specialized databases rather than one unified storage engine. Useful as per-signal reference, not as Parallax's unified backend. |
| Grafana LGTM | Mature observability stack, but intentionally split across Loki, Mimir, Tempo, and other components. Useful as an ecosystem reference, not a database candidate. |
| InfluxDB / TimescaleDB / M3DB | Strong time-series or metrics heritage, but weaker fit for unified logs/traces/metrics evidence bundles. |

## What We Need To Prove

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

## Candidate Dataset

The benchmark should use synthetic-but-realistic data, then later replay real
open-source project CI data when available.

### Telemetry Signals

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

## Query Workload

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

## Benchmark Dimensions

### Performance

- ingest throughput;
- p50/p95/p99 query latency;
- concurrent ingest plus query behavior;
- cold-cache query latency;
- hot-cache query latency;
- high-cardinality query behavior;
- batch backfill performance.

### Cost Shape

- retained size on disk or object storage;
- compression ratio by signal type;
- CPU and memory per ingested GB;
- CPU and memory per query class;
- local SSD requirement;
- object storage request cost and egress sensitivity;
- compaction/indexing overhead.

### Operability

- number of required services;
- local single-node setup time;
- Kubernetes setup complexity;
- backup/restore story;
- schema evolution behavior;
- retention and downsampling;
- authentication and multi-tenancy available in OSS;
- observability of the database itself;
- failure recovery under node restart or disk pressure.

### Product Fit

- OpenTelemetry ingestion quality;
- Prometheus remote write/read quality;
- PromQL completeness for Parallax needs;
- SQL ergonomics for cross-signal joins;
- support for dynamic attributes;
- support for trace/log correlation;
- ease of exporting portable evidence bundles.

## Test Matrix

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

## Candidate-Specific Notes

### GreptimeDB

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

### ClickHouse

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

## Decision Criteria

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

## Current Benchmark Shortlist

1. GreptimeDB.
2. ClickHouse.

Everything else is watch-list or sanity-check material until it proves it is a
database-level alternative rather than an observability platform.
