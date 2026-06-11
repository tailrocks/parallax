# AGENTS.md

This repository uses `main` as its active working branch during the research
stage. This file is the canonical home for rules that apply only to AI agents.
Rules that apply equally to humans and agents live in the topic-specific files
linked under **Shared conventions** below.

## Project Status: Research Draft

Parallax is currently a market-research and product-discovery repository. The
goal is to capture ideas, sources, comparisons, product hypotheses, and evolving
structure quickly.

- Prefer small, source-linked Markdown updates over heavy process.
- Keep findings in the repository, not only in chat.
- Commit and push findings after creating or updating research documents.
- Keep research under `docs/research/` unless a topic-specific root file is a
  better fit.
- Do not build a documentation site yet. Plain Markdown is enough for this
  stage.
- Concept-proving Rust code is allowed under `poc/` (operator, 2026-06-11):
  small, runnable, test-covered proofs of designed mechanisms. PoC code is not
  product code and supports no product claims; keep each PoC scoped to the
  mechanism it verifies.
- Version policy (operator, 2026-06-12): always use the latest stable versions
  everywhere — crates, engines, UI dependencies, toolchains. Version tables in
  docs are known-compatible floors, not freezes; when implementing, resolve
  the latest mutually-compatible stable set and update the table in the same
  commit.
- V1 implementation is authorized (operator, 2026-06-12): product code lands
  under `crates/` (Cargo workspace) and `ui/` (TanStack Start app), following
  [docs/research/architecture/v1-implementation-spec.md](docs/research/architecture/v1-implementation-spec.md)
  and the [v1-implementation prompt](prompts/v1-implementation.md). Contract
  changes go to the implementation spec first, then code; update
  `PROJECT_STRUCTURE.md` when the directories appear.
- License and attribution (operator, 2026-06-11; extended 2026-06-12): the
  **entire repository is Apache-2.0** — the root `LICENSE` carries the
  canonical Apache-2.0 text and `NOTICE` the copyright line. Every artifact
  that declares a license (crate metadata, schemas, examples, future product
  code) declares **Apache-2.0**, and the company name used in examples,
  attribution, and metadata is **Tailrocks** (e.g. `github.com/tailrocks/...`
  in fixtures, not `acme`). License ≠ visibility: the repository stays private
  until the operator publishes it, and pre-release access follows
  [REPOSITORY_PROTECTION.md](REPOSITORY_PROTECTION.md).

## Branching and Pull Requests

Do not open pull requests for routine work in this repository yet. The project
is too early and will iterate directly on `main`.

- Work directly on `main`.
- Commit focused changes.
- Push after each durable research or structure update.
- Revisit this rule later when the project has enough implementation surface or
  collaborators to justify pull requests.

See [BRANCHING.md](BRANCHING.md) for the current branch policy.

## Tool-Specific Files

Durable agent rules for this repository belong in this local `AGENTS.md` file,
not in tool-specific config files. When the operator gives a new durable rule or
changes an existing one, update `AGENTS.md` in the same change.

Current linkers:

- [CLAUDE.md](CLAUDE.md) contains only `@AGENTS.md`.
- [prompts/CLAUDE.md](prompts/CLAUDE.md) contains only `@AGENTS.md` and points to the
  prompts-folder rule in [prompts/AGENTS.md](prompts/AGENTS.md).

`CLAUDE.md` must always remain a thin pointer to `AGENTS.md` only. Do not put
real instructions in `CLAUDE.md`. If another tool needs a linker file later, add
the same kind of thin pointer and keep the real instructions here or in the
shared convention files.

## Commit Attribution

Every commit created by an AI agent in this repository must include exactly one
`Co-authored-by` trailer identifying the agent tool that made the commit. The
trailer identifies the agent product, not the underlying model.

Use these trailers:

- Codex:

  ```text
  Co-authored-by: Codex <codex@openai.com>
  ```

- Claude Code:

  ```text
  Co-authored-by: Claude <noreply@anthropic.com>
  ```

- Amp:

  ```text
  Co-authored-by: Amp <amp@ampcode.com>
  ```

- Opencode:

  ```text
  Co-authored-by: opencode-agent[bot] <opencode-agent[bot]@users.noreply.github.com>
  ```

Do not stack multiple agent trailers on one ordinary commit. If several agents
materially contribute to one change, ask the operator how to attribute it before
committing.

See [COMMITS.md](COMMITS.md) for commit message conventions.

## Research Notes

Market and product research should be concise, sourced, and easy to extend.

- Use Markdown under `docs/research/`.
- Prefer comparison tables plus short analysis sections.
- Include a research date for time-sensitive market notes.
- Link primary sources whenever possible.
- When a new finding changes the repo shape, update
  [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) in the same change.

## Benchmarking Rule (four builds, always)

Every performance benchmark in the GreptimeDB-vs-ClickHouse research must be measured on **all four
builds**, never stable-only and never a two-way:

1. **GreptimeDB — latest stable** (e.g. `v1.0.2`).
2. **GreptimeDB — latest nightly** (e.g. `v1.1.0-nightly-YYYYMMDD`).
3. **ClickHouse — latest stable feature line, NOT LTS** (e.g. `v26.5.1.882-stable`). Always the
   newest feature release; never an LTS/backport line.
4. **ClickHouse — latest nightly** (`clickhouse/clickhouse-server:head`).

**Two tiers — local small, server large.** On a laptop run a **small but meaningful preliminary** tier
(default `N=100,000`, minimum 50,000) — big `N` (millions) with four DB containers freezes a MacBook.
The **detailed, large-scale** test (`N=5,000,000`+) runs on a **server**, not the dev machine. Don't
keep all four containers standing with big data on a laptop: `docker start` the nightlies → `gen.sh`
(small) → `bench.sh` → `docker stop` the nightlies. Build identical data on all four (`range()` on
GreptimeDB / `numbers()` on ClickHouse). Re-pull + rebuild when a new nightly tag drops. **Every benchmark must update the
consolidated matrix [`docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md`](docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md)** (every query × 4 builds, a *Faster*
column, per-query *Details* links to the mechanism note + the reproducible run in
`local-benchmark-results.md`). A stable-only number is not the result; the four-build row is. The
operator-facing detail of this rule also lives in the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](prompts/greptimedb-vs-clickhouse-internals.md).

## Research Prompt Maintenance

Research prompts are durable operator intent, not disposable one-off inputs.
When the operator clarifies the research direction, confirms a product decision,
changes evaluation criteria, adds or removes target domains, or names tools that
must be compared, update the relevant prompt under `prompts/` in the same
change if a future `/goal` or `/loop` run would otherwise repeat stale
instructions.

Research run instructions should use supported long-running commands only:
`/goal` for condition-driven continuation and Claude Code `/loop` for scheduled
repeat prompts. Do not document unsupported command aliases in runbooks or prompt
examples.

If a prompt does not need an update after such clarification, say why in the
final response.

Prompt files must also stay **runnable as-is**: keep them goal-only and never embed
run-mechanics or "how to use" notes — those live in `prompts/README.md`. The form rule for
prompt files is in [prompts/AGENTS.md](prompts/AGENTS.md).

## Shared Conventions

Rules in these files apply to everyone working in the repo:

- [BRANCHING.md](BRANCHING.md) - current `main`-first workflow and when to
  revisit pull requests.
- [COMMITS.md](COMMITS.md) - Conventional Commits and agent attribution
  trailers.
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - current repository layout and
  ownership of each folder/file.
