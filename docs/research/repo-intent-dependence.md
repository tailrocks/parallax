# Repo-Intent Dependence and the Degraded Mode

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Answer strategic question 13 directly, because the corpus has only handled it in
one-line table cells: the vision assumes a context-rich monorepo (code **plus**
docs, design decisions, tasks, roadmap). How much of Parallax's value actually
depends on that, and what happens for the large majority of teams that do not work
that way? This matters because the answer sizes the addressable market and is the
sharpest edge of the founder-market-fit risk in the [bear case](risks-and-bear-case.md)
(A2 / n=1).

## The Value Decomposes Into Two Separable Layers

Parallax's value is not one thing that needs the monorepo. It is two layers, and
only the second needs repo-intent:

1. **Runtime evidence layer (the floor).** Error grouping, cross-signal
   correlation, evidence bundles, CLI/agent/CI traces, retention. This needs
   **telemetry + the source code**, nothing more. Any team that emits Sentry/OTLP
   data and has a repo gets the full floor. No monorepo, no ADRs, no task tracker
   required.
2. **Repo-intent layer (the multiplier).** Linking a failure to *why the code is
   the way it is* — design decisions, ADRs, tasks, roadmap. This needs the
   context-rich repo the vision assumes. It makes proposals better and avoids
   "why is this here" mistakes, but it is **additive**, not required.

The strategic rule that follows: **the product must be fully valuable on the
runtime-evidence floor alone. Repo-intent is an optional multiplier, never a
prerequisite.** If Parallax's pitch requires the monorepo+intent setup, its
market collapses to teams that work exactly like the operator — which is the
n=1 founder-market-fit trap.
The result contract for proving or narrowing this claim is the
[Repo-intent value ledger](repo-intent-value-ledger.md).

## Degraded Mode (Teams Without Repo-Intent)

Most teams have code + telemetry but not curated docs/decisions/tasks. Their
experience must still be excellent:

- They get grouping, correlation, trace/log/metric joins, release-regression
  detection, and bounded bundles.
- A coding agent fixes from **code + runtime evidence** — and SWE-bench shows
  agents already fix real bugs from code alone, so code + telemetry is a strong
  floor (see [bundle-value evaluation](bundle-value-evaluation.md)).
- What they lose is only the "why" layer: the agent may not know a piece of code
  exists to satisfy a constraint it cannot see. That is a real but bounded loss,
  and it degrades gracefully — the bundle simply omits intent edges rather than
  breaking.

Degraded mode is the **common case**, so it is the case the product must be
designed for. The context-rich monorepo is the power-user case.

## What Repo-Intent Adds When Present (The Upside)

When a team *does* keep decisions/tasks/roadmap in the repo (like the operator):

- proposals can cite the decision a fix must not violate;
- the agent can align a fix with stated intent, not just make tests pass;
- "this code is intentional, do not "simplify" it" is knowable.
- agent instruction files such as `AGENTS.md`, `CLAUDE.md`, and Copilot
  instructions can expose repo-local operating intent, but they are still
  context. They do not become policy enforcement unless hooks, settings, CI, or
  another control enforces them outside the prose file.

This is also a **moat seed**: telemetry-only competitors (Datadog, Sentry,
OpenObserve, SigNoz) do not link runtime evidence to repo-held intent. So
repo-intent is simultaneously something to *not depend on* (for market size) and
something that *differentiates* (for high-context teams). Offer it as an
opt-in enrichment: point Parallax at `docs/`, ADRs, tasks, and approved
instruction surfaces; it adds source-cited intent nodes/edges to the bundle when
available.

## Implication For The Bundle-Value Eval

This question is empirically testable and should be an arm in the
[bundle-value evaluation](bundle-value-evaluation.md): run the agent with
**bundle + repo-intent** vs **bundle + code only (no intent)**. The delta
measures how much repo-intent actually buys. Two outcomes:

- Small delta → degraded mode is nearly as good; market is broad; repo-intent is
  a nice-to-have. (Good for A2.)
- Large delta → the product leans on repo-intent; market narrows toward
  operator-like teams; flag the founder-market-fit risk and either invest in
  making intent capture trivial or accept the narrower wedge.

Either way, do not assume; measure.

## Bottom Line

Parallax's runtime-evidence value does not depend on the monorepo, and it must
not be allowed to. Build for the team that has code + telemetry and nothing else;
treat repo-intent (docs, decisions, tasks, roadmap) as an opt-in multiplier that
differentiates for high-context teams and seeds a moat telemetry-only tools
cannot match. The monorepo is the operator's advantage, not the product's
requirement — and the bundle-value eval should measure the size of that
advantage rather than presume it.

## Relationship To Other Research

- [Verdict](verdict.md) — Q13 row this expands; the GO depends on broad
  addressable market, i.e. on degraded mode being good.
- [Risks and the bear case](risks-and-bear-case.md) — A2 / founder-market-fit;
  repo-intent dependence is the sharpest form of that risk.
- [Bundle-value evaluation](bundle-value-evaluation.md) — add the
  intent-vs-no-intent arm proposed here.
- [Repo-intent value ledger](repo-intent-value-ledger.md) — defines the paired
  runtime-only versus runtime-plus-intent result rows, claim levels, and
  stale/conflicting-intent fixtures.
- [AI-native observability](ai-native-observability-and-incident-intelligence.md)
  — repo-intent linkage as a differentiator vs telemetry-only products.
