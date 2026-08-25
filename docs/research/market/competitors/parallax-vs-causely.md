# Parallax vs Causely

> One-to-one comparison. **No pro-Parallax bias.** Where Causely is ahead, ahead
> is written. Where Parallax's edge is only *planned* or *unproven*, that is
> stated, not hidden.
>
> Research date: **2026-07-17**. First canonical comparison — Causely was named
> in Parallax's own [`00-vision/ai-native-observability.md`](../../00-vision/ai-native-observability.md)
> but was missing from the canonical roster; added this pass. **Different layer
> than the telemetry competitors** — Causely is a causal-context layer that sits
> *on top of* telemetry, not a telemetry store.

## TL;DR verdict (scoped per axis)

- **Causely is a different-layer competitor that overlaps Parallax's exact
  thesis: "deliver context so agents stop guessing, burn fewer tokens, and act."**
  It ships a **causal-intelligence MCP layer** over your existing telemetry.
- **Causal modeling + MCP-grounding maturity: Causely wins, plainly** over
  pre-release Parallax — it is the *shipped* version of "give agents a causal
  model, not raw queries."
- **The layer split is the crux:** Causely = **BYO-telemetry causal-context MCP
  layer** (you bring Prometheus/Loki/Tempo/etc.; Causely reasons over them).
  Parallax = **telemetry-native evidence store** (owns the data: OTLP + Sentry
  ingest → derived errors → bundle). They overlap on *agent context* but differ
  on *who owns the telemetry*.
- **Parallax's differentiated edges are all unproven (A1 gate):** owning the
  telemetry + deriving error events + a bounded redacted bundle + a fix-outcome
  loop. Causely owns none of those — it is a reasoning layer, not a store.

## Causely — what it is (verified 2026-07-17)

A **causal-intelligence layer** for observability: builds a **live causal model
of your system and delivers it via an MCP server** so AI agents (and engineers)
get *why* an incident happened and the correct fix, instead of guessing over raw
metrics/logs/traces. Sits **on top of your existing telemetry** (metrics, logs,
traces, Kubernetes) — it is a reasoning/grounding layer, **not a telemetry
backend**.

| | Causely | Source |
|---|---|---|
| **Layer** | causal-intelligence **MCP layer** over BYO telemetry (Prom/Loki/Tempo/K8s primitives) | [causely.ai/product](https://www.causely.ai/product), [arXiv 2605.18327](https://arxiv.org/pdf/2605.18327) |
| **MCP server** | ✅ shipped — "Causal Reasoning Engine" into the IDE/agent; agents diagnose+understand+remediate | [Cloud Native Now — MCP launch](https://www.causely.ai/blog/cloud-native-now-causely-adds-mcp-server-to-causal-ai-platform) |
| **Pitch** | "live causal model via MCP — stop guessing, burn fewer tokens, act before things break" | [causely.ai/product](https://www.causely.ai/product) |
| **Telemetry ownership** | **none** — consumes your existing observability primitives via curated MCP servers | arXiv paper |
| **License / model** | **closed-source commercial** (SaaS reasoning engine + in-cluster agents; **not OSS**, **not full self-host** — agents only, engine managed) — resolved pass 35 | [causely.ai/pricing](https://www.causely.ai/pricing), [docs.causely.ai/installation](https://docs.causely.ai/installation/) |
| **Pricing** | **Professional $2,000/mo (≤500 services); Enterprise custom** — no free/OSS tier; **30-day free trial** (pass 35 + **pass 64** re-confirm) | [causely.ai/pricing](https://www.causely.ai/pricing) |
| **MCP server** | ✅ shipped — **30+ structured tools across 5 categories** for IDE/agents (Cursor/Claude/Copilot); **Gemini integration** (Oct 2025); blast-radius + remediation guidance | [causely.ai/blog](https://www.causely.ai/blog) |
| **Benchmark claim** | lower time / tokens / tool-calls when agents get causal context | causely.ai/product |
| **Category fit** | "AI investigation / causal-context layer" — complementary to telemetry stores; competitive with Parallax's *agent-context* thesis | this analysis |

> Causely pricing (resolved pass 35): **Professional $2,000/mo (≤500 services); Enterprise custom** ([causely.ai/pricing](https://www.causely.ai/pricing)); no free/OSS tier. Self-host: **agents deploy in-cluster** (Helm/CLI/Docker/Nomad/FluxCD/ArgoCD); the **causal reasoning engine is managed SaaS** (Enterprise on-prem via sales) — not a fully air-gapped self-host engine. Parallax pricing: **no public number** (pre-release). Direct comparison **N/A — different layers**.

## Axis-by-axis comparison

### The layer split (the crux)

- **Causely = a reasoning layer over your telemetry.** You keep Prometheus /
  Loki / Tempo / Datadog / etc.; Causely builds a causal model and exposes it to
  agents via MCP. It **does not ingest, store, or own your telemetry.**
- **Parallax = the telemetry store + evidence layer.** OTLP + shipped Sentry-
  envelope ingest → derived `error_event`s → fingerprinted → bounded bundle.

> They overlap on **"deliver context so an agent can act safely"** — the exact
> thesis axis — but split on **who owns the data**. Causely assumes you already
> have telemetry; Parallax *is* (part of) the telemetry + the evidence artifact.

### Agent-context story (Parallax's wedge — be most honest)

- **Causely ships the "grounding" version of Parallax's thesis today:** a causal
  model delivered to agents via MCP, with a benchmarked claim of fewer
  tokens/tool-calls/better RCA. This is the **strongest shipped "context layer
  for production agents"** in the set — directly competitive with Parallax's
  bounded-bundle-for-agents idea.
- **Parallax's claim:** a bounded, redacted, agent-use (safety/value unproven) *evidence bundle* served
  to coding agents, **derived from telemetry Parallax owns** (incl. Sentry
  envelopes + error events + outcome loop).

> **Honest verdict:** on *shipped agent-grounding*, **Causely is ahead of pre-
> release Parallax** — it delivers a causal model to agents via MCP today,
> benchmarked. Parallax's differentiation is (a) **owning the telemetry** (so the
> context can include Sentry-derived errors + outcomes Causely can't see), and
> (b) the **bounded/redacted bundle + fix-outcome loop** — all **unproven (A1
> gate).** The burden is on Parallax to show that owning-telemetry + a bounded
> bundle beats Causely's BYO-telemetry causal model for agent fix quality.

### Telemetry / signal coverage

- **Causely: none of its own** — reasons over whatever telemetry you point it at.
  No ingest, no storage, no Sentry path, no error-event derivation.
- **Parallax: OTLP (logs/traces/metrics) + shipped Sentry-envelope ingest +**
  derived error events + fingerprints.

> On telemetry ownership, Parallax is broader by design; Causely deliberately
> defers it. Not a head-to-head on coverage — different scopes.

### Causal modeling / RCA

- **Causely: ✅ core strength** — causal graph, topology-aware RCA, "why did this
  happen" (the Dynatrace-Davis / New-Relic-iRCA family, but as an MCP-grounding
  layer). Shipped.
- **Parallax: evidence-graph correlation + planned causal edges** (designed,
  **unproven**). On causal-modeling maturity, **Causely wins.**

### Dashboards, storage, deployment

N/A-for-comparison or deferred: Causely is a layer, not a store/UI suite — it
has no telemetry store, no dashboard suite (it feeds agents/IDEs). Parallax has
a telemetry store + minimal UI. Different surfaces.

### Openness, licensing & lock-in

- **Causely: closed commercial** (verify self-host); locks you into its causal
  model + MCP surface over your telemetry. Low *data* lock-in (your telemetry
  stays in Prom/Loki/Tempo), moderate *reasoning* lock-in.
- **Parallax: Apache-2.0**, fully open, owns the telemetry.

> On openness, **Parallax wins** (Apache-2.0 vs closed). On data lock-in, mixed
> (Causely leaves your telemetry where it is; Parallax owns it but openly).

## Where Causely plainly wins (no bias)

1. **Shipped causal-modeling + RCA maturity** (topology-aware "why").
2. **MCP-grounding for agents — delivered today**, benchmarked (fewer
   tokens/tool-calls). The strongest *shipped* "context layer for production
   agents" in the comparison.
3. **BYO-telemetry** — works with the stack you already have (no rip-and-replace).
4. **Narrow, focused product** (causal reasoning), not a sprawling platform.

## Where Parallax honestly edges Causely

1. **Telemetry ownership** — Parallax ingests/owns OTLP + Sentry envelopes +
   derives error events; Causely sees only what you point it at (no Sentry-derived
   errors, no outcome data). *(Real scope difference.)*
2. **Bounded, redacted, agent-use (safety/value unproven) evidence bundle** — Causely's MCP is a live
   causal-model query surface, not a portable/redacted/versioned artifact.
   *(Thesis, **unproven** — A1 gate.)*
3. **Fix-outcome loop** — Causely reasons to a cause; it does not track
   accepted/rejected/reverted/recurred fixes. *(Thesis, **unproven** — A1 gate.)*
4. **Openness** — Apache-2.0 vs closed commercial.
5. **Single source of truth** — Parallax can correlate errors + traces + logs +
   CI/deploy from owned data; Causely's quality depends on your underlying
   telemetry's coverage (the arXiv paper's own caveat: agent quality is bounded
   by the primitives the MCP servers expose).

## The honest synthesis

Causely and Parallax are the **two clearest "context-layer-for-production-agents"
bets in the set**, from opposite ends:

- **Causely = reasoning layer, BYO telemetry, closed, shipped.** Best if you
  already have telemetry and want a causal model fed to agents.
- **Parallax = telemetry-native evidence store, open, pre-release, unproven
  bundle/outcome thesis.** Best if you want the context *and* the underlying
  owned evidence (errors, Sentry, outcomes) in one bounded artifact.

**Neither subsumes the other today.** Causely is the maturity benchmark for the
agent-grounding axis; Parallax's bet is that owning the telemetry + a bounded
redacted bundle + an outcome loop beats a BYO-telemetry causal layer for
*production-incident* agent fixes — and that bet is the **A1 gate, unproven.**

## Watch triggers — re-evaluate Causely if it:

- Adds its **own telemetry ingestion / storage** (becoming a store, not just a
  layer) → would close the layer gap.
- Adds a **bounded/portable evidence artifact** or **fix-outcome tracking** →
  direct collision with Parallax's thesis.
- **Open-sources** its causal model/MCP → changes the openness verdict.

## Sources (checked 2026-07-17; license/pricing/MCP-detail resolved pass 35)

- [causely.ai/product](https://www.causely.ai/product) — "live causal model via MCP"; [blog](https://www.causely.ai/blog).
- [causely.ai/pricing](https://www.causely.ai/pricing) — Professional $2,000/mo (≤500 services); Enterprise custom (pass 35).
- [docs.causely.ai/installation](https://docs.causely.ai/installation/) — in-cluster agents (Helm/CLI/Docker/Nomad/FluxCD/ArgoCD); managed SaaS engine (pass 35).
- [Causely blog — MCP Server (30+ tools, 5 categories)](https://www.causely.ai/blog); Gemini integration (Oct 2025) (pass 35).
- [Cloud Native Now — Causely adds MCP server](https://www.causely.ai/blog/cloud-native-now-causely-adds-mcp-server-to-causal-ai-platform).
- [arXiv 2605.18327 — A Causal Intelligence Layer for Enterprise AI](https://arxiv.org/pdf/2605.18327) (ops AI agents use curated MCP servers for observability primitives).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md) (named Causely as a causal/MCP layer), [architecture/causal-reconstruction.md](../../architecture/causal-reconstruction.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
