# Parallax vs LangSmith

> An unbiased, one-to-one comparison. Research date: **2026-07-17**.
> Sources: [LangChain pricing](https://www.langchain.com/pricing), [LangSmith docs](https://docs.smith.langchain.com/), and 2026 third-party pricing analyses. Completes the AI-observability trio alongside [Langfuse](parallax-vs-langfuse.md) and [Arize Phoenix](parallax-vs-arize-phoenix.md).
>
> **Bottom line up front:** LangSmith is LangChain's **closed, commercial LLM/agent
> observability + eval platform** — the native tool for the LangChain/LangGraph
> ecosystem (the most-deployed agent framework). On **agent tracing (LangGraph),
> evals, prompt hub, datasets/experiments, and ecosystem lock-in as a strength,
> LangSmith is far ahead of pre-release Parallax.** It is **closed and commercial** —
> SaaS by default; self-host/hybrid exist only on the Enterprise contract (a heavy
> K8s stack, not an OSS path), per-seat + usage-metered pricing. Parallax's honest
> edges are **open-source/self-host** (LangSmith is closed), **Apache-2.0 vs
> proprietary**, production-telemetry breadth, production-error + outcome loop, and
> the *unproven* bounded agent bundle (A1 gate).

## What each product is

- **LangSmith** (LangChain, Inc.) — the **commercial LLM/agent observability + evaluation platform** native to the **LangChain/LangGraph** ecosystem: tracing (LLM + agent/LangGraph spans), evaluation (automated + human), datasets, **prompt hub** (versioned prompts), experiments, playground, and tight LangGraph agent-tracing. **Closed-source SaaS** (proprietary); self-host is Enterprise-limited, not a real OSS path. Per-seat + per-trace usage pricing.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/agent traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves bounded/redacted evidence bundles to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

Both touch agent/LLM tracing, but LangSmith is a closed LLMOps platform tied to LangChain; Parallax is an open self-hosted production-incident evidence engine. Compare axis-by-axis.

## Signal coverage

| Signal | LangSmith (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| LLM / model spans | ✅ core | ✅ (🏗) |
| Agent spans (LangGraph nodes) | ✅ core (native LangGraph) | ✅ (🏗) |
| Non-LLM / tool / retrieval spans | ✅ | ✅ (🏗) |
| Production app traces (OTLP) | ✅ traces-only — dedicated OTLP endpoint (`/otel/v1/traces`, HTTP proto/JSON); **no OTLP metrics or logs** | ✅🧪 OTLP-native (shipped, pre-release) |
| Logs / Metrics | ❌ (not a log/metrics platform) | ✅🧪 OTLP logs/metrics (shipped, pre-release) |
| Errors / exceptions (production) | 🟡 (LLM-eval failures; not prod error events) | ✅🧪 derived `error_event` (shipped, pre-release) |
| Eval scores / annotations | ✅ core (automated + human) | ✅ planned (A1) |
| Prompt hub / datasets / experiments | ✅ core | ❌ (out of scope) |

**Verdict:** on **LLM/agent tracing + eval + prompt/dataset tooling, LangSmith wins decisively** (native to the dominant agent framework). On production telemetry breadth (logs/metrics/errors), Parallax's design is broader — LangSmith is not a general backend.

## Ingestion & transport

- **Tracing:** LangSmith captures via LangChain/LangGraph SDKs (native), **OpenTelemetry end-to-end** ([docs](https://docs.langchain.com/langsmith/trace-with-opentelemetry), [announcement](https://www.langchain.com/blog/end-to-end-opentelemetry-langsmith)) — a dedicated OTLP endpoint (`https://api.smith.langchain.com/otel/v1/traces`, HTTP proto/JSON, `x-api-key` auth; regional endpoints for EU/APAC/AWS-US), plus `LANGSMITH_OTEL_ENABLED=true` in its SDKs and an official [`langsmith-collector-proxy`](https://github.com/langchain-ai/langsmith-collector-proxy) for fan-out. **Traces only — no OTLP metrics or logs**; token/latency arrive as span attributes (OpenLLMetry conventions). Strongest where you're already on LangChain/LangGraph.
- **LangGraph agent tracing:** native — per-node execution tracing (third-party-cited at $0.001/node, unconfirmed on the live pricing page).
- **Parallax:** OTel SDKs + CLI/agent tracing, OTLP-native storage.

**Verdict:** on **LangChain/LangGraph-native tracing, LangSmith wins** (it's the native platform). On general OTLP-native telemetry storage, Parallax's design is broader. The **ecosystem lock-in cuts both ways**: LangSmith is strongest inside LangChain; teams not on LangChain get less native value.

## Storage architecture

- **LangSmith:** proprietary closed backend; internals not public. Retention tiers per live pricing page (2026-07-17): **14-day base, 180-day extended** (extra fee to upgrade a trace). Self-host/hybrid exist on Enterprise only — see *Architecture & deployment*.
- **Parallax:** GreptimeDB (native OTLP tables) + Turso, self-host single-binary.

**Verdict:** on **self-host + open storage, Parallax wins by design** (LangSmith is closed SaaS). On proven-at-scale, LangSmith (LangChain's commercial arm, large customer base) is mature; Parallax unproven.

## Query & correlation

- **LangSmith:** trace-centric (drill LLM/agent trace → nested spans → evals → prompt version → dataset/experiment). Strong within the LLM/agent domain. Not a cross-signal (metrics↔logs↔traces↔infra) engine.
- **Parallax:** evidence-graph correlation + bounded bundle (unproven, A1).

**Verdict:** on **LLM/agent-trace + eval linkage, LangSmith wins** (native, mature). On cross-signal production correlation, Parallax's design is broader but unproven.

## Evaluation & the LLMOps loop — LangSmith's moat

- **LangSmith:** the trace → eval → prompt → experiment loop, with the **prompt hub** (versioned, shared prompts) + datasets + experiments + playground + automated/human evals. The canonical LLMOps loop for the LangChain ecosystem.
- **Parallax:** A1 gate = does a bounded bundle beat raw context for agent *fix* outcomes (different eval target, unbuilt/unproven).

**Verdict:** on **LLM-app evaluation + the dev loop, LangSmith wins decisively.** Not Parallax's domain.

## AI-native / agent-context story

- **LangSmith's position:** an **LLMOps + eval platform for developers building LLM apps on LangChain/LangGraph** — trace, evaluate, manage prompts, experiment. A human dev loop + analytics; **not a bounded, read-only, redacted agent-context projection for production-incident resolution.** No production-error derivation, no fix-outcome loop.
- **Parallax's claim:** bounded, redacted, agent-safe evidence bundle for production incidents (planned, A1 gate).

**Honest verdict:** LangSmith is **far more mature** on capturing/structuring agent traces (esp. LangGraph) and evals. On shipped capability, **LangSmith leads.** Parallax's differentiation is entirely in cells LangSmith doesn't occupy: production-error derivation, fix-outcome loop, bounded/redacted agent-context artifact — all **unproven (A1 gate).** Fair read: a team on LangChain/LangGraph gets far more from LangSmith today than from pre-release Parallax.

## Architecture & deployment

- **LangSmith:** **closed, commercial, SaaS-default.** Self-host and hybrid **do exist** ([docs](https://docs.langchain.com/langsmith/architectural-overview), cloud guides for [AWS](https://docs.langchain.com/langsmith/aws-self-hosted)/[GCP](https://docs.langchain.com/langsmith/gcp-self-hosted)/[Azure](https://docs.langchain.com/langsmith/azure-self-hosted)) — but **only on the Enterprise contract** (custom pricing, annual invoice, license beacon to `beacon.langchain.com` unless air-gapped), and the production path is **Kubernetes + Helm** with **PostgreSQL 14+** (metadata), **Redis 5+/Valkey 8** (queues), **ClickHouse** (trace analytics; LangChain recommends externally-managed ClickHouse Cloud or a LangSmith-managed option), and **blob storage (S3/GCS/Azure Blob) required in production**, baseline **16 vCPU / 64 GB RAM**. Docker Compose is dev/test-only. This is a real but heavy, paid-gated self-host path — not an OSS self-host option like Parallax/Langfuse/OpenObserve.
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** on **self-host accessibility / data sovereignty, Parallax wins by design** (free Apache single-binary vs a paid-Enterprise-gated, multi-component K8s stack). Note honestly: "LangSmith has no self-host" is **false** — it has a real one, just commercial and operationally heavy. On managed SaaS scale/maturity, LangSmith wins.

## Operational footprint

- **LangSmith:** SaaS = zero backend ops. Cost is money (per-seat + per-trace).
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target.

**Verdict:** on **operator burden, LangSmith (SaaS) is lower.** On cash cost + vendor dependency, Parallax (self-host) is lower. Scoped.

## Scalability & performance

- **LangSmith:** proven at scale (LangChain's commercial platform, large LangGraph customer base). Specific numbers vendor; not independently measured.
- **Parallax:** unproven; benchmark-dependent.

**Verdict:** on **proven-at-scale + maturity, LangSmith wins conclusively.**

## Security

- **LangSmith:** SSO/SAML, RBAC, audit (Enterprise). Mature SaaS posture.
- **Parallax:** SSO/RBAC/audit planned; redaction (A6) designed.

**Verdict:** on **shipped security, LangSmith wins.**

## Privacy & compliance

- **LangSmith:** SOC 2 (Enterprise), data residency. SaaS.
- **Parallax:** none yet; data ownership via self-host.

**Verdict:** on **compliance, LangSmith wins.** On **data sovereignty, Parallax wins by design** (LangSmith is SaaS-only).

## Openness, licensing & vendor lock-in

- **LangSmith:** **closed-source proprietary SaaS.** High vendor lock-in — especially **ecosystem lock-in to LangChain/LangGraph** (the deeper you integrate, the harder to leave). Proprietary trace/prompt/dataset formats.
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness and lock-in cost, Parallax wins decisively** (Apache OSS + OTLP-native + self-host vs closed SaaS + ecosystem lock-in). This is a real, structural Parallax edge — the strongest non-thesis differentiator vs LangSmith.

## Pricing & economics — real numbers

LangSmith pricing is **public** ([langchain.com/pricing](https://www.langchain.com/pricing), re-fetched 2026-07-17, pass 14):

| Plan | Price | Traces | Notes |
| --- | --- | --- | --- |
| **Developer** | **Free** | 5K base traces/mo, then pay-as-you-go | max 1 seat, 14-day retention |
| **Plus** | **$39 / seat / month** | 10K base traces/mo included, then pay-as-you-go | unlimited seats |
| **Enterprise** | custom (annual invoice) | custom | self-hosted + hybrid options, custom SSO/RBAC, SLA |

**Metering units (live page, 2026-07-17):** usage beyond the included traces is metered in **LCU ($1.50 / LCU)** and **LSU ($1.00 / LSU)** — the page does **not** publish a per-1K-trace overage price. Retention: **base traces 14 days; extended traces 180 days** (upgrading a trace to extended costs an additional fee). ⚠️ **Correction vs pass 13:** the earlier figures ($0.50/1K base, $2.50/1K extended, 400-day extended retention) came from secondary 2026 analyses and do **not** match the live pricing page today — either LangSmith changed its metering (per-1K → LCU/LSU, 400d → 180d) or the secondary sources were stale/wrong. The live page is authoritative; per-1K figures are retained here only as historical context.

**LangGraph agent tracing:** third-party sources cite **$0.001 / node execution** (first 100K free) — **not confirmed on the live pricing page** (LCU/LSU abstractions may have replaced it); treat as unproven. Sources: [pecollective](https://pecollective.com/blog/langsmith-pricing/), [checkthat.ai](https://checkthat.ai/brands/langsmith/pricing), [laminar.sh](https://laminar.sh/blog/2026-01-29-laminar-vs-langfuse-vs-langsmith-llm-observability-compared). Per-seat + usage metering compounds at scale (a documented TCO concern).

**Parallax pricing:** none public yet (pre-release).

**Honest cost read:** LangSmith's per-seat + per-trace + per-node model compounds for high-volume agent workloads (third-party TCO guides flag this). Whether Parallax self-host is cheaper is benchmark-dependent/unmeasured. Langfuse (MIT, free self-host) is the cheaper OSS alternative in the same category.

## Where LangSmith plainly wins

- LangChain/LangGraph-native agent tracing (the dominant agent framework's native platform).
- Eval loop + prompt hub + datasets/experiments/playground (mature LLMOps).
- Proven-at-scale, SaaS maturity, SOC2/SSO/RBAC.
- Ecosystem integration depth (for LangChain users).

## Where Parallax honestly edges LangSmith

- **Openness / lock-in** — Apache-2.0 OTLP-native self-host vs closed SaaS + LangChain ecosystem lock-in. *(Real, decisive.)*
- **Self-host accessibility / data sovereignty** — Parallax: free Apache single-binary. LangSmith: self-host/hybrid exist but are Enterprise-contract-gated and operationally heavy (K8s+Helm, Postgres+Redis+ClickHouse+blob storage, 16 vCPU/64 GB baseline). *(Real edge on accessibility, not on existence — LangSmith self-host is real, just paid and heavy.)*
- **Production telemetry breadth** — OTLP-native logs/metrics/errors; LangSmith is LLM-only. *(Real design difference.)*
- **Production error events + fix-outcome loop** — LangSmith has neither. *(Thesis, unproven, A1.)*
- **Bounded, redacted, agent-safe evidence bundle** — LangSmith is an LLMOps eval tool, not an incident-context engine. *(Thesis, unproven, A1.)*

## Open questions / what measurement would settle

- **A1 gate vs LangSmith:** if a team is on LangChain/LangGraph + LangSmith, does a Parallax bounded bundle measurably improve coding-agent fix outcomes for *production incidents*? Unproven — and LangSmith's LangGraph tracing already covers much agent-context ground.
- ~~**LangSmith self-host reality (2026)**~~ — **answered (pass 14):** real but Enterprise-only; K8s+Helm production path with Postgres 14+/Redis/ClickHouse/blob storage, 16 vCPU/64 GB baseline, license beacon unless air-gapped ([architectural overview](https://docs.langchain.com/langsmith/architectural-overview)).
- **LangSmith metering drift** — live page (pass 14) shows LCU/LSU units + 180-day extended retention, contradicting the per-1K/400-day figures in secondary analyses. Open: what exactly an LCU/LSU maps to (trace? span? byte?) — pin from LangSmith docs next pass.

## Sources (accessed 2026-07-17)

- [LangChain pricing](https://www.langchain.com/pricing) (re-fetched 2026-07-17, pass 14 — LCU/LSU metering, 180-day extended retention); [LangSmith docs](https://docs.smith.langchain.com/).
- [Trace with OpenTelemetry — LangChain docs](https://docs.langchain.com/langsmith/trace-with-opentelemetry); [End-to-end OpenTelemetry in LangSmith — LangChain blog](https://www.langchain.com/blog/end-to-end-opentelemetry-langsmith); [langsmith-collector-proxy](https://github.com/langchain-ai/langsmith-collector-proxy).
- Self-host: [architectural overview](https://docs.langchain.com/langsmith/architectural-overview), [AWS](https://docs.langchain.com/langsmith/aws-self-hosted)/[GCP](https://docs.langchain.com/langsmith/gcp-self-hosted)/[Azure](https://docs.langchain.com/langsmith/azure-self-hosted) guides.
- 2026 pricing analyses: [pecollective](https://pecollective.com/blog/langsmith-pricing/), [checkthat.ai](https://checkthat.ai/brands/langsmith/pricing), [laminar.sh](https://laminar.sh/blog/2026-01-29-laminar-vs-langfuse-vs-langsmith-llm-observability-compared), [inference.net](https://inference.net/content/langsmith-pricing/).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [reference/agent-observability-review.md](../../reference/agent-observability-review.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Sibling deep-dives: [parallax-vs-langfuse.md](parallax-vs-langfuse.md), [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md).
