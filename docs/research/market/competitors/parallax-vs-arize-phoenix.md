# Parallax vs Arize Phoenix

> One-to-one comparison. **No pro-Parallax bias.** Where Phoenix is ahead, ahead
> is written. Where Parallax's edge is only *planned* or *unproven*, that is
> stated, not hidden.
>
> Research date: **2026-07-17**. **Pass 108 + pass 146 + pass 168 + pass 198
> pin:** GitHub **arize-phoenix-v18.1.0** (2026-07-17), **10,600★** (pass
> **198**); LICENSE still **ELv2**. **`@arizeai/phoenix-mcp@4.2.0`** still
> shipping; MCP includes **mutating** prompt tools — agent surface over Phoenix
> traces/evals, **not** RO portable production-incident evidence bundle. Same
> job as Langfuse (LLMOps/dev loop). A1 unproven.

## TL;DR verdict (scoped per axis)

- **LLM/agent tracing depth, evaluation tooling (LLM-as-judge heritage),
  OpenInference (the OTel-for-LLM semantic standard Arize drives), datasets/
  experiments, OSS self-host-free economics, and Python-native ergonomics:
  Phoenix wins, plainly** over pre-release Parallax — same as Langfuse, its
  direct sibling in this category.
- **Like Langfuse, Phoenix serves a different primary job** (LLM/agent-app
  experimentation → eval → improve) than Parallax (production-incident evidence
  for coding agents). On the narrow overlap (agent execution traces + OTLP),
  Phoenix is far more mature today.
- **Parallax's differentiated claims are all unproven (A1 gate):** production
  telemetry breadth (Phoenix is not a logs/metrics/errors backend), production
  error derivation + fix-outcome loop, and the bounded redacted agent-context
  bundle.
- **One real license difference:** Phoenix is **ELv2** (self-host free, but
  restricts resale-as-managed-service; not OSI-open) — *less* permissive than
  Langfuse's MIT and Parallax's Apache-2.0. (comparison-set previously mislabeled
  Phoenix "Apache-2.0"; corrected here.)

## Phoenix — what it is (verified 2026-07-17)

Open-source **AI observability + evaluation** platform (Arize AI): tracing for
LLMs and agents, evaluations (incl. LLM-as-judge), datasets, experiments,
prompt playground/management, and analytics (latency / cost / token usage).
Originated in notebook-based prompt experimentation + LLM-as-judge evals.

| | Phoenix | Source |
|---|---|---|
| **Latest release** | **arize-phoenix-v18.1.0** (2026-07-17; still latest pass **168**) | [github.com/Arize-ai/phoenix/releases](https://github.com/Arize-ai/phoenix/releases) |
| **Stars** | **10,600** (pass **168**) | GitHub API |
| **MCP package** | **@arizeai/phoenix-mcp@4.2.0** — prompts include **write** tools (`upsert-prompt`, tags); projects list/get | npm registry + package README |
| **Cadence** | Very high version number, fast-moving (v18 line) | releases |
| **Language** | **Python** (core); TypeScript UI | GitHub |
| **License** | **Elastic License 2.0 (ELv2)** — *not* Apache/OSI-open. Free to use + self-host; restricts offering it as a **managed service**. Self-host is 100% free, **no feature gates, no usage limits**. | [arize.com/docs/phoenix/self-hosting/license](https://arize.com/docs/phoenix/self-hosting/license), [GitHub](https://github.com/arize-ai/phoenix) |
| **Tracing standard** | **OpenTelemetry (OTLP) + OpenInference** semantic conventions for AI/LLM spans — Arize drives OpenInference | [arize.com/docs/ax/concepts/otel-openinference/overview](https://arize.com/docs/ax/concepts/otel-openinference/overview) |
| **OTLP ingest** | ✅ **accepts traces over OTLP natively** (Phoenix-aware OTel defaults) | [arize.com/docs/phoenix](https://arize.com/docs/phoenix) |
| **Self-host** | ✅ Docker / containers / Railway; free, unlimited | [self-host docs](https://arize.com/docs/phoenix/self-hosting) |
| **Backing store** | **file-based SQLite (default)**, configurable via DB URL ([self-hosting/configuration](https://arize.com/docs/phoenix/self-hosting/configuration), 2026-07-17); **no native ClickHouse** (only in third-party Docker Compose guides). | pinned |
| **Company** | Arize AI (commercial); Phoenix OSS + Phoenix Cloud + Arize AX (enterprise platform) | [arize.com/phoenix](https://arize.com/phoenix/) |

### Pricing (**pass 61** live [arize.com/pricing](https://arize.com/pricing) — prior Core $29/Pro $199 **stale/wrong**)

| Tier | Price | Notes |
|---|---|---|
| **Self-host Phoenix OSS (ELv2)** | **$0** | free, unlimited, local-first; move to AX when needed |
| **Arize AX Free** | **$0** | 25k spans/mo, 1 GB ingest, 15-day retention, SaaS |
| **Arize AX Pro** | **$50/mo** | 50k spans/mo, 10 GB, 30-day retention, SaaS |
| **Arize AX Enterprise** | **custom** | custom volume/retention; SaaS **or self-hosted**; SSO/audit/HIPAA path |

**Correction:** pass-7 “Phoenix Cloud Core $29 / Pro $199” matched **Langfuse** Cloud shape and is **not** on live Arize AX pricing (2026-07-17). Treat secondary Cekura/Laminar tier names as **stale** unless re-proven. AX Enterprise table also lists agent **Signal** (find failure modes / open PRs) and managed agents as Enterprise-only — another fixer surface, not Parallax-unique.

Sources: live [arize.com/pricing](https://arize.com/pricing) (pass 61); [arize.com/phoenix](https://arize.com/phoenix/).

> Parallax pricing: **no public number** (pre-release). Direct comparison
> **benchmark-dependent, unmeasured**.

## Axis-by-axis comparison

### Signal coverage

| Signal | Phoenix (shipped) | Parallax (pre-release; ✅🧪=code-shipped) | Who |
|---|---|---|---|
| LLM / model spans (prompt, completion, tokens, cost) | ✅ core | ✅ (🏗) | **Phoenix** |
| Agent / tool / retrieval spans (multi-agent) | ✅ hierarchical, OpenInference | ✅ (🏗) | **Phoenix** |
| Non-LLM spans (API, embeddings) | ✅ in same trace | ✅ (🏗) | **Phoenix** |
| Production app traces (OTLP) | 🟡 receives OTLP traces (LLM-focused, not a general telemetry backend) | ✅🧪 OTLP-native (shipped, pre-release) | tie (different scope) |
| Logs | 🟡 trace-scoped, not a log platform | ✅🧪 OTLP logs (shipped, pre-release) | **Parallax** (breadth) |
| Metrics | ❌ not a metrics platform | ✅🧪 OTLP metrics (shipped, pre-release) | **Parallax** (breadth) |
| Production errors / exceptions | 🟡 LLM-eval failures, not prod error events | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) | **Parallax** (design; unproven) |
| Eval scores / annotations | ✅ core (LLM-as-judge heritage) | ✅ planned (A1) | **Phoenix** |
| Datasets / experiments | ✅ core | ❌ out of scope | **Phoenix** |

**Verdict:** on **LLM/agent-tracing + eval/experiment tooling, Phoenix wins decisively.** On **production telemetry breadth (logs/metrics/errors), Parallax's design is broader** — Phoenix is not a general observability backend.

### Ingestion & transport

- **Phoenix: OTLP-native receiver** — accepts OTel traces (Phoenix-aware defaults),
  built on **OpenTelemetry + OpenInference** (Arize authored OpenInference, the
  semantic-convention standard for AI/LLM spans — a real standards-leadership
  edge). Multi-agent tracing auto-logs each agent/tool interaction as a span.
- **Parallax: OTLP-native (traces/logs/metrics) + shipped Sentry-envelope.**

> Both OTLP-native. Phoenix's edge: **OpenInference** (the AI-span semantic
> standard) + deep LLM-framework instrumentation. Parallax's edge: general
  multi-signal OTLP storage. **Direct overlap on agent-trace OTLP ingest.**

### Storage architecture

Phoenix: **default file-based SQLite** (configurable via DB URL; no native ClickHouse). Parallax: GreptimeDB native OTLP tables + Turso.
Both self-hostable; Phoenix's is shipped/mature, Parallax's is newer and
**benchmark-dependent, unproven**.

### Query & correlation

Phoenix: trace-centric drill-down (LLM trace → nested tool/model spans →
attached eval scores → dataset/experiment). Strong within the LLM domain, **not**
a cross-signal (metrics↔logs↔traces↔infra) engine. Parallax: evidence-graph +
run_id stitching + bounded bundle (**unproven**, A1).

### Evaluation & the LLMOps loop — Phoenix's moat

Phoenix's **trace → eval → experiment** loop is the product: LLM-as-judge
evaluators (its heritage), human annotations, datasets built from traces,
experiments comparing prompt/model variants, prompt playground. **This is the
canonical AI-app dev loop, shipped and mature.** Parallax's A1 eval is about
agent *outcomes*, unbuilt/unproven.

### Dashboards & visualization

Phoenix: trace explorer, eval/dataset UI, analytics (latency/cost/tokens),
prompt playground. Mature for the AI-app domain. Parallax: minimal V1.
**Phoenix wins** within its domain.

### AI-native / agent-context story (Parallax's wedge — the crux)

- **Phoenix:** an **AI-app observability + eval platform for developers improving
  LLM/agent applications** — trace, evaluate, experiment. A human dev loop +
  analytics, **not** a context engine that serves bounded, redacted evidence to
  autonomous coding agents for *production-incident* resolution. No production-
  error derivation, no fix-outcome loop, no read-only bounded agent projection.
- **Parallax's claim:** bounded, redacted, agent-use (safety/value unproven) evidence bundle for coding
  agents (CLI/HTTP first, MCP after gates) for production incidents.

> **Honest verdict:** Phoenix (like Langfuse) is **far more mature** on capturing
> + structuring agent/LLM execution traces, and drives **OpenInference** — the
> standard Parallax would likely align to. On shipped capability, **Phoenix
> leads.** Parallax's differentiation lives entirely in cells Phoenix does not
> occupy: production-error derivation, fix-outcome loop, bounded/redacted agent
> bundle — all **unproven (A1 gate).** A team wanting "agent traces + LLM-as-judge
> evals" gets far more from Phoenix today than from pre-release Parallax.

### Architecture & deployment

Phoenix: self-host (Docker, ELv2, free, unlimited) **or** Phoenix Cloud (managed).
Arize AX for enterprise. Parallax: single-binary self-host target, local-first,
Apache-2.0. Both open + self-hostable; **Phoenix shipped/mature, Parallax
pre-release.**

### Scalability & performance

Phoenix: proven at scale (OSS community + Cloud + AX customers). Specific numbers
vendor/marketing; not independently measured. Parallax: **benchmark-dependent,
unproven.** On proven scale, **Phoenix wins.**

### Security & compliance

Phoenix Cloud / Arize AX: enterprise security (Arize AX). Self-host = your own
posture (OSS). Parallax: SSO/RBAC/audit planned; redaction (A6) designed.
**Phoenix/Arize wins on shipped enterprise posture.**

### Openness, licensing & lock-in (real difference)

- **Phoenix: ELv2** — free to use + self-host with **no feature gates**, but
  **not OSI-open** and **restricts offering it as a managed service**. This is
  *less* permissive than **Langfuse (MIT)** and **Parallax (Apache-2.0)**.
  Low lock-in via OTLP/OpenInference (vendor-neutral standards).
- **Parallax: Apache-2.0**, fully open, OTLP-native.

> **Verdict:** on **license permissiveness, Parallax (Apache-2.0) edges Phoenix
> (ELv2)**, and Langfuse (MIT) edges Phoenix too. ELv2's managed-service
> restriction is a real consideration for embedders — but for a self-hosting
> end-user it is effectively free + unlimited. Honest, scoped.

### Extensibility

Phoenix: LLM-framework integrations (OpenAI/Anthropic/LangChain/etc.), OTel/
OpenInference SDKs, Python-first, public API, datasets/experiments API.
**Mature for the AI-app ecosystem.** Parallax: OTel-native, CLI/HTTP/MCP,
pipeline/processor (planned).

### Pricing & economics

Phoenix self-host = **free, unlimited, ELv2**. Managed path = **AX Free / AX Pro
$50 / Enterprise custom** (pass 61). **Hard to undercut on price for the LLM-tracing job.** Parallax cost
edge only applies to its *different* job (production telemetry evidence);
**benchmark-dependent, unmeasured.**

## Where Phoenix plainly wins (no bias)

1. **LLM/agent-tracing + OpenInference** — purpose-built + drives the semantic standard.
2. **Evaluation maturity** — LLM-as-judge heritage, datasets, experiments.
3. **OSS self-host-free** (ELv2, unlimited).
4. **Python-native** ergonomics + large OSS community.
5. **Cloud + Arize AX** scale + enterprise.
6. **Proven-at-scale, shipped today.**

## Where Parallax honestly edges Phoenix

1. **Production telemetry breadth** — OTLP-native logs/metrics/errors; Phoenix is not a general telemetry backend. *(Real design difference.)*
2. **Production error events + fix-outcome loop** — Phoenix has neither. *(Thesis, **unproven** — A1 gate.)*
3. **Bounded, redacted, agent-use (safety/value unproven) evidence bundle for production incidents** — Phoenix is an AI-app eval loop, not an incident-context engine. *(Thesis, **unproven** — A1 gate; existential question.)*
4. **License permissiveness** — Apache-2.0 vs ELv2 (managed-service restriction). *(Narrow but real.)*
5. **Single-binary local-first** — Phoenix self-host is a Docker/container stack. *(Minor design edge.)*

## Watch triggers — re-evaluate Phoenix/Arize if it:

- Adds **production-error derivation / an error-issue lifecycle** → erodes Parallax's error-workflow edge.
- Adds a **bounded, versioned, redacted evidence-bundle artifact** with outcome semantics → pressures A3.
- Adds a **fix-outcome loop** → closes the core-thesis differentiator.
- Ships **production logs/metrics** storage (becoming a general telemetry backend) → erodes Parallax's breadth edge.
- **Changes license** away from ELv2 (Arize has historically kept Phoenix open).

## Sources (checked 2026-07-17)

- [github.com/Arize-ai/phoenix](https://github.com/Arize-ai/phoenix) — README, releases (**arize-phoenix-v18.1.0**, 2026-07-17)
- [arize.com/docs/phoenix](https://arize.com/docs/phoenix) — OTLP-native ingest
- [arize.com/docs/phoenix/self-hosting/license](https://arize.com/docs/phoenix/self-hosting/license) — **ELv2**
- [arize.com/docs/ax/concepts/otel-openinference/overview](https://arize.com/docs/ax/concepts/otel-openinference/overview) — OpenTelemetry + OpenInference
- [arize.com/phoenix](https://arize.com/phoenix/); live [arize.com/pricing](https://arize.com/pricing) (**pass 61** AX Free / Pro $50 / Enterprise); secondary analyses demoted (stale Core $29/Pro $199)
- Parallax side: [00-vision/ai-native-observability.md](../../00-vision/ai-native-observability.md), [reference/agent-observability-review.md](../../reference/agent-observability-review.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/)
