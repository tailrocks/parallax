# Parallax

> Proprietary and confidential. Copyright (c) 2026 Tailrocks Pte. Ltd. All
> rights reserved. This private repository is owned by Tailrocks Pte. Ltd. Do
> not copy, publish, distribute, disclose, or use its contents outside
> Tailrocks Pte. Ltd. without prior written permission. See [LICENSE](LICENSE)
> and [NOTICE](NOTICE). See [REPOSITORY_PROTECTION.md](REPOSITORY_PROTECTION.md)
> before sharing access outside Tailrocks Pte. Ltd.

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

Parallax is the **context engine, not the fixer** — it serves bounded, redacted
evidence bundles to a separate coding agent that proposes the fix.

## Current Status

This repository is in research and product-discovery mode. Expect fast
iteration on `main`, plain Markdown notes, and frequent restructuring as the
idea becomes sharper.

## Start Here

The research record lives under [`docs/research/`](docs/research/) and is organized so you can
reach "what is Parallax, which storage engine, and why" in a few minutes:

- **[Research index](docs/research/README.md)** — the navigable map (vision, decisions, architecture, capture, storage, validation, market, reference) with a "current answers" table.
- **[Problem, audience, and product shape](docs/research/00-vision/problem-audience-product-shape.md)** — what Parallax solves, who it is for (developer on a dev machine first), and the shape: best of three worlds (OTel collect, Sentry organize, Grafana understand), agent-first, CLI + API + UI over one canonical API.
- **[North star: the autonomous fix loop](docs/research/00-vision/north-star-autonomous-fix-loop.md)** — the named moonshot (earned autonomy, the impossible triangle) and how it coexists with the narrow wedge. Build-order note: the moonshot is the ceiling, not the schedule.
- **[V1 build plan](docs/research/architecture/v1-build-plan.md)** — what gets built and how: goals 1+2 (local visibility slightly first, then the server profile), crate layout, milestones M0–M6 with dogfood exit criteria; autonomous fixing parked at the schema level.
- **[Deployment architecture map](docs/research/architecture/deployment-architecture-map.md)** — the three angles (local laptop, own server, cloud + object storage) with diagrams, setup flows, and the GreptimeDB/Turso/Postgres role split per angle.
- **[Go / no-go verdict](docs/research/decisions/go-no-go.md)** — GO, for the narrow evidence/context engine.
- **[Storage engine decision](docs/research/decisions/storage-engine.md)** — current lean GreptimeDB (not yet settled), ClickHouse fallback, both behind a `StorageAdapter`.
- **[Risks and the bear case](docs/research/decisions/risks-and-bear-case.md)** — the adversarial counterweight.
- **[Strategic synthesis + coverage map](docs/research/decisions/strategic-coverage.md)** — every prompt area mapped to its evidence.
- **[Implementation concept](docs/research/architecture/implementation-concept.md)** — the end-to-end blueprint.

Other entry points: [Repository structure](PROJECT_STRUCTURE.md) · [Agent instructions](AGENTS.md) · [Research prompt runbook](prompts/README.md).

## Indefinite Research Runs

The preferred research workflow is an indefinite re-verification loop over
[`prompts/deep-research-parallax.md`](prompts/deep-research-parallax.md), run
through `/goal` in Codex or Claude Code. Use Claude Code `/loop` only when you
want scheduled re-triggers inside an open Claude Code session. `/goal` is the
standard choice for long-running research because the next turn starts when the
previous turn finishes; `/loop` is Claude Code-only and starts the next pass when
its interval fires.

Treat every existing note under `docs/research/` as a theory until current
primary-source evidence supports it. Each pass should re-check a weak, stale,
important, or suspicious claim; reconsider it against the Parallax goal; add
missing important research; update the relevant Markdown; commit; push; and then
continue to the next gap.

The ordinary deep-research loop should focus on quality, trustworthiness,
current source verification, explicit uncertainty, and falsification criteria.
Do not spend those passes benchmarking storage or infrastructure performance
differences; use separate benchmark-agent artifacts when they exist and mark
benchmark-dependent claims as unproven until measured.

See [`prompts/README.md`](prompts/README.md) for the verified `/goal` and Claude
Code `/loop` runbook.

## Working Direction

The current recommended wedge is:

1. Start OpenTelemetry-native: ingest OTLP traces, logs, and metrics from Rust
   services and CLI apps, and derive Parallax-owned error events from exception
   spans and ERROR/FATAL logs.
2. Treat Sentry-compatible error ingestion as a future migration adapter, not
   V1 scope.
3. Store high-volume observability data in a simple self-hosted backend behind a
   `StorageAdapter`; current lean is GreptimeDB (see the
   [storage engine decision](docs/research/decisions/storage-engine.md)), with
   ClickHouse as the fallback.
4. Use a Rust message stream such as Apache Iggy only if replay, buffering, or
   processor separation is worth the operational cost.
5. Trace coding-agent sessions and CLI invocations as first-class execution
   evidence.
6. Produce evidence-backed context bundles for humans and coding agents.
7. Keep the UI and deployment model much simpler than self-hosted Sentry.
