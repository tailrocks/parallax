# Parallax

Parallax is an early research project exploring an open-source, Rust-first,
self-hosted observability and debugging system for production errors, logs,
traces, metrics, CLI runs, coding-agent sessions, and agent-ready failure
context.

The current working thesis is narrower than generic AI observability and more
specific than a CI debugging tool:

> Build a Sentry-compatible, OpenTelemetry-native execution context system that
> is simpler and cheaper to self-host than Sentry, while giving humans and
> coding agents the surrounding logs, traces, metrics, releases, CLI runs,
> agent actions, and runtime context needed to fix software failures.

## Current Status

This repository is in research and product-discovery mode. Expect fast
iteration on `main`, plain Markdown notes, and frequent restructuring as the
idea becomes sharper.

## Start Here

- [Go / no-go verdict](docs/research/verdict.md)
- [Risks and the bear case](docs/research/risks-and-bear-case.md)
- [Business model and economics](docs/research/business-model-and-economics.md)
- [Business model validation ledger](docs/research/business-model-validation-ledger.md)
- [Repo-intent dependence](docs/research/repo-intent-dependence.md)
- [Repo-intent value ledger](docs/research/repo-intent-value-ledger.md)
- [User interview and deployment intent gate](docs/research/user-interview-and-deployment-intent-gate.md)
- [A2 interview evidence ledger](docs/research/a2-interview-evidence-ledger.md)
- [Schema adoption and corpus moat gate](docs/research/schema-adoption-and-corpus-moat-gate.md)
- [A3 schema adoption and corpus ledger](docs/research/a3-schema-adoption-corpus-ledger.md)
- [Self-hosted simplicity gate](docs/research/self-hosted-simplicity-gate.md)
- [Self-hosted deployment baseline inventory](docs/research/self-hosted-deployment-baseline-inventory.md)
- [Self-hosted simplicity ledger](docs/research/self-hosted-simplicity-ledger.md)
- [Project thesis](docs/research/project-thesis.md)
- [Market landscape](docs/research/market-landscape.md)
- [Open self-hosted competitor watch](docs/research/open-self-hosted-competitor-watch.md)
- [Lightweight Sentry-compatible competitor watch](docs/research/lightweight-sentry-compatible-competitor-watch.md)
- [Agentic observability competitor drift ledger](docs/research/agentic-observability-competitor-drift-ledger.md)
- [Self-hosted observability architecture](docs/research/self-hosted-observability-architecture.md)
- [CI failure context MVP](docs/research/ci-failure-context-mvp.md)
- [GreptimeDB storage evaluation](docs/research/greptimedb-storage-evaluation.md)
- [Observability storage benchmark plan](docs/research/observability-storage-benchmark-plan.md)
- [Storage benchmark prototype (runnable)](docs/research/storage-benchmark-prototype.md)
- [Storage freshness and bundle latency gate](docs/research/storage-freshness-and-bundle-latency-gate.md)
- [Storage size and object cost gate](docs/research/storage-size-and-object-cost-gate.md)
- [Retention cost model](docs/research/retention-cost-model.md)
- [Metadata store benchmark plan and prototype](docs/research/metadata-store-benchmark-plan.md)
- [Turso metadata production readiness](docs/research/turso-metadata-production-readiness.md)
- [Messaging and ingestion layer](docs/research/messaging-and-ingestion-layer.md)
- [Ingest log replay and backpressure gate](docs/research/ingest-log-replay-and-backpressure-gate.md)
- [A5 stack decision ledger](docs/research/a5-stack-decision-ledger.md)
- [Rust data collection and instrumentation](docs/research/rust-data-collection-and-instrumentation.md)
- [Rust stacktrace grouping and symbolication](docs/research/rust-stacktrace-grouping-and-symbolication.md)
- [Rust stacktrace grouping ledger](docs/research/rust-stacktrace-grouping-ledger.md)
- [Frontend collection and cross-tier correlation](docs/research/frontend-collection-and-cross-tier-correlation.md)
- [Correlation reliability on real telemetry gate](docs/research/correlation-reliability-real-telemetry-gate.md)
- [A4 correlation reliability ledger](docs/research/a4-correlation-reliability-ledger.md)
- [Deploy, change, and issue-tracker context](docs/research/deploy-change-and-issue-context.md)
- [Deploy/change context ledger](docs/research/deploy-change-context-ledger.md)
- [Redaction pipeline and secret safety](docs/research/redaction-pipeline-and-secret-safety.md)
- [Redaction detector toolchain](docs/research/redaction-detector-toolchain.md)
- [A6 redaction red-team ledger](docs/research/a6-redaction-red-team-ledger.md)
- [A7 scope discipline ledger](docs/research/a7-scope-discipline-ledger.md)
- [Causal reconstruction and agent safety](docs/research/causal-reconstruction-and-agent-safety.md)
- [AI-native observability and incident intelligence](docs/research/ai-native-observability-and-incident-intelligence.md)
- [Flaky test investigation and replay](docs/research/flaky-test-investigation-and-replay.md)
- [Agent and CLI execution tracing](docs/research/agent-and-cli-execution-tracing.md)
- [Agent and CLI OTel semantic-convention mapping](docs/research/agent-cli-otel-semconv-mapping.md)
- [Agent session tracing across real tools](docs/research/agent-session-tracing-real-tools.md)
- [Agent session tracing ledger](docs/research/agent-session-tracing-ledger.md)
- [CLI trace overhead and redaction](docs/research/cli-trace-overhead-and-redaction.md)
- [Agent access surface: CLI, HTTP API, and MCP](docs/research/agent-access-surface-cli-api-mcp.md)
- [Agent access surface safety ledger](docs/research/agent-access-surface-safety-ledger.md)
- [Fixer component and outcome loop](docs/research/fixer-component-and-outcome-loop.md)
- [Production database evidence access gate](docs/research/production-database-evidence-access.md)
- [Production database evidence ledger](docs/research/production-database-evidence-ledger.md)
- [Agent observability technical review](docs/research/agent-observability-technical-review.md)
- [Bundle-value evaluation](docs/research/bundle-value-evaluation.md)
- [Bundle-value seed corpus](docs/research/bundle-value-seed-corpus.md)
- [Phase 0 telemetry overlay contract](docs/research/phase0-telemetry-overlay-contract.md)
- [A1 eval result ledger and model refresh](docs/research/a1-eval-result-ledger-and-model-refresh.md)
- [Bundle-value Phase 0 runbook](docs/research/bundle-value-phase0-runbook.md)
- [Build roadmap and validation sequence](docs/research/build-roadmap-and-validation-sequence.md)
- [Future platform direction](docs/research/future-platform-direction.md)
- [Evidence bundle and open schema specification](docs/research/evidence-bundle-and-schema.md)
- [Strategic verdict and research coverage](docs/research/strategic-verdict-and-research-coverage.md)
- [Technical implementation concept](docs/research/technical-implementation-concept.md)
- [Sentry-compatible ingestion](docs/research/sentry-compatible-ingestion.md)
- [Sentry SDK fixture compatibility gate](docs/research/sentry-sdk-fixture-compatibility.md)
- [Sentry SDK compatibility ledger](docs/research/sentry-sdk-compatibility-ledger.md)
- [OpenTelemetry protocol and context layer](docs/research/opentelemetry-protocol-and-context-layer.md)
- [OTLP receiver conformance and Collector equivalence](docs/research/otlp-receiver-conformance-and-collector-equivalence.md)
- [OTLP conformance ledger](docs/research/otlp-conformance-ledger.md)
- [Repository structure](PROJECT_STRUCTURE.md)
- [Agent instructions](AGENTS.md)

## Working Direction

The current recommended wedge is:

1. Start with Sentry-compatible error ingestion for Rust services and CLI apps.
2. Add OpenTelemetry logs, traces, and metrics correlation.
3. Store high-volume observability data in a simple self-hosted backend,
   starting with GreptimeDB as the first prototype candidate.
4. Use a Rust message stream such as Apache Iggy only if replay, buffering, or
   processor separation is worth the operational cost.
5. Trace coding-agent sessions and CLI invocations as first-class execution
   evidence.
6. Produce evidence-backed context for humans and coding agents.
7. Keep the UI and deployment model much simpler than self-hosted Sentry.
