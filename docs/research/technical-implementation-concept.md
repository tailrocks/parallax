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

> A Rust-first, Sentry-compatible, OpenTelemetry-native execution context system
> for services, CLI apps, CI runs, and coding agents that stores observability
> evidence in GreptimeDB, keeps product metadata in Turso, and exposes bounded
> evidence bundles through an HTTP API and MCP server.

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
| Execution surfaces | Treat services, CLI apps, CI runs, and coding agents as first-class trace sources. | Parallax should explain software execution and the agent work performed on that execution, not only long-running services. |
| Human surface | Minimal Sentry-like issue UI later. | Humans need inspection and trust, but the differentiator is the context API. |

## Phase 2 Blueprint Decisions

This section locks the decisions required after the GO verdict in
[Parallax Go / No-Go Verdict](verdict.md).

### API Standard Decision

Support **both OpenTelemetry and Sentry**, with different jobs:

| API surface | Decision | What Parallax supports | What Parallax stores |
| --- | --- | --- | --- |
| OpenTelemetry / OTLP | Native telemetry standard. | OTLP/gRPC on `4317`, OTLP/HTTP on `4318`, `/v1/traces`, `/v1/logs`, `/v1/metrics`, binary protobuf first, gzip, partial-success responses, retryable overload responses, strict body limits. | Normalized spans, logs, metric samples, resources, trace/span IDs, service identity, deployment attributes, semantic attributes, and raw payload refs. High-volume rows go to GreptimeDB; raw refs go to local disk/object storage. |
| Sentry envelope API | Compatibility and migration standard for error events. | `POST /api/<project_id>/envelope/`, starting with `event` items only. Parse `exception`, stacktrace, release, environment, tags, breadcrumbs, `contexts.trace`, `debug_meta`, and client fingerprints. Reject or metadata-only-store high-risk items until explicitly supported. | Raw envelope refs, normalized error events, stack frames, Rust panic/error-chain fields, grouping material, issue/fingerprint metadata, release linkage, and trace/span correlation keys. High-volume event rows go to GreptimeDB; project/issue metadata goes to Turso. |
| Parallax context API | Product API above OTEL/Sentry. | JSON and Markdown evidence bundles by issue, event, trace, CI run, CLI invocation, or agent session. All responses include redaction status, evidence refs, confidence, missing-data warnings, and query manifest. | Product/query state, investigation runs, bundle manifests, audit records, agent access logs, and accepted/rejected fix outcomes in Turso or Postgres fallback. |

Do not invent a new telemetry wire protocol. OTLP is the wire format for
logs/traces/metrics. Sentry envelopes are the wire format for error migration.
Parallax's unique API is the context bundle and evidence graph above them.

### Component Boundary And Agent Access

Parallax has one job:

> ingest, store, correlate, and serve evidence.

Parallax does **not** fix production bugs. The fixing layer is separate:

```text
Parallax evidence store
  -> CLI / HTTP API / MCP context surface
  -> fixer component
  -> coding agent
  -> proposal PR or fix PR
```

The fixer component owns repository checkout, agent orchestration, patch
generation, test execution, and pull-request creation. Parallax records those
actions as evidence after the fixer performs them.
The detailed fixer request/outcome contract and autonomy gates are specified in
[Fixer component and outcome loop](fixer-component-and-outcome-loop.md).

Access decision:

- **CLI first:** required from day one because coding agents already operate
  through shell commands, CI can call a CLI, and humans need local exports.
- **HTTP API underneath:** the CLI and MCP server must call the same HTTP/JSON
  context contract so behavior is testable and stable.
- **MCP required, not optional:** the CLI is enough for the earliest local
  prototype, but not enough for first-class agent integration. MCP gives agents
  discoverable tools, structured arguments, transport-level sessions, and an
  authorization model. Keep the MCP server thin and read-only at first.
- **No generic mutation tools:** no `run_sql`, `run_shell`, production deploy,
  rollback, or database mutation tools in Parallax core.

### Named Stack Per Layer

| Layer | Simple default | Scalable path | Very scalable path |
| --- | --- | --- | --- |
| Language/runtime | Rust, Tokio async runtime. | Same. | Same. |
| HTTP API | `axum` REST/JSON service in `parallax-server`. | Separate `parallax-api` nodes behind a load balancer. | Horizontally scaled `parallax-api` fleet. |
| OTLP/gRPC | `tonic` + `prost` receiver for OTLP/gRPC; `axum` route for OTLP/HTTP. | Dedicated `parallax-ingest` nodes. | Regional ingest tiers with collector compatibility and overload control. |
| App collection | Rust `tracing`, `tracing-error`, `opentelemetry-otlp`, Sentry-compatible Rust panic/error capture. | Add SDK fixtures for more languages through Sentry envelope compatibility and OTLP. | Collector/agent integrations, sampling policy, tenant routing. |
| CLI tracing | `parallax` CLI built with `clap`; wrapper/subcommand mode records structural command metadata, sanitized args/env/cwd, stdout/stderr policy refs, exit code, and overhead metrics. | CI and deploy systems call CLI with project token and redaction policy after the [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md) gate passes. | Organization-wide CLI/agent gateway and policy templates. |
| Agent-session tracing | JSON event schema for model calls, MCP/tool calls, shell commands, file reads/writes, tests, patches, PRs, approvals, outcomes. | Fixer component and agent adapters emit session traces to Parallax. | Multi-agent session graph with policy, review, and accepted-fix feedback loops. |
| Stream / buffer | Local append-only WAL/outbox segment files. | Apache Iggy standalone when replay, backpressure, or worker separation is needed. | Iggy cluster or storage-backed stream fallback if Iggy fails scale tests. |
| Observability storage | GreptimeDB standalone on local disk. | GreptimeDB standalone with S3/object storage. | GreptimeDB distributed with object storage; ClickHouse fallback cluster if benchmarks force it. |
| Metadata store | Turso Database for projects, DSNs, policies, issue state, audit, agent sessions, CLI invocations, outcomes. | Turso with benchmarked backup/restore and concurrency gates; Postgres fallback if those fail. | Postgres fallback for large multi-node metadata if Turso fails production gates. |
| Raw evidence retention | Local disk raw refs with TTL. | S3-compatible object storage for raw envelopes, attachments, logs, and bundle manifests. | Tiered object storage with lifecycle policy and per-tenant retention. |
| Processing | In-process Rust normalizer/grouping/evidence-graph worker. | Separate Rust worker services and consumer groups. | Worker pools by normalization, grouping, symbolication, graph, bundle indexing. |
| Context surface | CLI + HTTP API + read-only MCP in the same binary. | Separate API/MCP service. | Horizontally scaled API/MCP tier with tenant isolation and audit indexing. |
| Human UI | No dashboard suite; optional minimal issue/evidence view later. | Same UI over API. | Same UI over API; dashboards remain non-core. |

The stack deliberately keeps Tier 1 small while preserving the seams needed for
Tier 2 and Tier 3.

Two version-freshness caveats on this table as of 2026-05:

- **Apache Iggy has no clustering yet.** Iggy is still in the Apache Incubator
  (latest `server-0.8.0`, 2026-04-22) and is **single-node only**; replication
  via Viewstamped Replication is in progress but not yet used by the server. So
  the Tier 3 "Iggy cluster" cell is a future target, not a shipping capability.
  Tier 3 must assume the storage-backed stream fallback until Iggy ships and
  proves clustering; do not design Tier 3 around Iggy HA that does not exist.
- **Rotel is alpha.** A Rust OTLP collector (Rotel, `v0.2.x`) is attractive but
  pre-1.0; the conservative collector default is the OpenTelemetry Collector
  (Go, allowed by the runtime filter) until Rotel reaches 1.0. Treat Rotel as an
  optional high-performance hop, not load-bearing infrastructure.

Related research:

- [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md)
- [Self-hosted observability architecture](self-hosted-observability-architecture.md)
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md)
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
- [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md)
- [Turso metadata production readiness](turso-metadata-production-readiness.md)
- [Messaging and ingestion layer](messaging-and-ingestion-layer.md)
- [Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md)
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
- [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md)
- [Flaky test investigation and replay](flaky-test-investigation-and-replay.md)
- [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md)
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md)
- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
- [Agent observability technical review](agent-observability-technical-review.md)
- [Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md)
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)

Metadata-store source:

- [Turso Database GitHub repository](https://github.com/tursodatabase/turso)

Use the `tursodatabase/turso` engine for the embedded metadata slot, not the old
C SQLite default. As of 2026-05 Turso Database is still pre-1.0 (latest stable
`v0.6.1`, 2026-05-22) and the repository still carries an explicit beta warning;
Turso Cloud has separate documented durability, PITR, export, and sync behavior,
but those managed-cloud guarantees do not prove the embedded local store is safe
under Parallax crash, backup, migration, and audit workloads. This is an
operator-chosen default, not a maturity claim: Parallax must pair it with its
own backup path and the [metadata-store benchmark](metadata-store-benchmark-plan.md)
before relying on it for large production installs, and Postgres remains the
scale-out fallback the moment Turso fails those gates. Treat the metadata slot
as the most likely place the named stack changes under benchmarking.

The current Turso-specific production gate is stricter than "it runs locally":
[Turso metadata production readiness](turso-metadata-production-readiness.md)
separates local embedded Turso from Turso Sync/Cloud behavior, requires MVCC
conflict/retry tests, treats CDC and MVCC as mutually exclusive for the audit
path, and keeps Postgres as an active fallback until backup/restore and
migration rollback are proven.

## Why This Is The Right First System

The product should start where the user pain is sharp:

1. self-hosted Sentry is useful but operationally heavy;
2. small Rust-heavy teams want ownership and predictable cost;
3. agents need structured context around failures;
4. teams need an audit trail for what agents saw, did, changed, and validated;
5. dashboards are secondary to evidence retrieval and question-answering;
6. complete root cause proof is unrealistic, but deterministic context assembly
   is achievable and valuable.

The smallest useful loop is:

```text
Rust service panics or emits error
  -> existing Sentry SDK or Parallax Rust setup sends event
  -> OpenTelemetry sends traces/logs/metrics
  -> CLI or agent execution trace records bounded local work when applicable
  -> Parallax groups the error
  -> Parallax fetches same-trace logs/spans/metrics and deploy context
  -> Parallax builds a bounded evidence bundle
  -> coding agent receives bundle and opens a fix PR or proposal PR
```

That is narrower than "AI observability" but much more buildable.

## Default Storage Decision

Use **GreptimeDB** as the default v0.1 observability store.

GreptimeDB reached **v1.0 GA in April 2026** (latest `v1.0.2`, 2026-05-14), with
stable APIs, a unified engine for metrics/logs/traces on object storage, and
SQL + PromQL + native OTLP ingestion. This removes the largest risk that existed
in earlier passes, when it was still beta: the storage layer is now a
production-grade choice, not a bet on an unreleased database. Distributed mode
(Frontend/Datanode/Metasrv with region failover and online repartition) is real
but younger than standalone, so the storage benchmark still applies to the
scale-out tiers.

This remains an opinionated default, not a claim that GreptimeDB has already
beaten ClickHouse on every workload. The storage benchmark still has veto power.
But the first prototype should optimize for architectural fit rather than
inherited incumbency.

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
Rust app / service / CLI / coding agent
  -> Sentry SDK compatible envelope endpoint
  -> OTLP HTTP/gRPC endpoint
  -> agent/CLI execution trace endpoint
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
Rust app / service / CLI / coding agent
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
apps / collectors / CI systems / coding agents
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
   - Convert CLI invocations and coding-agent sessions into execution traces.
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
     change, CI event, CLI invocation, agent session, and agent action.
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

## Agent And CLI Trace Model

Agent sessions and CLI invocations are first-class execution traces, not
attachments on an error.

### CLI Invocation Row

Store one invocation root plus child events:

| Field group | Required fields |
| --- | --- |
| Identity | `invocation_id`, `project_id`, `started_at`, `ended_at`, `duration_ms`, `status`, `exit_code` |
| Command | command name, subcommand, sanitized args, arg redaction report, cwd, repo, branch, commit SHA |
| Environment | sanitized env refs, config refs, host/container info, user/service actor, policy version |
| Process tree | parent invocation, spawned child processes, test/build/deploy step refs |
| Output | bounded stdout/stderr excerpts, object-storage refs for larger output, panic/error chain when present |
| Side effects | files read/written when available, database/resource action refs, network/deploy refs, generated artifact refs |
| Correlation | trace ID/span ID when emitted, CI run/job/step ID, release/deploy ID, agent session ID |
| Safety | redaction policy version, secret-detection result, raw access policy, audit actor |

High-cardinality command/output events go to GreptimeDB. Long stdout/stderr,
patches, and artifacts go to object storage with refs. Invocation metadata,
policy, and outcome state go to Turso.

### Coding-Agent Session Row

Store one session root plus ordered actions:

| Field group | Required fields |
| --- | --- |
| Identity | `agent_session_id`, `project_id`, `agent_product`, `started_at`, `ended_at`, `status` |
| Context | prompt refs, repo refs, files read, docs read, Parallax bundles requested, external issue/PR refs |
| Tool calls | MCP/API calls, CLI commands, shell commands, database/query templates, tool result refs |
| Model steps | model/provider metadata when available, token counts when available, reasoning summary refs, confidence |
| Code actions | files edited, patch refs, commits, PR URLs, review comments, rollback/revert refs |
| Validation | tests run, build commands, lint/typecheck results, failure excerpts, skipped checks |
| Outcome | accepted, edited, rejected, reverted, inconclusive, production recurrence, human approver |
| Safety | approvals, denied actions, policy version, redaction report, prompt-injection flags |

Parallax stores facts about the agent run. It does not need full private model
reasoning to be useful; it needs enough structured action history to reconstruct
what context the agent used and what it changed.

### Audit Graph Edges

Agent and CLI events become evidence graph nodes with typed edges:

| Edge | Meaning |
| --- | --- |
| `agent_used_bundle` | Agent session consumed a specific Parallax context bundle. |
| `agent_ran_command` | Agent invoked a CLI/shell command. |
| `command_spawned_process` | CLI command created a child process or test/build/deploy step. |
| `command_touched_resource` | Command interacted with a file, database object, queue, deploy target, or external API. |
| `agent_changed_file` | Agent produced a patch touching a file. |
| `agent_opened_pr` | Agent or fixer created a PR from a patch/proposal. |
| `validation_checked_patch` | Test/build/lint command validated or rejected a patch. |
| `fix_addressed_issue` | Human or recurrence data linked a fix outcome to an issue. |
| `fix_worsened_issue` | Revert, recurrence, or human review linked a bad outcome to prior action. |

This is what lets Parallax answer audit questions such as "what did the agent
do before this deploy?" and "which command touched this database object?"

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

The agent surface has three concrete forms:

1. CLI commands for shell-native agents and CI.
2. HTTP API for stable service-to-service integration.
3. MCP tools for first-class agent clients.

The CLI is the first usable surface. MCP is still required because a dedicated
MCP server makes Parallax discoverable as a structured, least-privilege context
provider rather than a bag of shell commands. The focused decision is captured
in [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md):
canonical HTTP API first, day-one CLI, and a read-only MCP adapter before broad
agent pilots. All surfaces must call the same authorization, redaction, and
bundle-building code.

First CLI commands:

| Command | Purpose |
| --- | --- |
| `parallax issue list --project <id>` | List grouped issues by project/environment. |
| `parallax issue show <issue_id>` | Show issue detail and representative stacktrace. |
| `parallax issue context <issue_id> --window 10m --format json|markdown` | Return a bounded evidence bundle. |
| `parallax event raw <event_id>` | Return redacted normalized event data and raw refs. |
| `parallax trace context <trace_id>` | Return spans, logs, and metric deltas for a trace. |
| `parallax hypothesis check <issue_id> --file hypothesis.md` | Run deterministic checks for a proposed cause. |
| `parallax agent session show <session_id>` | Show what an agent saw, queried, changed, tested, and produced. |
| `parallax cli invocation show <invocation_id>` | Show sanitized command, environment refs, output refs, exit status, and side effects. |

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
| `parallax_pr_proposal` | Return evidence-backed proposal context and checklist, not code changes or production action. |
| `parallax_agent_session_show` | Return agent-session timeline, tools, files, tests, patch refs, and outcome. |
| `parallax_cli_invocation_show` | Return sanitized CLI invocation evidence and side-effect refs. |

Build the MCP server against the current official spec revision shown by the MCP
site on the research date: **2025-11-25**. Do not cite or implement a
future-dated spec revision until the official site publishes it as current. MCP
is supported by the major checked agent clients (Claude Code, Codex, Cursor,
and Copilot/VS Code), which is why a read-only MCP context server is the right
agent-native surface alongside the CLI.

Sources:

- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
- [MCP specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)

## Scaling Trajectory

### Tier 1: Simple

Target: personal projects, startups, and small teams.

```text
parallax-server
  - Sentry envelope endpoint
  - OTLP HTTP/gRPC endpoint
  - CLI/agent trace endpoint
  - local WAL/outbox
  - in-process normalizer/grouping/evidence graph
  - HTTP API + read-only MCP
greptimedb standalone on local disk
turso metadata
local disk raw retention
parallax CLI
```

Properties:

- one Parallax binary;
- no broker;
- no Kubernetes requirement;
- bounded local raw WAL;
- CLI, HTTP API, and MCP available from the same process;
- easiest migration path from self-hosted Sentry SDKs.

This is the product that proves Parallax can be simpler than Sentry.
That proof is measured by the
[self-hosted simplicity gate](self-hosted-simplicity-gate.md), not assumed from
the component diagram.

### Tier 2: Scalable

Target: a team running production Rust services.

```text
parallax-ingest
parallax-worker
greptimedb standalone with object storage
turso metadata
optional Apache Iggy standalone
parallax-api / MCP
parallax CLI
```

Properties:

- stateless ingest split from API;
- raw replay and worker separation if Iggy is enabled;
- object storage for retained telemetry;
- Turso for metadata and audit;
- still small enough for one VM or a simple Compose deployment.

The explicit seams are ingest, stream, workers, storage, and API/MCP. Moving
from Tier 1 to Tier 2 should not change the event contract, the bundle schema,
or the CLI/MCP tools.

### Tier 3: Very Scalable

Target: larger companies and high-volume telemetry.

```text
parallax-ingest x N
iggy cluster or fallback stream
worker pools x N
greptimedb distributed or clickhouse fallback cluster
turso metadata or postgres scale-out fallback
object storage
api/mcp nodes x N
parallax CLI across CI/dev/agent environments
```

Properties:

- stateless ingest and API nodes;
- stream owns burst buffering and replay;
- storage owns long retention;
- processors scale by consumer group;
- the evidence graph and API contract stay the same.

The important design constraint: scale-out should change topology, not the event
contract. Tier 3 is not the first product, but Tier 1 must avoid choices that
make Tier 3 impossible.

## Benchmark Gates

Do not declare the architecture proven until these gates pass with latest
candidate versions:

### Storage

- GreptimeDB versus ClickHouse on Parallax-shaped datasets.
- Ingest-to-queryable latency under concurrent writes, with the initial proof
  gate in
  [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
- Evidence-bundle query latency by issue, trace, release window, and metric
  anomaly, with the initial proof gate in
  [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
- Retained size and object-storage cost for 7, 30, and 90 days, with the
  initial proof gate in
  [Storage size and object cost gate](storage-size-and-object-cost-gate.md).
- High-cardinality labels and attributes.
- Cold-cache and hot-cache behavior.

### Stream

- local WAL versus Iggy, with the replay/backpressure gate in
  [Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md).
- producer ack latency and crash durability.
- replay throughput.
- worker restart and consumer group behavior.
- disk-full and segment-corruption behavior.
- memory use on a tiny VPS.

### Metadata

- Turso versus Postgres for product metadata, agent sessions, CLI invocations,
  outcomes, and audit records.
- Crash/restart and backup/restore correctness.
- Concurrent ingest, agent-session, CLI-invocation, and API metadata writes.
- Migration/export path from Turso to Postgres if Turso fails scale-out gates.

### Agent Context

- bundle size limits;
- redaction quality;
- prompt-injection resistance;
- evidence citation completeness;
- "inconclusive" behavior when data is missing;
- production database evidence safety, with the initial gate in
  [Production database evidence access gate](production-database-evidence-access.md);
- PR correctness rate by failure class.

### Agent And CLI Execution

- coding-agent session schema across Codex, Claude Code, Amp, and OpenCode,
  with the initial adapter/value gate in
  [Agent session tracing across real tools](agent-session-tracing-real-tools.md);
- CLI redaction and overhead for args, env, config, stdout, and stderr, with
  the initial gate in [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md);
- A6 detector/runtime redaction architecture, with Parallax owning a Rust
  default-deny runtime engine and using external scanners only as offline
  validators, in [Redaction detector toolchain](redaction-detector-toolchain.md);
- agent-session query latency when linked to production events and CI runs;
- outcome feedback quality for accepted, edited, rejected, and reverted fixes.

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
  - CLI invocation trace ingestion
  - coding-agent session trace ingestion
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
self-hosted execution context engine that turns telemetry, deploys, code, CI,
CLI, and agent evidence into bounded, auditable context for humans and coding
agents.
