# Parallax vs Splunk Observability Cloud

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (AI surface
> re-verified **pass 36** against the [AI Agent Monitoring](https://www.splunk.com/en_us/blog/observability/monitor-llm-and-agent-performance-with-ai-agent-monitoring-in-splunk-observability-cloud.html)
> + [Cisco Live Agentic Observability](https://www.splunk.com/en_us/blog/observability/splunk-observability-at-cisco-live.html) blogs).
> Sources: [Splunk Observability Cloud](https://www.splunk.com/en_us/products/observability-cloud.html), [OTel-native blog](https://www.splunk.com/en_us/blog/devops/unlock-the-power-of-observability-with-opentelemetry-logs-data-model.html), [cubeapm 2026 pricing](https://cubeapm.com/blog/splunk-observability-cloud-pricing-and-review/), [BITSIO 2026 guide](https://www.bitsioinc.com/blog-post/splunk-observability-cloud-2026-guide).
>
> **Bottom line up front:** Splunk Observability Cloud (now **Cisco**-owned) is a
> **100% OpenTelemetry-native** full-stack SaaS with a distinctive **NoSample™
> tracing** (head-based 100% sampling — no lost traces). On **OTel-native parity,
> NoSample tracing, enterprise scale, Cisco backing, and the Splunk SIEM adjacency,
> it is far ahead of pre-release Parallax.** Parallax's honest edges are
> **open-source/self-host** (Obs Cloud is closed SaaS), **Apache-2.0**, **cost**
> (Splunk skews enterprise-expensive), and the *unproven* bounded agent bundle (A1).

## What each product is

- **Splunk Observability Cloud** (Cisco, post-2024 acquisition) — a **full-stack, OpenTelemetry-native** SaaS unifying **APM/traces, Infrastructure Monitoring (metrics), Log Observer (logs), RUM, Synthetics** through one OTel Collector pipeline. Distinctive: **NoSample™ tracing** (head-based, 100% trace sampling — no statistical loss). Components built on OTel standards; the Splunk Distribution of the OTel Collector is the collection path. Closed SaaS (Cisco-owned). Adjacent to **Splunk Enterprise/Cloud** (the log/SIEM/security incumbent). Continuous SaaS release.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OTLP/OTel-native. Splunk Obs Cloud is a closed enterprise full-stack SaaS; Parallax is an open self-hosted agent-context engine.

## Signal coverage

| Signal | Splunk Obs Cloud (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Traces / APM | ✅ (**NoSample™** 100% sampling) | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ (Log Observer) | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ (Infrastructure Monitoring) | ✅🧪 OTLP metrics (shipped, pre-release) |
| RUM | ✅ | ❌ |
| Synthetics | ✅ | ❌ |
| Errors / exceptions | 🟡 (queryable; no Sentry-grade issue lifecycle) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Real-time / NoSample tracing | ✅ (distinctive) | ❌ (samples by design) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Splunk's coverage is comprehensive and shipped. On coverage, **Splunk wins decisively.** Parallax ships Sentry-envelope ingest (Splunk has none) — a real Parallax-favorable cell.

## Ingestion & transport — OTel-native parity, plus NoSample

- **OTLP/OTel:** Splunk Obs Cloud is **100% OpenTelemetry-native** — logs/traces/metrics through the **Splunk Distribution of the OTel Collector**; data in the OTel schema. **Genuine OTLP-native parity with Parallax's design** (unlike Sentry/Datadog which transform). Splunk even publishes the OTel Logs Data Model adaptation.
- **NoSample™ tracing:** Splunk's distinctive — head-based sampling that retains **100% of traces** (no statistical loss). A real strength for complete-trace investigation.
- **Sentry envelope:** Splunk has **none**. Parallax ships it.

**Verdict:** on OTLP-native ingest, **tied in design; Splunk ships it.** On NoSample (100% trace retention), **Splunk wins** (Parallax samples). On Sentry-envelope, **Parallax wins** (shipped; Splunk none).

## Storage architecture

- **Splunk Obs Cloud:** proprietary SaaS backend (the SignalFx-derived metrics engine + log store); internals not public. Real-time ingest strength.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, self-host single-binary.

**Verdict:** on **self-host + open storage, Parallax wins by design.** On proven-at-scale + real-time ingest maturity, Splunk wins. Unmeasurable head-to-head (Splunk backend proprietary).

## Query & correlation

- **Splunk Obs Cloud:** unified cross-signal (trace↔log↔metric↔tag) investigation via the OTel schema; Splunk-style search/SPL for logs; APM service maps. Mature.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **general cross-signal investigation, Splunk wins** (mature, unified). Parallax's bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **Splunk Obs Cloud:** errors are queryable signals; **no native Sentry-grade issue lifecycle** (Splunk's issue workflow is log/SIEM-centric, not error-tracking-centric).
- **Parallax:** derived `error_event` + fingerprint + (planned) fix-outcome loop.

**Verdict:** on **error-issue workflow, Parallax targets a gap** — but planned/unproven.

## AI-native / agent-context story

- **Splunk's AI (pass-36 re-verify):** far beyond the pass-17 "Splunk AI = anomaly/alerting/assistant" read. Splunk/Cisco shipped a dedicated **AI/agent surface** in 2026:
  - **AI Agent Monitoring** — monitor LLM + agent **performance, quality, security, and cost**; built on **OpenTelemetry + Cisco AGNTCY** (no vendor lock-in); Q1-2026 update added deeper AI insights.
  - **Agentic Observability** (Cisco Live 2026) — AI that **detects, investigates, summarizes, and recommends actions** with end-to-end AI-workload visibility = a **shipped autonomous investigator / RCA** surface.
  - **Cisco AI Defense integration** — compliance with AI standards + threat detection for AI apps.
  - **AI Agent Governance + Federated Analytics** — managing autonomous agent behavior at scale.
  - Still **not** a bounded, read-only, redacted, *portable* agent-context projection.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (planned, A1).

**Honest verdict (pass-36, no-bias):** Splunk now **ships agent-obs (AI Agent Monitoring) AND autonomous investigation (Agentic Observability)** — two cells Parallax aspired to are occupied by Splunk. This is the **4th shipped autonomous investigator** in the set (after HolmesGPT, Causely, Honeycomb Auto-investigations) — Parallax's "context-engine, *not* the fixer" thesis now faces four shipped "fixers." Parallax's remaining wedge narrows to the bounded/redacted/portable production-incident bundle + outcome loop (A1 unproven).

## Architecture & deployment

- **Splunk Obs Cloud:** **closed SaaS** (Cisco). The OTel Collector runs in your env but ships to Splunk. No OSS self-host of Obs Cloud (Splunk *Enterprise* self-hosts logs/SIEM separately, but that's a different product).
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **self-host / data sovereignty, Parallax wins by design** (Splunk Obs Cloud is SaaS-only). On managed SaaS + enterprise maturity, Splunk wins.

## Operational footprint

- **Splunk Obs Cloud:** SaaS = zero backend ops; you run the OTel Collector.
- **Parallax:** self-hosted GreptimeDB + Turso + engine.

**Verdict:** on **operator burden, Splunk (SaaS) is lower.** On cash cost + vendor dependency, Parallax. Scoped.

## Scalability & performance

- **Splunk Obs Cloud:** proven at hyperscale (Cisco-backed, large enterprise base; SignalFx metrics heritage). Specific numbers vendor; not independently measured.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale, Splunk wins conclusively.**

## Security

- **Splunk Obs Cloud:** SSO/SAML, RBAC, audit; **+ the Splunk SIEM/security heritage** (a major strength — Cisco security stack). Mature enterprise.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security (esp. SIEM heritage), Splunk wins decisively.**

## Privacy & compliance

- **Splunk Obs Cloud:** SOC2/ISO27001/HIPAA/FedRAMP/PCI; data residency. Mature enterprise.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** on **compliance, Splunk wins decisively.**

## Openness, licensing & vendor lock-in

- **Splunk Obs Cloud:** **closed-source proprietary SaaS** (Cisco). High vendor lock-in (proprietary backend, SPL, OTel-collector distribution). No self-host of Obs Cloud.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness and lock-in cost, Parallax wins decisively** (Apache OSS + OTLP-native + self-host vs closed Cisco SaaS). Strongest structural Parallax edge.

## Pricing & economics — real numbers

Splunk Observability Cloud pricing is **public** ([splunk.com/products/pricing/observability](https://www.splunk.com/en_us/products/pricing/observability.html) + [FAQ](https://www.splunk.com/en_us/products/pricing/faqs/observability.html), **confirmed pass 40** — the pass-17 third-party $15/$60/$75 figures match the official page):

| Component | Price | Notes |
| --- | --- | --- |
| Infrastructure Monitoring | **$15 / host / mo** | billed annually; metrics |
| Cloud App & Infra (IM + APM) | **$60 / host / mo** | + APM/traces |
| End-to-End Observability Cloud Suite | **$75 / host / mo** | + Log Observer + RUM + synthetics |
| **Free Edition** | **$0** | **15 hosts free forever**, all features |

**Pricing models:** host-based (unique hosts, hourly-averaged) **or** usage-based (custom metrics/containers/serverless). APM counts unique APM hosts/min, averaged across the cycle.

> **No public number (marked explicitly):** à-la-carte **credit rates** for individual Infrastructure/APM/Log-Observer/RUM components, standalone **Log Observer** pricing, and **log-overage rates** are **NOT published** on splunk.com — they require a sales quote. The published rates are the bundled host tiers ($15/$60/$75). Government (UK G-Cloud) lists the Suite at $95/host/mo (100-host min). Third-party full-stack-with-add-ons ranges cite $95–$200/host/mo.

Cisco-era enterprise contracts skew expensive (Splunk's historical reputation); the base host tiers are the published floor.

**Parallax pricing:** none public yet (pre-release).

**Honest cost read:** Splunk skews enterprise-expensive (per-host tiers + Cisco enterprise positioning). Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured, but cost is a real Parallax opening for price-sensitive buyers (Parallax pre-release/unproven).

## Where Splunk Obs Cloud plainly wins

- **100% OTel-native** (genuine OTLP-native parity — the cleanest of the closed incumbents on OTel).
- **NoSample™ tracing** (100% trace retention — distinctive).
- Full-stack breadth + RUM/synthetics + real-time ingest.
- Hyperscale + Cisco backing + SIEM/security heritage.
- Enterprise compliance (SOC2/ISO/HIPAA/FedRAMP/PCI).

## Where Parallax honestly edges Splunk Obs Cloud

- **Openness / lock-in** — Apache-2.0 OTLP-native self-host vs closed Cisco SaaS. *(Real, decisive.)*
- **Self-host / data sovereignty** — Parallax designed for it; Obs Cloud is SaaS-only. *(Real.)*
- **Cost** — Splunk enterprise-expensive; Parallax self-host targets a price opening. *(Real gap; Parallax pre-release.)*
- **Sentry-envelope compatibility** — Splunk has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-safe evidence bundle + fix-outcome loop** — Splunk has neither. *(Thesis, unproven, A1.)*

> **Honest summary:** Splunk Obs Cloud is the cleanest OTel-native closed incumbent (100% OTel + NoSample tracing) — far ahead of pre-release Parallax on OTel-maturity, breadth, scale, security/SIEM, compliance. Parallax's defensible delta is **openness/cost/self-host** (Apache vs closed Cisco SaaS; Splunk is enterprise-expensive) + **Sentry-envelope** + the **bounded+outcome bundle** (A1 unproven). Notably: on the OTLP-native axis, Splunk is at parity with Parallax's design (both truly native) — Parallax cannot claim OTel-native as a wedge vs Splunk.

## Open questions / what measurement would settle

- **A1 gate vs Splunk AI:** does a Parallax bounded bundle beat Splunk-AI-assistant-as-context for coding-agent fix outcomes? Unproven.
- **Cisco-era exact pricing + log overage** — **RESOLVED pass 40** ([splunk.com](https://www.splunk.com/en_us/products/pricing/observability.html)): Infra $15 / IM+APM $60 / Suite $75 per host/mo + Free Edition 15 hosts; pass-17 third-party numbers **confirmed correct**. **No public number** (marked): à-la-carte credit rates, standalone Log Observer, log-overage — sales-quote only. Still-open (NOT desk-research): NoSample storage-cost vs sampled GreptimeDB (benchmark); A1-vs-Splunk-Agentic-Obs.
- **NoSample cost** — 100% trace retention has a storage cost; how does it compare to Parallax's sampled GreptimeDB at parity? Benchmark-dependent.

## Sources (accessed 2026-07-17)

- [Splunk Observability Cloud](https://www.splunk.com/en_us/products/observability-cloud.html); [observability explainer](https://www.splunk.com/en_us/products/observability-explainer.html).
- [OTel-native / OTel Logs Data Model blog](https://www.splunk.com/en_us/blog/devops/unlock-the-power-of-observability-with-opentelemetry-logs-data-model.html).
- [cubeapm 2026 pricing](https://cubeapm.com/blog/splunk-observability-cloud-pricing-and-review/); [BITSIO 2026 guide](https://www.bitsioinc.com/blog-post/splunk-observability-cloud-2026-guide).
- **Pass-36 AI sources:** [Monitor LLM/Agent performance with AI Agent Monitoring](https://www.splunk.com/en_us/blog/observability/monitor-llm-and-agent-performance-with-ai-agent-monitoring-in-splunk-observability-cloud.html) (OTel + Cisco AGNTCY); [Splunk at Cisco Live: Agentic Observability](https://www.splunk.com/en_us/blog/observability/splunk-observability-at-cisco-live.html); [Q1-2026 observability update (Cisco AI Defense)](https://www.splunk.com/en_us/blog/observability/splunk-observability-ai-agent-monitoring-innovations.html); [Cisco/Splunk federated analytics + AI agent governance (TFiR)](https://tfir.io/cisco-splunk-federated-analytics-ai-agent-governance/).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
