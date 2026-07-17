# Parallax vs Mezmo

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 65** —
> live [pricing](https://www.mezmo.com/pricing) re-verify: **no public $/GB unit rates**
> on page; AI SRE + MCP marketing). Sources: live [mezmo.com/pricing](https://www.mezmo.com/pricing),
> [mezmo.com](https://www.mezmo.com/), [Mezmo Flow](https://www.mezmo.com/newsroom/mezmo-flow-released),
> historical **[2025-05-14 newsroom $0.20 rates](https://www.mezmo.com/newsroom/mezmo-disrupts-market-by-reducing-observability-cost-structure-by-90)**,
> [AURA OSS](https://github.com/mezmo/aura).
>
> **Bottom line up front:** Mezmo (formerly **LogDNA**) is a **telemetry data pipeline
> + AI-ready context layer** — profiles/transforms/routes telemetry in flight, plus
> **agentic AI SRE** and **MCP with RBAC**. **Different stack layer from Parallax**
> (pipeline/governance + AI RCA over prepared telemetry, **not** a portable redacted
> evidence bundle backend). **Complementary, not head-to-head.** On pipeline + shipped
> AI RCA + MCP, **Mezmo is ahead of pre-release Parallax**. Parallax edges remain layers
> Mezmo does not own (OTLP store, Sentry envelope, versioned redacted bundle, outcome).

## What each product is

- **Mezmo** (formerly **LogDNA**) — a **telemetry data pipeline + AI SRE platform**: **Active Telemetry Pipelines** (profile/transform/route logs/metrics/traces; OTel-aligned); **Agentic root cause analysis** (AI SRE investigates incidents; **AI RCA included in platform license — no metered AI queries** per live pricing copy); **MCP / agent access with built-in RBAC** (users/agents only inspect authorized logs); **context engineering** (dedupe/cluster/enrich before LLM). **AURA** open-source agentic harness ([mezmo/aura](https://github.com/mezmo/aura), **Apache-2.0**, **~223★** pass 65) for production AI SRE agents. Retains LogDNA-era log management. **Closed SaaS platform** (pipeline/core); AURA is the OSS harness slice. **Pricing (pass 65):** live page markets **platform / “transparent platform pricing”** — **no public $/GB unit rates** on the page; historical 2025 newsroom **$0.20/GB ingest + $0.20/GB retain/mo** is **not shown on live pricing** (treat as historical proxy only).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

**Crucial framing:** Mezmo is a **pipeline/control/governance layer** (route/optimize/govern telemetry in flight); Parallax is a **backend + context engine** (ingest, store, derive, serve). They sit at **different stack layers** and are **complementary** — Mezmo can feed Parallax. Same layer-distinction logic as [Odigos](parallax-vs-odigos.md) (instrumentation) — Mezmo is the routing/cost-governance peer (akin to Chronosphere's Telemetry Pipeline / Cribl).

## Signal coverage — Mezmo routes; Parallax consumes

| Signal | Mezmo (shipped) | Parallax (planned/shipped) |
| --- | --- | --- |
| Telemetry pipeline (profile/transform/route in flight) | ✅ **(the core)** | ❌ (no pipeline layer) |
| Log management (LogDNA) | ✅ (K8s-native, streaming) | ✅🧪 OTLP logs (shipped, pre-release) |
| Cost/volume governance (drop/sample/optimize) | ✅ (vendor savings claims) | ❌ |
| Mezmo Flow / auto-optimize | ✅ | ❌ |
| **AI SRE / agentic RCA** | ✅ (included in platform license per live pricing) | 🏗 planned (AI RCA) |
| **MCP + agent RBAC** | ✅ (MCP/Agent with access controls) | ✅🧪 local-stdio MCP (read-only); remote MCP planned |
| **AURA OSS agent harness** | ✅ Apache-2.0 (~223★) | ❌ (no AURA-class harness) |
| **Telemetry storage / backend** | 🟡 (LogDNA store; pipeline forwards) | ✅🧪 GreptimeDB (shipped, pre-release) |
| Error derivation / fingerprinting | ❌ | ✅ derived `error_event` (🧪 shipped) |
| Portable redacted versioned evidence bundle | ❌ | 🟡🧪 code (A1 unproven) |
| Sentry envelope / DSN | ❌ | ✅ shipped |

**Verdict (pass 65):** **still different layers**, but Mezmo now **ships AI SRE + MCP** — closer to “agent context over telemetry” than pass-28 pipeline-only framing. **No-bias:** Mezmo’s agent surface is **SaaS-grounded AI RCA over prepared telemetry**, not Parallax’s portable redacted coding-agent bundle. Mezmo still does **not** own OTLP evidence store + Sentry + outcome loop.

## Ingestion & transport — the layer relationship

- **Mezmo:** OTel-aligned pipeline — collect/transform/route telemetry to destinations (SIEM/backends/storage). It is a **telemetry processor/router.**
- **Parallax:** OTLP ingest gateway (consumer/destination) + shipped Sentry-envelope adapter.

**Verdict:** Mezmo is a **pre-processor/router Parallax can sit behind.** They are **pipeline-adjacent, not competitive.** On the pipeline/cost-governance axis, **Mezmo is ahead of Parallax** (Parallax has no pipeline layer — it depends on SDKs/collectors/tools like Mezmo to shape input).

## Storage / Query / Error / AI / Deployment — pass 65 AI surface

Mezmo retains **LogDNA's log-management store** (so it *is* a log backend too) while the strategic pitch is **pipeline + AI-ready context + agentic SRE**. Live pricing copy: AI RCA **included** (no pay-per-query AI surcharges); model-agnostic (native agents **or** bring-your-own via **MCP**); MCP has **RBAC**.

**Verdict:** on **pipeline + cost-governance + shipped AI SRE/MCP, Mezmo is ahead of pre-release Parallax.** On **portable redacted versioned evidence bundle + Sentry + OTLP-native error derivation**, Parallax targets layers Mezmo does not ship. **Do not claim “agent MCP” as Parallax-unique vs Mezmo.**

## Openness, licensing & vendor lock-in

- **Mezmo platform:** **closed SaaS** (proprietary). Moderate lock-in (pipeline config, LogDNA formats). No full-platform OSS self-host.
- **AURA:** **Apache-2.0** OSS harness (~223★) — open slice for agent runtime; **not** the full Mezmo pipeline SaaS.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **full-platform openness, Parallax edges** (uniform Apache self-host vs closed SaaS). AURA narrows the “Mezmo has zero OSS” claim — agent harness is open; pipeline/control plane is not.

## Pricing & economics — pass 65: **no public unit rate on live page**

| Source | What it says |
| --- | --- |
| **Live [mezmo.com/pricing](https://www.mezmo.com/pricing)** (pass 65) | Markets **AI-driven SRE** platform pricing: AI RCA included, no AI surcharges, MCP at no extra cost claim, “transparent platform pricing.” **FAQ exists** (“How is pricing calculated?”) but **no static $/GB or $/host table on the page**. **No public number** for current unit rates → sales/trial. |
| **Historical** ([newsroom 2025-05-14](https://www.mezmo.com/newsroom/mezmo-disrupts-market-by-reducing-observability-cost-structure-by-90)) | **$0.20 / GB ingested** + **$0.20 / GB retained / mo** — was pass-42 primary unit cite. **Not reproduced on live pricing page (pass 65)** → keep only as **dated historical proxy**, not current list. |

**Parallax pricing:** none public yet (pre-release); self-host = no per-event tax by design.

**Honest cost read:** Mezmo’s **pipeline-reduces-downstream-cost** pitch remains the economic story (like Cribl / Chronosphere pipeline). **Cannot claim live $0.20/GB as verified 2026-07 list** — page moved to platform/AI packaging without public unit card. Free trial available. Not a like-for-like cost contest with Parallax (different layer).

## Where Mezmo plainly wins

- **Telemetry pipeline** (profile/transform/route; cost/volume governance).
- **Agentic AI SRE + included AI RCA** (live pricing; no metered AI-query story).
- **MCP with RBAC** for agents/users.
- **AURA** Apache-2.0 production agent harness.
- LogDNA log management + OTel-aligned ingest/routing.

## Where Parallax and Mezmo differ (not "Parallax wins")

- **Different stack layer** — Mezmo routes/optimizes/governs + AI-RCA over prepared telemetry; Parallax stores/derives/serves portable evidence. **Mostly complementary.**
- **Parallax layers Mezmo doesn’t ship:** Sentry-envelope, OTLP-native evidence graph, portable redacted versioned bundle, fix-outcome loop (A1 unproven).
- **Mezmo layers Parallax doesn’t ship:** in-flight pipeline, AI SRE agent product, AURA harness.

> **Honest summary (pass 65):** Mezmo is **still not a head-to-head production-evidence backend**, but it is **no longer “pipeline-only.”** Live product markets **AI SRE + MCP + context engineering** on top of Active Telemetry Pipelines — another field-convergence signal against claiming “agent context” as unique. **$0.20/GB is historical, not live-page-verified.** Realistic stack remains **collectors → Mezmo (route/optimize) → Parallax (ingest/derive/bundle)** if A1 holds. Track Mezmo as complementary **pipeline + AI-SRE layer** and a **public-unit-rate gap** (sales-only).

## Open questions / what would matter

- **Mezmo → Parallax integration** — OTLP out into Parallax? (Likely; PoC open.)
- **Public unit rates** — if Mezmo republishes $/GB, re-pin; until then **no public number**.
- **AURA vs HolmesGPT vs Parallax** — AURA is another OSS “fixer harness”; A1 must beat/complement raw-agent-over-telemetry.
- **Parallax pipeline gap** — integrate (Mezmo/Vector/FluentBit) or build.

## Sources (accessed 2026-07-17; pass 65)

- Live **[mezmo.com/pricing](https://www.mezmo.com/pricing)** (no public $/GB); [mezmo.com](https://www.mezmo.com/); [Agentic SRE](https://www.mezmo.com/platform/agentic-sre).
- Historical unit rates: [newsroom 2025-05-14](https://www.mezmo.com/newsroom/mezmo-disrupts-market-by-reducing-observability-cost-structure-by-90).
- [AURA](https://github.com/mezmo/aura) (~223★ Apache-2.0); [Mezmo Flow](https://www.mezmo.com/newsroom/mezmo-flow-released).
- Parallax side: [capture/otlp.md](../../capture/otlp.md), [architecture/integration-contract.md](../../architecture/integration-contract.md).
- Sibling layers: [parallax-vs-odigos.md](parallax-vs-odigos.md), [parallax-vs-chronosphere.md](parallax-vs-chronosphere.md), [parallax-vs-holmesgpt.md](parallax-vs-holmesgpt.md).
