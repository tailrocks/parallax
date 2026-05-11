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

Durable rules belong in shared project files, not in tool-specific config files.
Tool-specific files should only link back here.

Current linker:

- [CLAUDE.md](CLAUDE.md) contains only `@AGENTS.md`.

If another tool needs a linker file later, add the same kind of thin pointer and
keep the real instructions here or in the shared convention files.

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

## Shared Conventions

Rules in these files apply to everyone working in the repo:

- [BRANCHING.md](BRANCHING.md) - current `main`-first workflow and when to
  revisit pull requests.
- [COMMITS.md](COMMITS.md) - Conventional Commits and agent attribution
  trailers.
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - current repository layout and
  ownership of each folder/file.
