# Technical Implementation Concept

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This document turns the Parallax research into a concrete build concept. It is
not a neutral menu. It names the first system to build, the default components,
the deployment topology, the data model, and the alternatives rejected for now.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

## Short Recommendation

Build Parallax as:

> A Rust-first, Sentry-compatible, OpenTelemetry-native error context system that
> stores observability evidence in GreptimeDB, keeps product metadata in
> Turso, and exposes bounded evidence bundles through an HTTP API and MCP server.

The first product should beat self-hosted Sentry on operational simplicity. It
should not start as a full observability dashboard or autonomous production SRE.

## Layer Decisions

| Layer | Recommendation | Why |
| --- | --- | --- |
| Rust app collection | `tracing`, `tracing-error`, `opentelemetry-otlp`, and a Sentry-compatible panic/error layer. | Only in-process collection sees panic messages, typed error chains, span fields, release/env, and backtraces. |
| External protocol | Accept Sentry envelopes and OTLP HTTP/gRPC. | Preserves existing Sentry SDK setup while making OTEL the native logs/traces/metrics path. |
| Ingest gateway | Build a Rust `parallax-ingest` service. | Parallax needs auth, redaction, size limits, raw evidence retention, grouping hooks, and idempotency before storage. |
| Message stream | No external broker in the tiny deployment. Use a local WAL/outbox. Add Apache Iggy for the durable profile. | The first version must stay simpler than Sentry. Iggy is the best Rust-native append-only stream once replay and processor isolation matter. |
| Storage default | GreptimeDB for v0.1 observability storage. Keep a ClickHouse adapter as the benchmark fallback. | GreptimeDB is the closest architectural fit: Rust, observability-native, OTLP, Prometheus/PromQL, SQL, object-storage-oriented deployment. |
| Metadata store | Turso Database for local/dev and tiny single-node; keep Postgres only as a scale-out fallback until Turso production behavior is proven. | Users, projects, DSNs, issue status, policies, and audit records are relational product state, not telemetry. Turso keeps the embedded metadata path Rust-native and SQLite-compatible without choosing C SQLite. |
| Processing | Rust workers, in-process for tiny mode and separate services for durable/scale-out mode. | Normalization, symbolication, grouping, correlation, and graph building need deterministic logic and strong testability. |
| Causal layer | Typed evidence graph stored as tables first. | Materialize graph edges before adopting a graph database. Causality needs explicit evidence and confidence. |
| Agent surface | HTTP context API plus MCP server, read-only first. | Agents need structured evidence, not dashboards. MCP makes Codex, Claude Code, Amp, OpenCode, and IDE agents first-class clients. |
| Human surface | Minimal Sentry-like issue UI later. | Humans need inspection and trust, but the differentiator is the context API. |

Related research:

- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
- [Self-hosted observability architecture](self-hosted-observability-architecture.md)
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md)
- [Messaging and ingestion layer](messaging-and-ingestion-layer.md)
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
- [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md)
- [Flaky test investigation and replay](flaky-test-investigation-and-replay.md)
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)

Metadata-store source:

- [Turso Database GitHub repository](https://github.com/tursodatabase/turso)

Use the `tursodatabase/turso` engine for the embedded metadata slot, not the old
C SQLite default. The repository currently describes Turso Database as a
Rust-written, SQLite-compatible in-process SQL database and marks it beta, so
Parallax should pair this choice with backups and a metadata-store benchmark
before relying on it for large production installs.

## Why This Is The Right First System

The product should start where the user pain is sharp:

1. self-hosted Sentry is useful but operationally heavy;
2. small Rust-heavy teams want ownership and predictable cost;
3. agents need structured context around failures;
4. dashboards are secondary to evidence retrieval;
5. complete root cause proof is unrealistic, but deterministic context assembly
   is achievable and valuable.

The smallest useful loop is:

```text
Rust service panics or emits error
  -> existing Sentry SDK or Parallax Rust setup sends event
  -> OpenTelemetry sends traces/logs/metrics
  -> Parallax groups the error
  -> Parallax fetches same-trace logs/spans/metrics and deploy context
  -> Parallax builds a bounded evidence bundle
  -> coding agent receives bundle and opens a fix PR or proposal PR
```

That is narrower than "AI observability" but much more buildable.

## Default Storage Decision

Use **GreptimeDB** as the default v0.1 observability store.

This is an opinionated default, not a claim that GreptimeDB has already beaten
ClickHouse on every workload. The storage benchmark still has veto power. But
the first prototype should optimize for architectural fit rather than inherited
incumbency.

### Why GreptimeDB Wins The First Build

| Axis | GreptimeDB rationale |
| --- | --- |
| Speed | Direct OTLP ingestion, SQL, PromQL, and observability-shaped tables reduce pipeline work before query. Public performance evidence is not enough, so Parallax must benchmark evidence-bundle latency. |
| Cost | Object-storage-oriented deployment fits long observability retention better than pure local SSD retention. Cost still depends on cache, compaction, query load, and object-store requests. |
| Scaling | Standalone mode fits small teams; distributed mode and compute/storage separation fit the future scale-out trajectory. |
| Architecture | Purpose-built for metrics, logs, traces, events, SQL, PromQL, and OpenTelemetry rather than a generic analytics database adapted to observability. |
| Rust/open-source lens | Rust-native, Apache-2.0 core, inspectable and agent-contributable. |
| Agent context | Cross-signal query surface is closer to the evidence-bundle use case than separate metrics/logs/traces databases. |

Current source anchors:

- [GreptimeDB docs](https://docs.greptime.com/)
- [GreptimeDB OpenTelemetry ingestion](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)
- [GreptimeDB Prometheus ingestion](https://docs.greptime.com/user-guide/ingest-data/for-observability/prometheus/)
- [GreptimeDB PromQL](https://docs.greptime.com/user-guide/query-data/promql/)
- [GreptimeDB architecture](https://docs.greptime.com/user-guide/concepts/architecture/)
- [GreptimeDB storage options](https://docs.greptime.com/user-guide/deployments-administration/configuration/)

### Why Not ClickHouse As Default?

ClickHouse remains the strongest mature fallback. It may beat GreptimeDB on raw
log/trace analytics or operational maturity. But it is not the best default for
the first Parallax implementation because:

- core metrics semantics require more application-layer work;
- PromQL compatibility is not the database's native center;
- the observability story often depends on platform wrappers such as ClickStack;
- the user specifically wants systems purpose-built for this new AI-native
  context use case, not only incumbent analytical strength.

ClickHouse should stay behind a storage abstraction and be benchmarked with the
latest stable/public version. It becomes the default only if GreptimeDB fails the
Parallax-shaped benchmark on freshness, evidence-bundle latency, trace/log
performance, operational simplicity, or OSS production viability.

Current source anchors:

- [ClickHouse observability docs](https://clickhouse.com/docs/use-cases/observability)
- [ClickHouse OpenTelemetry integration](https://clickhouse.com/docs/use-cases/observability/integrating-opentelemetry)
- [ClickHouse object storage docs](https://clickhouse.com/docs/operations/storing-data)

## Component Diagram

Tiny single-node:

```text
Rust app / service
  -> Sentry SDK compatible envelope endpoint
  -> OTLP HTTP/gRPC endpoint
  -> parallax-server
       - auth / DSN validation
       - redaction and size limits
       - local WAL / outbox
       - normalizer
       - deterministic grouping
       - storage writer
       - evidence graph builder
       - context API / MCP server
  -> GreptimeDB standalone
  -> Turso metadata
```

Durable single-server:

```text
Rust app / service
  -> parallax-ingest
       - auth / redaction / raw append
  -> Apache Iggy standalone
  -> parallax-worker
       - normalize
       - symbolicate
       - group
       - correlate
       - build evidence graph
  -> GreptimeDB standalone + object storage
  -> Turso metadata
  -> parallax-api / MCP
```

Scale-out:

```text
apps / collectors / CI systems
  -> parallax-ingest x N
  -> Iggy cluster or fallback stream
  -> normalizer workers x N
  -> grouping workers x N
  -> symbolication workers x N
  -> context-index workers x N
  -> GreptimeDB distributed or ClickHouse fallback cluster
  -> Turso metadata or Postgres scale-out fallback
  -> object storage
  -> context API / MCP / UI
```

## Data Flow From Event To Evidence Bundle

1. **Accept event.**
   - Sentry envelope arrives at `POST /api/:project_id/envelope/`.
   - OTLP logs/traces/metrics arrive over HTTP/gRPC.
   - Ingest validates DSN/token, project, size, and content type.

2. **Persist raw evidence.**
   - Tiny profile: append to local WAL/outbox.
   - Durable profile: append to Apache Iggy.
   - Store raw payload reference with TTL for parser recovery.

3. **Normalize.**
   - Convert Sentry events into Parallax error-event rows.
   - Convert OTLP spans/logs/metrics into queryable records.
   - Extract release, environment, service, trace ID, span ID, runtime, SDK, and
     resource attributes.

4. **Symbolicate and enrich.**
   - Use Rust line tables or split debuginfo when available.
   - Normalize Rust frame names and panic locations.
   - Attach build ID, release, commit SHA, service owner, and deploy context.

5. **Group deterministically.**
   - Respect explicit client fingerprint.
   - Otherwise compute stack-based fingerprint.
   - Fall back to error type plus normalized first message line.
   - Store grouping algorithm version.

6. **Store high-volume evidence.**
   - Error events, logs, spans, metric samples, and deploy markers go to
     GreptimeDB.
   - Users, projects, DSNs, issue status, redaction policy, and audit go to
     Turso.

7. **Build evidence graph.**
   - Create nodes for error, span, log, metric window, release, deploy, code
     change, CI event, and agent action.
   - Create typed edges such as `error_in_span`, `log_in_span`,
     `same_fingerprint`, `same_release_regression`, and
     `metric_anomaly_on_path`.

8. **Build context bundle.**
   - Anchor on issue, event, trace, alert, or CI failure.
   - Fetch deterministic neighbors first.
   - Expand along topology only after strong evidence is assembled.
   - Rank hypotheses with supporting evidence, contradictions, and missing data.

9. **Serve agent/human context.**
   - HTTP API returns JSON or Markdown bundle.
   - MCP exposes tools such as `parallax_issue_context` and
     `parallax_hypothesis_check`.
   - UI renders the same context later.

## Error Event Data Model

The first internal error event should be Sentry-inspired but Parallax-owned:

| Field group | Required fields |
| --- | --- |
| Identity | `event_id`, `project_id`, `received_at`, `timestamp`, `environment`, `release`, `service_name` |
| Error | `error_type`, `message`, `level`, `mechanism`, `handled`, `panic_location` |
| Stack | frames ordered oldest to newest: crate/module/function/file/line/in_app/build_id |
| Rust context | source chain, `SpanTrace`, panic hook data, `anyhow`/`eyre` context when present |
| Correlation | `trace_id`, `span_id`, transaction/route, request ID, deployment ID |
| Runtime | SDK, runtime, OS, arch, hostname/container/pod metadata |
| Grouping | fingerprint, grouping algorithm version, top in-app frame, normalized message |
| Evidence refs | raw envelope ref, trace refs, log-window refs, metric-window refs |
| Safety | redaction policy version, PII flags, raw access policy |

The data model must make grouping and evidence retrieval deterministic before AI
touches the event.

## Deterministic Grouping

Grouping should be deterministic for v0.1:

```text
if client_fingerprint exists:
  use client_fingerprint
else if stacktrace has in_app frames:
  hash(platform, error_type, normalized top in_app frames, panic location)
else if stacktrace exists:
  hash(platform, error_type, normalized top frames)
else:
  hash(platform, error_type, normalized first message line)
```

Rust-specific normalization:

- strip symbol hash suffixes where safe;
- preserve crate/module/function boundaries;
- preserve panic file/line as a strong signal;
- include release and environment for regression analysis, not grouping identity;
- version the algorithm so future changes can be audited.

AI grouping may become a secondary suggestion layer, but it should not decide
issue identity in the MVP.

## Correlation And Causal Reconstruction

Correlation should be layered by evidence strength:

| Layer | Query |
| --- | --- |
| Strong | Same trace ID, same span ID, parent/child spans, span links, same fingerprint. |
| Medium | Same release regression window, dependency path, service topology, metric anomaly on trace path. |
| Weak | Same time window, semantic similarity, free-text match. |

Causal reconstruction happens in the **evidence graph builder** and
**hypothesis engine**, not in the database and not inside a free-form LLM prompt.

The LLM receives:

- a bounded bundle;
- edge strengths;
- raw evidence links;
- contradictions;
- missing evidence;
- allowed actions.

It should not receive unlimited logs or direct production credentials.

## Agent-Facing Context API

First HTTP endpoints:

```text
GET /api/projects/:project/issues
GET /api/projects/:project/issues/:issue_id
GET /api/projects/:project/issues/:issue_id/context?window=10m
GET /api/projects/:project/events/:event_id/raw
GET /api/projects/:project/traces/:trace_id/context
POST /api/projects/:project/hypotheses/check
```

First MCP tools:

| Tool | Purpose |
| --- | --- |
| `parallax_issue_list` | List grouped issues by project/environment. |
| `parallax_issue_show` | Return issue detail and representative stacktrace. |
| `parallax_issue_context` | Return bounded evidence bundle. |
| `parallax_event_raw` | Return raw normalized event, redacted by default. |
| `parallax_trace_context` | Return spans, logs, and metric deltas for a trace. |
| `parallax_hypothesis_check` | Run deterministic checks for one proposed cause. |
| `parallax_pr_proposal` | Produce patch/proposal text, not production action. |

Sources:

- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)

## Deployment Profiles

### Profile 1: Tiny

Target: personal projects, startups, and small teams.

```text
parallax-server
greptimedb standalone
turso metadata
local disk retention
```

Properties:

- one Parallax binary;
- no broker;
- no Kubernetes requirement;
- bounded local raw WAL;
- easiest migration path from self-hosted Sentry SDKs.

This is the product that proves Parallax can be simpler than Sentry.

### Profile 2: Small Production

Target: a team running production Rust services.

```text
parallax-ingest
parallax-worker
greptimedb standalone with object storage
turso metadata
optional Apache Iggy standalone
```

Properties:

- raw replay and worker separation if Iggy is enabled;
- object storage for retained telemetry;
- Turso for metadata and audit;
- still small enough for one VM or a simple Compose deployment.

### Profile 3: Scale-Out

Target: larger companies and high-volume telemetry.

```text
parallax-ingest x N
iggy cluster or fallback stream
worker pools x N
greptimedb distributed or clickhouse fallback cluster
turso metadata or postgres scale-out fallback
object storage
api/mcp nodes x N
```

Properties:

- stateless ingest and API nodes;
- stream owns burst buffering and replay;
- storage owns long retention;
- processors scale by consumer group;
- the evidence graph and API contract stay the same.

The important design constraint: scale-out should change topology, not the event
contract.

## Benchmark Gates

Do not declare the architecture proven until these gates pass with latest
candidate versions:

### Storage

- GreptimeDB versus ClickHouse on Parallax-shaped datasets.
- Ingest-to-queryable latency under concurrent writes.
- Evidence-bundle query latency by issue, trace, release window, and metric
  anomaly.
- Retained size and object-storage cost for 7, 30, and 90 days.
- High-cardinality labels and attributes.
- Cold-cache and hot-cache behavior.

### Stream

- local WAL versus Iggy.
- producer ack latency and crash durability.
- replay throughput.
- worker restart and consumer group behavior.
- disk-full and segment-corruption behavior.
- memory use on a tiny VPS.

### Agent Context

- bundle size limits;
- redaction quality;
- prompt-injection resistance;
- evidence citation completeness;
- "inconclusive" behavior when data is missing;
- PR correctness rate by failure class.

## Rejected Alternatives

| Alternative | Decision | Reason |
| --- | --- | --- |
| Full Sentry clone | Reject. | Too much product surface and too much operational complexity. |
| Dashboard-first observability | Reject. | The differentiator is agent-ready context, not charts. |
| eBPF-first error capture | Reject. | eBPF cannot see Rust panic messages, typed error chains, or span fields. |
| Kafka/Pulsar | Reject as deployable candidates. | JVM and operational profile violate the language/runtime filter. |
| Required broker in v0.1 | Reject. | The tiny deployment must stay simpler than self-hosted Sentry. |
| ClickHouse as automatic default | Reject for first build. | Strong fallback, but less purpose-built for unified metrics/logs/traces plus PromQL semantics. |
| Elasticsearch/OpenSearch storage | Reject. | JVM/search-index architecture is the wrong performance and operations profile. Keep only object-centric log UI lessons. |
| Generic `run_sql` / `run_shell` MCP tools | Reject. | Too much blast radius and prompt-injection risk. |
| Autonomous production rollback | Reject for MVP. | Requires a separate safety, approval, and policy system. |

## What To Build First

The first implementation milestone should be:

```text
parallax-server
  - Sentry envelope ingest subset for error events
  - OTLP ingest path for logs/traces/metrics
  - local WAL/outbox
  - deterministic Rust-focused grouping
  - GreptimeDB writer
  - Turso metadata
  - issue context API
  - MCP read-only context tools
```

First useful command/API:

```bash
parallax issue context ISSUE_ID --window 10m --format markdown
```

First useful agent result:

```text
This panic first appeared in release 2026.05.25-4.
The top in-app frame is checkout::discount::apply at src/discount.rs:118.
The failing event is in trace 4f..., span checkout.apply_discount.
The same trace has a database lookup returning an empty rule set 12 ms before
the panic.
No matching failures exist in the prior release window.
Suggested fix: guard empty rule set and add regression test.
```

That is the correct first proof point: deterministic context that makes an
agent's fix proposal materially better than reading the stacktrace alone.

## Bottom Line

Parallax is technically plausible if it stays disciplined:

- start as a small Rust/Sentry/OTLP error-context system;
- use GreptimeDB as the default v0.1 observability store, with ClickHouse as the
  latest-version benchmark fallback;
- use no broker in the tiny profile and Apache Iggy in the durable profile;
- build deterministic grouping and evidence graphs before AI claims;
- expose safe API/MCP context before autonomous action;
- measure speed, cost, and scaling on Parallax-shaped workloads before claiming
  a storage or stream victory.

This direction is not fundamentally flawed. The flawed version is promising
omniscient AI root cause analysis. The defensible company is an open-source,
self-hosted runtime context engine that turns telemetry, deploys, code, and CI
evidence into bounded, auditable context for humans and coding agents.
