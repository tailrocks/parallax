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
| Agent data access danger? | Major risk. Use read-only scoped templates, redaction, limits, just-in-time grants, audit logs, and no production mutation in MVP. |
| Why trace agents? | Agents will become a primary system interface. Teams need audit, observability, and question-answering over agent actions, not only final outputs. |

## Prompt Coverage Map

| Prompt area | Repository evidence |
| --- | --- |
| Market and product thesis | [Project thesis](project-thesis.md), [Market landscape](market-landscape.md), [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md) |
| Evaluation lens and benchmark methodology | [Observability storage benchmark plan](observability-storage-benchmark-plan.md), [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md), [Messaging and ingestion layer](messaging-and-ingestion-layer.md) |
| Language/runtime filter and Rust preference | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md), [Technical implementation concept](technical-implementation-concept.md) |
| Messaging/streaming | [Messaging and ingestion layer](messaging-and-ingestion-layer.md) |
| Unified observability storage | [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md), [Observability storage benchmark plan](observability-storage-benchmark-plan.md) |
| OpenTelemetry | [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md) |
| Sentry-compatible ingestion | [Sentry-compatible ingestion](sentry-compatible-ingestion.md) |
| Collection method and eBPF tradeoff | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md) |
| Rust applications first | [Rust data collection and instrumentation](rust-data-collection-and-instrumentation.md), [Technical implementation concept](technical-implementation-concept.md) |
| AI-native observability | [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md), [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md) |
| Flaky-test investigation | [CI failure context MVP](ci-failure-context-mvp.md), [Flaky test investigation and replay](flaky-test-investigation-and-replay.md) |
| Agent and CLI execution tracing | [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) |
| Agent-observability technical references | [Agent observability technical review](agent-observability-technical-review.md) |
| Core architecture | [Self-hosted observability architecture](self-hosted-observability-architecture.md), [Technical implementation concept](technical-implementation-concept.md) |
| CLI/API/MCP philosophy | [Self-hosted observability architecture](self-hosted-observability-architecture.md), [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md), [Technical implementation concept](technical-implementation-concept.md) |
| Critical strategic questions | [AI-native observability and incident intelligence](ai-native-observability-and-incident-intelligence.md), this document |
| Final implementation blueprint | [Technical implementation concept](technical-implementation-concept.md) |

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

The research validates direction, not performance claims. These must be tested:

1. GreptimeDB ingest-to-queryable freshness for mixed logs/traces/metrics/errors.
2. Evidence-bundle query latency under concurrent ingest.
3. GreptimeDB versus ClickHouse storage size and object-storage cost.
4. Iggy replay and backpressure behavior versus local WAL and NATS/Redpanda.
5. Sentry envelope compatibility across real SDKs.
6. Rust stacktrace grouping stability across release/debug-info variants.
7. Agent fix quality with bounded Parallax bundles versus raw Sentry/CI context.
8. Redaction quality for logs, events, attachments, database query output, and
   agent prompt bundles.
9. CLI trace capture overhead and secret redaction for args, env, config,
   stdout, and stderr.
10. Agent-session tracing value across real Codex, Claude Code, Amp, and
    OpenCode runs.

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

## Final Position

Proceed with Parallax, but keep the claim precise:

> Parallax does not prove every root cause. It makes the best available runtime,
> CI, CLI, deploy, repo, and agent evidence cheap to retain, fast to query, safe
> to expose, and structured enough for humans and agents to act on.

That is the buildable company.
