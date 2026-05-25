# Strategic Verdict and Research Coverage

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This is the final synthesis for the Parallax deep-research prompt. It ties the
research notes into one verdict and maps the prompt requirements to current
repository evidence.

Version freshness rule: all comparisons must use the latest reasonably
available stable/public version of each candidate as of the research date. Older
benchmarks, architecture posts, and feature matrices are historical evidence
unless explicitly rechecked.

## Verdict

Parallax is not fundamentally flawed if it stays narrow and evidence-first.

The flawed version is:

> A generic AI root-cause chatbot over logs, metrics, and traces.

That version is crowded, easy for Sentry/Datadog/Grafana to absorb, unsafe for
agents, and likely to overpromise causality.

The defensible version is:

> An open-source, Rust-first, self-hostable execution context engine that
> accepts Sentry-compatible errors, OTLP telemetry, CLI invocation traces, and
> coding-agent session traces, builds deterministic evidence graphs, and serves
> bounded context bundles to humans and coding agents through API/MCP.

That version can become the intelligence layer between telemetry systems, CI,
CLI tools, deploys, issue trackers, repos, and autonomous coding agents.

The strategic reason is that agent-driven work makes action trails harder to
see. As more engineering and operations work moves through agents, teams need an
audit system that can answer what happened, who or what initiated it, what the
agent saw, what tools it used, what it changed, and whether the result fixed or
worsened the original problem.

## Build Direction

Build this first:

```text
Rust service / CLI / coding agent
  -> Sentry-compatible envelope ingest
  -> OTLP logs/traces/metrics ingest
  -> CLI invocation and agent-session trace ingest
  -> Parallax Rust ingest gateway
  -> local WAL for tiny mode
  -> GreptimeDB for observability evidence
  -> Turso for metadata/product state
  -> deterministic grouping/correlation/evidence graph
  -> API/MCP context bundle
```

Add Apache Iggy only when replay, burst buffering, or worker separation is worth
the extra process. Keep ClickHouse as the latest-version benchmark fallback if
GreptimeDB fails Parallax-shaped storage tests.

## Direct Answers To Strategic Questions

| Question | Answer |
| --- | --- |
| Company-sized? | Yes, if framed as context/evidence layer for autonomous software maintenance. No, if framed as generic AI RCA. |
| Just a Sentry/Grafana feature? | Generic AI investigation is a feature. Self-hosted Rust-first evidence bundles plus open schema plus agent workflow can be a product. |
| Market too crowded? | Broad market is crowded. Narrow wedge remains: Sentry-compatible, OTLP-native, self-hosted execution context for services, CI, CLIs, and coding agents. |
| Hardest technical problems? | High-quality error grouping, cross-signal joins, symbolication, missing-data handling, causal graph modeling, retention cost, redaction, safe agent tools. |
| Hidden operational problems? | Cardinality, schema evolution, object-storage cost, backpressure, retries/duplicates, symbol files, tenant isolation, upgrade path, source/release mapping. |
| Scaling bottlenecks? | Ingest-to-queryable freshness, trace/log joins, high-cardinality attributes, stream replay, GreptimeDB/ClickHouse compaction, evidence-bundle query fanout. |
| Commodity parts? | OTLP ingestion, natural-language summaries, basic log search, dashboards, anomaly alerts, generic chat assistant. |
| Moat? | Failure corpus, accepted-fix feedback, open evidence schema, Rust capture quality, repo intent integration, safe agent policy. |
| True moat category? | Evidence graph plus failure/fix feedback loop. Agent integration matters, but only if grounded in evidence. |
| Does AI change architecture? | Yes. It shifts priority from dashboards to machine-readable context, evidence edges, APIs, and MCP tools. |
| Can agents open correct PRs? | Sometimes. High for clear app errors and deterministic CI failures; low for data corruption, privacy, infra outages, broad multi-service incidents. |
| Is lifecycle reconstruction achievable? | Partially. Strong for traced request/workflow/test lifecycles; weak for arbitrary true root cause without topology, change data, and counterfactual evidence. |
| Monorepo dependency? | Context-rich repos make Parallax much stronger. Without docs/tasks/decisions, Parallax still helps with runtime evidence but loses the "why" layer. |
| Agent data access danger? | Major risk. Use read-only scoped templates, default-deny redaction, limits, just-in-time grants, audit logs, and no production mutation in MVP; see the [production database evidence access gate](production-database-evidence-access.md). |
| Why trace agents? | Agents will become a primary system interface. Teams need audit, observability, and question-answering over agent actions, not only final outputs. |

## Prompt Coverage Map

| Prompt area | Repository evidence |
| --- | --- |
| Market and product thesis | [Project thesis](project-thesis.md), [Market landscape](market-landscape.md), [Open self-hosted competitor watch](open-self-hosted-competitor-watch.md), [Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md), [Agentic observability competitor drift ledger](agentic-observability-competitor-drift-ledger.md), [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md), [Repo-intent dependence](repo-intent-dependence.md), [Repo-intent value ledger](repo-intent-value-ledger.md), [Business model and economics](business-model-and-economics.md), [Business model validation ledger](business-model-validation-ledger.md), [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md), [A2 interview evidence ledger](a2-interview-evidence-ledger.md), [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md), [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md) |
| Evaluation lens and benchmark methodology | [Observability storage benchmark plan](observability-storage-benchmark-plan.md), [Storage benchmark prototype (runnable)](storage-benchmark-prototype.md), [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md), [Storage size and object cost gate](storage-size-and-object-cost-gate.md), [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md), [A5 stack decision ledger](a5-stack-decision-ledger.md), [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md), [Messaging and ingestion layer](messaging-and-ingestion-layer.md) |
| Language/runtime filter and Rust preference | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md), [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md), [Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md), [Technical implementation concept](technical-implementation-concept.md) |
| Messaging/streaming | [Messaging and ingestion layer](messaging-and-ingestion-layer.md), [Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md) |
| Unified observability storage | [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md), [Observability storage benchmark plan](observability-storage-benchmark-plan.md), [Storage benchmark prototype (runnable)](storage-benchmark-prototype.md), [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md), [Storage size and object cost gate](storage-size-and-object-cost-gate.md), [A5 stack decision ledger](a5-stack-decision-ledger.md) |
| Metadata store | [Metadata store benchmark plan and prototype](metadata-store-benchmark-plan.md), [Turso metadata production readiness](turso-metadata-production-readiness.md), [Technical implementation concept](technical-implementation-concept.md) |
| OpenTelemetry | [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md), [OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md), [OTLP conformance ledger](otlp-conformance-ledger.md) |
| Sentry-compatible ingestion | [Sentry-compatible ingestion](sentry-compatible-ingestion.md), [Sentry SDK fixture compatibility gate](sentry-sdk-fixture-compatibility.md), [Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md) |
| Self-hosted operational simplicity | [Self-hosted simplicity gate](self-hosted-simplicity-gate.md), [Self-hosted deployment baseline inventory](self-hosted-deployment-baseline-inventory.md), [Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md), [A7 scope discipline ledger](a7-scope-discipline-ledger.md), [Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md), [Self-hosted observability architecture](self-hosted-observability-architecture.md), [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md) |
| Collection method and eBPF tradeoff | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md) |
| Rust applications first | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md), [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md), [Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md), [Technical implementation concept](technical-implementation-concept.md) |
| AI-native observability | [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md), [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md) |
| Flaky-test investigation | [CI failure context MVP](ci-failure-context-mvp.md), [Flaky test investigation and replay](flaky-test-investigation-and-replay.md) |
| Deploy/change/issue tracker context | [Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md), [Deploy/change context ledger](deploy-change-context-ledger.md), [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md), [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md) |
| Agent and CLI execution tracing | [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md), [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md), [Agent session tracing across real tools](agent-session-tracing-real-tools.md), [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md) |
| Fixer and outcome loop | [Fixer component and outcome loop](fixer-component-and-outcome-loop.md), [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md), [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md) |
| Agent-observability technical references | [Agent observability technical review](agent-observability-technical-review.md), [Agent session tracing across real tools](agent-session-tracing-real-tools.md) |
| Frontend collection and cross-tier correlation | [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md), [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md), [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md), [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Storage benchmark prototype](storage-benchmark-prototype.md) |
| Redaction/privacy/agent exposure safety | [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md), [Redaction detector toolchain](redaction-detector-toolchain.md), [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md), [Production database evidence access gate](production-database-evidence-access.md), [Agent session tracing across real tools](agent-session-tracing-real-tools.md), [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md), [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Frontend collection and cross-tier correlation](frontend-collection-and-cross-tier-correlation.md), [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) |
| Evidence bundle and open schema | [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md), [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md), [Bundle-value evaluation](bundle-value-evaluation.md), [Bundle-value seed corpus](bundle-value-seed-corpus.md), [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md), [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md), [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md) |
| Core architecture | [Self-hosted observability architecture](self-hosted-observability-architecture.md), [Technical implementation concept](technical-implementation-concept.md) |
| CLI/API/MCP philosophy | [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md), [Agent access surface safety ledger](agent-access-surface-safety-ledger.md), [Self-hosted observability architecture](self-hosted-observability-architecture.md), [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md), [Technical implementation concept](technical-implementation-concept.md) |
| Critical strategic questions | [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md), this document |
| Final implementation blueprint | [Technical implementation concept](technical-implementation-concept.md), [Evidence bundle and open schema specification](evidence-bundle-and-schema.md), [Storage benchmark prototype (runnable)](storage-benchmark-prototype.md) |

## Key Decisions

| Layer | Decision |
| --- | --- |
| App collection | Rust `tracing`, `tracing-error`, `opentelemetry-otlp`, panic hooks, error-chain capture, Sentry-compatible path. |
| External protocols | Sentry envelopes and OTLP HTTP/gRPC. |
| Ingest | Rust `parallax-ingest` gateway. |
| Stream | No external broker for tiny mode; Apache Iggy for durable profile. |
| Observability storage | GreptimeDB default v0.1; ClickHouse benchmark fallback. |
| Metadata | Turso for tiny/local metadata and product state; Postgres only as scale-out fallback. |
| Processing | Rust workers, deterministic normalization/grouping/correlation before AI. |
| Context model | Typed evidence graph in tables first. |
| Execution surfaces | Services, CI runs, CLI apps, and coding agents. |
| Agent surface | Read-only API/MCP context first; PR workflow later; no production mutation. |
| UI | Minimal issue/evidence UI later; object-centric evidence, not dashboard suite. |

## What Is Still Unproven

The research validates direction, not demand or performance claims. These must
be tested:

Market/product gates:

- A2 real-user demand beyond the operator, using the
  [User interview and deployment intent gate](user-interview-and-deployment-intent-gate.md)
  and [A2 interview evidence ledger](a2-interview-evidence-ledger.md).
- Business-model validation for hosted, fixer, enterprise ops, support/services,
  conversion, and paid-pilot seams, using the
  [business model validation ledger](business-model-validation-ledger.md).
- Repo-intent dependence and degraded-mode breadth, using
  [Repo-intent dependence](repo-intent-dependence.md) and
  [Repo-intent value ledger](repo-intent-value-ledger.md). Runtime-only bundles
  must remain useful for teams without curated docs, decisions, or roadmap.
- A3 schema/corpus moat, using the
  [Schema adoption and corpus moat gate](schema-adoption-and-corpus-moat-gate.md)
  and [A3 schema adoption and corpus ledger](a3-schema-adoption-corpus-ledger.md).

Technical proof gates:

- A5 stack-decision roll-up across storage speed/cost, metadata, ingest-log,
  setup, and integration rows, specified in the
  [A5 stack decision ledger](a5-stack-decision-ledger.md). This ledger controls
  when component benchmarks are allowed to become stack-default claims.

- A7 scope-discipline roll-up across component inventory, dependency rows,
  feature intake, interface surfaces, and phase budgets, specified in the
  [A7 scope discipline ledger](a7-scope-discipline-ledger.md). This ledger
  controls when roadmap breadth is allowed to enter active build scope.

- Deterministic cross-signal correlation reliability on real telemetry,
  specified further in the
  [Correlation reliability on real telemetry gate](correlation-reliability-real-telemetry-gate.md)
  and made auditable by the
  [A4 correlation reliability ledger](a4-correlation-reliability-ledger.md).

1. GreptimeDB ingest-to-queryable freshness for mixed logs/traces/metrics/errors,
   specified further in
   [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
2. Evidence-bundle query latency under concurrent ingest, specified further in
   [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md).
3. GreptimeDB versus ClickHouse storage size and object-storage cost, specified
   further in
   [Storage size and object cost gate](storage-size-and-object-cost-gate.md).
4. Iggy replay and backpressure behavior versus local WAL and NATS/Redpanda,
   specified further in
   [Ingest log replay and backpressure gate](ingest-log-replay-and-backpressure-gate.md).
5. Sentry envelope compatibility across real SDKs, starting with the
   [Sentry SDK fixture gate](sentry-sdk-fixture-compatibility.md) and made
   claimable only through the
   [Sentry SDK compatibility ledger](sentry-sdk-compatibility-ledger.md).
6. Phase 1 setup simplicity versus current Sentry, SigNoz, and OpenObserve
   baselines, specified further in the
   [Self-hosted simplicity gate](self-hosted-simplicity-gate.md) and made
   claimable through the
   [Self-hosted simplicity ledger](self-hosted-simplicity-ledger.md).
7. Rust stacktrace grouping stability across release/debug-info variants,
   specified as a proof gate in
   [Rust stacktrace grouping and symbolication](rust-stacktrace-grouping-and-symbolication.md)
   and made claimable through the
   [Rust stacktrace grouping ledger](rust-stacktrace-grouping-ledger.md).
8. Agent fix quality with bounded Parallax bundles versus raw Sentry/CI context,
   with the first task-source selection specified in the
   [Bundle-value seed corpus](bundle-value-seed-corpus.md), the no-cheat
   telemetry overlay specified in
   [Phase 0 telemetry overlay contract](phase0-telemetry-overlay-contract.md),
   the first runnable pass specified in
   [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md), and the
   claim ledger specified in
   [A1 eval result ledger and model refresh](a1-eval-result-ledger-and-model-refresh.md).
9. Redaction quality for logs, events, attachments, database query output, and
   agent prompt bundles; the [redaction pipeline](redaction-pipeline-and-secret-safety.md)
   and [redaction detector toolchain](redaction-detector-toolchain.md) have veto
   power before agent exposure, with auditable results specified in the
   [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md).
10. CLI trace capture overhead and secret redaction for args, env, config,
   stdout, and stderr, specified further in
   [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md).
11. Agent-session tracing value across real Codex, Claude Code, Amp, and
    OpenCode runs, specified further in
    [Agent session tracing across real tools](agent-session-tracing-real-tools.md),
    with OpenTelemetry GenAI/MCP/CLI normalization specified in
    [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md).
12. Turso correctness, backup/restore, concurrency, migration, and fallback
    behavior for metadata, agent session state, CLI invocation state, outcomes,
    and audit records, specified further in
    [Turso metadata production readiness](turso-metadata-production-readiness.md).
13. CLI/HTTP/MCP projection equivalence and read-only MCP safety, specified
    further in
    [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
    and made claimable through the
    [Agent access surface safety ledger](agent-access-surface-safety-ledger.md).
14. Production database evidence access safety, specified further in
    [Production database evidence access gate](production-database-evidence-access.md)
    and made claimable through the
    [Production database evidence ledger](production-database-evidence-ledger.md).
15. OTLP receiver conformance and direct-SDK/Collector/Rotel normalization
    equivalence, specified further in
    [OTLP receiver conformance and Collector equivalence](otlp-receiver-conformance-and-collector-equivalence.md)
    and made claimable through the
    [OTLP conformance ledger](otlp-conformance-ledger.md).
16. Release/deploy/code-change/work-item context completeness and edge strength,
    specified further in
    [Deploy, change, and issue-tracker context](deploy-change-and-issue-context.md)
    and made claimable through the
    [Deploy/change context ledger](deploy-change-context-ledger.md).

## First Prototype Gate

Prototype should prove this loop:

```text
Rust app error
  -> Sentry-compatible event
  -> OTLP trace/log context
  -> optional CLI invocation or coding-agent session context
  -> Parallax grouping
  -> GreptimeDB evidence query
  -> Turso issue metadata
  -> bounded MCP/API bundle
  -> agent produces diagnosis or PR proposal with evidence refs
```

Acceptance criteria:

- event visible within target freshness window;
- error grouped deterministically;
- trace/log context joined by `trace_id`/`span_id`;
- release/commit context attached;
- evidence bundle includes raw refs and redaction report;
- CLI and agent traces link back to source events, commands, tests, files, and
  outcome refs when available;
- agent output cites evidence and says "inconclusive" when evidence is weak.
- fixer or agent PR workflows write outcome records instead of treating opened
  pull requests as successful fixes; see the
  [fixer component and outcome loop](fixer-component-and-outcome-loop.md).

## Final Position

Proceed with Parallax, but keep the claim precise:

> Parallax does not prove every root cause. It makes the best available runtime,
> CI, CLI, deploy, repo, and agent evidence cheap to retain, fast to query, safe
> to expose, and structured enough for humans and agents to act on.

That is the buildable company.
