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

> A Rust-first execution context system with fixture-gated Sentry envelope
> error-event ingest and OpenTelemetry-compatible telemetry ingest for services,
> CLI apps, CI runs, and coding agents. It starts as a **local-first one-binary
> evidence server** with embedded Turso/SQLite-like storage so a developer can
> hand a `run_id` to an agent and get logs, traces, spans, metrics, and grouped
> failures back as a bundle. It stores production/high-volume observability
> evidence behind a swappable ClickHouse/GreptimeDB adapter, with the **current
> production lean GreptimeDB (not yet settled)** — fit + cost + Rust — and
> ClickHouse the fallback that wins raw analytics; see
> [storage engine decision](../decisions/storage-engine.md). It keeps Postgres
> as the production relational fallback until metadata gates pass and exposes
> bounded schema-bound evidence bundles through a CLI and local API, with a later
> read-only MCP adapter once the canonical bundle and projection contracts are
> stable.

The first product should beat self-hosted Sentry on operational simplicity. It
should not start as a full observability dashboard or autonomous production SRE.

## Layer Decisions

| Layer | Recommendation | Why |
| --- | --- | --- |
| Rust app collection | `tracing`, `tracing-error`, `opentelemetry-otlp`, and a Sentry envelope panic/error-event layer. | Only in-process collection sees panic messages, typed error chains, span fields, release/env, and backtraces. |
| External protocol | Accept the Sentry envelope `event` subset and OTLP HTTP/gRPC. | Preserves the error-event migration path while avoiding sessions, replay, profiles, release-health, exact grouping parity, and full Sentry API/UI parity until fixture gates prove each surface. |
| Ingest gateway | Build a Rust `parallax-ingest` service. | Parallax needs auth, redaction, size limits, raw evidence retention, grouping hooks, and idempotency before storage. |
| Message stream | No external broker in the tiny deployment. Use a local WAL/outbox. Add Apache Iggy for the durable profile. | The first version must stay simpler than Sentry. Iggy is the best Rust-native append-only stream once replay and processor isolation matter. |
| Storage default | Do not hard-code one engine in the product contract. **V1 local default: embedded Turso/SQLite-like profile. Production/server lean: GreptimeDB, not settled**, with ClickHouse fallback; all behind the adapter ([v1-storage-adapter-vision.md](../decisions/v1-storage-adapter-vision.md), [storage-engine.md](../decisions/storage-engine.md)). | Local V1 optimizes one-binary developer/agent debugging by `run_id`. Server mode optimizes retained observability evidence; the resolved anchored-retrieval query mix takes ClickHouse's scan-speed lead off the hot path, so cost + Rust decide — where GreptimeDB leads. Full A5 gates still have veto power. |
| Metadata store | Turso Database for local/dev and tiny single-node prototypes; keep Postgres as the production and scale-out fallback until Turso production behavior is proven. | Users, projects, DSNs, issue status, policies, and audit records are relational product state, not telemetry. Turso keeps the embedded metadata path Rust-native and SQLite-compatible without choosing C SQLite. |
| Processing | Rust workers, in-process for tiny mode and separate services for durable/scale-out mode. | Normalization, symbolication, grouping, correlation, and graph building need deterministic logic and strong testability. |
| Causal layer | Typed evidence graph stored as tables first. | Materialize graph edges before adopting a graph database. Causality needs explicit evidence and confidence. |
| Agent surface | CLI plus canonical HTTP context API first; read-only MCP adapter after the access-surface gate. | Agents need structured evidence, not dashboards. CLI/HTTP keeps the tiny tier testable; MCP becomes valuable once the bundle contract and safety model are stable. |
| Execution surfaces | Treat services, CLI apps, CI runs, and coding agents as first-class trace sources. | Parallax should explain software execution and the agent work performed on that execution, not only long-running services. |
| Human surface | Minimal Sentry-like issue UI later. | Humans need inspection and trust, but the differentiator is the context API. |

## Phase 2 Blueprint Decisions

This section locks the decisions required after the GO verdict in
[Parallax Go / No-Go Verdict](../decisions/go-no-go.md).

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
Normalized telemetry rows are not themselves the agent-facing product; they
become claimable agent context only when they assemble into schema-valid
evidence bundles with canonical hashes, projection manifests, redaction reports,
and matching CLI/HTTP/MCP projections.

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
[Fixer component and outcome loop](../decisions/fixer-boundary.md), and
the result rows required before fixer claims become measurable are specified in
[Fixer outcome ledger](../decisions/fixer-boundary.md).

Access decision:

- **CLI first:** required from day one because coding agents already operate
  through shell commands, CI can call a CLI, and humans need local exports.
- **HTTP API underneath:** the CLI and later MCP adapter must call the same
  HTTP/JSON context contract so behavior is testable and stable.
- **MCP later, not required for the tiny tier:** the CLI is enough for the
  earliest local prototype and Phase 1 proof. MCP becomes the right first-class
  agent integration once the bundle API, authorization model, audit rows, and
  redaction gates are stable. Keep the MCP adapter thin and read-only.
- **Projection equivalence before agent claims:** CLI JSON, HTTP JSON, MCP
  `structuredContent`, and Markdown must be projections of the same canonical
  bundle. Text-only MCP output or renderer-only equivalence does not count.
- **No generic mutation tools:** no `run_sql`, `run_shell`, production deploy,
  rollback, or database mutation tools in Parallax core.

### Named Stack Per Layer

| Layer | Simple default | Scalable path | Very scalable path |
| --- | --- | --- | --- |
| Language/runtime | Rust, Tokio async runtime. | Same. | Same. |
| HTTP API | `axum` REST/JSON service in `parallax-server`. | Separate `parallax-api` nodes behind a load balancer. | Horizontally scaled `parallax-api` fleet. |
| OTLP/gRPC | `tonic` + `prost` receiver for OTLP/gRPC; `axum` route for OTLP/HTTP. | Dedicated `parallax-ingest` nodes. | Regional ingest tiers with collector compatibility and overload control. |
| App collection | Rust `tracing`, `tracing-error`, `opentelemetry-otlp`, and fixture-gated Sentry envelope Rust panic/error capture. | Add SDK fixtures for more languages through Sentry envelope `event` compatibility and OTLP. | Collector/agent integrations, sampling policy, tenant routing. |
| CLI tracing | `parallax` CLI built with `clap`; wrapper/subcommand mode records structural command metadata, sanitized args/env/cwd, stdout/stderr policy refs, exit code, and overhead metrics. | CI and deploy systems call CLI with project token and redaction policy after the [CLI trace overhead and redaction](../capture/agent-cli-tracing.md) gate passes. | Organization-wide CLI/agent gateway and policy templates. |
| Agent-session tracing | Normalized `agent_session` / `agent_action` schema fed by bounded adapters for native OTel, hooks/plugins, JSONL or stream JSON, exports, server/API protocols, wrappers, and raw refs. | Fixer component and real-tool adapters source session traces with per-tool/version/config coverage, lossiness, redaction, and projection rows in the ledger. | Multi-agent session graph with policy, review, and fixer outcome feedback loops after ledger gates. |
| Stream / buffer | Local append-only WAL/outbox segment files. | Apache Iggy standalone when replay, backpressure, or worker separation is needed. | Iggy cluster or storage-backed stream fallback if Iggy fails scale tests. |
| Observability storage | Local embedded Turso/SQLite-like profile first; server storage adapter with GreptimeDB and ClickHouse profiles; current production lean GreptimeDB (not settled), ClickHouse fallback ([v1-storage-adapter-vision.md](../decisions/v1-storage-adapter-vision.md), [storage-engine.md](../decisions/storage-engine.md)). | GreptimeDB S3-native profile or ClickHouse hot tier depending on freshness/cost gates. | ClickHouse cluster, GreptimeDB distributed/object-storage profile, or a later hot/cold split only if A5 cost and latency gates justify the added operations. |
| Metadata store | Turso Database for prototype projects, DSNs, policies, issue state, audit, agent sessions, CLI invocations, and outcomes. | Turso with benchmarked backup/restore and concurrency gates; Postgres production fallback if those fail. | Postgres fallback for production or large multi-node metadata if Turso fails production gates. |
| Raw evidence retention | Local disk raw refs with TTL. | S3-compatible object storage for raw envelopes, attachments, logs, and bundle manifests. | Tiered object storage with lifecycle policy and per-tenant retention. |
| Processing | In-process Rust normalizer/grouping/evidence-graph worker. | Separate Rust worker services and consumer groups. | Worker pools by normalization, grouping, symbolication, graph, bundle indexing. |
| Context surface | CLI + HTTP API in the same binary; optional read-only MCP adapter only after the access-surface gate. | Separate API and optional MCP service. | Horizontally scaled API/MCP tier with tenant isolation and audit indexing. |
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

- [Rust data collection and instrumentation](../capture/rust.md)
- [Self-hosted observability architecture](overview.md)
- [GreptimeDB storage evaluation](../storage/evaluation.md)
- [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md)
- [Storage size and object cost gate](../storage/size-and-object-cost.md)
- [Metadata store benchmark plan and prototype](../storage/metadata/metadata-store-benchmark-plan.md)
- [Turso metadata production readiness](../storage/metadata/turso-metadata-production-readiness.md)
- [Messaging and ingestion layer](../storage/streaming/messaging-and-ingestion-layer.md)
- [Ingest log replay and backpressure gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md)
- [A7 scope discipline ledger](../validation/a7-scope.md)
- [Causal reconstruction and agent safety](causal-reconstruction.md)
- [AI-native observability and incident intelligence](../00-vision/ai-native-observability.md)
- [Flaky test investigation and replay](../capture/ci-and-flaky-tests.md)
- [Frontend collection and cross-tier correlation](../capture/frontend.md)
- [Frontend capture safety ledger](../capture/frontend.md)
- [Agent and CLI execution tracing](../capture/agent-cli-tracing.md)
- [Agent session tracing across real tools](../capture/agent-cli-tracing.md)
- [CLI trace overhead and redaction](../capture/agent-cli-tracing.md)
- [Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md)
- [Agent access surface safety ledger](../decisions/agent-access-surface.md)
- [Agent observability technical review](../reference/agent-observability-review.md)
- [Strategic verdict and research coverage](../decisions/strategic-coverage.md)
- [OpenTelemetry protocol and context layer](../capture/otlp.md)
- [OTLP receiver conformance and Collector equivalence](../capture/otlp.md)
- [OTLP conformance ledger](../capture/otlp.md)
- [Evidence bundle and open schema specification](evidence-bundle-schema.md)
- [Deploy, change, and issue-tracker context](../capture/deploy-change-context.md)
- [Deploy/change context ledger](../capture/deploy-change-context.md)

Metadata-store source:

- [Turso Database GitHub repository](https://github.com/tursodatabase/turso)
- [Turso v0.6.1 release](https://github.com/tursodatabase/turso/releases/tag/v0.6.1)
- [Turso v0.7.0-pre.3 release](https://github.com/tursodatabase/turso/releases/tag/v0.7.0-pre.3)

Use the `tursodatabase/turso` engine for the embedded metadata slot, not the old
C SQLite default. As of 2026-05 Turso Database is still pre-1.0 (latest
non-prerelease checked `v0.6.1`, 2026-05-22; newest checked pre-release
`v0.7.0-pre.3`) and the repository still carries an explicit beta warning;
same-day GitHub metadata also shows `main` moving after those tags, including
MVCC-internal changes, so benchmark rows must distinguish stable release,
pre-release, and moving `main` commit results;
Turso Cloud has separate documented durability, PITR, export, and sync behavior,
but those managed-cloud guarantees do not prove the embedded local store is safe
under Parallax crash, backup, migration, and audit workloads. This is an
operator-chosen default, not a maturity claim: Parallax must pair it with its
own backup path and the [metadata-store benchmark](../storage/metadata/metadata-store-benchmark-plan.md)
before relying on it for large production installs, and Postgres remains the
scale-out fallback the moment Turso fails those gates. Treat the metadata slot
as the most likely place the named stack changes under benchmarking. Metadata
benchmarks must record whether they use a stable or pre-release Turso tag; a
pre-release result can inform development but should not satisfy production
default claims without a stable rerun.

The current Turso-specific production gate is stricter than "it runs locally":
[Turso metadata production readiness](../storage/metadata/turso-metadata-production-readiness.md)
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
  -> Parallax fetches same-trace logs/spans/metrics and deploy/change context
  -> Parallax builds a bounded evidence bundle
  -> separate fixer/coding agent receives bundle and may open a proposal or
     draft PR
  -> outcome rows decide whether the fix helped
```

That is narrower than "AI observability" but much more buildable.

## Default Storage Decision

Keep **ClickHouse and GreptimeDB** behind a storage adapter. The current lean is
**GreptimeDB (not yet settled)**, with ClickHouse the fallback; full reasoning in
[storage-engine.md](../decisions/storage-engine.md).

GreptimeDB reached **v1.0 GA in April 2026** (latest stable checked `v1.0.2`,
2026-05-14). An intermediate proxy lens once tilted toward ClickHouse: Parallax
itself owns OTLP ingestion, routing, normalization, and format conversion before
writing to storage, which neutralizes much of GreptimeDB's native-ingest
advantage, leaving retrieval speed, mature SQL/build-on-top ecosystem, cost
shape, and scale/operations — ClickHouse leads retrieval/ecosystem, GreptimeDB
leads cost/cardinality/PromQL/auto-rebalance. The deciding input then resolved:
the query mix is **anchored-retrieval-dominant** (operator 2026-05-29), so both
engines serve Parallax's hot path interactively (≪300 ms) and ClickHouse's
retrieval lead falls off it — leaving cost + Rust, where GreptimeDB leads. Hence
the current lean is **GreptimeDB, not yet settled**, with ClickHouse the fallback
for analytics-heavy use.

This is still a benchmark-controlled decision. The storage freshness,
bundle-latency, object-cost, cold-read, durability, setup, and
operational-complexity gates keep veto power.

### Current Lean

| Axis | Current interpretation |
| --- | --- |
| Retrieval speed | ClickHouse is faster on scans, broad log search, dynamic JSON, joins, and mature analytical SQL — but this is **off Parallax's anchored hot path**, where both engines stay interactive (≪300 ms) when schema keys/indexes are correct. So retrieval speed does not decide the lean. |
| Build-on-top ecosystem | ClickHouse leads: SigNoz, Uptrace, HyperDX, and ClickStack prove a large observability platform surface over ClickHouse. GreptimeDB can express the core grouped-error/evidence-window queries, but the ecosystem is younger. |
| Cost and retention | GreptimeDB remains the important branch for self-hosted 1x object-storage economics, fewer always-on compute assumptions, and deep retained history. ClickHouse can use object storage too, so this must be priced rather than assumed. |
| Metrics/cardinality/PromQL | GreptimeDB remains strategically relevant for PromQL-native and high-cardinality metric workflows. This can flip the storage choice if metrics become a first-class product surface rather than background evidence. |
| Rust/open-source lens | GreptimeDB is the operator-contributable Rust substrate. That matters for a long-term bet, but does not override current retrieval/ecosystem evidence. |
| First build discipline | The first build should not depend on either engine-specific magic. It should keep the schema, bundle, and storage adapter boundaries honest until A5 decides. |

Current source anchors:

- [GreptimeDB docs](https://docs.greptime.com/)
- [GreptimeDB v1.0.2 release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [GreptimeDB OpenTelemetry ingestion](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/)
- [GreptimeDB Prometheus ingestion](https://docs.greptime.com/user-guide/ingest-data/for-observability/prometheus/)
- [GreptimeDB PromQL](https://docs.greptime.com/user-guide/query-data/promql/)
- [GreptimeDB trace read/write docs](https://docs.greptime.com/user-guide/traces/read-write/)
- [GreptimeDB architecture](https://docs.greptime.com/user-guide/concepts/architecture/)
- [GreptimeDB storage options](https://docs.greptime.com/user-guide/deployments-administration/configuration/)

- [ClickHouse observability docs](https://clickhouse.com/docs/use-cases/observability)
- [ClickHouse OpenTelemetry integration](https://clickhouse.com/docs/use-cases/observability/integrating-opentelemetry)
- [ClickHouse object storage docs](https://clickhouse.com/docs/operations/storing-data)
- [ClickHouse v26.5.1.882-stable release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable)
- [ClickHouse v26.3.12.3-lts release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts)
- [ClickHouse production version guidance](https://clickhouse.com/docs/faq/operations/production#how-to-choose-between-clickhouse-releases)
- [Storage benchmark artifact interpretation](../storage/benchmark-plan.md)
- [Platform fit and alternatives](../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md)
- [Storage architecture, cost, and tiering](../storage/greptimedb-vs-clickhouse/storage-cost-and-tiering.md)

## Component Diagram

Tiny single-node:

```text
Rust app / service / CLI / coding agent
  -> Sentry envelope event endpoint for tested SDK fixtures
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
       - CLI / HTTP context API
       - optional MCP adapter after access-surface gate
  -> columnar storage adapter (GreptimeDB lean / ClickHouse fallback)
  -> Turso prototype metadata
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
  -> columnar storage adapter + object storage profile
  -> Turso prototype metadata
  -> parallax-api
  -> optional MCP adapter
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
  -> context API / optional MCP adapter / UI
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
| Adapter provenance | `adapter_name`, `adapter_version`, capture surface, source tool binary/version/config, source schema snapshot, lossiness report |
| Context | prompt refs, repo refs, files read, docs read, Parallax bundles requested, external issue/PR refs |
| Tool calls | MCP/API calls, CLI commands, shell commands, database/query templates, tool result refs |
| Model steps | model/provider metadata when available, token counts when available, reasoning summary refs, confidence |
| Code actions | files edited, patch refs, commits, PR URLs, review comments, rollback/revert refs |
| Validation | tests run, build commands, lint/typecheck results, failure excerpts, skipped checks |
| Outcome | accepted, edited, rejected, reverted, inconclusive, production recurrence, human approver |
| Safety | approvals, denied actions, policy version, hook/plugin source and trust mode, dangerous flags, content-capture level, raw-ref policy, redaction/source-field/projection status, prompt-injection flags |

Parallax stores facts about the agent run. It does not need full private model
reasoning to be useful; it needs enough structured action history to reconstruct
what context the agent used and what it changed.

Do not collapse tool-specific capture surfaces into one support claim. Claude
Code native OTel is separate from Claude print-mode `stream-json`; Codex hooks
are separate from `codex exec --json` JSONL and plugin/managed hook surfaces;
Amp plugin events are separate from execute-mode streaming JSON; and OpenCode
run JSON, export JSON, plugin hooks, server/API, and ACP surfaces are separate.
Product wording must point to the dated tool/version/config matrix in the
[Agent session tracing ledger](../capture/agent-cli-tracing.md), not to generic
"agent tracing" support.

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
2. HTTP API for stable service-to-service integration and the canonical
   schema-bound context contract.
3. MCP tools later for first-class agent clients.

The CLI is the first usable surface. MCP should not be required for Phase 1,
because that would add protocol and security scope before the bundle contract is
stable. The focused decision is captured in
[Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md):
canonical HTTP API first, day-one CLI, and then a read-only MCP adapter once A7
scope discipline and A6 redaction safety are green. All surfaces must call the
same authorization, redaction, and bundle-building code, and every agent-visible
surface must expose the same canonical bundle hash plus projection manifest for
the same anchor, principal, schema version, and redaction policy.

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

First MCP tools, after the access-surface gate:

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

Build the later MCP adapter against the current official spec revision shown by
the MCP site on the research date: **2025-11-25**. Do not cite or implement a
future-dated spec revision until the official site publishes it as current. MCP
is supported by the major checked agent clients (Claude Code, Codex, Cursor,
and Copilot/VS Code), which is why a read-only MCP context adapter is the right
later agent-native surface alongside the CLI.
Bundle-returning MCP tools must declare an output schema and return the bundle
as `structuredContent`; text content is only a bounded Markdown compatibility
projection.

Sources:

- [Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md)
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
  - HTTP API
  - optional read-only MCP adapter after the access-surface gate
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
- CLI and HTTP API available from the same process, with MCP optional after the
  access-surface gate;
- easiest migration path from self-hosted Sentry SDKs.

This is the product that proves Parallax can be simpler than Sentry.
That proof is measured by the
[self-hosted simplicity gate](../validation/self-hosted-simplicity.md) and made claimable
through the [self-hosted simplicity ledger](../validation/self-hosted-simplicity.md),
not assumed from the component diagram.

### Tier 2: Scalable

Target: a team running production Rust services.

```text
parallax-ingest
parallax-worker
greptimedb standalone with object storage
turso metadata
optional Apache Iggy standalone
parallax-api, plus optional MCP adapter after the access-surface gate
parallax CLI
```

Properties:

- stateless ingest split from API;
- raw replay and worker separation if Iggy is enabled;
- object storage for retained telemetry;
- Turso for prototype metadata and audit, with Postgres ready if production
  gates fail;
- still small enough for one VM or a simple Compose deployment.

The explicit seams are ingest, stream, workers, storage, API, and optional MCP. Moving
from Tier 1 to Tier 2 should not change the event contract, the bundle schema,
or the CLI/MCP tool semantics.

### Tier 3: Very Scalable

Target: larger companies and high-volume telemetry.

```text
parallax-ingest x N
iggy cluster or fallback stream
worker pools x N
greptimedb distributed or clickhouse fallback cluster
turso metadata or postgres scale-out fallback
object storage
api nodes x N, plus optional mcp adapter nodes
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
  [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md).
- Evidence-bundle query latency by issue, trace, release window, and metric
  anomaly, with the initial proof gate in
  [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md).
- Retained size and object-storage cost for 7, 30, and 90 days, with the
  initial proof gate in
  [Storage size and object cost gate](../storage/size-and-object-cost.md).
- High-cardinality labels and attributes.
- Cold-cache and hot-cache behavior.

### Stream

- local WAL versus Iggy, with the replay/backpressure gate in
  [Ingest log replay and backpressure gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md).
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

- Rust stacktrace grouping and symbolication stability, with the proof gate in
  [Rust stacktrace grouping and symbolication](../capture/rust.md)
  and claim levels in
  [Rust stacktrace grouping ledger](../capture/rust.md);
- bundle size limits;
- redaction quality;
- prompt-injection resistance;
- evidence citation completeness;
- "inconclusive" behavior when data is missing;
- production database evidence safety, with the initial gate in
  [Production database evidence access gate](../capture/production-db-evidence.md)
  and the claim ledger in
  [Production database evidence ledger](../capture/production-db-evidence.md);
- PR correctness rate by failure class.

### Agent And CLI Execution

- adapter claims across native OTel, hooks/plugins, JSONL and stream JSON,
  export/API/ACP, wrapper, and raw-ref surfaces across Codex, Claude Code, Amp,
  and OpenCode, with the initial adapter/value gate in
  [Agent session tracing across real tools](../capture/agent-cli-tracing.md)
  and result ledger in
  [Agent session tracing ledger](../capture/agent-cli-tracing.md);
- CLI redaction and overhead for args, env, config, stdout, and stderr, with
  the initial gate in [CLI trace overhead and redaction](../capture/agent-cli-tracing.md)
  and result ledger in [CLI trace safety ledger](../capture/agent-cli-tracing.md);
- A6 detector/runtime redaction architecture, with Parallax owning a Rust
  default-deny runtime engine and using external scanners only as offline
  validators, in [Redaction detector toolchain](../capture/redaction.md);
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
  - Turso prototype metadata
  - issue context API
  - no MCP requirement until CLI/HTTP bundle contract and safety gates are green
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
- keep the columnar observability store behind an adapter; the current lean is
  GreptimeDB (not yet settled — cost + Rust, anchored hot path), with ClickHouse
  the fallback for analytics ([storage-engine.md](../decisions/storage-engine.md));
- use no broker in the tiny profile and Apache Iggy in the durable profile;
- build deterministic grouping and evidence graphs before AI claims;
- expose safe CLI/API context before autonomous action, then add MCP after the
  access-surface and redaction gates;
- measure speed, cost, and scaling on Parallax-shaped workloads before claiming
  a storage or stream victory.

This direction is not fundamentally flawed. The flawed version is promising
omniscient AI root cause analysis. The defensible company is an open-source,
self-hosted execution context engine that turns telemetry, deploys, code, CI,
CLI, and agent evidence into bounded, auditable context for humans and coding
agents.
