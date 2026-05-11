# Parallax

Parallax is an early research project exploring an open-source, CLI-first
debugging system for CI failures, flaky tests, and eventually production
incidents.

The current working thesis is narrower than generic AI observability:

> Build a local-first failure context compiler that turns test failures,
> CI logs, retries, git history, and related evidence into portable debugging
> bundles for humans and coding agents.

## Current Status

This repository is in research and product-discovery mode. Expect fast
iteration on `main`, plain Markdown notes, and frequent restructuring as the
idea becomes sharper.

## Start Here

- [Project thesis](docs/research/project-thesis.md)
- [Market landscape](docs/research/market-landscape.md)
- [Repository structure](PROJECT_STRUCTURE.md)
- [Agent instructions](AGENTS.md)

## Working Direction

The current recommended wedge is:

1. Start with CI failures and flaky tests.
2. Produce evidence-backed explanations, not opaque LLM answers.
3. Package the evidence as agent-ready context.
4. Avoid competing head-on with full observability platforms at the start.
