# Parallax

> Copyright (c) 2026 Tailrocks Pte. Ltd. Licensed under the
> [Apache License, Version 2.0](LICENSE) — see [LICENSE](LICENSE) and
> [NOTICE](NOTICE). The repository is now public and governed by
> [REPOSITORY_PROTECTION.md](REPOSITORY_PROTECTION.md).

Parallax is an early research project exploring an open-source, Rust-first,
self-hosted observability and debugging system for production errors, logs,
traces, metrics, CLI runs, coding-agent sessions, and agent-ready failure
context.

The current working thesis is narrower than generic AI observability and more
specific than a CI debugging tool:

> Build a Sentry-compatible, OpenTelemetry-native execution context system for
> self-hosted investigation, while giving humans and
> coding agents the surrounding logs, traces, metrics, releases, CLI runs,
> agent actions, and runtime context needed to fix software failures.

Parallax is the **context engine, not the fixer** — it serves bounded, redacted
evidence bundles to a separate coding agent that proposes the fix.

## Current Status

V1 product code ships under [`crates/`](crates/) and [`ui/`](ui/) on `main`
(local GreptimeDB + Turso profile). Research under
[`docs/research/`](docs/research/) is the evidence and decision record; treat
notes as theories until primary sources or the
[code-reality ledger](docs/research/code-reality-ledger.md) support them.
Expect iteration on `main` as open plans and validation gates close.

## Using It

The V1 implementation (workspace under [`crates/`](crates/), web UI under
[`ui/`](ui/)) is usable today on the local profile:

- **Preview Homebrew package** — after the preview workflow publishes a build:
  `brew tap tailrocks/parallax` then `brew install parallax@preview`. See the
  [release verification guide](docs/guide/releases.md) for independent asset
  checks and local rehearsal.
- **[Quickstart](docs/guide/quickstart.md)** — install → serve → connect a Rust app → first evidence bundle.
- **[Footprint](docs/guide/footprint.md)** — idle ~24 MiB Parallax + ~139 MiB Greptime (2026-08-13, Apple M5 Max).
- **[CLI reference](docs/guide/cli.md)** — every `parallax` command.
- **[Agent how-to](docs/guide/agent-howto.md)** — point your coding agent at `parallax issue context`.
- **[Conventions](docs/guide/conventions.md)** — resource attributes, `parallax.run.id`, exception encodings, DB wrapper spans.
- **[Grouping](docs/guide/grouping.md)** — why events share an issue (`fp-v1`).
- **[Upgrade and durability](docs/guide/upgrade-and-durability.md)** — data-dir upgrade contract and loss counters.
- **[Evidence bundle schema (`bundle-v1`)](schema/evidence-bundle.v1.schema.json)** — portable JSON Schema for the canonical bundle bytes (versioning policy in [`schema/README.md`](schema/README.md)).
- **Developing Parallax itself** — see [CONTRIBUTING.md § Development](CONTRIBUTING.md#development).

## Start Here

The research record lives under [`docs/research/`](docs/research/) and is organized so you can
reach "what is Parallax, which storage engine, and why" in a few minutes:

- **[Research index](docs/research/README.md)** — the navigable map (vision, decisions, architecture, capture, storage, validation, market, reference) with a "current answers" table.
- **[Code-reality ledger](docs/research/code-reality-ledger.md)** — research claims vs shipped `crates/`/`ui/` status (use this before trusting older research prose).
- **[Problem, audience, and product shape](docs/research/00-vision/problem-audience-product-shape.md)** — what Parallax solves, who it is for (developer on a dev machine first), and the shape: best of three worlds (OTel collect, Sentry organize, Grafana understand), agent-first, CLI + API + UI over one canonical API.
- **[North star: the autonomous fix loop](docs/research/00-vision/north-star-autonomous-fix-loop.md)** — the named moonshot (earned autonomy, the impossible triangle) and how it coexists with the narrow wedge. Build-order note: the moonshot is the ceiling, not the schedule.
- **[V1 scope](docs/research/architecture/v1-scope.md)** — the self-sufficient local-machine contract and shipped delivery inventory (install, engine supervision, ingest, run wrapper, CLI, retention, docs, exclusions, and acceptance scenarios).
- **[V1 build record](docs/research/architecture/v1-build-plan.md)** — historical crate/milestone sequencing and dogfood criteria; unfinished residuals live in research decisions, not a `plans/` index.
- **[Deployment architecture map](docs/research/architecture/deployment-architecture-map.md)** — the three historical topology angles; current implementation policy is GreptimeDB telemetry plus Turso metadata in every supported profile.
- **[Go / no-go verdict](docs/research/decisions/go-no-go.md)** — GO, for the narrow evidence/context engine.
- **[Storage engine decision](docs/research/decisions/storage-engine.md)** — GreptimeDB is the committed telemetry engine; ClickHouse remains a research/benchmark comparator, never a product fallback.
- **[Risks and the bear case](docs/research/decisions/risks-and-bear-case.md)** — the adversarial counterweight.
- **[Strategic synthesis + coverage map](docs/research/decisions/strategic-coverage.md)** — every prompt area mapped to its evidence.
- **[Historical implementation concept](docs/research/architecture/implementation-concept.md)** — the original end-to-end architecture reasoning; shipped work lives under `crates/` and `ui/`.

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

What is already on the critical path (see
[code-reality ledger](docs/research/code-reality-ledger.md) for status detail):

1. **OTLP-native ingest** of traces, logs, and metrics; derive Parallax-owned
   error events from exception spans and ERROR/FATAL logs.
2. **Sentry-envelope HTTP ingest is shipped** (bounded adapter; multi-SDK
   compatibility ledger still unproven — not a deferred V1 exclusion).
3. **GreptimeDB native OTLP tables + Turso metadata are mandatory.**
   `StorageAdapter` is a capability/test boundary, not an engine-substitution
   promise. ClickHouse is a research/benchmark comparator only.
4. A durable message broker (e.g. Iggy) only if replay/backpressure measurement
   justifies the ops cost — not a default product dependency.
5. CLI invocations and coding-agent sessions as first-class execution evidence
   (capture adapters still deepen under active plans).
6. Bounded, redacted evidence bundles for humans and coding agents (bundle
   **value** vs raw context remains the open A1 gate).
7. Measure self-host operations against relevant alternatives before making
   simplicity or cost claims; no comparative result is established yet.
