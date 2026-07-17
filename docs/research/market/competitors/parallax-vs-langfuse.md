# Parallax vs Langfuse

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 107**
> + **pass 145** pin recheck). Sources: [Langfuse docs](https://langfuse.com/docs),
> OTel integration, pricing/self-host, GitHub **v3.221.1** (2026-07-17) /
> **31,340★** (push 2026-07-17). **Still LLMOps/dev-loop product** (trace → eval
> → prompt → experiment), **not** production-incident OTLP full-signal + portable
> redacted evidence bundle + outcome ledger. Complementary on agent traces;
> A1 still unproven for Parallax vs raw/Langfuse context.
>
> **Bottom line up front:** Langfuse is the archetypal **open-source LLM/agent
> observability platform** and the most direct AI-wedge competitor to Parallax's
> agent-context thesis. On **LLM/agent tracing maturity, evaluations, prompt
> management, datasets/experiments, OSS community, and self-host-free economics,
> Langfuse is far ahead of pre-release Parallax.** The honest nuance: the two
> serve *different loops* — Langfuse is an LLMOps dev loop (improve your LLM app:
> trace → eval → prompt → experiment), Parallax is a production-incident evidence
> engine for coding agents. On the narrow overlap (agent execution traces + safe
> context for agents), Langfuse is far more mature today; Parallax's only
> differentiated claims (derived production errors + fix-outcome loop + bounded
> redacted agent bundle) are **unproven (A1 gate).**

## What each product is

- **Langfuse** — open-source (**MIT core**; `ee/` folders excepted per LICENSE)
  LLM engineering platform: tracing (LLM + non-LLM spans, hierarchical, multi-turn),
  evaluation scores (human + automated/model-based), prompt management (versioned,
  linked to traces), datasets, experiments, analytics (latency / cost / token usage),
  and a prompt playground. Open-core: MIT self-host (free core) + Langfuse Cloud +
  self-host Enterprise (RBAC/SCIM). **Latest: v3.221.1 (2026-07-17); 31,340★;
  extremely fast cadence (pass 107).** v3.x self-hostable; **2025-06-04** open-sourced
  remaining product features under MIT ([changelog](https://langfuse.com/changelog/2025-06-04-open-sourcing-langfuse)).
  **Pass 107 air-gap nuance:** README states self-hosted instances **default to
  reporting basic usage stats to a centralized PostHog** (not raw traces/prompts);
  opt-out via [telemetry docs](https://langfuse.com/self-hosting/security/telemetry).
  Do **not** claim Langfuse CE is phone-home-free without that opt-out.
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**: OTLP-native ingest of traces/logs/metrics + CLI/coding-agent execution traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, serves **bounded, redacted, schema-valid evidence bundles** to humans and coding agents. GreptimeDB + Turso. **Pre-release.**

These overlap on **agent/LLM tracing and "context for agents,"** but were built for different primary jobs. Compare axis-by-axis.

## Signal coverage — Langfuse is the LLM-tracing specialist

| Signal | Langfuse (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| LLM / model spans (prompt, completion, tokens, cost) | ✅ core, first-class | ✅ (🏗) |
| Agent / tool / retrieval spans | ✅ hierarchical, nested | ✅ (🏗) |
| Non-LLM spans (API calls, embeddings, retrieval) | ✅ in same trace | ✅ (🏗) |
| Production app traces (OTLP) | 🟡 receives OTLP traces (not a general telemetry backend) | ✅🧪 OTLP-native (shipped, pre-release) |
| Logs | 🟡 (trace-scoped, not a log platform) | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ❌ (not a metrics platform) | ✅🧪 OTLP metrics (shipped, pre-release) |
| Errors / exceptions (production) | 🟡 (LLM-eval failures, not prod error events) | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Eval scores / annotations | ✅ core (human + automated) | ✅ planned (A1 eval design) |
| Prompt versions / datasets / experiments | ✅ core | ❌ (out of scope) |

**Verdict:** on **LLM/agent-tracing depth + eval/prompt/dataset tooling, Langfuse wins decisively** (it is purpose-built for that). On **production telemetry breadth (full OTLP logs/metrics/errors), Parallax's design is broader** — Langfuse is not a general observability backend.

## Ingestion & transport

- **OTLP:** Langfuse **operates as an OpenTelemetry backend** — it receives traces on the `/api/public/otel` OTLP endpoint ([docs](https://langfuse.com/integrations/native/opentelemetry)). Plus its own SDKs (Python, JS/TS, etc.) and native framework integrations (LangChain, OpenAI, etc.). So Langfuse is **OTLP-receivable for traces** — but it is **not a general OTLP telemetry store** (no OTLP metrics, no OTLP logs-as-a-log-platform); it consumes OTLP traces into its LLM-trace model. Parallax is OTLP-native across traces/logs/metrics into GreptimeDB native tables.
- **SDKs / integrations:** Langfuse ships many LLM-framework integrations (LangChain, OpenAI/Anthropic/Bedrock, LiteLLM, etc.) + OTel + HTTP API. Notably the **OTEL-native Langfuse SDK v4** is a thin layer over the official OpenTelemetry client and emits traces via **OTLP natively** ([docs](https://langfuse.com/integrations/native/opentelemetry)), and **MCP tracing** links MCP client + server traces end-to-end via W3C trace-context propagation ([docs](https://langfuse.com/docs/observability/features/mcp-tracing)). This is a direct, shipped overlap with Parallax's OTLP-native + agent-trace wedge. Parallax relies on OTel SDKs + CLI/agent tracing.

**Verdict:** on **LLM-framework integration breadth, Langfuse wins.** On **general OTLP-native telemetry storage, Parallax's design is broader.** Scoped, not head-to-head.

## Storage architecture

- **Langfuse:** self-hosted via Docker (MIT); **v3 backing store pinned: PostgreSQL (state) + ClickHouse (traces/observations/scores) + Redis + S3**, plus an async worker container ([infra-evolution blog](https://langfuse.com/blog/2024-12-langfuse-v3-infrastructure-evolution), v3 stable 2024-12-09). Cloud = managed.
- **Parallax:** GreptimeDB (telemetry native OTLP tables) + Turso (metadata), single-binary self-host target.

**Verdict:** both self-hostable; Langfuse's is more mature/shipped. Parallax's GreptimeDB-native design is benchmark-dependent and **unproven** vs Langfuse's shipped stack.

## Query & correlation

- **Langfuse:** trace-centric exploration — drill an LLM trace to its nested tool/retrieval/model spans, attach scores, link the prompt version, jump to the dataset/experiment. Strong within the LLM-app domain. **Not** a cross-signal (metrics↔logs↔traces↔infra) correlation engine.
- **Parallax:** evidence-graph correlation across production signals + run_id/invocation stitching + the bounded evidence bundle (unproven, A1).

**Verdict:** on **LLM-trace drill-down + eval linkage, Langfuse wins** (purpose-built). On **cross-signal production correlation, Parallax's design is broader** (but unproven). Different axes.

## Evaluation & the LLMOps loop — Langfuse's moat

- **Langfuse:** the **trace → eval → prompt → experiment** loop is the product: human annotations, automated/model-based evaluators, scores on traces/observations, versioned prompt management linked to traces for per-version metrics, datasets built from production traces, experiments comparing prompt/model variants. This is the canonical LLMOps loop, shipped and mature.
- **Parallax:** the A1 validation gate asks whether a bounded evidence bundle beats raw context for agent fix quality — Parallax's eval design is **about agent outcomes, not LLM-app quality**, and is **unbuilt/unproven.**

**Verdict:** on **LLM-app evaluation and the dev loop, Langfuse wins decisively.** This is not Parallax's domain.

## Dashboards & visualization

- **Langfuse:** analytics dashboards (latency, cost, token usage, score distributions), trace explorer, prompt manager UI, sessions/conversations. Mature for the LLM domain.
- **Parallax:** V1 UI = Sentry-grade issues + dashboards (TanStack/shadcn). Narrower, different focus.

**Verdict:** **Langfuse wins** within the LLM-analytics domain; different purpose.

## AI-native / agent-context story — the crux

This is the axis that matters most for Parallax's thesis, so be most honest.

- **Langfuse's position:** it is an **LLMOps platform for developers improving their LLM applications** — trace, evaluate, iterate prompts, run experiments. It is a **human dev loop + analytics**, not a *context engine that serves bounded, redacted evidence to autonomous coding agents for production incident resolution*. Langfuse does not derive production error events, does not run a fix-outcome loop, and does not serve a read-only bounded agent-context projection.
- **Parallax's claim:** a bounded, redacted, agent-safe evidence bundle served to coding agents (CLI/HTTP first, local-stdio MCP graduated (plan 112 DONE; remote deferred)) for *production incidents* — a context engine, not an LLMOps dashboard.

**Honest verdict:** Langfuse is **far more mature** on the thing both touch — capturing and structuring agent/LLM execution traces. On shipped capability, **Langfuse leads.** Parallax's differentiation is entirely in the cells Langfuse does not occupy: production-error derivation, fix-outcome loop, and a bounded/redacted agent-context artifact — all **unproven (A1 gate).** A fair read: today, a team wanting "agent traces + evals" gets far more from Langfuse than from pre-release Parallax. Parallax's bet is that *production-incident evidence for coding agents* is a different, valuable job Langfuse doesn't do — and that bet is unvalidated.

## Architecture & deployment model

- **Langfuse:** self-host (Docker/K8s, MIT, free, unlimited) **or** Langfuse Cloud (managed, multi-region). Open-core; self-host Enterprise is **custom-priced** (project RBAC/SCIM/audit/retention + ClickHouse commercial bundle) — **no public $** (pass 61).
- **Parallax:** single-binary self-host target, local-first, air-gap-capable, Apache-2.0.

**Verdict:** both are open + self-hostable. **Langfuse is shipped and mature today; Parallax is pre-release.** Parallax's single-binary local-first target is a simplicity edge (by design), unproven in production.

## Operational footprint

- **Langfuse self-host:** Docker stack (Langfuse web + worker + Postgres + ClickHouse + Redis + S3); moderate ops. Cloud = zero backend ops.
- **Parallax:** self-hosted GreptimeDB + Turso + engine; single-binary target lowers burden but production operation is real work.

**Verdict:** **Langfuse wins on operational maturity** (shipped + Cloud zero-ops option). Scoped.

## Scalability & performance

- **Langfuse:** proven at scale (large OSS community, Cloud customers). Specific numbers vendor/marketing; not independently measured here.
- **Parallax:** unproven at production scale; **benchmark-dependent.**

**Verdict:** on **proven-at-scale + maturity, Langfuse wins conclusively.** Parallax cannot yet make a measured scale claim.

## Security

- **Langfuse Cloud:** ISO 27001, SOC 2, GDPR; Pro+ has SOC2/ISO reports + HIPAA path; Enterprise adds SCIM/audit/SLA. **Self-host OSS includes org RBAC + Enterprise SSO** ([pricing-self-host](https://langfuse.com/pricing-self-host)); paid self-host EE gates **project-level RBAC, data retention mgmt, audit logs, SCIM, server-side masking** (custom $).
- **Parallax:** SSO/RBAC/audit planned, not shipped; redaction (A6) designed as first-class.

**Verdict:** on **shipped security/compliance posture, Langfuse Cloud wins.** Parallax's redaction-before-agent-access is a narrower, unproven edge.

## Openness, licensing & vendor lock-in

- **Langfuse:** **MIT core** (self-host, free, all core features, unlimited) — genuinely open, no feature gating on core. Enterprise features under a commercial license. Low lock-in (OTLP-receivable, standard formats, export). **More open than Sentry's FSL or Datadog's closed model; comparable to Parallax's Apache-2.0** (both permissive; MIT vs Apache-2.0 is a minor difference — Apache-2.0 adds patent grant).
- **Parallax:** Apache-2.0, fully open, OTLP-native, portable bundle.

**Verdict:** on **openness, roughly tied** — both permissive OSS, self-hostable, OTLP-receivable/native. Neither has a lock-in advantage over the other. (An honest draw, not a Parallax win.)

## Extensibility

- **Langfuse:** many LLM-framework integrations, SDKs, OTel, public API, webhooks, prompt-API. Mature for the LLM ecosystem.
- **Parallax:** OTel-native, CLI/HTTP/MCP surfaces, pipeline/processor, webhooks (planned).

**Verdict:** on **ecosystem breadth, Langfuse wins** (mature LLM integrations).

## Pricing & economics — real numbers

Langfuse pricing is **public** ([langfuse.com/pricing](https://langfuse.com/pricing) + [pricing-self-host](https://langfuse.com/pricing-self-host), **pass 61** 2026-07-17). **Pass-4 table was stale.**

| Plan | Price | Notes |
| --- | --- | --- |
| **Self-hosted OSS** | **$0 / MIT** | all core features, unlimited usage; org RBAC + Enterprise SSO free in OSS |
| **Cloud Hobby** | **Free** | **50k units**/mo, **30-day** access, **2 users**, no credit card |
| **Cloud Core** | **$29/mo** | **NEW vs pass 4** — 100k units included; +usage; **90-day** access; unlimited users |
| **Cloud Pro** | **$199/mo** | 100k units included; **3-year** data access (not 90-day); SOC2/ISO/HIPAA path |
| **Teams add-on** | **+$300/mo** on Pro | Enterprise SSO enforcement, fine-grained RBAC, Slack support |
| **Cloud Enterprise** | **$2,499/mo** | Pro+Teams + audit logs, SCIM, SLA, dedicated engineer |
| **Usage overage (paid Cloud)** | graduated | **$8 / $7 / $6.50 / $6** per 100k units by volume tier |
| **Self-host Enterprise** | **custom (no public $)** | project RBAC, retention policies, audit logs, SCIM, server-side masking; bundled ClickHouse Cloud/BYOC/Private — **~$500/mo pass-4 figure retired** |

A billable unit = trace / observation / score. **Langfuse Assistant (in-app agent)** ships on **Cloud** plans (Hobby+); self-host matrix marks Assistant **unavailable** on OSS and EE (**Cloud-only AI teammate** — same class of gate as SigNoz Noz). **Self-host OSS is free with no unit limits** — very strong economics.

**Parallax pricing:** none public yet (pre-release). Stated shape: Apache-2.0 open core + gated enterprise-ops + managed cloud + outcome-priced fixer.

**Honest cost read:** Langfuse self-host is **free, unlimited, MIT** — hard to undercut on price for the LLM-tracing job. Parallax's cost edge only applies to the *different* job (production telemetry evidence); on Langfuse's home turf, Langfuse's free self-host cannot be beaten on price.

## Where Langfuse plainly wins

- LLM/agent tracing depth + hierarchical traces (purpose-built).
- Evaluation loop: human + automated scores, datasets, experiments.
- Prompt management (versioned, trace-linked, per-version metrics).
- OSS maturity + community + MIT self-host-free economics.
- Cloud scale + ISO27001/SOC2/GDPR compliance.
- LLM-framework integration breadth.
- Proven-at-scale, shipped today.

## Where Parallax honestly edges Langfuse

- **Production telemetry breadth** — OTLP-native logs/metrics/errors; Langfuse is not a general telemetry backend. *(Real design difference.)*
- **Production error events + fix-outcome loop** — Langfuse has neither; unoccupied cells. *(Thesis, **unproven** — A1 gate.)*
- **Bounded, redacted, agent-safe evidence bundle for production incidents** — Langfuse is an LLMOps dev loop, not an incident-context engine. *(Thesis, **unproven** — A1 gate; this is the crux of whether Parallax is a real product vs a feature Langfuse could add.)*
- **Single-binary local-first** — Langfuse self-host is a Docker stack. *(Minor design edge.)*

## Open questions / what measurement would settle

- **A1 gate vs Langfuse:** if a team already has Langfuse for agent traces + evals, does adding a Parallax bounded bundle measurably improve coding-agent fix outcomes for *production incidents*? Unproven — and this is the existential question for Parallax's wedge against the AI-observability category Langfuse leads.
- **Langfuse extension risk:** Langfuse could add production-error derivation / a bounded export. If it does, Parallax's AI-wedge differentiation collapses. Track Langfuse changelog.
- ~~Langfuse exact version + backing store~~ → **pinned v3.221.1 (pass 58)** + v3 stack (Postgres + ClickHouse + Redis + S3 + async worker). Prod-error-export watch still open.

## Sources (accessed 2026-07-17)

- [Langfuse docs home](https://langfuse.com/docs); [observability overview](https://langfuse.com/docs/observability/overview).
- [Langfuse OTLP/OTel integration](https://langfuse.com/integrations/native/opentelemetry) (OTLP backend at `/api/public/otel`; OTEL-native **SDK v4**); [MCP tracing docs](https://langfuse.com/docs/observability/features/mcp-tracing).
- [Langfuse changelog: open-sourced all remaining product features under MIT (2025-06-04)](https://langfuse.com/changelog/2025-06-04-open-sourcing-langfuse); [self-host SSO](https://langfuse.com/self-hosting/security/authentication-and-sso).
- [GitHub releases — latest v3.221.1 (2026-07-17)](https://github.com/langfuse/langfuse/releases).
- [Langfuse pricing](https://langfuse.com/pricing) (**pass 61:** Hobby/Core $29/Pro $199/Enterprise $2499 + Teams $300; Assistant Cloud); [self-host pricing](https://langfuse.com/pricing-self-host) (EE **custom**, OSS free + org RBAC/SSO).
- 2026 comparisons: [OpenObserve LLM obs tools](https://openobserve.ai/blog/llm-observability-tools/), [Firecrawl](https://www.firecrawl.dev/blog/best-llm-observability-tools), [MLflow top-5](https://mlflow.org/top-5-agent-observability-tools/).
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [reference/agent-observability-review.md](../../reference/agent-observability-review.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
