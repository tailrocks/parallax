# Strategic Verdict and Research Coverage

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25 · Restructured into a decision record 2026-05-29

> **Decision record — strategic synthesis and coverage map.** Status: GO (narrow), tying the
> research notes into one verdict and mapping every prompt area to its evidence. The current
> open proof gates are enumerated in "What Is Still Unproven" below. See
> [go-no-go.md](go-no-go.md) for the verdict, [risks-and-bear-case.md](risks-and-bear-case.md)
> for the bear case, and [storage-engine.md](storage-engine.md) for the engine decision.

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
> accepts fixture-gated Sentry envelope error events, OTLP telemetry, CLI
> invocation traces, and coding-agent session records from tested capture
> adapters, builds
> deterministic evidence graphs, and serves bounded context bundles to humans
> and coding agents through API first and MCP after projection/safety gates.

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
  -> fixture-gated Sentry envelope event ingest
  -> OTLP logs/traces/metrics ingest
  -> CLI invocation trace ingest and tested agent-session adapter ingest
  -> Parallax Rust ingest gateway
  -> local WAL for tiny mode
  -> columnar storage adapter (GreptimeDB lean / ClickHouse fallback)
  -> Turso prototype metadata / Postgres production fallback
  -> deterministic grouping/correlation/evidence graph
  -> API context bundle / later read-only MCP projection
```

Add Apache Iggy only when replay, burst buffering, or worker separation is worth
the extra process. Keep ClickHouse and GreptimeDB runnable behind the storage
adapter; the current lean is **GreptimeDB (not yet settled)** — the resolved
anchored-retrieval query mix takes ClickHouse's scan-speed lead off the hot
path, so cost + Rust decide — with ClickHouse the fallback until Parallax-shaped
storage gates settle it (see [storage-engine.md](storage-engine.md)).

## Direct Answers To Strategic Questions

| Question | Answer |
| --- | --- |
| Company-sized? | Yes, if framed as context/evidence layer for autonomous software maintenance. No, if framed as generic AI RCA. |
| Just a Sentry/Grafana feature? | Generic AI investigation is a feature. Self-hosted Rust-first evidence bundles plus open schema plus agent workflow can be a product. |
| Market too crowded? | Broad market is crowded. Narrow wedge remains: fixture-gated Sentry envelope error migration, OTLP-conformance-gated ingest, and self-hosted execution context for services, CI, CLIs, and coding agents. |
| Hardest technical problems? | High-quality error grouping, cross-signal joins, symbolication, missing-data handling, causal graph modeling, retention cost, redaction, safe agent tools. |
| Hidden operational problems? | Cardinality, schema evolution, object-storage cost, backpressure, retries/duplicates, symbol files, tenant isolation, upgrade path, source/release mapping. |
| Scaling bottlenecks? | Ingest-to-queryable freshness, trace/log joins, high-cardinality attributes, stream replay, GreptimeDB/ClickHouse compaction, evidence-bundle query fanout. |
| Commodity parts? | OTLP ingestion, natural-language summaries, basic log search, dashboards, anomaly alerts, generic chat assistant. |
| Moat? | Failure/fixer-outcome corpus with accepted, rejected, reverted, and recurrence rows; open evidence schema; Rust capture quality; repo intent integration; safe agent policy. |
| True moat category? | Evidence graph plus measured fixer outcome loop. Agent integration matters, but only if grounded in evidence. |
| Does AI change architecture? | Yes. It shifts priority from dashboards to machine-readable context, evidence edges, APIs, and MCP tools. |
| Can agents open correct PRs? | Sometimes. High for clear app errors and deterministic CI failures; low for data corruption, privacy, infra outages, broad multi-service incidents. |
| Is lifecycle reconstruction achievable? | Partially. Strong for traced request/workflow/test lifecycles; weak for arbitrary true root cause without topology, change data, and counterfactual evidence. |
| Monorepo dependency? | Context-rich repos make Parallax much stronger. Without docs/tasks/decisions, Parallax still helps with runtime evidence but loses the "why" layer. |
| Agent data access danger? | Major risk. Use read-only scoped templates, default-deny redaction, limits, just-in-time grants, audit logs, and no production mutation in MVP; see the [production database evidence access gate](../capture/production-db-evidence.md). |
| Why trace agents? | Agents will become a primary system interface. Teams need audit, observability, and question-answering over agent actions, not only final outputs. |

## Prompt Coverage Map

| Prompt area | Repository evidence |
| --- | --- |
| Market and product thesis | [Project thesis](../00-vision/thesis.md), [Market landscape](../market/landscape.md), [Open self-hosted competitor watch](../market/competitor-watch.md), [Lightweight Sentry-compatible competitor watch](../market/competitor-watch.md), [Agentic observability competitor drift ledger](../market/competitor-watch.md), [AI-native observability and incident intelligence](../00-vision/ai-native-observability.md), [Repo-intent dependence](../validation/repo-intent.md), [Repo-intent value ledger](../validation/repo-intent.md), [Business model and economics](../validation/business-model.md), [Business model validation ledger](../validation/business-model.md), [User interview and deployment intent gate](../validation/a2-user-demand.md), [A2 interview evidence ledger](../validation/a2-user-demand.md), [Schema adoption and corpus moat gate](../validation/a3-schema-corpus.md), [A3 schema adoption and corpus ledger](../validation/a3-schema-corpus.md) |
| Evaluation lens and benchmark methodology | [Observability storage benchmark plan](../storage/benchmark-plan.md), [Storage benchmark prototype (runnable)](../storage/benchmark-plan.md), [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md), [Storage size and object cost gate](../storage/size-and-object-cost.md), [Metadata store benchmark plan and prototype](../storage/metadata/metadata-store-benchmark-plan.md), [A5 stack decision ledger](stack-decision.md), [GreptimeDB storage evaluation](../storage/evaluation.md), [Messaging and ingestion layer](../storage/streaming/messaging-and-ingestion-layer.md) |
| Language/runtime filter and Rust preference | [Rust data collection and instrumentation](../capture/rust.md), [Rust stacktrace grouping and symbolication](../capture/rust.md), [Rust stacktrace grouping ledger](../capture/rust.md), [Technical implementation concept](../architecture/implementation-concept.md) |
| Messaging/streaming | [Messaging and ingestion layer](../storage/streaming/messaging-and-ingestion-layer.md), [Ingest log replay and backpressure gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md) |
| Unified observability storage | [GreptimeDB storage evaluation](../storage/evaluation.md), [Observability storage benchmark plan](../storage/benchmark-plan.md), [Storage benchmark prototype (runnable)](../storage/benchmark-plan.md), [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md), [Storage size and object cost gate](../storage/size-and-object-cost.md), [A5 stack decision ledger](stack-decision.md) |
| Metadata store | [Metadata store benchmark plan and prototype](../storage/metadata/metadata-store-benchmark-plan.md), [Turso metadata production readiness](../storage/metadata/turso-metadata-production-readiness.md), [Technical implementation concept](../architecture/implementation-concept.md) |
| OpenTelemetry | [OpenTelemetry protocol and context layer](../capture/otlp.md), [OTLP receiver conformance and Collector equivalence](../capture/otlp.md), [OTLP conformance ledger](../capture/otlp.md) |
| Sentry envelope compatibility | [Sentry-compatible ingestion](../capture/sentry-ingest.md), [Sentry SDK fixture compatibility gate](../capture/sentry-ingest.md), [Sentry SDK compatibility ledger](../capture/sentry-ingest.md) |
| Self-hosted operational simplicity | [Self-hosted simplicity gate](../validation/self-hosted-simplicity.md), [Self-hosted deployment baseline inventory](../validation/self-hosted-simplicity.md), [Self-hosted simplicity ledger](../validation/self-hosted-simplicity.md), [A7 scope discipline ledger](../validation/a7-scope.md), [Lightweight Sentry-compatible competitor watch](../market/competitor-watch.md), [Self-hosted observability architecture](../architecture/overview.md), [Build roadmap and validation sequence](../architecture/build-roadmap.md) |
| Collection method and eBPF tradeoff | [Rust data collection and instrumentation](../capture/rust.md) |
| Rust applications first | [Rust data collection and instrumentation](../capture/rust.md), [Rust stacktrace grouping and symbolication](../capture/rust.md), [Rust stacktrace grouping ledger](../capture/rust.md), [Technical implementation concept](../architecture/implementation-concept.md) |
| AI-native observability | [AI-native observability and incident intelligence](../00-vision/ai-native-observability.md), [Causal reconstruction and agent safety](../architecture/causal-reconstruction.md) |
| Flaky-test investigation | [CI failure context MVP](../capture/ci-and-flaky-tests.md), [Flaky test investigation and replay](../capture/ci-and-flaky-tests.md) |
| Deploy/change/issue tracker context | [Deploy, change, and issue-tracker context](../capture/deploy-change-context.md), [Deploy/change context ledger](../capture/deploy-change-context.md), [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Correlation reliability on real telemetry gate](../capture/correlation.md), [A4 correlation reliability ledger](../capture/correlation.md) |
| Agent and CLI execution tracing | [Agent and CLI execution tracing](../capture/agent-cli-tracing.md), [Agent and CLI OTel semantic-convention mapping](../capture/agent-cli-tracing.md), [Agent session tracing across real tools](../capture/agent-cli-tracing.md), [Agent session tracing ledger](../capture/agent-cli-tracing.md), [CLI trace overhead and redaction](../capture/agent-cli-tracing.md), [CLI trace safety ledger](../capture/agent-cli-tracing.md) |
| Fixer and outcome loop | [Fixer component and outcome loop](fixer-boundary.md), [Fixer outcome ledger](fixer-boundary.md), [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Agent and CLI execution tracing](../capture/agent-cli-tracing.md), [Schema adoption and corpus moat gate](../validation/a3-schema-corpus.md) |
| Agent-observability technical references | [Agent observability technical review](../reference/agent-observability-review.md), [Agent session tracing across real tools](../capture/agent-cli-tracing.md) |
| Frontend collection and cross-tier correlation | [Frontend collection and cross-tier correlation](../capture/frontend.md), [Frontend capture safety ledger](../capture/frontend.md), [Correlation reliability on real telemetry gate](../capture/correlation.md), [A4 correlation reliability ledger](../capture/correlation.md), [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Storage benchmark prototype](../storage/benchmark-plan.md) |
| Redaction/privacy/agent exposure safety | [Redaction pipeline and secret safety](../capture/redaction.md), [Redaction detector toolchain](../capture/redaction.md), [A6 redaction red-team ledger](../capture/redaction.md), [Production database evidence access gate](../capture/production-db-evidence.md), [Agent session tracing across real tools](../capture/agent-cli-tracing.md), [CLI trace overhead and redaction](../capture/agent-cli-tracing.md), [CLI trace safety ledger](../capture/agent-cli-tracing.md), [Frontend capture safety ledger](../capture/frontend.md), [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Frontend collection and cross-tier correlation](../capture/frontend.md), [Agent and CLI execution tracing](../capture/agent-cli-tracing.md) |
| Evidence bundle and open schema | [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Schema adoption and corpus moat gate](../validation/a3-schema-corpus.md), [A3 schema adoption and corpus ledger](../validation/a3-schema-corpus.md), [Bundle-value evaluation](../validation/a1-bundle-value/bundle-value-evaluation.md), [Bundle-value seed corpus](../validation/a1-bundle-value/bundle-value-seed-corpus.md), [A1 task source freeze check](../validation/a1-bundle-value/a1-task-source-freeze-check.md), [Phase 0 telemetry overlay contract](../validation/a1-bundle-value/phase0-telemetry-overlay-contract.md), [Bundle-value Phase 0 runbook](../validation/a1-bundle-value/bundle-value-phase0-runbook.md), [A1 eval result ledger and model refresh](../validation/a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md) |
| Core architecture | [Self-hosted observability architecture](../architecture/overview.md), [Technical implementation concept](../architecture/implementation-concept.md) |
| CLI/API/MCP philosophy | [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface.md), [Agent access surface safety ledger](agent-access-surface.md), [Self-hosted observability architecture](../architecture/overview.md), [Causal reconstruction and agent safety](../architecture/causal-reconstruction.md), [Technical implementation concept](../architecture/implementation-concept.md) |
| Critical strategic questions | [AI-native observability and incident intelligence](../00-vision/ai-native-observability.md), this document |
| Final implementation blueprint | [Technical implementation concept](../architecture/implementation-concept.md), [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md), [Storage benchmark prototype (runnable)](../storage/benchmark-plan.md) |

## Key Decisions

| Layer | Decision |
| --- | --- |
| App collection | Rust `tracing`, `tracing-error`, `opentelemetry-otlp`, panic hooks, error-chain capture, Sentry envelope error path. |
| External protocols | Sentry envelope `event` subset and OTLP HTTP/gRPC. |
| Ingest | Rust `parallax-ingest` gateway. |
| Stream | No external broker for tiny mode; Apache Iggy for durable profile. |
| Observability storage | Storage adapter; current lean GreptimeDB (not settled, see [storage-engine.md](storage-engine.md)), ClickHouse fallback, until A5 gates decide. |
| Metadata | Turso Database for prototype/tiny local metadata; Postgres remains the production and scale-out fallback until Turso passes the production-readiness gate. |
| Processing | Rust workers, deterministic normalization/grouping/correlation before AI. |
| Context model | Typed evidence graph in tables first. |
| Execution surfaces | Services, CI runs, CLI apps, and coding agents. |
| Agent surface | CLI/HTTP context first; read-only MCP after the access-surface gate; PR workflow later; no production mutation. |
| UI | Minimal issue/evidence UI later; object-centric evidence, not dashboard suite. |

## What Is Still Unproven

The research validates direction, not demand or performance claims. These must
be tested:

Market/product gates:

- A2 real-user demand beyond the operator, using the
  [User interview and deployment intent gate](../validation/a2-user-demand.md)
  and [A2 interview evidence ledger](../validation/a2-user-demand.md).
- Business-model validation for hosted, fixer, enterprise ops, support/services,
  conversion, and paid-pilot seams, using the
  [business model validation ledger](../validation/business-model.md).
- Repo-intent dependence and degraded-mode breadth, using
  [Repo-intent dependence](../validation/repo-intent.md) and
  [Repo-intent value ledger](../validation/repo-intent.md). Runtime-only bundles
  must remain useful for teams without curated docs, decisions, or roadmap.
- A3 schema/corpus moat, using the
  [Schema adoption and corpus moat gate](../validation/a3-schema-corpus.md)
  and [A3 schema adoption and corpus ledger](../validation/a3-schema-corpus.md).

Technical proof gates:

- A5 stack-decision roll-up across storage speed/cost, metadata, ingest-log,
  setup, and integration rows, specified in the
  [A5 stack decision ledger](stack-decision.md). This ledger controls
  when component benchmarks are allowed to become stack-default claims.

- A7 scope-discipline roll-up across component inventory, dependency rows,
  feature intake, interface surfaces, and phase budgets, specified in the
  [A7 scope discipline ledger](../validation/a7-scope.md). This ledger
  controls when roadmap breadth is allowed to enter active build scope.

- Deterministic cross-signal correlation reliability on real telemetry,
  specified further in the
  [Correlation reliability on real telemetry gate](../capture/correlation.md)
  and made auditable by the
  [A4 correlation reliability ledger](../capture/correlation.md).

1. GreptimeDB ingest-to-queryable freshness for mixed logs/traces/metrics/errors,
   specified further in
   [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md).
2. Evidence-bundle query latency under concurrent ingest, specified further in
   [Storage freshness and bundle latency gate](../storage/freshness-and-latency.md).
3. GreptimeDB versus ClickHouse storage size and object-storage cost, specified
   further in
   [Storage size and object cost gate](../storage/size-and-object-cost.md).
4. Iggy replay and backpressure behavior versus local WAL and NATS/Redpanda,
   specified further in
   [Ingest log replay and backpressure gate](../storage/streaming/ingest-log-replay-and-backpressure-gate.md).
5. Sentry envelope compatibility across real SDKs, starting with the
   [Sentry SDK fixture gate](../capture/sentry-ingest.md) and made
   claimable only through the
   [Sentry SDK compatibility ledger](../capture/sentry-ingest.md).
6. Phase 1 setup simplicity versus current Sentry, SigNoz, and OpenObserve
   baselines, specified further in the
   [Self-hosted simplicity gate](../validation/self-hosted-simplicity.md) and made
   claimable through the
   [Self-hosted simplicity ledger](../validation/self-hosted-simplicity.md).
7. Rust stacktrace grouping stability across release/debug-info variants,
   specified as a proof gate in
   [Rust stacktrace grouping and symbolication](../capture/rust.md)
   and made claimable through the
   [Rust stacktrace grouping ledger](../capture/rust.md).
8. Agent fix quality with bounded Parallax bundles versus raw Sentry/CI context,
   with the first task-source selection specified in the
   [Bundle-value seed corpus](../validation/a1-bundle-value/bundle-value-seed-corpus.md), the no-cheat
   telemetry overlay specified in
   [Phase 0 telemetry overlay contract](../validation/a1-bundle-value/phase0-telemetry-overlay-contract.md),
   the first runnable pass specified in
   [Bundle-value Phase 0 runbook](../validation/a1-bundle-value/bundle-value-phase0-runbook.md), and the
   claim ledger specified in
   [A1 eval result ledger and model refresh](../validation/a1-bundle-value/a1-eval-result-ledger-and-model-refresh.md).
9. Redaction quality for logs, events, attachments, database query output, and
   agent prompt bundles; the [redaction pipeline](../capture/redaction.md)
   and [redaction detector toolchain](../capture/redaction.md) have veto
   power before agent exposure, with auditable results specified in the
   [A6 redaction red-team ledger](../capture/redaction.md).
10. CLI trace capture overhead and secret redaction for args, env, config,
   stdout, and stderr, specified further in
   [CLI trace overhead and redaction](../capture/agent-cli-tracing.md) and
   made claimable through the
   [CLI trace safety ledger](../capture/agent-cli-tracing.md).
11. Agent-session capture value across real Codex, Claude Code, Amp, and
    OpenCode runs, specified further in
    [Agent session tracing across real tools](../capture/agent-cli-tracing.md)
    and made claimable through the
    [Agent session tracing ledger](../capture/agent-cli-tracing.md),
    with OpenTelemetry GenAI/MCP/CLI normalization specified in
    [Agent and CLI OTel semantic-convention mapping](../capture/agent-cli-tracing.md).
12. Turso correctness, backup/restore, concurrency, migration, and fallback
    behavior for metadata, agent session state, CLI invocation state, outcomes,
    and audit records, specified further in
    [Turso metadata production readiness](../storage/metadata/turso-metadata-production-readiness.md).
13. CLI/HTTP/MCP projection equivalence and read-only MCP safety, specified
    further in
    [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface.md)
    and made claimable through the
    [Agent access surface safety ledger](agent-access-surface.md).
14. Production database evidence access safety, specified further in
    [Production database evidence access gate](../capture/production-db-evidence.md)
    and made claimable through the
    [Production database evidence ledger](../capture/production-db-evidence.md).
15. OTLP receiver conformance and direct-SDK/Collector/Rotel normalization
    equivalence, specified further in
    [OTLP receiver conformance and Collector equivalence](../capture/otlp.md)
    and made claimable through the
    [OTLP conformance ledger](../capture/otlp.md).
16. Release/deploy/code-change/work-item context completeness and edge strength,
    specified further in
    [Deploy, change, and issue-tracker context](../capture/deploy-change-context.md)
    and made claimable through the
    [Deploy/change context ledger](../capture/deploy-change-context.md).
17. Separate fixer outcome quality from PR creation, specified further in
    [Fixer component and outcome loop](fixer-boundary.md)
    and made claimable only through the
    [Fixer outcome ledger](fixer-boundary.md).

## First Prototype Gate

Prototype should prove this loop:

```text
Rust app error
  -> fixture-gated Sentry envelope event
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
  [fixer component and outcome loop](fixer-boundary.md) and
  [fixer outcome ledger](fixer-boundary.md).

## Final Position

Proceed with Parallax, but keep the claim precise:

> Parallax does not prove every root cause. It makes the best available runtime,
> CI, CLI, deploy, repo, and agent evidence cheap to retain, fast to query, safe
> to expose, and structured enough for humans and agents to act on.

That is the buildable company.
