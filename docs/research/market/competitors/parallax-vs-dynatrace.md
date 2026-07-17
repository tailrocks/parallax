# Parallax vs Dynatrace

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [dynatrace.com/pricing](https://www.dynatrace.com/pricing/), [OneAgent+OTLP docs](https://docs.dynatrace.com/docs/ingest-from/dynatrace-oneagent/oneagent-and-opentelemetry/oneagent-otel), [Grail log-observability blog](https://www.dynatrace.com/news/blog/how-dynatrace-supercharged-log-observability-in-2025/), [Perform 2026 press](https://www.dynatrace.com/news/press-release/perform-2026-ignites-new-era/), third-party pricing (Spendhound/Vendr/CheckThat.ai).
>
> **Bottom line up front:** Dynatrace is a **closed, enterprise AIOps incumbent**
> with a genuinely distinctive axis: **Davis AI deterministic causal root-cause
> analysis** built on **OneAgent's** deep, zero-config, code-level auto-instrumentation
> and full dependency **topology** — causation, not just correlation. On **AI-driven
> RCA, deep auto-instrumentation, topology, the Grail long-retention store, and
> enterprise scale, Dynatrace is far ahead of pre-release Parallax.** Dynatrace is
> also pivoting (Perform 2026) to an **"agent control plane"** framing — a strategic
> overlap with Parallax's agent thesis. Parallax's honest edges are
> **open-source/self-host** (Dynatrace is closed SaaS), **Apache-2.0**, **cost**
> (Dynatrace is enterprise-expensive), and the *unproven* bounded agent bundle (A1).

## What each product is

- **Dynatrace** (NYSE: DT) — closed, enterprise **AIOps/observability platform**: **Davis AI** (anomaly detection + **automatic causal root-cause analysis**), **OneAgent** (deep, zero-config, code-level auto-instrumentation across full stack), **Grail** (the columnar store for logs/metrics/traces/events, up to **10-year** retention), **Smartscape/topology** (live dependency graph), **OTLP/OpenTelemetry** (hybrid OneAgent + OTLP ingest), **AI Observability** (LLM/agent via OTLP), K8s log module. Closed SaaS (proprietary; OneAgent/ActiveGate); **no real OSS self-host**. Perform 2026 (Jan 2026) pivoted positioning to **"agent control plane" / autonomous observability**. Continuous SaaS release (no OSS version number to pin).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both touch "AI-driven investigation" and (now) "agent" framing, but Dynatrace is a closed enterprise AIOps suite; Parallax is an open self-hosted agent-context engine. Compare axis-by-axis.

## Signal coverage

| Signal | Dynatrace (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Traces / APM | ✅ (OneAgent deep + OTLP) | ✅ OTLP traces (🏗) |
| Logs | ✅ (Grail; K8s log module) | ✅ OTLP logs (🏗) |
| Metrics | ✅ | ✅ OTLP metrics (🏗) |
| Real-user / digital experience | ✅ (Real User + synthetics) | ❌ |
| Continuous profiling | ✅ | ❌ |
| Topology / Smartscape (live dep graph) | ✅ (the Davis substrate) | ❌ (🏗 evidence graph) |
| Errors / exceptions | ✅ (OneAgent deep + Davis RCA) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Davis AI causal RCA | ✅ (distinctive) | 🟡 (🏗) |
| LLM / agent obs (AI Observability) | ✅ (OTLP) | ✅ (🏗) |
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
- **Parallax:** derived `error_event` + fingerprint + (planned) fix-outcome loop.

**Verdict:** on **shipped error/incident workflow + RCA, Dynatrace wins.** On the **fix-outcome loop**, Parallax targets an unoccupied cell (planned/unproven, A1).

## AI-native / agent-context story — the strategic overlap

- **Dynatrace's AI:** **Davis AI** (anomaly detection + **automatic causal RCA**) is genuinely strong and distinctive — topology-driven deterministic root cause. Plus **AI Observability** (LLM/agent via OTLP). And the **Perform 2026 "agent control plane"** pivot — Dynatrace positions its platform as an **OS/control-plane for AI agents** running IT/DevOps. This is a direct, announced overlap with Parallax's "context engine for agents" thesis.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle served to coding agents (planned, A1 gate).

**Honest verdict:** On shipped AI (Davis causal RCA + AI Observability), **Dynatrace leads decisively.** The "agent control plane" pivot is strategically important: **Dynatrace is explicitly moving into Parallax's "agent" framing** — if it ships a real bounded agent-context surface, it's a collision (watch). But today Davis is a **human-AIOps causation engine**, not a bounded, read-only, redacted agent-context projection. Parallax's differentiated bundle is **unproven (A1).** The burden of proof that Parallax's bundle beats Davis-causal-RCA-as-context is on Parallax.

## Architecture & deployment

- **Dynatrace:** **closed SaaS** (Dynatrace Platform); OneAgent/ActiveGate run in your env but ship to Dynatrace. No OSS self-host.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

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

Dynatrace pricing is **public** ([dynatrace.com/pricing](https://www.dynatrace.com/pricing/), accessed 2026-07-17), **consumption + seat based**:

| Component | Price | Notes |
| --- | --- | --- |
| Full-Stack Monitoring | ~**$0.08 / host-hr** | billed hourly |
| Infrastructure Monitoring | ~**$0.04 / host-hr** | |
| Application Observability | ~**$29 / host-mo** | |
| Kubernetes/App Observability | **$1.40 / pod-mo** | |
| Grail log overage | **$0.40–$0.60 / GB** | beyond 100 GB/mo free |
| User seats | **$49–$349 / user-mo** | plan-dependent |

**Real-world contracts (third-party):** SMB avg **~$100–182K/yr**; enterprise avg **~$1.05M/yr** ([Spendhound](https://www.spendhound.com/marketplace/dynatrace-pricing), [Vendr](https://www.vendr.com/marketplace/dynatrace) avg ~$170K/yr, [CheckThat.ai](https://checkthat.ai/brands/dynatrace/pricing)). **Dynatrace is enterprise-expensive** — a documented, structural fact.

**Parallax pricing:** none public yet (pre-release); stated shape = Apache open core + managed cloud + outcome-priced fixer.

**Honest cost read:** Dynatrace is among the most expensive observability platforms (enterprise contracts in the $100K–$1M+/yr range). Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured, but the **cost gap is a real Parallax opening** for price-sensitive buyers — though Parallax is pre-release and unproven at scale.

## Where Dynatrace plainly wins

- **Davis AI causal RCA** (topology-driven deterministic root cause — the distinctive axis).
- **OneAgent deep zero-config auto-instrumentation** (code-level, full-stack, auto service discovery).
- **Topology / Smartscape** (live dependency graph — Davis's substrate).
- Grail long-retention (10-yr) analytics + DQL.
- Proven-at-hyperscale + enterprise security/compliance (SOC2/ISO/HIPAA/FedRAMP/PCI).
- AI Observability (LLM/agent via OTLP) + the "agent control plane" pivot.

## Where Parallax honestly edges Dynatrace

- **Openness / lock-in** — Apache-2.0 OTLP-native self-host vs closed SaaS + deep proprietary instrumentation. *(Real, decisive.)*
- **Self-host / data sovereignty** — Parallax designed for it; Dynatrace is SaaS-only. *(Real.)*
- **Cost** — Dynatrace is enterprise-expensive ($100K–$1M+/yr); Parallax self-host targets a real price opening. *(Real gap; Parallax pre-release/unproven.)*
- **Sentry-envelope compatibility** — Dynatrace has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-safe evidence bundle + fix-outcome loop** — Dynatrace has neither (Davis is human-AIOps RCA). *(Thesis, unproven, A1; and Dynatrace's "agent control plane" pivot is a collision risk to watch.)*

> **Honest summary:** Dynatrace is the category leader for **deterministic causal RCA** (Davis + OneAgent topology) and enterprise AIOps — far ahead of pre-release Parallax on AI-RCA, auto-instrumentation, topology, scale, compliance. Its **"agent control plane" pivot is a strategic collision risk** with Parallax's agent thesis. Parallax's defensible delta is entirely **openness/cost/self-host** (Apache vs closed; Dynatrace is enterprise-expensive) + **Sentry-envelope** + the **bounded+outcome bundle** (A1 unproven). Do not claim "AI RCA" or "agent" as Parallax-unique — Dynatrace leads both today.

## Watch triggers (the point of tracking Dynatrace)

1. **"Agent control plane" → real bounded agent-context surface** — if Dynatrace ships a bounded/read-only agent projection (not just Davis-for-humans), it's a direct collision with Parallax's thesis. **Highest-priority watch.**
2. **Davis-as-agent-context** — does Dynatrace expose Davis causal-RCA to agents programmatically/safely?
3. **Cost** — track whether Dynatrace adds a cheaper/self-host tier (unlikely).

**As of 2026-07-17:** the "agent control plane" is **positioning**, not a shipped bounded agent-context surface. Davis remains human-AIOps. Trigger not yet fired — but the announced direction is the most serious strategic threat to Parallax's agent framing from an incumbent.

## Open questions / what measurement would settle

- **A1 gate vs Davis:** does a Parallax bounded bundle beat Davis-causal-RCA-as-context for coding-agent fix outcomes? Unproven — and Davis's deterministic causation is a high bar.
- **Dynatrace "agent control plane" substance** — is it a real bounded agent surface or marketing? Track Perform 2026 follow-through.
- **Dynatrace exact current pricing** — confirm host-hr/pod/GB/seat rates on the live pricing page (third-party figures are indicative).

## Sources (accessed 2026-07-17)

- [Dynatrace pricing](https://www.dynatrace.com/pricing/); [OneAgent+OTLP docs](https://docs.dynatrace.com/docs/ingest-from/dynatrace-oneagent/oneagent-and-opentelemetry/oneagent-otel); [Grail log-obs blog](https://www.dynatrace.com/news/blog/how-dynatrace-supercharged-log-observability-in-2025/).
- [Perform 2026 press release](https://www.dynatrace.com/news/press-release/perform-2026-ignites-new-era/); [Futurum "agent OS" analysis](https://futurumgroup.com/insights/dynatrace-perform-2026-is-observability-the-new-agent-os/).
- Third-party pricing: [Spendhound](https://www.spendhound.com/marketplace/dynatrace-pricing), [Vendr](https://www.vendr.com/marketplace/dynatrace), [CheckThat.ai](https://checkthat.ai/brands/dynatrace/pricing).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/), [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md).
