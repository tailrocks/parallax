# Parallax vs Dynatrace

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (agent-control-
> plane watch **resolved pass 38** against the [Perform 2026 agentic-AI blog](https://www.dynatrace.com/news/blog/dynatrace-introduces-a-new-foundation-for-agentic-ai-at-perform-2026/)).
> Sources: [dynatrace.com/pricing](https://www.dynatrace.com/pricing/), [OneAgent+OTLP docs](https://docs.dynatrace.com/docs/ingest-from/dynatrace-oneagent/oneagent-and-opentelemetry/oneagent-otel), [Grail log-observability blog](https://www.dynatrace.com/news/blog/how-dynatrace-supercharged-log-observability-in-2025/), [Perform 2026 agentic-AI blog](https://www.dynatrace.com/news/blog/dynatrace-introduces-a-new-foundation-for-agentic-ai-at-perform-2026/), [Perform 2026 press](https://www.dynatrace.com/news/press-release/perform-2026-ignites-new-era/), third-party pricing (Spendhound/Vendr/CheckThat.ai).
>
> **Bottom line up front:** Dynatrace is a **closed, enterprise AIOps incumbent**
> with a genuinely distinctive axis: **Davis AI deterministic causal root-cause
> analysis** built on **OneAgent's** deep, zero-config, code-level auto-instrumentation
> and full dependency **topology** — causation, not just correlation. On **AI-driven
> RCA, deep auto-instrumentation, topology, the Grail long-retention store, and
> enterprise scale, Dynatrace is far ahead of pre-release Parallax.** **Pass-38
> WATCH FIRED:** Dynatrace **shipped** (Perform 2026, 2026-03-04) an **"agentic
> operations platform / agent control plane"** — Dynatrace Intelligence + new
> Smartscape real-time dependency graph + Intelligence Agents (auto-remediate) +
> Dynatrace Assist + **Dynatrace MCP Server** — explicitly providing **"bounded
> agent context"** (its words) to autonomous agents. This is a **direct, named
> collision with Parallax's "bounded context for agents" thesis**, executed on a
> far more mature substrate. Parallax's honest edges narrow to
> **open-source/self-host** (Dynatrace is closed SaaS), **Apache-2.0**, **cost**
> (Dynatrace is enterprise-expensive), and the *unproven* bounded agent bundle (A1).

## What each product is

- **Dynatrace** (NYSE: DT) — closed, enterprise **AIOps/observability platform**: **Davis AI** (anomaly detection + **automatic causal root-cause analysis**), **OneAgent** (deep, zero-config, code-level auto-instrumentation across full stack), **Grail** (the columnar store for logs/metrics/traces/events, up to **10-year** retention), **Smartscape/topology** (live dependency graph), **OTLP/OpenTelemetry** (hybrid OneAgent + OTLP ingest), **AI Observability** (LLM/agent via OTLP), K8s log module. Closed SaaS (proprietary; OneAgent/ActiveGate); **no real OSS self-host**. **Perform 2026 (2026-03-04) shipped the "agentic operations platform / agent control plane"**: Dynatrace Intelligence + new Smartscape (real-time dep graph) + Intelligence Agents (auto-remediate) + Dynatrace Assist + **Dynatrace MCP Server** — explicitly "bounded agent context" for autonomous agents (see AI-native section). Continuous SaaS release (no OSS version number to pin).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both touch "AI-driven investigation" and (now) "agent" framing, but Dynatrace is a closed enterprise AIOps suite; Parallax is an open self-hosted agent-context engine. Compare axis-by-axis.

## Signal coverage

| Signal | Dynatrace (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Traces / APM | ✅ (OneAgent deep + OTLP) | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ (Grail; K8s log module) | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ | ✅🧪 OTLP metrics (shipped, pre-release) |
| Real-user / digital experience | ✅ (Real User + synthetics) | ❌ |
| Continuous profiling | ✅ | ❌ |
| Topology / Smartscape (live dep graph) | ✅ (the Davis substrate) | ❌ (🏗 evidence graph) |
| Errors / exceptions | ✅ (OneAgent deep + Davis RCA) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Davis AI causal RCA | ✅ (distinctive) | 🟡 (🏗) |
| LLM / agent obs (AI Observability) | ✅ (OTLP) | 🟡🧪 agent-session modules (partial) |
| Sentry envelope / DSN | ❌ | ✅ shipped (sentry_http.rs) |

**Verdict:** Dynatrace's coverage is comprehensive and all shipped. On coverage, **Dynatrace wins decisively.** Parallax ships Sentry-envelope ingest (Dynatrace has none) — a real Parallax-favorable cell.

## Ingestion & transport

- **OneAgent + OTLP hybrid:** Dynatrace's signature is **OneAgent** — deep, zero-config, code-level auto-instrumentation (no SDK wiring) with automatic service discovery, **plus** full **OTLP/OpenTelemetry** ingest (send OTLP traces/logs alongside OneAgent). Davis AI works on OTLP-ingested data too.
- **Sentry envelope:** Dynatrace has **none**. Parallax **ships** Sentry-envelope ingest.

**Verdict:** on **deep auto-instrumentation (OneAgent), Dynatrace wins decisively** (Parallax relies on OTel SDKs). On OTLP, both. On Sentry-envelope, **Parallax wins** (shipped; Dynatrace has none).

## Storage architecture

- **Dynatrace:** **Grail** — the columnar, lakehouse-style store for all telemetry (logs/metrics/traces/events), with up to **10-year** retention; DQL (Dynatrace Query Language). Proprietary, SaaS.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, self-host single-binary.

**Verdict:** on **long-retention analytics (Grail, 10-yr) + proven-at-scale, Dynatrace wins.** On self-host + open storage, Parallax. GreptimeDB-vs-Grail is benchmark-dependent/unmeasurable (Grail internals proprietary).

## Query & correlation — Davis's substrate

- **Dynatrace:** **topology-driven** correlation — Smartscape's live dependency graph + Davis AI perform **causal** root-cause analysis (deterministic paths through the topology), not just statistical correlation. This is the distinctive axis: causation from the dependency graph.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **causal/topology-driven RCA, Dynatrace wins decisively** — it is the category leader for deterministic causation-based investigation. Parallax's evidence-graph is a different, agent-facing abstraction, unproven (A1).

## Error tracking & workflow

- **Dynatrace:** OneAgent-captured errors + Davis RCA + the Problems app (managed incidents) + AI-assisted ticket creation. A real, mature incident workflow.
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **shipped error/incident workflow + RCA, Dynatrace wins.** On the **fix-outcome loop**, Parallax targets an unoccupied cell (offline residual plan **123 DONE**; live value **unproven**).

## AI-native / agent-context story — the strategic overlap

- **Dynatrace's AI (pass-38 re-verify — WATCH TRIGGER FIRED):** the pass-16
  "Perform-2026 agent control plane → real bounded agent surface" watch is now
  **confirmed shipped** (Perform 2026, 2026-03-04). Dynatrace has evolved into an
  **"agentic operations platform"** with concrete, named components:
  - **Dynatrace Intelligence** — the reasoning/decision-making layer; fuses
    deterministic AI + contextual analytics to **ground agentic decisions in
    real-time facts** and "minimize hallucinations so organizations can trust
    automated actions" — Dynatrace's own words for *bounded agent context*.
  - **New Smartscape®** — real-time dependency graph, "**a source of truth for
    AI**" that both humans **and AI agents** use; precise, always-current view of
    every entity/dependency (cloud, K8s, metadata, agentless-discovered).
    **= a shipped real-time context/topology graph for agents.**
  - **Dynatrace Intelligence Agents** — auto-**remediate**, auto-**prevent**,
    auto-**optimize**; ready-made agents or build-your-own. **= autonomous action.**
  - **Dynatrace Assist** — conversational portal pulling context from Grail +
    Smartscape + collaborating with agents.
  - **Dynatrace MCP Server** — governed bridge via **Model Context Protocol**
    that "delivers real-time insights into agentic workflows" and "gives
    autonomous agents the observability truth they need to reason accurately,
    take decisive actions, and operate safely." **= shipped MCP for agent context.**
  - Plus pre-existing **Davis AI causal RCA** + **AI Observability** (LLM/agent via OTLP).
- **Parallax's claim:** bounded, redacted, agent-use (safety/value unproven) evidence bundle served to coding agents (**code-shipped**, A1 value unproven).

**Honest verdict (pass-38, no-bias — the strongest incumbent collision yet):**
Dynatrace now **ships a production-grade "context/grounding layer for agents" at
enterprise scale** — Smartscape (real-time truth graph) + an MCP server +
agent grounding ("bounded context" in Dynatrace's own framing) + autonomous
remediation. This is a **direct, named collision with Parallax's "bounded context
for agents" thesis**, and Dynatrace executes it with a far more mature substrate
(Davis causal RCA + Smartscape topology + Grail 10-yr lakehouse). **Important
no-bias nuance — the mechanisms differ, so this is not literal parity:** Dynatrace's
"bounded context" = grounding agents in **live, proprietary, SaaS topology/causal
facts via MCP** (for enterprise DevOps/ops agents, governed inside Dynatrace);
Parallax's = a **portable, redacted, versioned evidence bundle for a coding
agent's fix loop** (self-hosted, Apache). Overlapping *intent* (reliable agent
context), different *mechanism* + *deployment* + *agent*. Parallax's surviving
edges narrow to: **open/self-host** (Dynatrace closed SaaS, enterprise-expensive),
**cost**, **redaction as a first-class property**, **portable/versioned bundle**,
**Sentry-envelope + prod-error/outcome loop** — and **all are A1-unproven**, now
under direct pressure from a shipped enterprise competitor claiming the same
"bounded context for agents" ground.

## Architecture & deployment

- **Dynatrace:** **closed SaaS** (Dynatrace Platform); OneAgent/ActiveGate run in your env but ship to Dynatrace. No OSS self-host.
- **Parallax:** single-binary self-host target, local-first, offline/local deployment target (air-gap unverified), Apache-2.0.

**Verdict:** on **self-host / data sovereignty, Parallax wins by design** (Dynatrace is closed SaaS). On managed SaaS + enterprise deployment maturity, Dynatrace wins.

## Operational footprint

- **Dynatrace:** SaaS = zero backend ops; you run OneAgent/ActiveGate. Enterprise-grade day-2.
- **Parallax:** self-hosted GreptimeDB + Turso + engine.

**Verdict:** on **operator burden, Dynatrace (SaaS) is lower.** On cash cost + vendor dependency, Parallax. Scoped.

## Scalability & performance

- **Dynatrace:** proven at hyperscale (large enterprise customers, NYSE-listed). Specific numbers vendor; not independently measured.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale + maturity, Dynatrace wins conclusively.**

## Security

- **Dynatrace:** SSO/SAML, RBAC, audit, compliance — enterprise-grade. Mature.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped enterprise security, Dynatrace wins decisively.**

## Privacy & compliance

- **Dynatrace:** SOC2/ISO27001/HIPAA/FedRAMP/PCI, data residency. Mature enterprise.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** on **compliance, Dynatrace wins decisively.**

## Openness, licensing & vendor lock-in

- **Dynatrace:** **closed-source proprietary SaaS.** High vendor lock-in (OneAgent instrumentation, DQL, Grail, Smartscape — all proprietary). No self-host path.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness and lock-in cost, Parallax wins decisively** (Apache OSS + OTLP-native + self-host vs closed SaaS + deep proprietary instrumentation). This is the strongest structural Parallax edge vs Dynatrace.

## Pricing & economics — real numbers

Dynatrace pricing is **public** ([dynatrace.com/pricing](https://www.dynatrace.com/pricing/) + [rate-card](https://www.dynatrace.com/pricing/rate-card/), accessed 2026-07-17; **pricing corrected pass 39** — pass-16's "$0.08/host-hr Full-Stack" was a third-party error; the official rate-card is **memory-metered**, not flat per host; **pass 65 live re-confirm** holds host/telemetry rates; **log Grail rates corrected** — see table):

| Component | Price | Notes |
| --- | --- | --- |
| **Full-Stack Monitoring** | **$0.01 / memory-GiB-hour** | **memory-metered** (8 GiB host ≈ **$58/mo**); Davis AI **included**; OTel metrics/traces included |
| **Infrastructure Monitoring** | **$0.04 / host-hr** flat | ~$29/mo |
| **Foundation & Discovery** | **$0.01 / host-hr** | ~$7/mo basic host health |
| **Kubernetes Platform Monitoring** | **$1.40 / pod-mo** | free when host already Full-Stack |
| **Logs — Pay-per-Query** | Ingest **$0.20/GiB** · Retain **$0.0007/GiB-day** · Query **$0.0035/GiB-scanned** | live pricing pass 65 |
| **Logs — Bundled Queries** | Ingest **$0.20/GiB** · Retain+queries **$0.02/GiB-day** | min 10–35 days queries included |
| **Telemetry Metrics** | Ingest **$0.15 / 100k DP** · Retain **$0.0007/GiB-day** · Query **$0** | 15 mo @ 1-min included |
| **Telemetry Traces/Events** | Ingest **$0.20/GiB** · Retain **$0.0007/GiB-day** · Query **$0.0035/GiB-scanned** | 10-day traces included on Full-Stack |
| **RUM** | **$2.25 / 1k sessions** (replay **$4.50 / 1k**) | |
| Mainframe | $0.10 / MSU-hour | |

> ⚠️ **Correction (no-bias):** Full-Stack is **$0.01/GiB-hr memory-metered**, NOT pass-16's wrong "$0.08/host-hr". **Pass 65 log correction:** prior deep-dive "Grail log $0.40–0.60/GB" is **stale vs live page** — public Log Analytics is **$0.20/GiB ingest** + separate retain/query (pay-per-query or bundled). Dynatrace remains enterprise-expensive in aggregate (DPS contracts); public unit rates are more transparent and lower-floor than older third-party summaries.

**Real-world contracts (third-party):** SMB avg **~$100–182K/yr**; enterprise avg **~$1.05M/yr** ([Spendhound](https://www.spendhound.com/marketplace/dynatrace-pricing), [Vendr](https://www.vendr.com/marketplace/dynatrace) avg ~$170K/yr, [CheckThat.ai](https://checkthat.ai/brands/dynatrace/pricing)). **Dynatrace is enterprise-expensive at aggregate scale** — but the corrected per-host floor (~$58/mo 8GiB, Davis bundled) is more competitive than pass-16 stated.

**Parallax pricing:** none public yet (pre-release); stated shape = Apache open core + managed cloud + outcome-priced fixer.

**Honest cost read:** Dynatrace is among the most expensive observability platforms (enterprise contracts in the $100K–$1M+/yr range). Whether Parallax self-host is cheaper is benchmark-dependent and unmeasured; any cost opening remains a hypothesis while Parallax is pre-release and unproven at scale.

## Where Dynatrace plainly wins

- **Davis AI causal RCA** (topology-driven deterministic root cause — the distinctive axis).
- **OneAgent deep zero-config auto-instrumentation** (code-level, full-stack, auto service discovery).
- **Topology / Smartscape** (live dependency graph — Davis's substrate).
- Grail long-retention (10-yr) analytics + DQL.
- Proven-at-hyperscale + enterprise security/compliance (SOC2/ISO/HIPAA/FedRAMP/PCI).
- AI Observability (LLM/agent via OTLP) + **the shipped Perform-2026 agentic operations platform** (Dynatrace Intelligence + Smartscape truth-graph + Intelligence Agents + **MCP Server** = "bounded agent context" at enterprise scale — the pass-16 watch trigger **FIRED**).

## Where Parallax honestly edges Dynatrace

- **Openness / lock-in** — Apache-2.0 OTLP-native self-host vs closed SaaS + deep proprietary instrumentation. *(Real, decisive.)*
- **Self-host / data sovereignty** — Parallax designed for it; Dynatrace is SaaS-only. *(Real.)*
- **Cost** — Dynatrace is enterprise-expensive ($100K–$1M+/yr); Parallax self-host targets a real price opening. *(Real gap; Parallax pre-release/unproven.)*
- **Sentry-envelope compatibility** — Dynatrace has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-use (safety/value unproven) evidence bundle + fix-outcome loop** — *narrowed by pass-38:* Dynatrace now ships its own "bounded agent context" (Smartscape + MCP Server + Intelligence grounding), so the *intent* overlaps. Parallax's residual edge is specifically **portable + redacted + versioned bundle for a coding-agent fix loop** (vs Dynatrace's live, proprietary, SaaS, enterprise-ops-agent grounding). *(Thesis, unproven, A1 — and now directly pressured by a shipped enterprise competitor.)*

> **Honest summary:** Dynatrace is the category leader for **deterministic causal RCA** (Davis + OneAgent topology) and enterprise AIOps — far ahead of pre-release Parallax on AI-RCA, auto-instrumentation, topology, scale, compliance. **Pass-38: its "agent control plane" is no longer a forthcoming pivot but a SHIPPED agentic-operations platform** (Dynatrace Intelligence + Smartscape truth-graph + Intelligence Agents + MCP Server) that explicitly delivers **"bounded agent context"** to autonomous agents — a **direct, named collision with Parallax's "bounded context for agents" thesis**, on a far more mature substrate. Parallax's defensible delta narrows to **openness/cost/self-host** (Apache vs closed; Dynatrace is enterprise-expensive) + the **portable+redacted+versioned bundle for a coding-agent fix loop** specifically (A1 unproven, now under direct enterprise pressure) + **Sentry-envelope**. Do not claim "AI RCA," "agent," or now **"bounded context for agents"** as Parallax-unique — Dynatrace leads all three today.

## Watch triggers (the point of tracking Dynatrace)

1. **"Agent control plane" → real bounded agent-context surface** — **FIRED pass 38** (Perform 2026): Dynatrace Intelligence + Smartscape + Intelligence Agents + **MCP Server**. Residual watch: portable redacted coding-agent dossier (not live SaaS MCP).
2. **Davis-as-agent-context** — MCP exposes observability truth to agents (**shipped**); A1 vs Davis still unmeasured.
3. **Cost / self-host tier** — still SaaS-first; no Apache self-host backend. UNFIRED for self-host product.

**As of 2026-07-17 (pass 65):** trigger 1 remains **FIRED**. Live pricing re-confirm holds Full-Stack **$0.01/GiB-hr**; logs **$0.20/GiB ingest** (not $0.40–0.60).

## Open questions / what measurement would settle

- **A1 gate vs Davis/MCP grounding:** does a Parallax portable redacted bundle beat Dynatrace MCP-grounded agents for coding-agent fix outcomes? Unproven (not desk-research).
- ~~Dynatrace "agent control plane" substance~~ → **FIRED pass 38** (shipped MCP + Intelligence Agents).
- ~~Dynatrace exact current pricing~~ → **RESOLVED pass 39 + pass 65 re-confirm:** Full-Stack **$0.01/GiB-hr**, Infra **$0.04/host-hr**, K8s **$1.40/pod-mo**, logs **$0.20/GiB ingest** + retain/query split (live page).

## Sources (accessed 2026-07-17; agent-control-plane watch resolved pass 38)

- [Dynatrace pricing](https://www.dynatrace.com/pricing/); [OneAgent+OTLP docs](https://docs.dynatrace.com/docs/ingest-from/dynatrace-oneagent/oneagent-and-opentelemetry/oneagent-otel); [Grail log-obs blog](https://www.dynatrace.com/news/blog/how-dynatrace-supercharged-log-observability-in-2025/).
- **[Perform 2026 agentic-AI blog (2026-03-04)](https://www.dynatrace.com/news/blog/dynatrace-introduces-a-new-foundation-for-agentic-ai-at-perform-2026/)** — Dynatrace Intelligence, new Smartscape (truth graph for AI), Intelligence Agents (auto-remediate/prevent/optimize), Dynatrace Assist, **Dynatrace MCP Server** ("observability truth" to agents).
- [Perform 2026 press release](https://www.dynatrace.com/news/press-release/perform-2026-ignites-new-era/); [Futurum "agent OS" analysis](https://futurumgroup.com/insights/dynatrace-perform-2026-is-observability-the-new-agent-os/); [Diginomica: determinism-first](https://diginomica.com/dynatrace-perform-2026-why-agentic-ai-only-works-when-determinism-comes-first); [Dynatrace agentic-AI control-plane blog](https://www.dynatrace.com/news/blog/agentic-ai-report-reliable-autonomous-operations/).
- Third-party pricing: [Spendhound](https://www.spendhound.com/marketplace/dynatrace-pricing), [Vendr](https://www.vendr.com/marketplace/dynatrace), [CheckThat.ai](https://checkthat.ai/brands/dynatrace/pricing).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/), [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md).
