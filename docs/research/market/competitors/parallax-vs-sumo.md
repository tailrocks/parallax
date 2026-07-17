# Parallax vs Sumo Logic

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 41
> Flex pricing + Dojo AI RESOLVED**). Sources: live [sumologic.com/pricing](https://www.sumologic.com/pricing),
> [Cloud Flex Credit overview](https://www.sumologic.com/pricing/cloud-flex-credit),
> [OTel guide](https://www.sumologic.com/guides/opentelemetry),
> [CNCF OTel-bet blog](https://cncf.io/blog/2022/12/13/why-sumo-logic-is-betting-its-future-on-opentelemetry/).
>
> **Bottom line up front:** Sumo Logic is a **cloud logs-first observability + SIEM
> SaaS** (Francisco Partners-owned since 2023) with distinctive **Flex (ingest free;
> pay scan + storage credits)** pricing and early OpenTelemetry adoption. **Dojo AI**
> (Mobot + Query/Knowledge/Summary/SOC-Analyst agents) is now on the live pricing
> page — another incumbent AI surface. On logs/SIEM/scan-pricing/SaaS maturity, Sumo
> is far ahead of pre-release Parallax. Parallax edges = open/self-host, Apache-2.0,
> full-signal + error workflow, unproven bundle (A1).

## What each product is

- **Sumo Logic** — a **cloud-native logs-first observability + SIEM/security SaaS**: log analytics (the origin strength), metrics, traces/APM, real-user monitoring, **Cloud SIEM** (security), **Cloud Log Analytics**, "Continuous Intelligence." Distinctive **Flex pricing** (charge for data *scanned at query time*, not ingested — log ingest itself free). **Early OpenTelemetry adopter** (OTel is core strategy; official Sumo Logic OTel Collector exporter). **Acquired by Francisco Partners in 2023 (~$1.7B take-private).** Closed SaaS. Continuous SaaS release.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both OTLP/OTel-capable. Sumo is a closed logs-first + SIEM SaaS; Parallax is an open self-hosted agent-context engine.

## Signal coverage

| Signal | Sumo Logic (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Logs (analytics) | ✅ **(origin strength — logs-first intelligence)** | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ | ✅🧪 OTLP metrics (shipped, pre-release) |
| Traces / APM | ✅ | ✅🧪 OTLP traces (shipped, pre-release) |
| RUM | ✅ | ❌ |
| **Cloud SIEM / security** | ✅ (distinctive) | ❌ |
| Errors / exceptions | 🟡 (log/metric-centric; no Sentry-grade lifecycle) | ✅ derived `error_event` + fingerprint (🧪 shipped) |
| Continuous Intelligence / alerts | ✅ | 🟡 (🏗) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict:** Sumo's coverage is broad and shipped, logs-first + SIEM-strong. On coverage, **Sumo wins decisively.** No Sentry-envelope, no fix-outcome loop (same gap).

## Ingestion & transport

- **OTLP/OTel:** Sumo is **OTel-capable** — early adopter, official Sumo Logic OTel Collector exporter for metrics/logs/traces. Not OTLP-native-storage (data lands in Sumo's proprietary backend), but OTel is the ingestion standard.
- **Sentry envelope:** none.
- **Collection:** Sumo agents + OTel Collector.

**Verdict:** on OTel ingestion, **Sumo wins on maturity** (shipped). On OTLP-native storage + Sentry-envelope, **Parallax ships both**.

## Storage architecture

- **Sumo Logic:** proprietary SaaS backend (logs-first, scan-optimized for Flex pricing); internals not public. Flex = query-scan-based cost.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, self-host single-binary.

**Verdict:** on **self-host + open storage, Parallax wins by design.** On proven-at-scale + the logs-analytics/SIEM niche, Sumo wins. Unmeasurable head-to-head (Sumo backend proprietary).

## Query & correlation

- **Sumo Logic:** log-search-centric query + dashboards + Continuous Intelligence; cross-signal correlation; **Cloud SIEM** investigation (logs + security signals). Mature.
- **Parallax:** evidence-graph correlation + bounded bundle for agents.

**Verdict:** on **log-search + SIEM investigation, Sumo wins** (mature, logs-first). Parallax's bundle is a different axis (bounded agent context), unproven (A1).

## Error tracking & workflow

- **Sumo Logic:** errors are queryable log/metric signals; **no native Sentry-grade error-issue lifecycle.**
- **Parallax:** derived `error_event` + fingerprint (**shipped**) + fix-outcome offline residual (**plan 123 DONE**; live value **unproven**).

**Verdict:** on **error-issue workflow, Parallax ships error derivation + fingerprint** (pre-release); fix-outcome offline residual plan 123 DONE, live value **unproven**.

## AI-native / agent-context story

- **Sumo Logic (pass 41 re-verify on live pricing page):** **Dojo AI** ships a multi-agent surface — **Mobot** conversational UI + **Query Agent** (NL→query) + **Knowledge Agent** (platform how-to) + **Summary Agent** (Insight signal condensation; SIEM) + **SOC Analyst Agent (preview)** (alert triage; SIEM). Plus AI-driven alerting / ML RCA / anomaly. This is a **shipped multi-agent assistive/investigation surface** — human+security-ops oriented, **not** a bounded/redacted/portable coding-agent evidence bundle.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding agents (**code-shipped**, A1 value unproven).

**Honest verdict (no-bias):** Sumo now ships **agent-flavored investigation tools** (Dojo AI), so “Sumo has only classic ML anomaly” is **stale**. On shipped AI investigation, **Sumo leads** pre-release Parallax. Parallax’s residual claim is still the **bounded/redacted/portable prod-incident bundle + outcome loop** (A1 unproven) — not “AI exists.”

## Architecture & deployment

- **Sumo Logic:** **closed SaaS** (Francisco Partners-owned). Collectors run in your env but ship to Sumo. No OSS self-host.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **self-host / data sovereignty, Parallax wins by design** (Sumo is SaaS-only). On managed SaaS + enterprise maturity, Sumo wins.

## Operational footprint / Scalability

- **Sumo Logic:** SaaS = zero backend ops; proven at scale (long-standing log-analytics incumbent). Specific numbers vendor; not independently measured.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale + zero-ops, Sumo wins conclusively.**

## Security / compliance

- **Sumo Logic:** SSO/SAML, RBAC, audit; **+ Cloud SIEM** (a major strength — Sumo is a security player). SOC2/ISO27001/HIPAA/PCI/FedRAMP. Mature.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security/compliance (esp. SIEM), Sumo wins decisively.**

## Openness, licensing & vendor lock-in

- **Sumo Logic:** **closed-source proprietary SaaS.** High vendor lock-in (proprietary backend, query language, Flex metering). No self-host.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness and lock-in cost, Parallax wins decisively** (Apache OSS + OTLP-native + self-host vs closed SaaS).

## Pricing & economics — Flex RESOLVED pass 41 (no fixed public $/GB without calculator)

Primary: live [sumologic.com/pricing](https://www.sumologic.com/pricing) (accessed 2026-07-17).

| Fact | Status |
| --- | --- |
| **Plans** | **Essentials** (self-serve trial) + **Enterprise Suite** (sales) |
| **Flex model** | **$0 log ingest** (excl. Cloud SIEM path); credits consumed by **storage + query scan**; SIEM ingest *does* consume credits |
| **Scan-usage profiles (MSRP estimator)** | **500–750** / **750–1,500** / **1,500–2,000** scans per GB ingested (low / mid / high analytics intensity) |
| **Estimated $/TB scanned** | **Interactive calculator only** — page does **not** publish a static dollar rate without region + profile inputs. Secondary proxies historically ~**$2.05–$3.14 / TB scanned** by usage band ([Exabeam explainer](https://www.exabeam.com/explainers/sumo-logic/sumo-logic-solution-overview-limitations-and-alternatives/), older LinkedIn/teardowns) — treat as **medium/low confidence**, not live list price |
| **Credit unit** | Annual credit buckets; self-serve credit-card up to **$25,000**; real-time credit tracking |
| **Essentials capacity (live matrix)** | Metrics up to **50,000/day**; tracing up to **5 GB/day**; log retention up to **365 days**; 300/500 real-time log/metric monitors |
| **Enterprise Suite capacity** | Metrics/tracing **unlimited** (volume-quoted); customer-defined log retention; SIEM + SOAR activation subject to mins |
| **DPM (metrics)** | Defined on pricing FAQ as average metric data points per minute in 1k increments (licensing unit) |

**What is *not* public as a single number:** a universal list **$/GB scanned** or **$/credit** independent of region, commitment, and analytics profile. Honest label: **no single public number** — quote/estimator required.

**Parallax pricing:** **no public number** (pre-release); self-host = no per-event/per-scan tax by design.

**Honest cost read:** Flex inverts ingest-tax (ingest free, pay scan+storage) — distinctive vs Datadog-style ingest. Still **SaaS-metered**. Head-to-head vs Parallax self-host **unmeasured**.

## Where Sumo Logic plainly wins

- **Logs-first intelligence** (the origin strength — log analytics, search, classification).
- **Cloud SIEM / security** (a major differentiator — observability + security).
- **Flex scan-pricing** (distinctive — ingest free, pay for query scan).
- Early OTel adopter + OTel Collector exporter.
- Proven-at-scale + mature compliance (SOC2/ISO/HIPAA/PCI/FedRAMP).

## Where Parallax honestly edges Sumo Logic

- **Openness / lock-in** — Apache-2.0 OTLP-native self-host vs closed SaaS. *(Real, decisive.)*
- **Self-host / data sovereignty** — Parallax designed for it; Sumo is SaaS-only. *(Real.)*
- **Full-signal-native + production error-workflow** — Sumo is logs-first; Parallax **ships** production error derivation + fix-outcome offline residual (plan **123 DONE**; live value unproven). *(Real gap in Sumo; Parallax error path **shipped**.)*
- **Sentry-envelope compatibility** — Sumo has none; Parallax ships it. *(Real.)*
- **Bounded, redacted, agent-safe evidence bundle** — Sumo has none. *(Thesis, unproven, A1.)*

> **Honest summary:** Sumo Logic is a mature **logs-first + SIEM** cloud SaaS (Francisco Partners-owned) — far ahead of pre-release Parallax on logs analytics, security/SIEM, Flex scan-pricing, scale, compliance. Parallax's defensible delta is **openness/self-host** (Apache vs closed SaaS), **full-signal + production-error-native** (vs logs-first), **Sentry-envelope**, and the **bounded+outcome bundle** (A1 unproven). Sumo's Flex scan-pricing is a distinctive cost model but still SaaS-metered — not the same as Parallax's self-host-no-metering bet.

## Open questions / what measurement would settle

- **A1 gate:** does a Parallax bundle add value beyond Sumo's logs/SIEM + Dojo AI for coding-agent incident fixes? Unproven.
- **Sumo list $/TB scanned** — **resolved as no static public number**; estimator/quote only. Capacity matrix + Flex model verified.
- **Francisco Partners strategy** — track product direction/cadence under private ownership.
- **Dojo AI → bounded agent artifact** — if Sumo ships a portable redacted coding-agent bundle, collision with Parallax wedge. **Pass 58: UNFIRED** (Dojo remains assistive multi-agent on Sumo SaaS, not a portable redacted dossier).

## Sources (accessed 2026-07-17; pass 41 re-verify)

- [Sumo Logic pricing](https://www.sumologic.com/pricing) (live Flex estimator, Essentials/Enterprise matrix, Dojo AI feature list); [Cloud Flex Credit overview](https://www.sumologic.com/pricing/cloud-flex-credit).
- [OTel guide](https://www.sumologic.com/guides/opentelemetry); [CNCF: Sumo betting on OTel](https://cncf.io/blog/2022/12/13/why-sumo-logic-is-betting-its-future-on-opentelemetry/).
- Secondary scan-rate proxies only: [Exabeam](https://www.exabeam.com/explainers/sumo-logic/sumo-logic-solution-overview-limitations-and-alternatives/), [Coralogix](https://coralogix.com/blog/coralogix-vs-sumo-logic-features/).
- Parallax side: [decisions/storage-engine.md](../../decisions/storage-engine.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
