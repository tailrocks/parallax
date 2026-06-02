# Project Structure

This file is the lightweight map of the Parallax repository. Keep it current as
the project evolves.

## Current Stage

Parallax is in research and product-discovery mode. The repository should stay
simple: root-level project rules, a README, and Markdown research notes under
`docs/`.

There is no docs UI, application source tree, package manager, release process,
or CI contract yet.

## Root Files

| Path | Purpose |
| --- | --- |
| `README.md` | Short repository entry point with links to current research. |
| `AGENTS.md` | Canonical AI-agent instructions for this repository. |
| `CLAUDE.md` | Claude Code linker that points to `AGENTS.md`. |
| `BRANCHING.md` | Current `main`-first workflow and pull-request policy. |
| `COMMITS.md` | Commit message and AI-agent attribution conventions. |
| `PROJECT_STRUCTURE.md` | This repository map. |
| `.gitignore` | Local files that should not be committed. |

## Directories

| Path | Purpose |
| --- | --- |
| `docs/` | Documentation and research notes. No generated docs UI yet. |
| `docs/research/` | Market, product, and strategy research, grouped by topic. The canonical per-note index is [`docs/research/README.md`](docs/research/README.md). |
| `docs/research/00-vision/` | Why this product: thesis, world-before-Parallax stack, platform direction, AI-native observability synthesis. |
| `docs/research/decisions/` | ADR-style decision records — current truth, conclusion first (go/no-go, strategic coverage, risks, the dated skeptical re-assessment, storage engine, V1 storage adapter vision, stack decision, metadata store, agent access surface, fixer boundary). |
| `docs/research/architecture/` | How the pieces fit: implementation concept, overview, evidence-bundle schema, API concept, causal reconstruction, local-first V1, simple UI V2, build roadmap. |
| `docs/research/capture/` | How each signal is collected and made safe: rust, frontend, OTLP-first ingest, future sentry-ingest, agent/CLI tracing, deploy/change context, CI/flaky tests, production-DB evidence, correlation (A4), redaction (A6). |
| `docs/research/storage/` | Telemetry-store evaluation, benchmark plan, freshness/latency and size/object-cost gates, plus `metadata/` and `streaming/` evidence subdirs. |
| `docs/research/storage/greptimedb-vs-clickhouse/` | Deep white-box GreptimeDB vs ClickHouse internals comparison: one-page verdict, run-log, 30+ mechanism notes, the four-build version matrix, and benchmarks. Produced by an indefinite `/goal` or Claude Code `/loop`. |
| `docs/research/validation/` | A1–A7 assumption gates and ledgers (A1 bundle-value subdir, A1 trajectory-source checks, A2 user demand, A3 schema/corpus, A7 scope, self-hosted simplicity, business model, repo intent, profitability analysis). |
| `docs/research/market/` | Market landscape, consolidated competitor watch, alternatives deep analysis, competitive comparison matrix, wider alternatives survey, and focused agent-debugging competitor drift notes. |
| `docs/research/market/maple-deep-research.md` | Deep research on Maple (maple.dev): architecture, features, UX, threat assessment, and Parallax comparison. |
| `docs/research/market/open-source-observability-tools-survey.md` | Broad survey of 37 open-source observability tools across three tiers (major platforms, established, emerging). |
| `docs/research/reference/ai-native-debugging-tools.md` | Survey of 13 AI-native debugging, SRE agent, and coding-agent observability tools. |
| `docs/research/capture/sentry-compatible-oss-tools.md` | Survey of 10 Sentry-compatible or Sentry-alternative open-source tools for future adapter/market tracking. |
| `docs/research/reference/` | External technical reviews. Includes `ai-native-debugging-tools.md` (open-source AI debugging agents, SRE agents, coding-agent observability tools). |
| `prompts/` | Reusable research and agent prompts. |
| `bench/` | Local storage-benchmark scaffolding: pinned `compose.yml` for GreptimeDB + ClickHouse smoke runs. Generated datasets/results are gitignored; only compose/scripts are tracked. Consistent with [`docs/research/storage/benchmark-plan.md`](docs/research/storage/benchmark-plan.md). |

## Research Record

The research is grouped by topic under `docs/research/`. For the full,
conclusion-first per-note index, see [`docs/research/README.md`](docs/research/README.md).
The decisions front door is [`docs/research/decisions/`](docs/research/decisions/): each
settled question has exactly one ADR (conclusion first, linking to its evidence), and the
still-open proof gates are enumerated in
[`decisions/strategic-coverage.md`](docs/research/decisions/strategic-coverage.md) and the
relevant `validation/` notes. The prioritized, decision-moving research backlog (what to research
and compare next, ranked) is [`docs/research/research-agenda.md`](docs/research/research-agenda.md).

| Area | Where |
| --- | --- |
| Vision and thesis | [`00-vision/`](docs/research/00-vision/) |
| Decisions (ADRs) | [`decisions/`](docs/research/decisions/) |
| Architecture | [`architecture/`](docs/research/architecture/) |
| Signal capture and safety | [`capture/`](docs/research/capture/) |
| Storage and the engine sub-study | [`storage/`](docs/research/storage/) |
| Assumption validation (A1–A7) | [`validation/`](docs/research/validation/) |
| Market and competitors | [`market/`](docs/research/market/) |
| External reference | [`reference/`](docs/research/reference/) |

## Prompts

| Path | Purpose |
| --- | --- |
| `prompts/README.md` | How to use the prompts in this folder (one-off, `/goal`, `/loop`). |
| `prompts/AGENTS.md` | Authoring rule for this folder: prompt files stay goal-only and runnable; run-mechanics and how-to-use live in `prompts/README.md`. |
| `prompts/CLAUDE.md` | Thin pointer to `prompts/AGENTS.md`. |
| `prompts/parallax-vision-and-restructure.md` | North-star brief: product vision, the GreptimeDB-vs-ClickHouse decision rule, and the research-record restructure mission. |
| `prompts/deep-research-parallax.md` | Deep research brief for validating the AI-native debugging/investigation direction. |
| `prompts/greptimedb-vs-clickhouse-internals.md` | Never-ending `/goal` or Claude Code `/loop` brief for the under-the-hood GreptimeDB vs ClickHouse comparison; writes to `docs/research/storage/greptimedb-vs-clickhouse/`. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
