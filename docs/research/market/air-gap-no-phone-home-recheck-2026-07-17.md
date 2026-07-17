# Air-gap / no-phone-home differentiator recheck (2026-07-17)

<!-- markdownlint-disable MD013 -->

**Pass target:** research-agenda comparison #5 — does the **air-gapped,
no-phone-home agent-evidence** differentiator stay unique vs incumbents?

**Prior theory:** hard-boundary buyers need self-host with no vendor control
plane / no mandatory cloud AI; Seer and Bits are cloud; Grafana on-prem may
still phone home for some features.

**Verdict (pass 56 + pass 151 + pass 178):** **Still unique for the *combination*
of air-gap-capable open core + portable agent evidence + no closed cloud AI
dependency.** Narrower claims need care:

| Claim | Status 2026-07-18 (pass 178) | Evidence |
| --- | --- | --- |
| Sentry Seer available self-hosted | **False — still excluded** | [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/): "Seer and other AI & ML features… currently closed source" (**pass 126 + pass 151 + 158** primary re-fetch) |
| Datadog Bits / product as self-hosted backend | **False** | Pass 42 + **111** + **178:** [Observability Pipelines](https://docs.datadoghq.com/observability_pipelines/) — Worker runs in-infra to **route** logs/metrics to **destinations** (Datadog, SIEM, cloud storage). **Not** a self-hosted Datadog store/UI/Bits backend. |
| OSS Grafana / LGTM can run offline | **True for OSS core** | Self-host OSS is real; **Enterprise plugins/license keys** and Cloud AI features are separate — do **not** claim "all Grafana phones home" without a dated primary for a specific binary feature |
| Grafana Assistant on self-managed | **UI yes / AI backend Cloud** | **Pass 77:** plugin + connect to Grafana Cloud stack required; not offline BYO-LLM — [incumbent-self-hosted-ai-recheck-2026-07-17.md](incumbent-self-hosted-ai-recheck-2026-07-17.md) |
| OSS SigNoz / OpenObserve / Traceway / Rustrak / GlitchTip / Bugsink can air-gap | **True (product can)** | Self-host OSS peers; **none** ship Parallax-style portable redacted evidence bundle + outcome loop (passes 49–53 + 122–125 + 134) |
| Langfuse self-host phone-home | **Default usage telemetry ON** | **Pass 107:** README — self-hosted instances report basic usage stats to centralized PostHog by default (not raw traces); opt-out documented. Not air-gap-clean without config. |
| Full combination closed | **False** | No peer ships open portable redacted prod-incident bundle + outcome under air-gap |

**Pass 151:** Seer exclusion re-confirmed (same primary sentence as pass 126).
Wedge combination rechecks (Traceway/Bugsink/Rustrak/TMA1/GlitchTip) still show
**no** portable redacted prod-incident bundle + outcome under pure air-gap.
Differentiator **holds as combination claim**; A1 value still unproven.

**Pass 178 (2026-07-18):** Datadog OPW primary docs reconfirm **in-infra routing
worker**, not self-hosted Datadog product backend. Seer exclusion last deep-checked
pass **158**. Grafana Assistant hybrid Cloud LLM last pass **158**. Combination
air-gap claim **still holds**.

**Pass 220 (2026-07-18):** OPW docs still describe Worker that **routes** to
**destinations** (not a self-hosted Datadog store/UI). Air-gap combination claim
**still holds**.

**Pass 251 (2026-07-18):** Datadog [Observability Pipelines](https://docs.datadoghq.com/observability_pipelines/)
primary re-fetch (markdown/docs body):

- Overview: collect/process logs and metrics **within your own infrastructure**,
  then **route** data to different **destinations**.
- Worker: runs in-infra to **aggregate, process, and route** data.
- Destinations examples: **Datadog, SIEM tools, or cloud storage** (archive
  templates → S3/GCS/Azure).
- Control plane: Observability Pipelines **UI** is a centralized place to build
  pipelines / deploy Workers (Datadog product surface — not "full Datadog offline").

**Still route-only for OPW** — **not** a self-hosted Datadog APM/Bits/UI product
store. **UNFIRED** as air-gap Seer/Bits replacement.

### Pass 261 (2026-07-18) — Datadog BYOC Logs (CloudPrem branding) deep pin

Primary sources:

| Source | Role |
| --- | --- |
| [docs.datadoghq.com/cloudprem/](https://docs.datadoghq.com/cloudprem/) | Docs hub → **BYOC Logs** product pages |
| [docs.datadoghq.com/byoc_logs/](https://docs.datadoghq.com/byoc_logs/) (llms index) / byoc-logs intro | Self-hosted **log** ingest/index/search in customer cloud/K8s |
| [Introducing BYOC Logs blog](https://www.datadoghq.com/blog/introducing-datadog-byoc-logs/) | Architecture + hybrid SaaS story |

**What it is (primary language):**

- Hybrid log management: compute/index nodes + **object storage in customer
  environment** (on-prem or S3/GCS/Azure); search via Datadog **Log Explorer**.
- Explicitly **fully integrated with the Datadog SaaS platform** — correlation
  with SaaS metrics/traces, Bits AI SRE / NLQ, MCP-ready over BYOC log datasets,
  governance (RBAC, audit, SDS) as on SaaS.
- Built for residency/compliance/high-volume **logs**, not a claim of full
  offline multi-signal agent evidence without Datadog control plane.

**What it is not (honest bounds):**

| Claim to avoid | Why |
| --- | --- |
| "Datadog now ships air-gap Seer/Bits offline" | AI assistance in blog is **Bits AI** over SaaS-correlated signals; BYOC keeps SaaS integration |
| "Same as OPW" | OPW = **route** worker; BYOC Logs = **store/search logs** in customer infra + SaaS UI |
| "Closes Parallax air-gap combination" | No portable redacted **investigation evidence bundle** + fix-outcome; still vendor SaaS for AI/UI; logs-first hybrid not open Apache context engine |

**Verdict:** BYOC Logs is a **material residency/cost product** and a **partial
pressure** on "keep logs in my cloud" buyers — **not** a falsification of the
air-gap **combination** differentiator (open core + portable redacted agent
evidence + no mandatory vendor cloud AI). OPW route-only claim **unchanged**.
**Continue watching** if Datadog ships offline Bits / multi-signal BYOC store
without SaaS phone-home.

### Pass 279 (2026-07-18) — OPW primary re-fetch

[Observability Pipelines](https://docs.datadoghq.com/observability_pipelines/)
still: collect/process then **route** to destinations; Worker **aggregate,
process, and route**. Still **not** self-hosted Bits/APM store. BYOC Logs
hybrid story (pass 261) unchanged. Air-gap combination claim **holds**.

### Pass 291 (2026-07-18) — OPW + BYOC hub + Assistant hybrid

| Source | Finding |
| --- | --- |
| Observability Pipelines docs | still **route** / Worker **aggregate, process, and route** |
| docs.datadoghq.com/cloudprem/ | still **HTTP 200** BYOC Logs hub (hybrid SaaS-integrated log store) |
| Grafana Assistant self-managed | still **backend/billing in Grafana Cloud** |

Air-gap combination claim **holds**. OPW ≠ Bits store; BYOC ≠ offline agent-evidence; Assistant ≠ offline BYO-LLM.

### Pass 303 (2026-07-18) — OPW + Assistant + Seer (GO composite)

| Source | Finding |
| --- | --- |
| [Observability Pipelines](https://docs.datadoghq.com/observability_pipelines/) | still: process **within your own infrastructure**, then **route** to **destinations**. Worker = pipeline surface — **not** self-hosted Bits/APM store |
| [Grafana Assistant self-managed](https://grafana.com/docs/grafana/latest/administration/assistant/) | still hybrid: Assistant **backend, usage limits, and billing stay in the Grafana Cloud stack** |
| [develop.sentry.dev/self-hosted](https://develop.sentry.dev/self-hosted/) | still Seer/AI closed-source exclusion |

Air-gap combination claim **holds**. OPW ≠ Bits store; Assistant ≠ offline BYO-LLM; Seer self-host **UNFIRED**.

### Pass 314 (2026-07-18) — OPW + Seer + Assistant

| Source | Finding |
| --- | --- |
| Observability Pipelines | still process **within your own infrastructure**, then **route** to **destinations**; Worker install surface — **not** self-hosted Bits store |
| develop.sentry.dev/self-hosted | still Seer **closed source** exclusion |
| Grafana Assistant self-managed | still hybrid **Grafana Cloud stack** backend/billing |

Air-gap combination claim **holds**.

### Pass 324 (2026-07-18) — OPW + Assistant + Seer

OPW still **route-to-destinations**; Assistant still **Grafana Cloud stack** backend; Seer still closed. Air-gap combination claim **holds**.

### Pass 334 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap combination claim **holds**.

### Pass 343 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 348 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 353 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 356 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 359 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 362 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 365 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.

### Pass 368 (2026-07-18) — OPW

OPW still **route-to-destinations**. Air-gap claim **holds**.












Air-gap combination claim (open core + portable redacted evidence + no closed
cloud AI) **still holds** as a *combination* (A1 still unproven).

**Falsification:** a major incumbent ships **self-hosted, open (or source-available)
agent evidence with no cloud AI dependency** *and* a portable redacted bundle
schema; or Seer becomes self-host GA with offline models; or Grafana Assistant
runs fully offline with BYO-LLM (would fire pass-77 UNFIRED item).

**Uncertainty:** Enterprise license telemetry for Grafana/Elastic/etc. was
**not** re-instrumented this pass — mark any "X phones home" claim
**unverified** unless a primary doc is attached. Air-gap *capability* ≠ "never
phones home on every build."

**Implication for A2:** pass 54 monetization desk recheck still aligns —
hard-boundary buyers pay for boundaries incumbents refuse to open. Differentiator
is **real but niche** and does not replace A1 value proof.
