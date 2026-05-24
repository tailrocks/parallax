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
| `docs/research/` | Market, product, and strategy research. |
| `prompts/` | Reusable research and agent prompts. |

## Research Documents

| Path | Purpose |
| --- | --- |
| `docs/research/project-thesis.md` | Original thesis. |
| `docs/research/market-landscape.md` | Market research. |
| `docs/research/self-hosted-observability-architecture.md` | Architecture research for the Sentry-compatible, OpenTelemetry-native self-hosted observability direction. |
| `docs/research/ci-failure-context-mvp.md` | MVP research for GitHub Actions failure-context bundles. |
| `docs/research/greptimedb-storage-evaluation.md` | Storage-layer evaluation for GreptimeDB, ClickHouse, and observability backends. |
| `docs/research/observability-storage-benchmark-plan.md` | Database-only benchmark plan for observability storage candidates. |
| `docs/research/rust-data-collection-and-instrumentation.md` | Rust-first data-collection decision (SDK/OTLP vs eBPF) and error-capture data model. |

## Prompts

| Path | Purpose |
| --- | --- |
| `prompts/deep-research-parallax.md` | Deep research brief for validating the AI-native debugging/investigation direction. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
