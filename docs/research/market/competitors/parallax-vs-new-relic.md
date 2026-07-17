# Parallax vs New Relic

> One-to-one comparison. **No pro-Parallax bias.** Where New Relic is ahead,
> ahead is written. Where Parallax's edge is only *planned* or *unproven*, that
> is stated, not hidden.
>
> Research date: **2026-07-17**. Pricing, OTLP, and AI surfaces re-checked
> against live primary sources this pass. No legacy deep-research note exists
> for New Relic — this is the first canonical comparison.

## TL;DR verdict (scoped per axis)

- **Full-platform APM breadth/maturity, the entity model, OTLP-native ingest
  (GA since 2021), SaaS scale, compliance, and — critically — shipped AI
  coding-agent observability: New Relic wins, plainly** over pre-release
  Parallax.
- **On Parallax's agent wedge specifically, New Relic is a real, shipped
  threat:** its June-2026 **AI Coding Observability** supports **Claude Code,
  Cursor, GitHub Copilot, Windsurf, and Amazon Q** — multi-coding-agent spend +
  tracing, live. This overlaps Parallax's CLI/agent-tracing thesis directly, and
  New Relic is ahead today.
- **Parallax's honest edges are self-host/data-ownership (New Relic is SaaS-only),
  Apache-2.0 vs closed, and the *unproven* bounded-redacted-bundle + fix-outcome
  thesis (A1 gate).** None shipped at parity.

## New Relic — what it is (verified 2026-07-17)

Full-stack **SaaS observability platform**: APM (entity model, golden signals),
infrastructure, logs, metrics, distributed tracing, browser/mobile (RUM),
serverless, synthetics, dashboards, alerting, + AI. One of the category-defining
incumbents alongside Datadog and Dynatrace.

| | New Relic | Source |
|---|---|---|
| **Model** | **SaaS-only** (no self-host product) — multi-tenant cloud | newrelic.com |
| **OTLP ingest** | ✅ **native OTLP GA since 2021-09-23** (traces GA; metrics+logs GA'd after); now supports OTLP 0.18.0, exponential histograms, stable logs. Collection **reversible** (swap OTel exporter to leave). | [GA announcement](https://docs.newrelic.com/whats-new/2021/09/whats-new-09-23-2021-otel-native-ga/), [what's new](https://docs.newrelic.com/whats-new/) |
| **Entity model** | APM entities + golden signals (throughput/error/latency/saturation) + service maps; entity-centric navigation | docs |
| **AI — NRAI / New Relic AI** | AI assistant: instrument guidance, system-health reports, alert-coverage gaps; generative-AI features | [newrelic.com/platform/new-relic-ai](https://newrelic.com/platform/new-relic-ai) |
| **AI — Agent Platform** | **Feb 2026: AI Agent Platform + OTel tools** launch (create/manage AI agents, OTel stream integration) | [TechCrunch 2026-02-24](https://techcrunch.com/2026/02/24/new-relic-launches-new-ai-agent-platform-and-opentelemetry-tools/) |
| **AI — Coding Observability** | **June 2026: AI Coding Observability** — supports **Claude Code, Cursor, GitHub Copilot, Windsurf, Amazon Q**; track/forecast AI spend, kill black-box invoices | [press release 20260608](https://newrelic.com/press-release/20260608) |
| **AI — AIM** | **AI Monitoring (AIM)** — "APM for AI" for LLM/AI apps | [AIM video](https://newrelic.com/resources/video/introducing-new-relic-ai-monitoring-aim) |
| **MCP** | AI-native features incl. MCP + Canvas (per competitive landscape) | [OpenObserve alt](https://openobserve.ai/blog/top-10-new-relic-alternatives/) |
| **Ownership** | Taken private **2023** (Francisco Partners + TPG, ~$6.5B); no longer public | public record |
| **Compliance** | SOC 2, FedRAMP, HIPAA-eligible, GDPR — enterprise-grade | newrelic.com |

### Pricing (re-cited 2026-07-17; verify live page)

| Component | Price | Notes |
|---|---|---|
| **Free tier** | **$0** | **100 GB/mo** data ingest + **1 full-platform user**, perpetual, no credit card |
| **Original/Standard data** | **$0.40 / GB** | beyond free 100 GB *(one source cites $0.35 — legacy/promo; $0.40 is most consistent across 2026 sources)* |
| **Data Plus** | **$0.60 / GB** | higher-value/longer-retention tier |
| **Full-platform user** | ~**$49 / user / mo** | beyond the 1 free user (first user sometimes from ~$10) |

Formula: data cost = (GB − 100) × per-GB rate; + user cost. Sources:
[newrelic.com/pricing](https://newrelic.com/pricing), [Motadata](https://www.motadata.com/blog/new-relic-pricing),
[SigNoz guide](https://signoz.io/guides/new-relic-pricing/), [CubeAPM calc](https://cubeapm.com/pricing-calculator/new-relic/).

> Parallax pricing: **no public number** (pre-release). Direct comparison
> **benchmark-dependent, unmeasured.**

## Axis-by-axis comparison

### Signal coverage

| Signal | New Relic (shipped) | Parallax (planned) | Who |
|---|---|---|---|
| Traces / distributed tracing | ✅ mature (OTLP GA) | ✅ OTLP traces (🏗) | **New Relic** |
| Logs | ✅ (OTLP GA) | ✅ OTLP logs (🏗) | **New Relic** |
| Metrics | ✅ (OTLP GA, exp. histograms) | ✅ OTLP metrics (🏗) | **New Relic** |
| Errors / exceptions | ✅ (error-rate, golden signals) | ✅ derived `error_event` + fingerprint (🏗) | **New Relic** (maturity; Parallax's error-as-event model unproven) |
| Continuous profiling | 🟡 (limited/profile features) | ❌ | **New Relic** |
| RUM / browser / mobile | ✅ mature | ❌ | **New Relic** |
| Synthetics | ✅ | ❌ | **New Relic** |
| LLM / agent spans | ✅ AIM "APM for AI" | 🟡 planned | **New Relic** |
| **AI coding-agent obs (Claude Code/Cursor/Copilot/Windsurf/Q)** | ✅ **shipped June 2026** | ✅ CLI/agent tracing (🏗) | **New Relic** (shipped, multi-agent) |

**Verdict:** New Relic's coverage is comprehensive and all shipped. On the
**AI coding-agent axis specifically, New Relic is ahead of Parallax today** —
multi-coding-agent observability is live.

### Ingestion & transport

- **New Relic: OTLP-native, GA since 2021** — traces/metrics/logs via OTLP
  (0.18.0, exp. histograms, stable logs). **Collection is reversible** (swap the
  OTel exporter to leave — low ingest lock-in). Plus proprietary agents/SDKs.
- **Parallax: OTLP-native (all signals) + planned Sentry-envelope.**

> **Both OTLP-native; New Relic has been GA for ~5 years.** On OTLP maturity +
  ingest reversibility, New Relic wins. Parallax's only ingest edge is the
  planned Sentry-envelope path (not shipped).

### Storage architecture

New Relic: proprietary multi-tenant SaaS backend (**NerdGraph** API,
**NRQL** query language, Time-Stream/proprietary store). Closed. Parallax:
GreptimeDB native OTLP tables + Turso. New Relic's store is proven at scale but
opaque; Parallax's is open but **unproven/benchmark-dependent.**

### Query & correlation

New Relic: **NRQL** + entity-centric navigation + cross-signal correlation via
the entity model (service → golden signals → traces → logs). Mature, broad.
Parallax: evidence-graph + bounded bundle (**unproven**, A1).

### Dashboards & visualization

New Relic: mature dashboards + entity explorer + service maps + Canvas. Parallax:
minimal V1. **New Relic wins.**

### Alerting & on-call

New Relic: mature alerting + AI anomaly detection + NRAI alert-coverage gaps.
Parallax: minimal. **New Relic wins.**

### AI-native / agent-context story (Parallax's wedge — the crux, be most honest)

- **New Relic ships broad AI today:** NRAI assistant, **AI Agent Platform**
  (Feb 2026), **AI Coding Observability** (June 2026 — Claude Code/Cursor/
  Copilot/Windsurf/Amazon Q; spend + tracing), **AIM** (APM for AI), MCP, Canvas.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for coding
  agents (CLI/HTTP, MCP after gates) for *production incidents*.

> **Honest verdict:** on every *shipped* AI axis — including the AI
> coding-agent-observability axis that is closest to Parallax's wedge — **New
> Relic is ahead of pre-release Parallax**, with multi-coding-agent coverage
> already live. Parallax's only differentiated AI claim is the **bounded,
> redacted bundle + fix-outcome loop** — **unproven (A1 gate).** The burden is on
> Parallax to show that a bounded bundle beats New Relic's raw context + NRAI for
> agent fix quality, and that is unmet. Do not read "Parallax wins AI."

### Architecture & deployment

New Relic: **SaaS-only**, multi-tenant, multi-region. **No self-host product.**
Parallax: single-binary self-host, local-first, air-gap, Apache-2.0.

> **Self-host / data-ownership / air-gap is Parallax's one real, structural edge
> vs New Relic** — New Relic offers none of it (SaaS-only). This is the same edge
> Parallax holds over Datadog (both closed SaaS). But it is a *deployment-model*
> edge, not a capability edge — teams who can use SaaS get far more from New
> Relic today.

### Scalability & performance

New Relic: proven at hyperscale (incumbent, large enterprise customer base).
Specific numbers vendor; not independently measured. Parallax:
**benchmark-dependent, unproven.** On proven scale, **New Relic wins.**

### Security & compliance

New Relic: SSO/SAML, RBAC, audit, **SOC 2 / FedRAMP / HIPAA-eligible / GDPR** —
enterprise-grade. Parallax: SSO/RBAC/audit planned; redaction (A6) designed.
**New Relic wins decisively** on shipped security + compliance (esp. FedRAMP —
gov-relevant).

### Openness, licensing & lock-in

- **New Relic: closed SaaS**, proprietary backend (NRQL, NerdGraph). **High
  lock-in** on the backend (though OTLP ingest + reversible exporter lowers
  *ingest* lock-in — you can stop sending, but your stored data/query history
  is New Relic's).
- **Parallax: Apache-2.0**, fully open, OTLP-native, portable bundle.

> **Verdict:** on **openness/lock-in, Parallax wins** (Apache-2.0 self-host vs
> closed SaaS). New Relic's OTLP-native ingest softens ingest lock-in but not
> backend lock-in. Honest, scoped.

### Extensibility

New Relic: large integration ecosystem, NerdGraph API, partners, marketplace.
Mature. Parallax: OTel-native, CLI/HTTP/MCP, pipeline (planned). **New Relic
wins** on ecosystem breadth.

### Pricing & economics

New Relic: 100 GB free + $0.40–0.60/GB + ~$49/user/mo — usage + user based,
public, transparent. Parallax: **no public number**. Direct cost comparison
**benchmark-dependent, unmeasured.** New Relic's free tier (100 GB + 1 user) is
generous for small teams; at scale the user+data metering compounds.

## Where New Relic plainly wins (no bias)

1. **Full-platform APM breadth/maturity** + entity model + golden signals.
2. **OTLP-native ingest GA since 2021** (years ahead; reversible exporter).
3. **AI — broadest shipped AI of the incumbents** incl. **AI Coding Observability
   (Claude Code/Cursor/Copilot/Windsurf/Q)** — direct overlap with Parallax's
   agent wedge, and ahead today.
4. **AIM "APM for AI"** + MCP + Canvas + NRAI.
5. **SaaS scale + zero-ops + multi-region.**
6. **Compliance (SOC2/FedRAMP/HIPAA/GDPR)** + SSO/RBAC/audit.
7. **100 GB free tier** + transparent usage pricing.
8. **Proven-at-scale, ~15yr incumbent.**

## Where Parallax honestly edges New Relic

1. **Self-host / data-ownership / air-gap** — New Relic is SaaS-only; none exists.
   *(Real, structural deployment-model edge.)*
2. **Openness** — Apache-2.0 vs closed SaaS; portable bundle. *(Real.)*
3. **Production error events + fix-outcome loop** — New Relic has neither as a
   managed artifact. *(Thesis, **unproven** — A1 gate.)*
4. **Bounded, redacted, agent-safe evidence bundle** — New Relic's AI is
   dashboard + raw-context + chat, not a bounded safe projection. *(Thesis,
   **unproven** — A1 gate; and New Relic's shipped AI Coding Obs shrinks the
   gap.)*
5. **Single-binary local-first** — N/A for SaaS New Relic. *(Design edge.)*

## Watch triggers — re-evaluate New Relic if it:

- Ships a **self-host / on-prem** product → would erode Parallax's deployment edge. *(Checked 2026-07-17: still SaaS-only.)*
- Adds a **bounded, versioned, redacted evidence-bundle artifact** with outcome semantics → pressures A3.
- Adds a **fix-outcome loop** to AI Coding Observability → closes the core-thesis differentiator.
- **GA's a portable evidence/export schema** beyond NRQL/NerdGraph data export.

## Sources (checked 2026-07-17)

- [newrelic.com/pricing](https://newrelic.com/pricing); [how pricing works](https://docs.newrelic.com/docs/accounts/accounts-billing/new-relic-one-pricing-billing/new-relic-one-pricing-billing/).
- [OTLP native ingest GA (2021-09-23)](https://docs.newrelic.com/whats-new/2021/09/whats-new-09-23-2021-otel-native-ga/); [what's new](https://docs.newrelic.com/whats-new/).
- [New Relic AI (NRAI)](https://newrelic.com/platform/new-relic-ai); [AI Agent Platform + OTel tools (TechCrunch 2026-02-24)](https://techcrunch.com/2026/02/24/new-relic-launches-new-ai-agent-platform-and-opentelemetry-tools/); [AI Coding Observability press release 20260608](https://newrelic.com/press-release/20260608); [AIM](https://newrelic.com/resources/video/introducing-new-relic-ai-monitoring-aim).
- Pricing analyses: [Motadata](https://www.motadata.com/blog/new-relic-pricing), [SigNoz guide](https://signoz.io/guides/new-relic-pricing/), [CubeAPM calc](https://cubeapm.com/pricing-calculator/new-relic/), [Nurbak ($0.35 discrepancy)](https://nurbak.com/en/blog/new-relic-pricing/).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
