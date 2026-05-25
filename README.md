# Parallax

Parallax is an early research project exploring an open-source, Rust-first,
self-hosted observability and debugging system for production errors, logs,
traces, metrics, and agent-ready failure context.

The current working thesis is narrower than generic AI observability and more
specific than a CI debugging tool:

> Build a Sentry-compatible, OpenTelemetry-native error context system that is
> simpler and cheaper to self-host than Sentry, while giving humans and coding
> agents the surrounding logs, traces, metrics, releases, and runtime context
> needed to fix production bugs.

## Current Status

This repository is in research and product-discovery mode. Expect fast
iteration on `main`, plain Markdown notes, and frequent restructuring as the
idea becomes sharper.

## Start Here

- [Project thesis](docs/research/project-thesis.md)
- [Market landscape](docs/research/market-landscape.md)
- [Self-hosted observability architecture](docs/research/self-hosted-observability-architecture.md)
- [CI failure context MVP](docs/research/ci-failure-context-mvp.md)
- [GreptimeDB storage evaluation](docs/research/greptimedb-storage-evaluation.md)
- [Observability storage benchmark plan](docs/research/observability-storage-benchmark-plan.md)
- [Messaging and ingestion layer](docs/research/messaging-and-ingestion-layer.md)
- [Causal reconstruction and agent safety](docs/research/causal-reconstruction-and-agent-safety.md)
- [Technical implementation concept](docs/research/technical-implementation-concept.md)
- [Sentry-compatible ingestion](docs/research/sentry-compatible-ingestion.md)
- [OpenTelemetry protocol and context layer](docs/research/opentelemetry-protocol-and-context-layer.md)
- [Repository structure](PROJECT_STRUCTURE.md)
- [Agent instructions](AGENTS.md)

## Working Direction

The current recommended wedge is:

1. Start with Sentry-compatible error ingestion for Rust services.
2. Add OpenTelemetry logs, traces, and metrics correlation.
3. Store high-volume observability data in a simple self-hosted backend,
   starting with GreptimeDB as the first prototype candidate.
4. Use a Rust message stream such as Apache Iggy only if replay, buffering, or
   processor separation is worth the operational cost.
5. Produce evidence-backed context for humans and coding agents.
6. Keep the UI and deployment model much simpler than self-hosted Sentry.
