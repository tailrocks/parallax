# GreptimeDB Storage Evaluation

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Executive Summary

> **⚠ V1 scope update (2026-06-18):** V1 is **decided GreptimeDB-only on the native OTLP model**;
> ClickHouse is deferred (not a V1 fallback or design constraint). The conclusion below ("ClickHouse
> stays the fallback") is retained as the engine-study record. Canonical:
> [../decisions/native-otel-tables.md](../decisions/native-otel-tables.md).

GreptimeDB is a credible storage-layer candidate for Parallax, but the strongest
technical argument is not that it is universally faster than every alternative.
The stronger argument is that GreptimeDB is designed as an open-source,
observability-native database that can ingest and query metrics, logs, and
traces through one system.

Current conclusion, after consuming the GreptimeDB-vs-ClickHouse benchmark
artifacts through Run 170:

> GreptimeDB remains a serious prototype-fit candidate for Parallax's unified
> observability-context layer, especially when Prometheus-compatible metrics,
> self-hosted 1x object-storage economics, cardinality tolerance, and
> auto-rebalance matter. The **current lean is GreptimeDB, not yet settled**
> ([decision](../decisions/storage-engine.md)): an intermediate proxy-lens
> interpretation once tilted toward ClickHouse (Parallax owns
> OTLP/routing/conversion, weighting retrieval speed + build-on-top ecosystem),
> but the resolved anchored-retrieval query mix takes ClickHouse's scan-speed
> lead off Parallax's hot path, so cost + Rust decide. ClickHouse stays the
> fallback; the full Parallax-shaped A5 gates still decide the production default.

But the skeptical view matters:

1. ClickHouse is still the safer mature choice for raw analytical performance,
   ecosystem depth, and log/trace analytics.
2. GreptimeDB's public performance evidence is mostly vendor-published, so
   Parallax should not assume "fastest" until running its own benchmark.
3. GreptimeDB traces are still explicitly marked experimental in the 1.0 docs,
   and the broader observability surface is young compared with mature
   single-signal systems.
4. GreptimeDB open source is usable, but some production operations features are
   positioned in GreptimeDB Enterprise.
5. The first Parallax MVP may not need a heavy observability database at all if
   it starts as a local CI failure context compiler.

## Current Source Freshness Update

As of 2026-05-25, the latest stable GreptimeDB release checked is
[`v1.0.2`](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2),
published 2026-05-14. GitHub also lists
`v1.1.0-nightly-20260525` as a pre-release, so benchmark and product claims
should stay pinned to the stable line unless a nightly is explicitly under test.
GreptimeDB's public docs describe it as an open-source observability database
for metrics, logs, and traces, and the getting-started docs install `v1.0.2`;
however, the trace read/write docs still say the trace section is experimental
and may change. Therefore GA reduces the "unreleased database" risk, but it does
not by itself prove trace maturity, production operations, retained cost, or
Parallax bundle-query performance.

Primary checks:

- [GreptimeDB v1.0.2 release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [GreptimeDB documentation home](https://docs.greptime.com/)
- [GreptimeDB standalone install](https://docs.greptime.com/getting-started/installation/greptimedb-standalone/)
- [GreptimeDB trace read/write docs](https://docs.greptime.com/user-guide/traces/read-write/)

## Language and Runtime Filter

Before fit, cost, or speed, candidates must pass a language/runtime gate. Only
high-performance, low-resource systems languages are in scope: Rust, Go, Zig,
C++, and C. Heavyweight managed or interpreted runtimes are excluded outright —
Java/JVM, Python, Ruby, PHP, and similar.

This removes the Java-family stores from contention regardless of features:
Elasticsearch, OpenSearch, QuestDB, Apache Doris, and Apache Pinot. The reason is
operational: the JVM profile (heap tuning, GC pauses, fat runtime) is the weight
Parallax exists to avoid — the same weight that makes self-hosted Sentry heavy.
The shortlist therefore stays GreptimeDB (Rust) versus ClickHouse (C++), with
Go/Rust/C++ systems (VictoriaMetrics, Parseable, Quickwit) as references. Prefer
Rust; when two candidates are close, Rust wins.

Note: Elasticsearch/OpenSearch is still worth studying for one thing only — its
UI/UX of presenting a log as a structured object (Kibana), not as storage.

## The Decision That Matters

Parallax should evaluate storage against this product question:

> Which backend best helps Parallax build evidence bundles by correlating test
> failures, logs, traces, metrics, deploys, and code changes without forcing
> users into a closed observability platform?

That is different from asking which database wins every benchmark. The useful
storage layer must be:

- self-hostable and open enough for teams to trust it with debugging data;
- good for metrics, not only logs and spans;
- able to query across telemetry types;
- cheap enough for high-volume observability retention;
- operationally reasonable for small teams;
- compatible with OpenTelemetry and Prometheus workflows;
- mature enough that Parallax is not blocked by storage bugs.

This is a database decision, not a platform decision. Platform products such as
ClickStack, SigNoz, Uptrace, OpenObserve, and Grafana LGTM can prove market
demand or provide useful schemas, but they should not be treated as storage
engine candidates. Parallax should compare the databases underneath them.

## Short Verdict

| Question | Current answer |
| --- | --- |
| Is GreptimeDB fully open source? | The core GreptimeDB repository is Apache-2.0 licensed. Self-hosting the core database is not blocked. Some advanced production features are sold as Enterprise. |
| Is it really faster than ClickHouse? | Not proven generally. GreptimeDB publishes strong benchmark results, but ClickHouse remains extremely strong for analytical/log workloads and has broader independent validation. |
| Is it cheaper? | Plausible for long-retention observability because GreptimeDB is object-storage-oriented, but total cost depends on query load, cache, compaction, egress, operations, and enterprise needs. Not proven by public data alone. |
| Is it better for metrics than ClickHouse? | Likely yes for Prometheus-compatible metrics workflows because GreptimeDB supports Prometheus remote write and PromQL. ClickHouse can store metrics, but it is less metrics-native in the core database. |
| Is it better for logs and traces than ClickHouse? | Not clearly. ClickHouse is a proven fit for logs/traces analytics, and ClickStack proves ClickHouse can be packaged into a unified observability stack. GreptimeDB's advantage is more metrics-native architecture, not obvious superiority on every log/trace query. |
| Is it mature enough? | Promising but young. GreptimeDB reached 1.0 GA and latest stable checked is `v1.0.2`, but trace docs remain experimental and ClickHouse has a much larger ecosystem and longer production history. |
| Should Parallax choose it now? | Keep it behind the storage abstraction; it is the **current lean (not yet settled)** as the Rust/object-storage/cardinality candidate, with ClickHouse the fallback — finalized only when the A5 sized-cost gates land (see [storage-engine.md](../decisions/storage-engine.md)). |

## GreptimeDB Strengths

### 1. Observability-Native Scope

GreptimeDB positions itself as an open-source observability database for metrics,
logs, and traces. Its documentation describes support for OpenTelemetry Protocol,
Prometheus remote write, Prometheus scrape, PromQL, SQL, logs, and traces.

This fits the Parallax thesis better than a database that treats metrics as an
afterthought. Parallax wants to assemble failure context across telemetry types,
not just store high-cardinality JSON logs.

Sources:

- [GreptimeDB GitHub repository](https://github.com/GreptimeTeam/greptimedb)
- [GreptimeDB OpenTelemetry Protocol docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)
- [GreptimeDB Prometheus ingestion docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/prometheus/)
- [GreptimeDB PromQL docs](https://docs.greptime.com/user-guide/query-data/promql/)
- [GreptimeDB traces overview](https://docs.greptime.com/user-guide/traces/overview/)

### 2. One System for Metrics, Logs, and Traces

GreptimeDB stores observability data in one database surface instead of asking
teams to operate separate systems for metrics, logs, and traces. This matters
for Parallax because the product value is context assembly:

```text
failed test / error
  -> nearby logs
  -> related spans
  -> correlated metrics
  -> deploy or code change window
  -> evidence bundle
```

The conceptual advantage is not merely fewer services. It is that Parallax can
ask cross-signal questions without stitching together Prometheus, Loki, Tempo,
Elasticsearch, and separate metrics stores itself.

This advantage is strongest against split observability stacks. It is less
unique against ClickHouse once ClickStack is considered, because ClickStack uses
ClickHouse as the telemetry store for logs, metrics, and traces. In that
comparison, GreptimeDB's differentiator is its
Prometheus-compatible metric model plus compute/storage separation in the
database architecture, not simply "one place for telemetry."

The caveat: "one database" does not automatically mean every cross-signal query
is easy, fast, or semantically clean. Schema design, labels/tags, trace IDs,
service names, timestamps, retention rules, and query planning still matter.

### 3. Metrics-Native Interfaces

GreptimeDB has Prometheus-compatible ingestion and PromQL query support. That is
important because observability metrics are not just generic rows; teams expect
Prometheus labels, time-series semantics, and PromQL-compatible workflows.

ClickHouse can store metrics and ClickStack supports OpenTelemetry metrics, but
ClickHouse core is a general analytical database. GreptimeDB has a clearer
metrics-first story.

This supports the user's intuition:

> ClickHouse is very good for logs and spans, but GreptimeDB may be a better
> conceptual fit when metrics are first-class and must be queried alongside logs
> and traces.

### 4. Object Storage and Cost Shape

GreptimeDB's architecture separates compute and storage and supports object
storage backends such as S3, GCS, Azure Blob Storage, and OSS. That can matter
for observability because raw telemetry volume grows quickly and long retention
on local SSD or block storage can become expensive.

The cost argument is plausible:

- object storage is commonly cheaper per retained GB than high-performance block
  storage;
- separating compute from storage can let teams scale query/ingest resources
  independently from retained data;
- a single database can reduce duplicated copies across metrics/logs/traces
  stacks.

But it is not automatically the cheapest system. Total cost includes:

- compute for ingestion and query;
- cache and local disk;
- compaction/indexing cost;
- object-store request and egress cost;
- retention policy design;
- operator time;
- support or enterprise features.

Sources:

- [GreptimeDB architecture docs](https://docs.greptime.com/user-guide/concepts/architecture/)
- [GreptimeDB storage configuration docs](https://docs.greptime.com/user-guide/deployments-administration/configuration/#storage-options)
- [AWS S3 pricing](https://aws.amazon.com/s3/pricing/)
- [Amazon EBS pricing](https://aws.amazon.com/ebs/pricing/)

### 5. Open-Source Core

GreptimeDB's repository is Apache-2.0 licensed, which is a strong signal for
self-hosting and extensibility. The core database can be inspected, forked, and
run by users without paying for a hosted service.

This is important for Parallax because debugging and observability data can be
sensitive. An open-source storage layer makes it easier to sell a local-first or
self-hosted story.

Source:

- [GreptimeDB license](https://github.com/GreptimeTeam/greptimedb/blob/main/LICENSE)

## GreptimeDB Risks

### 1. "Fastest" Is Not Proven

GreptimeDB publishes benchmarks showing competitive or superior performance in
log scenarios, JSONBench-style analytics, and comparisons against systems such
as Loki, Elasticsearch, and ClickHouse. These results are useful but not enough
to conclude that GreptimeDB is generally faster.

Reasons to be cautious:

- the strongest public data is vendor-published;
- observability workloads vary heavily by schema, cardinality, compression,
  query window, concurrency, and retention;
- ClickHouse is a benchmark-heavy project with a long history of high analytical
  performance;
- log, metrics, and trace workloads stress different storage/query paths;
- Parallax's actual workload may be smaller, burstier, and more context-oriented
  than vendor benchmarks.

The practical conclusion:

> GreptimeDB should be benchmarked against ClickHouse on Parallax-shaped data
> before claiming it is faster.

Sources:

- [GreptimeDB log scenario performance report](https://greptime.com/blogs/2025-08-07-beyond-loki-greptimedb-log-scenario-performance-report)
- [GreptimeDB and ClickHouse JSONBench discussion](https://greptime.com/tech-content/2025-07-10-database-architecture-jsonbench-performance-greptimedb)
- [GreptimeDB Elasticsearch comparison](https://greptime.com/blogs/2025-04-24-elasticsearch-greptimedb-comparison-performance)
- [GreptimeDB traces overview](https://docs.greptime.com/user-guide/traces/overview/)

### 2. Maturity Gap Versus ClickHouse

ClickHouse is more mature, has broader operational knowledge, more third-party
integration, more public production history, and a larger ecosystem. It is also
Apache-2.0 licensed and self-hostable.

GreptimeDB is younger. It reached 1.0 recently and is explicitly interesting
because it is early enough to be shaped around modern observability needs. But
that also means the Parallax project should expect sharper edges.

Source:

- [ClickHouse repository](https://github.com/ClickHouse/ClickHouse)
- [ClickHouse license](https://github.com/ClickHouse/ClickHouse/blob/master/LICENSE)
- [GreptimeDB releases](https://github.com/GreptimeTeam/greptimedb/releases)

### 3. Production Features May Be Enterprise

GreptimeDB core is open source, but GreptimeDB Enterprise advertises additional
features such as bulk ingestion, read replicas, automatic table repartitioning,
remote compaction, remote indexing, automated backup, data encryption, private
networking, and access control.

That does not block self-hosting the OSS database, but it matters for a serious
Parallax decision. If Parallax keeps GreptimeDB as a candidate default branch,
we need to know which production requirements are possible with OSS alone and
which require Enterprise.

Source:

- [GreptimeDB Enterprise](https://greptime.com/product/enterprise)

### 4. Unified Storage Does Not Remove Schema Work

GreptimeDB's metric engine can map logical metric tables into physical metric
tables. The docs note that all logical tables go into one physical table by
default, which can affect performance if that physical table is not partitioned.

For Parallax, this means the storage decision still requires careful modeling:

- metric labels and high-cardinality dimensions;
- trace span attributes;
- log body and structured fields;
- run/test/repository metadata;
- time windows for failure context;
- retention by signal and source.

Source:

- [GreptimeDB metric engine docs](https://docs.greptime.com/user-guide/administration/manage-data/metric-engine)

## ClickHouse Strengths and Weaknesses

ClickHouse remains the fallback and the skeptical alternative (faster raw analytics, more mature).

Strengths:

- extremely strong analytical SQL engine;
- mature OSS project with a large ecosystem;
- excellent for logs, events, spans, and high-cardinality analytics;
- ClickStack adds an open-source observability stack on top of ClickHouse;
- strong fit when Parallax wants flexible SQL over large raw event tables.

Weaknesses for this specific Parallax thesis:

- metrics are possible, but less native than Prometheus/PromQL-first systems;
- ClickStack narrows the unified-observability gap, but it is a stack built on
  ClickHouse rather than a metrics-native database design;
- self-hosted operations at scale are non-trivial;
- managed ClickStack has storage/compute separation advantages that are not the
  same as running the open-source stack yourself.

Important nuance:

> ClickHouse is not "bad for metrics." It can store and analyze metrics. The
> narrower point is that GreptimeDB appears more metrics-native for Prometheus
> workflows, while ClickHouse is more proven as a general analytical/event/log
> store.

Sources:

- [ClickHouse OpenTelemetry integration docs](https://clickhouse.com/docs/use-cases/observability/integrating-opentelemetry)
- [ClickHouse observability docs](https://clickhouse.com/docs/use-cases/observability)
- [ClickStack docs](https://clickhouse.com/docs/use-cases/observability/clickstack)
- [ClickStack architecture docs](https://clickhouse.com/docs/use-cases/observability/clickstack/architecture)
- [ClickStack repository](https://github.com/ClickHouse/ClickStack)

## Other Storage Alternatives

The search space is broader than GreptimeDB and ClickHouse, but many apparent
alternatives collapse into one of three buckets:

1. ClickHouse-based observability platforms, not independent databases.
2. Split stacks with separate metrics, logs, and traces stores.
3. Search/log-first systems that can ingest all three signals but are not
   metrics-native time-series engines.

That means Parallax should not say "GreptimeDB has no competitors." A more
accurate statement is:

> GreptimeDB and the ClickHouse ecosystem are the strongest open-source
> candidates for a fast, self-hostable observability storage layer that can
> handle metrics, logs, and traces together. Parseable is the most relevant
> non-ClickHouse challenger to watch, but it is still more of an observability
> datalake/platform than a direct GreptimeDB-style database substitute.

The primary benchmark should therefore compare databases only:

1. GreptimeDB.
2. ClickHouse.

Everything else is either excluded, watch-listed, or used only as a reference
for schemas and operational patterns.

## Unified-Storage Candidate and Exclusion Map

| Candidate | Stores metrics/logs/traces in one self-hostable system? | Query model | Storage design | Main weakness for Parallax |
| --- | --- | --- | --- | --- |
| GreptimeDB | Yes. One database engine targets metrics, logs, traces, and events. | SQL + PromQL + log/trace APIs. | Compute/storage separation, timestamp-first layout, object storage support. | Young; traces are still experimental in 1.0 docs; public speed/cost evidence is mostly vendor-published. |
| ClickHouse | Yes in practice. ClickStack stores OTel logs, metrics, traces, session replay, and errors in ClickHouse, but ClickStack itself is not a database. | SQL first; HyperDX search when using ClickStack; PromQL story is less native than GreptimeDB/Prometheus-style systems. | Mature columnar OLAP engine; open-source self-host is local-disk oriented, while managed ClickStack gets stronger object-storage economics. | Metrics semantics are stack-level rather than database-native; more operational surface than GreptimeDB for an OSS deployment. |
| Parseable | Yes by product direction. It describes itself as a telemetry/MELT observability datalake and documents logs, metrics, and traces via OpenTelemetry. | PostgreSQL-compatible SQL; PromQL docs exist. | Object-store-first datalake with local/distributed deployment. | Younger and less battle-tested; more platform-specific than GreptimeDB/ClickHouse as an embeddable storage choice. |
| OpenSearch Observability Stack | Partly. Logs/traces live in OpenSearch, but metrics discovery uses a Prometheus data source. | PPL for logs/traces; PromQL for metrics. | Search-index architecture with OpenSearch plus Prometheus and OTel/Data Prepper components. | Not one storage engine for all three; likely heavier and less cost-efficient for high-volume telemetry. |
| Elastic Observability | Yes as a product, and Elastic has an AGPL option for source code as of 2024, but releases/default distribution remain under Elastic licensing. | Elastic search/query stack plus OTel ingestion. | Search/index architecture. | Licensing and open-core history complicate "fully open"; heavier than Parallax wants for an OSS storage dependency. |
| VictoriaMetrics family | No single database. VictoriaMetrics, VictoriaLogs, and VictoriaTraces are separate specialized stores. | MetricsQL/PromQL, LogsQL, Jaeger-compatible trace APIs. | Highly optimized per signal. | Excellent specialized components, but Parallax would still stitch multiple stores together. |
| Grafana LGTM | No single database. Loki, Grafana/Mimir, Tempo, and Pyroscope are separate systems. | LogQL, PromQL, TraceQL, profiles APIs. | Mature composable stack. | Strong ecosystem, but operationally multi-system and not a single storage layer. |

Sources:

- [Parseable GitHub repository](https://github.com/parseablehq/parseable)
- [Parseable OpenTelemetry docs](https://www.parseable.com/docs/ingest-data/otel)
- [OpenSearch Observability Stack](https://observability.opensearch.org/docs/)
- [OpenSearch metrics discovery docs](https://observability.opensearch.org/docs/investigate/discover-metrics/)
- [Elastic licensing FAQ](https://www.elastic.co/pricing/faq/licensing)
- [Elastic OpenTelemetry docs](https://www.elastic.co/docs/solutions/observability/apm/opentelemetry/upstream-opentelemetry-collectors-language-sdks)
- [VictoriaMetrics product overview](https://victoriametrics.com/)
- [VictoriaLogs docs](https://docs.victoriametrics.com/victorialogs/)
- [VictoriaTraces docs](https://docs.victoriametrics.com/victoriatraces/)

## ClickHouse-Based Platforms Are Not Separate Storage Competitors

ClickStack should not be compared to GreptimeDB as a database. ClickStack is a
ClickHouse-backed observability stack. The official architecture docs describe
the open-source deployment as ClickHouse, HyperDX, and an OpenTelemetry
Collector, with MongoDB used for application state. The database under the stack
is ClickHouse.

Several strong "Datadog alternative" products support metrics, logs, and traces,
but they reinforce the ClickHouse side of the comparison rather than adding a new
storage engine:

| Platform | Storage implication |
| --- | --- |
| ClickStack / HyperDX | Official ClickHouse observability stack. Uses ClickHouse plus HyperDX and the OpenTelemetry Collector. |
| SigNoz | Open-source OpenTelemetry observability platform that uses ClickHouse as the datastore. |
| Uptrace | Open-source APM for traces, metrics, and logs; uses ClickHouse plus PostgreSQL metadata storage. |
| qryn | API compatibility layer for Loki, Prometheus, Tempo, and other protocols backed by ClickHouse. |

These are important market competitors, but for Parallax's storage decision they
mostly answer: "ClickHouse can be used as a unified telemetry backend if a stack
adds the observability semantics around it."

OpenObserve is also excluded from the database shortlist. It is a self-hostable
observability product/platform, not a clean storage engine candidate for
Parallax to embed or recommend as the default database. It can stay on a broad
market-watch list, but it should not drive the GreptimeDB-vs-ClickHouse storage
decision.

Sources:

- [ClickStack product page](https://clickhouse.com/clickstack)
- [ClickStack architecture docs](https://clickhouse.com/docs/use-cases/observability/clickstack/architecture)
- [SigNoz repository](https://github.com/SigNoz/signoz)
- [SigNoz architecture docs](https://signoz.io/docs/architecture/)
- [Uptrace repository](https://github.com/uptrace/uptrace)
- [Uptrace open-source APM page](https://uptrace.dev/get/hosted/open-source-apm)
- [qryn OpenTelemetry collector repository](https://github.com/metrico/otel-collector)
- [OpenObserve repository](https://github.com/openobserve/openobserve)

## Technical Difference: GreptimeDB vs. ClickHouse

The key technical difference is not that one can store telemetry and the other
cannot. Both can.

The real difference is where observability semantics live.

### GreptimeDB

GreptimeDB bakes more observability semantics into the database:

- time is required through a `TIME INDEX`;
- Prometheus remote write/read and PromQL are first-class interfaces;
- OTLP ingestion can target GreptimeDB directly;
- logs, traces, metrics, and wide events are part of the same product story;
- object storage is a primary architectural target;
- dynamic schema behavior is designed for changing OTLP attributes.

That makes GreptimeDB more naturally aligned with Parallax if Parallax wants a
database that already understands observability-shaped data.

### ClickHouse

ClickHouse is the stronger general analytical engine:

- mature columnar OLAP engine;
- broad ecosystem and operational knowledge;
- excellent high-cardinality event/log/trace analytics;
- powerful SQL and materialized-view patterns;
- many observability platforms already choose it as their store.

But observability behavior often lives above the database:

- OpenTelemetry schemas and transforms;
- HyperDX/ClickStack UX;
- Prometheus/PromQL compatibility layers;
- materialized views for rollups;
- adapters for trace and log workflows.

This is not necessarily bad. It is a strong architecture when the team wants SQL
control and raw analytical power. It is less attractive if Parallax wants the
smallest self-hosted path to "send OTLP/Prometheus in, query evidence bundles
out."

Sources:

- [GreptimeDB as an Alternative to ClickHouse for Time-Series and Observability](https://greptime.com/tech-content/2026-04-17-clickhouse-alternative-greptimedb)
- [ClickStack product page](https://clickhouse.com/clickstack)
- [ClickHouse OpenTelemetry integration docs](https://clickhouse.com/docs/use-cases/observability/integrating-opentelemetry)

## What We Can Finalize

We can finalize the following research position:

1. GreptimeDB is not the only open-source system that can store metrics, logs,
   and traces.
2. GreptimeDB is one of very few databases designed around all three signals
   rather than assembled into an observability stack later.
3. ClickHouse is the only equally serious mature baseline, and the ecosystem
   around it is now large: ClickStack, SigNoz, Uptrace, qryn, and custom
   ClickHouse observability pipelines.
4. Parseable is the main non-ClickHouse alternative worth watching, but it is
   more platform/datalake-shaped than database-shaped.
5. OpenObserve is excluded from the storage shortlist because it is better
   treated as an observability product, not a database competitor.
6. Grafana LGTM, VictoriaMetrics family, and OpenSearch are credible
   observability stacks, but they do not beat GreptimeDB on "single database for
   all three signals."
7. The remaining unproven claims are performance, cost, and trace maturity for
   Parallax-shaped workloads. GreptimeDB has strong published results, including
   a 2026 Greptime-authored comparison against ClickHouse, but Parallax should
   still run its own benchmark before saying GreptimeDB is faster, cheaper, or
   operationally safer for our workload.

The storage shortlist for Parallax should therefore be:

1. GreptimeDB as the purpose-built observability database candidate.
2. ClickHouse as the mature high-performance database baseline.

Parseable can stay on a watch list, but it should not be treated as equal to
GreptimeDB or ClickHouse until it proves database-level maturity, operational
fit, and Parallax-shaped performance.

| Option | Technical fit | Why it may lose for Parallax |
| --- | --- | --- |
| Prometheus + Thanos/Mimir | Excellent metrics ecosystem and PromQL. | Metrics-first; logs and traces require Loki/Tempo or other stores. Cross-signal evidence requires multi-system assembly. |
| Loki + Tempo + Mimir | Strong open observability stack around Grafana. | Mature but split across systems; operational and query model complexity remains. |
| VictoriaMetrics / VictoriaLogs / VictoriaTraces | Efficient, strong self-hosted story, metrics heritage. | More of a family of specialized systems than one unified SQL observability database. |
| Elasticsearch / OpenSearch | Strong log search ecosystem and mature operations knowledge. | Excluded by the language/runtime filter (Java/JVM). Also less metrics-native and expensive at scale. Useful only as a Kibana-style log-as-object UI/UX reference. |
| InfluxDB | Metrics/time-series heritage. | Weaker fit for unified logs/traces/context bundles than GreptimeDB or ClickHouse. |
| TimescaleDB | Postgres-based time-series familiarity. | Strong relational/time-series fit, but not a full observability-native logs/traces/metrics backend. |

Sources:

- [Grafana Mimir](https://grafana.com/oss/mimir/)
- [Grafana Loki](https://grafana.com/oss/loki/)
- [Grafana Tempo](https://grafana.com/oss/tempo/)
- [VictoriaMetrics](https://github.com/VictoriaMetrics/VictoriaMetrics)
- [VictoriaLogs](https://docs.victoriametrics.com/victorialogs/)
- [VictoriaTraces](https://docs.victoriametrics.com/victoriatraces/)
- [OpenSearch](https://opensearch.org/)
- [InfluxDB](https://github.com/influxdata/influxdb)
- [TimescaleDB](https://github.com/timescale/timescaledb)

## Recommended Parallax Position

Parallax should not describe GreptimeDB as "the fastest observability database"
unless we prove that with our own workload.

The stronger and more defensible claim is:

> GreptimeDB is a promising open-source, observability-native backend for
> unified metrics, logs, and traces, and it may let Parallax build cross-signal
> failure context without requiring separate Prometheus, Loki, Tempo, and
> ClickHouse deployments.

For the current research stage:

1. Treat GreptimeDB as the **current lean (not yet settled)** for unified
   observability storage — fit + cost + Rust — not a locked-in stack dependency.
2. Treat ClickHouse as the fallback behind the Parallax
   proxy, especially for logs, traces, spans, analytical SQL, and build-on-top
   ecosystem maturity.
3. Do not require either backend for the earliest CLI-first CI failure MVP.
4. Design Parallax's bundle format and ingestion interfaces so storage can be
   swapped.
5. Run a Parallax-shaped benchmark before confirming the production default.

## Benchmark We Should Run

The public benchmarks do not answer the Parallax question directly. Parallax
needs a focused benchmark that uses its own access patterns.

### Candidate Dataset

- OpenTelemetry traces with service, route, span kind, status, duration, and
  trace/span IDs.
- Structured logs with trace IDs, severity, service, message, error class, and
  repository/run metadata.
- Prometheus-style metrics for request latency, error rate, resource saturation,
  queue depth, and CI worker health.
- CI test events with repository, commit SHA, branch, test name, suite, duration,
  retry count, pass/fail, failure signature, and log excerpt pointers.

### Queries

1. Given a failing test, find related logs in the same CI run and time window.
2. Given a production error, find traces with matching error class and recent
   latency/error-rate changes.
3. Given a trace ID, fetch spans, logs, and metric deltas around the request.
4. Given a commit SHA, compare failure fingerprints before and after the commit.
5. Given a service and 30-minute window, build an evidence bundle with logs,
   traces, metric anomalies, and deploy metadata.

### Systems to Compare

1. GreptimeDB.
2. ClickHouse, optionally using ClickStack schemas/collector to represent the
   official ClickHouse observability path.
3. Parseable only if time allows.
4. Grafana stack and VictoriaMetrics family as sanity checks, not primary
   database candidates.

### Metrics

- ingest throughput;
- query latency for evidence-bundle queries;
- compression/retained storage size;
- CPU and memory under ingest/query concurrency;
- operational components required;
- ease of schema evolution;
- quality of OpenTelemetry and Prometheus compatibility;
- ease of self-hosting from zero.

## Current Decision

GreptimeDB is not proven to be the fastest, cheapest, or most mature storage
layer in general. But it is technically aligned with Parallax's long-term goal
better than most alternatives because it is open source, self-hostable,
Prometheus-compatible, OpenTelemetry-compatible, SQL-queryable,
object-storage-oriented, and designed around unified observability.

The recommended next step is to keep both ClickHouse and GreptimeDB behind the
storage abstraction while the A5 gates decide. A GreptimeDB GA release makes the
candidate less speculative; it does not close the benchmark, trace-maturity,
cost, cold-read, or operational-complexity questions. The **current lean is
GreptimeDB (not yet settled)** — the resolved anchored-retrieval query mix takes
ClickHouse's scan-speed lead off the hot path, so cost + Rust decide — with
ClickHouse the fallback to keep alive (see [storage-engine.md](../decisions/storage-engine.md)).
