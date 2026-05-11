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

## Research Documents

| Path | Purpose |
| --- | --- |
| `docs/research/project-thesis.md` | Original thesis. |
| `docs/research/market-landscape.md` | Market research. |
| `docs/research/greptimedb-storage-evaluation.md` | Storage-layer evaluation for GreptimeDB, ClickHouse, and observability backends. |

## Update Rule

When adding a new top-level file, directory, or durable research area, update
this file in the same commit.
