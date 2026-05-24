# Self-Hosted Observability Architecture

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-24

## Executive Summary

The stronger Parallax thesis is not CI-first debugging. It is:

> Build a Rust-first, self-hosted observability and error-context system that is
> compatible with Sentry SDK ingestion and OpenTelemetry ingestion, but is much
> simpler and cheaper to operate than self-hosted Sentry.

The target user is a small engineering team that:

- already uses Sentry SDKs for error capture and grouping;
- prefers self-hosting because cloud event pricing creates operational anxiety;
- wants logs, traces, metrics, and errors correlated in one system;
- wants enough context for a human or coding agent to fix production bugs;
- is willing to run a few simple services, but not a Sentry-sized service graph.

The recommended architecture direction:

```text
Applications
  -> Sentry-compatible ingest endpoint
  -> OpenTelemetry OTLP ingest endpoint
  -> Rust ingest gateway
  -> Apache Iggy durable stream
  -> Rust processors
  -> GreptimeDB for logs/traces/metrics/error events
  -> Postgres or embedded metadata store for projects/issues/users
  -> simple UI + CLI + agent context API
```

The product should not start as a full Sentry clone. It should start as a
Sentry-compatible event receiver and context API with deterministic grouping,
nearby telemetry lookup, and a small operational footprint.

## Product Boundary

Parallax should compete with self-hosted Sentry on the job the user actually
needs:

| Need | Parallax stance |
| --- | --- |
| Keep existing SDK setup | Accept Sentry envelopes and DSNs. |
| Group recurring errors | Implement deterministic grouping first. |
| Show what happened around an error | Correlate errors with logs, traces, metrics, releases, deploys, and host context. |
| Keep cost predictable | Self-host with bounded retention, object storage where useful, and no SaaS event quota anxiety. |
| Make agents useful | Produce compact evidence bundles and context APIs. |
| Avoid Sentry ops burden | Do not copy Sentry's full architecture, product surface, or background task graph. |

The first version should not include:

- session replay;
- profiling;
- cron monitoring;
- alerting matrix;
- multi-region clustering;
- billing;
- marketplace integrations;
- source map pipeline unless JavaScript becomes a target;
- complex organization/team administration.

## Why Not Copy Sentry Internals

Sentry's public self-hosted repository describes the packaged deployment as
"feature-complete" but intended for low-volume deployments and proofs of
concept. That is an important signal: even Sentry's own self-hosted posture is
not "simple production system for small teams."

Source:

- [Sentry self-hosted README](https://github.com/getsentry/self-hosted)

Sentry's architecture exists for a much broader product than Parallax needs:

- Relay for ingestion, filtering, rate limits, and processing;
- Kafka for buffering;
- Snuba and ClickHouse for event storage/query;
- Postgres for relational product data;
- Redis and workers for queues and product workflows.

Sources:

- [Sentry self-hosted data flow](https://develop.sentry.dev/self-hosted/data-flow/)
- [Snuba architecture overview](https://getsentry.github.io/snuba/architecture/overview.html)
- [Sentry Relay developer docs](https://develop.sentry.dev/ingestion/relay/)

The useful lesson is not "Sentry is wrong." It is that Sentry's architecture is
optimized for Sentry's full cloud product. Parallax should preserve the SDK
protocol surface while choosing a smaller internal design.

## Sentry-Compatible Ingestion

Sentry envelopes are the right compatibility target. Sentry's SDK docs define
envelopes as the format used for ingestion, forwarding, and offline storage.
Envelopes contain common headers and item payloads and are sent to:

```text
POST /api/<project_id>/envelope/
```

The older `/api/<project_id>/store/` endpoint is deprecated for event payloads.

Sources:

- [Sentry envelope format](https://develop.sentry.dev/sdk/foundations/envelopes/)
- [Sentry event payloads](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/)

MVP support should be a deliberately small subset:

| Envelope item | MVP decision | Reason |
| --- | --- | --- |
| `event` | Support first. | This is the core error-tracking object. |
| `transaction` | Parse enough for correlation, then store as trace-like telemetry. | Useful context, but not required for first grouping. |
| `attachment` | Store with limits or skip with metadata. | Useful for context, dangerous for cost and secrets. |
| `session` | Ignore initially. | Not needed for error-context MVP. |
| `replay_event` / `replay_recording` | Reject or drop explicitly. | Too much storage and product surface. |
| `profile` | Reject or drop explicitly. | Later feature. |
| `logs` / `metrics` / `spans` | Prefer OTLP path first. | Avoid chasing every Sentry telemetry extension early. |

The ingest gateway should still retain a bounded raw envelope reference for
debugging parser mistakes and forward compatibility. Sentry's Relay best
practices emphasize that protocol and API changes need forward compatibility
and that older relays should not drop data they do not understand. Parallax can
borrow that principle without implementing all of Relay.

Source:

- [Sentry Relay best practices](https://develop.sentry.dev/ingestion/relay/relay-best-practices/)

## Error Event Model

The Parallax internal event model should be Sentry-inspired but not Sentry-owned:

| Field group | Examples |
| --- | --- |
| Identity | `event_id`, `project_id`, `received_at`, `timestamp`, `environment`, `release`, `dist` |
| Error | exception type/value, mechanism, stacktrace frames, thread, level, logger |
| Runtime | platform, SDK name/version, modules/packages, server name, runtime context |
| Request | URL, method, route, headers allowlist, client IP policy |
| User | user ID/email/IP only if policy allows it |
| Correlation | trace ID, span ID, transaction name, service name, deployment ID |
| Grouping | fingerprint, grouping algorithm version, top in-app frame, normalized message |
| Evidence refs | log window refs, trace refs, metric window refs, raw envelope ref |

Sentry event payload docs require `event_id`, `timestamp`, and `platform`; they
also encourage fields such as level, logger, transaction, server name, release,
environment, tags, modules, and extra metadata. Stack traces should be part of
an exception or thread and contain frames ordered oldest to newest.

Sources:

- [Sentry event payloads](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/)
- [Sentry stack trace interface](https://develop.sentry.dev/sdk/foundations/envelopes/event-payloads/stacktrace/)

## Grouping Strategy

Grouping is the first product primitive to get right. Without grouping,
Parallax becomes only a log sink.

Sentry's grouping docs show the relevant design pressures:

- grouping starts as soon as events enter the infrastructure;
- client and server fingerprints can override default grouping;
- stack traces are the primary grouping signal;
- fallback message grouping is weaker;
- events are associated with issues/groups;
- Sentry now also uses AI-enhanced grouping after traditional hash lookup.

Source:

- [Sentry grouping internals](https://develop.sentry.dev/backend/application-domains/grouping/)

Parallax MVP grouping should be deterministic:

1. Respect explicit client `fingerprint` if present.
2. Build a Rust-focused stack fingerprint from in-app frames.
3. Normalize Rust symbol noise where possible:
   - remove hash suffixes;
   - normalize generic type parameter noise conservatively;
   - prefer crate/module/function/file/line when available;
   - treat panic location as a strong grouping signal.
4. Fall back to exception type plus normalized first message line.
5. Store the grouping algorithm version on each event.

AI grouping should be a later secondary pass. The first system must be trusted
without an LLM.

## OpenTelemetry Ingestion

OpenTelemetry should be the native path for logs, traces, and metrics.

OpenTelemetry supports traces, metrics, logs, and baggage as signals, with
events and profiles still in development/proposal stages. OTLP is stable for
trace, metric, and log signals and defines gRPC and HTTP transport plus protobuf
payloads.

Sources:

- [OpenTelemetry signals](https://opentelemetry.io/docs/concepts/signals/)
- [OTLP specification](https://opentelemetry.io/docs/specs/otlp/)

The OpenTelemetry Collector is also relevant. It is vendor-agnostic, receives,
processes, and exports telemetry data, and can handle retries, batching,
encryption, and sensitive-data filtering. Parallax should not require users to
run a Collector for a tiny setup, but it should support receiving from one.

Source:

- [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)

### Rust Collector Candidate: Rotel

Rotel (by Streamfold) is a Rust-native OpenTelemetry collector worth tracking
because it matches the Parallax bias: Rust, OTLP-native, Apache-2.0, small
footprint. It is not on the critical path, but it is a useful reference design
and a possible drop-in collector for users who want one.

What it is:

- "High Performance, Resource Efficient OpenTelemetry Collection," written in
  Rust with no garbage collector overhead;
- receivers: OTLP/gRPC, OTLP/HTTP, OTLP/HTTP-JSON, and a Kafka receiver for
  traces/metrics/logs;
- exporters: OTLP, ClickHouse, Kafka, Datadog, AWS X-Ray, AWS EMF, plus debug
  output;
- custom processors via native Python integration (pyo3 bindings);
- deployment: Docker, language packages (Python/Node.js), and an AWS Lambda
  extension layer with adaptive flushing and fast cold starts;
- license Apache-2.0; latest release v0.2.2 (2026-05-04); ~366 GitHub stars.

Performance claims (vendor/loadtest, verify independently):

- fewer resources than the upstream OpenTelemetry Collector for the same
  workloads;
- faster cold starts than the OpenTelemetry Lambda and Datadog OTEL Lambda
  layers.

Sources:

- [Rotel site](https://rotel.dev/)
- [Rotel GitHub repo](https://github.com/rotel-dev/rotel)

Why it matters for prototyping:

- a working example of a Rust OTLP receiver pipeline we can study instead of
  building one blind;
- its ClickHouse and Kafka exporters map directly onto the storage and stream
  layers Parallax is evaluating (GreptimeDB/ClickHouse + Iggy/Kafka);
- the pyo3 processor model is one concrete answer to "where do correlation and
  enrichment processors run."

Caveats:

- primary focus is serverless/Lambda collection, not a self-hosted ingestion
  backbone, so its strengths may not align with Parallax's durable-stream model;
- young (v0.2.2) and small ecosystem; treat as a reference and optional
  component, not a load-bearing dependency yet;
- no GreptimeDB exporter today, so OTLP would be the integration path.

Rust support matters because the target user is Rust-heavy. The OpenTelemetry
Rust docs currently mark traces, metrics, and logs as beta. That is good enough
for early adoption, but Parallax should expect API churn and should document
recommended `tracing` plus OpenTelemetry setup once implementation starts.

Source:

- [OpenTelemetry Rust](https://opentelemetry.io/docs/languages/rust/)

## Message Bus: Apache Iggy

Apache Iggy is worth prototyping as the internal durable stream because it fits
the system's philosophy:

- written in Rust;
- persistent append-only log;
- single binary deployment;
- multiple transports: QUIC, TCP, WebSocket, HTTP;
- streams, topics, partitions, consumer offsets, retention, and consumer groups;
- binary payload support without enforced schema;
- designed around thread-per-core shared-nothing architecture and `io_uring`;
- supports OpenTelemetry logs/traces and Prometheus metrics for its own
  observability.

Sources:

- [Apache Iggy about](https://iggy.apache.org/docs/introduction/about/)
- [Apache Iggy architecture](https://iggy.apache.org/docs/introduction/architecture/)

Iggy is a better conceptual fit than Kafka for Parallax's first architecture
because it avoids a JVM broker and keeps the stack Rust-centered. Its append-only
model also maps cleanly to ingestion replay:

```text
sentry.raw_envelopes
otel.raw_traces
otel.raw_logs
otel.raw_metrics
parallax.normalized_events
parallax.group_updates
```

Consumers can be separated by responsibility:

| Consumer | Responsibility |
| --- | --- |
| `event-normalizer` | Parse Sentry envelopes into Parallax event records. |
| `otel-normalizer` | Normalize OTLP logs/traces/metrics metadata. |
| `grouping-worker` | Compute fingerprints and issue membership. |
| `storage-writer` | Write normalized telemetry to GreptimeDB and metadata store. |
| `context-indexer` | Precompute correlation windows around new errors. |

### Iggy Caveats

Iggy should be a prototype candidate, not an unquestioned dependency.

Risks to validate:

- operational maturity compared with Kafka, NATS, Redpanda, or RabbitMQ;
- cluster story, because Iggy docs say Viewstamped Replication clustering is
  currently being implemented;
- Linux-specific performance assumptions around `io_uring`;
- default memory pool sizing may be too large for the smallest Parallax
  deployments;
- Docker on non-Linux can degrade performance;
- Iggy may be unnecessary for a single-node MVP if direct writes plus a local
  WAL are enough.

The first benchmark should compare:

1. no message bus: ingest gateway writes directly to storage plus local WAL;
2. Iggy as the durable stream;
3. NATS JetStream or Redpanda only if Iggy fails maturity tests.

## Storage: GreptimeDB First, Not Forever

GreptimeDB is the best first storage prototype for this product direction
because it is explicitly positioned as one database for metrics, logs, and
traces, with SQL, PromQL, OpenTelemetry support, object storage, and a Rust
implementation.

Sources:

- [GreptimeDB GitHub README](https://github.com/GreptimeTeam/greptimedb)
- [GreptimeDB OpenTelemetry docs](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)
- [GreptimeDB PromQL docs](https://docs.greptime.com/user-guide/query-data/promql/)

The most important GreptimeDB hypothesis for Parallax:

> A single observability-native database can make "show me everything around
> this error" cheaper to implement and operate than stitching together
> Prometheus, Loki, Tempo, Elasticsearch, ClickHouse, and Sentry.

The MVP schema should use GreptimeDB for high-volume, time-oriented data:

- error event rows;
- log rows;
- span rows;
- metric samples;
- context windows;
- deployment/release time markers.

Use Postgres, SQLite, or another simpler metadata store for low-volume product
state:

- users;
- projects;
- DSNs/tokens;
- issue status;
- comments;
- saved searches;
- retention settings.

Do not force all product metadata into GreptimeDB just because it can store
events. Keep the data model boring where query volume is low.

## Storage Alternatives To Track

| Candidate | Fit | Concern |
| --- | --- | --- |
| GreptimeDB | Best conceptual fit for unified metrics/logs/traces and PromQL plus SQL. | Young compared with ClickHouse; public performance evidence is partly vendor-published. |
| ClickHouse | Mature analytical baseline and proven for logs/events/traces. | Metrics semantics and cross-signal model require more application-layer work. |
| OpenObserve | Rust, single binary, logs/metrics/traces/RUM, S3-native Parquet architecture. | More platform-shaped; may compete with Parallax rather than act as a database layer. |
| Parseable | Full-stack observability platform for MELT telemetry and simple local startup. | Platform-shaped; verify query model and traces/metrics depth. |
| Quickwit | Rust cloud-native search for logs/traces, object-storage first, Apache-2.0. | Metrics are not the core product; likely complementary for search rather than primary unified store. |

Sources:

- [OpenObserve GitHub README](https://github.com/openobserve/openobserve)
- [Parseable GitHub README](https://github.com/parseablehq/parseable)
- [Quickwit GitHub README](https://github.com/quickwit-oss/quickwit)

## API and UI Shape

The UI should be simpler than Sentry because the primary consumer is often a
human plus an agent, not a human alone.

Core screens:

| Screen | Purpose |
| --- | --- |
| Issues | Grouped errors with count, last seen, release, environment, status. |
| Issue detail | Stack trace, recent events, correlated logs/traces/metrics, releases, agent context button. |
| Event detail | Raw normalized event plus raw envelope reference. |
| Trace/log context | Time-window query around an event. |
| Project settings | DSN, retention, redaction, grouping settings. |

Core CLI/API:

```bash
parallax issue list --project api --env prod
parallax issue show ISSUE_ID
parallax issue context ISSUE_ID --window 10m --format markdown
parallax event raw EVENT_ID
parallax project dsn PROJECT_ID
```

Agent endpoint:

```text
GET /api/projects/:project/issues/:issue_id/context?window=10m
```

Return a bounded evidence bundle:

- issue summary;
- representative stack trace;
- recent event examples;
- relevant log excerpts;
- trace/span waterfall summary;
- metric deltas;
- release/deploy changes;
- links to raw records.

## Minimal Deployment Profiles

### Profile 1: Tiny Single Server

For personal projects and small teams:

```text
parallax-server
greptimedb standalone
sqlite or postgres
local disk retention
```

No Iggy unless the ingest path needs buffering. This profile proves that
Parallax can beat self-hosted Sentry on operational simplicity.

### Profile 2: Durable Single Server

For production self-hosting with burst protection:

```text
parallax-ingest
iggy
parallax-worker
greptimedb standalone
postgres
object storage backups
```

This is the first serious target.

### Profile 3: Scale-Out

For higher ingest:

```text
multiple parallax-ingest nodes
iggy cluster or alternate stream
multiple workers
greptimedb distributed
postgres
object storage
```

Do not design this first. Keep interfaces clean enough that this can happen
later.

## MVP Sequence

1. Implement a Sentry envelope parser fixture, not a server.
2. Normalize one Rust panic event and one anyhow/eyre-style error event.
3. Compute deterministic grouping fingerprints.
4. Store normalized events in a local file or SQLite fixture.
5. Add a minimal HTTP endpoint compatible with Sentry SDK envelope submission.
6. Add GreptimeDB writes for error events and nearby logs.
7. Add OTLP logs/traces/metrics ingestion.
8. Add Iggy only when the direct ingest path needs replay, buffering, or
   independent processors.
9. Build the `issue context` API before building a full UI.

This order keeps the riskiest product question first:

> Can Parallax capture a production Rust error and provide more useful
> debugging context than Sentry, while remaining cheaper and simpler to run?

## Open Research Questions

1. Which subset of the Sentry envelope spec is enough for Rust users?
2. How close must the Sentry DSN/auth behavior be for existing SDKs to work
   unchanged?
3. What Rust stack frame normalization gives stable grouping across releases?
4. Can GreptimeDB query logs, spans, metrics, and error events around one error
   fast enough on a small server?
5. Does Iggy materially improve reliability versus a local WAL for the first
   deployment profile?
6. How much raw event/envelope data should be retained for debugging parser
   mistakes without exploding storage cost?
7. What redaction defaults are safe enough for self-hosted teams that still do
   not want secrets persisted forever?

## Current Recommendation

Parallax should pivot its primary research direction to:

> Sentry-compatible, OpenTelemetry-native, Rust-first self-hosted error context.

The CI failure context work remains useful as an adjacent agent-context workflow,
but it is not the first wedge if the real pain is production Rust observability
and self-hosted Sentry replacement.
