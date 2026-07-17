# Parallax vs Datadog

> An unbiased, one-to-one comparison. Research date: **2026-07-17** (**pass 111**
> self-host backend recheck: still **SaaS-only product**). Sources: live
> [Datadog pricing page](https://www.datadoghq.com/pricing/) (accessed 2026-07-17),
> [Datadog product/docs](https://www.datadoghq.com/), [Bits AI](https://www.datadoghq.com/blog/bits-ai-sre/)
> / [Bits Agent Builder](https://www.datadoghq.com/blog/bits-agent-builder/),
> [Agent Observability](https://www.datadoghq.com/products/ai/agent-observability/),
> plus third-party pricing analyses dated 2026.
>
> **Bottom line up front:** on breadth, maturity, scale, enterprise readiness, and shipped AI features, **Datadog is far ahead of pre-release Parallax.** Parallax's only honest edges are openness/self-hostability, cost transparency and data ownership, and an *unproven* evidence-bundle + fix-outcome thesis. A comparison that concluded otherwise would be dishonest.

## What each product is

- **Datadog** — the broadest commercial observability-and-security SaaS on the market: infrastructure, APM/traces, logs, metrics, profiling, RUM + session replay, database monitoring, serverless, network, CI/test visibility, incident/on-call management, cloud cost, cloud security (CSPM/CWPP/SIEM), LLM/agent observability, and an AI assistant (**Bits AI**). The collection Agent is open source; the backend and almost all product surfaces are proprietary SaaS. Founded 2010, public (NYSE:DDOG).
- **Parallax** — open-source (Apache-2.0), Rust-first, self-hostable **execution-context engine**. Ingests OTLP traces/logs/metrics + CLI/coding-agent execution traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, and serves **bounded, redacted, schema-valid evidence bundles** to humans and coding agents (CLI/HTTP first, read-only local-stdio MCP graduated (plan 112 DONE; remote deferred)). Storage: GreptimeDB (telemetry, native OTLP tables) + Turso (metadata). **Pre-release.**

These are not the same product. Datadog is a mature, closed, broad platform aimed at large enterprise SRE/SecOps teams. Parallax is a narrow, open, self-hostable context engine for coding agents and the local-dev/production-incident boundary. The honest comparison is axis-by-axis, not "which wins."

## Signal coverage — Datadog is far broader

Datadog ingests and correlates essentially **every signal** an enterprise cares about, at maturity Parallax does not have:

| Signal | Datadog (shipped) | Parallax (pre-release; ✅🧪=code-shipped) |
| --- | --- | --- |
| Traces / distributed tracing | ✅ full, 8 tracer languages + mobile + inferred spans | ✅🧪 OTLP traces (shipped, pre-release) |
| Logs | ✅ full, 200+ parsers, pipeline processing | ✅🧪 OTLP logs (shipped, pre-release) |
| Metrics | ✅ full, custom + 1000+ integrations | ✅🧪 OTLP metrics (shipped, pre-release) |
| Errors / exceptions | ✅ Error Tracking — auto-grouping into issues, lifecycle, Git/IDE pivot | ✅🧪 derived `error_event` + fingerprint (shipped, pre-release) |
| Continuous profiling | ✅ Continuous Profiler (wall/CPU/mem/lock/IO; Rust in beta) | ❌ |
| RUM + session replay | ✅ RUM Measure/Investigate + Session Replay (web + mobile) | ❌ |
| LLM / agent spans | ✅ Agent Observability — LLM tracing + evals + experiments + prompt tracking | ✅ (🏗) |
| CI / test visibility | ✅ CI Pipeline Visibility + Test Optimization + Code Coverage | ✅ (🏗) |
| Database monitoring | ✅ DBM ($70/db-host/mo; Postgres/MySQL/SQL Server/Oracle/Mongo/ClickHouse/Supabase) | ❌ |
| Synthetic testing | ✅ API/Browser/Mobile + Continuous Testing | ❌ |
| Cloud security | ✅ CSPM/KSPM/VM/CIEM/CWPP/SIEM | ❌ |
| Cloud cost | ✅ Cloud Cost Management | ❌ |
| Incident / on-call | ✅ Incident Response (On-Call + Incident Mgmt) | ❌ (out of scope) |

**Verdict:** Datadog's signal coverage is an order of magnitude broader than Parallax's target. Parallax is deliberately narrower (it is a context engine, not a pan-platform suite). On raw coverage, **Datadog wins decisively.**

## Ingestion & transport

- **OTLP support:** Datadog ingests OTLP — via the Datadog Agent's OTLP receiver (**GA**: `otlp_config` ingests **traces + metrics by default, logs with manual `logs: enabled: true`**, [docs](https://docs.datadoghq.com/opentelemetry/setup/otlp_ingest_in_the_agent/)) **and** a pure-consumption **Managed Platform OTLP** product ([$0.50/GB ingested spans](https://www.datadoghq.com/pricing/), no allotments, for Vercel/Cloudflare/Mulesoft/etc direct export). But Datadog is **not OTLP-native in storage** — OTLP data is transformed into Datadog's proprietary format and backend. Parallax's design is OTLP-native (raw telemetry stored in GreptimeDB's native `opentelemetry_logs`/`opentelemetry_traces`/per-metric tables). Whether OTLP-native storage beats OTLP-transform-into-proprietary is **benchmark- and architecture-dependent, unmeasured here.**
- **Sentry envelope/DSN:** Parallax **ships** a Sentry-envelope compatibility ingest (absorb Sentry's 30+ SDKs; plan 118 DONE). Datadog has no Sentry-envelope path. This remains one of Parallax's genuine wedges against the broad platforms.
- **Agent auto-instrumentation:** Datadog ships mature tracers for Java, Python, Ruby, Go, Node, .NET, PHP (APM) + mobile (iOS/Android/RN/Flutter/Unity) + Universal Service Monitoring (eBPF, language-agnostic RED metrics, $9/host). Parallax relies on OTel SDKs. On SDK breadth, **Datadog wins decisively.**
- **Cardinality handling / retention:** Datadog metering distinguishes ingested vs indexed spans (150 GB ingested + 1 M indexed spans per APM host/mo), and metrics vs logs vs events each separately. Parallax's cardinality ceiling is benchmark-dependent and unmeasured.

## Storage architecture — different bets

- **Datadog:** proprietary closed backend; internals not publicly documented. Operational properties (ingest throughput, query latency, compression) are vendor claims, not independently measured here. Retention: 15-month metrics/spans; 15-day default log/span index with **Flex Storage** up to 15 months ([pricing](https://www.datadoghq.com/pricing/)).
- **Parallax:** GreptimeDB (telemetry, native OTLP tables, Rust, self-hosted) + Turso/libSQL (metadata). Chosen for anchored evidence-bundle retrieval (≪300 ms interactive target) and the Rust self-hosted substrate. Parallax's storage performance vs Datadog's proprietary stack is **benchmark-dependent and unproven** — do not assume the open engine wins; it has not been measured against Datadog.

Honest framing: Datadog's storage is opaque but battle-tested at hyperscale (the company's existence proves it works at very large volume). Parallax's GreptimeDB bet is newer and its production scale is **unproven**. On proven-at-scale, **Datadog wins.**

## Query & correlation

- **Datadog:** unified pivoting across metrics↔traces↔logs↔RUM↔profiles↔infra↔network is the platform's core strength — click a trace, pivot to logs, profiles, the host, the deploy, the related synthetic test. Watchdog (anomaly/outlier/root-cause detection) and **Bits Investigation** auto-correlate across telemetry. This is genuinely best-in-class.
- **Parallax:** correlation into a typed evidence graph + trace-to-log + run_id/invocation stitching + evidence pinning. The *artifact* (bounded evidence bundle) **exists in code** but is **A1-unproven** for agent fix quality, and narrower in cross-signal breadth than Datadog's shipped pivots.

**Verdict:** on cross-signal correlation as it exists today, **Datadog wins decisively.** Parallax's evidence-bundle abstraction is a different axis (agent-actionable, bounded, redacted) — not a like-for-like better correlation engine, and its value is unproven (A1 gate).

## Dashboards & visualization

- **Datadog:** a mature dashboard builder, timeboards/screenboards, 1000+ out-of-the-box integration dashboards, service maps (USM), notebook-style investigation, and an **App Builder** (low-code internal tools, $35/published app/mo). React Flow not relevant here; Datadog's own viz stack.
- **Parallax:** V1 UI spec = Sentry-grade issues list/detail, predefined + user dashboards, trace lookup, chart→window→event→trace interactivity (TanStack Start + shadcn, graph viz via React Flow). Narrower by design.

**Verdict:** **Datadog wins** on dashboard breadth/maturity. Parallax's V1 UI is intentionally minimal.

## Alerting & on-call

- **Datadog:** monitors (metric/outlier/anomaly/forecast/composite/log/AI-based), Watchdog Insights, an **Event Management** correlation product ($0.10/evaluated event), full **Incident Response** suite (on-call schedules, escalation, paging, postmortems, status pages) — On-Call $20, Incident Mgmt $30, bundle $58 per seat/mo. End-to-end.
- **Parallax:** minimal alerting in V1 scope (planned, partial). No on-call/incident suite (explicitly out of scope).

**Verdict:** **Datadog wins decisively.** This is not a space Parallax competes in.

## Profiling

- **Datadog Continuous Profiler:** wall/CPU/memory/lock/IO/goroutine/exception profiles; Java/.NET/Go/Python/Ruby/Node/PHP; **Rust in beta**. Standalone $19/profiled host/mo (annual) or included in APM Enterprise. Low-overhead (JDK Flight Recorder). Mature.
- **Parallax:** none in V1 scope.

**Verdict:** **Datadog wins.** Profiling is out of Parallax's current scope.

## Developer experience

- **Datadog:** docs at scale, 1000+ integrations, quickstart in minutes for common stacks, a polished UI, mature SDKs, and AI-assisted onboarding (Claude Code/Cursor single-prompt setup for Agent Observability). The cost and complexity of billing is the main DX complaint (well-documented by third parties — see Pricing).
- **Parallax:** CLI-first, single binary, local-first, one-command local run (target). Smaller SDK surface (OTel-based); docs and quickstart maturity TBD (pre-release).

**Verdict:** on **time-to-first-value for a broad stack, Datadog wins** (mature SDKs + integrations). On **local-dev simplicity and a single-binary local loop, Parallax's target beats Datadog** (Datadog is SaaS-only and has no local-first story). Different DX axes; call each scoped.

## AI-native / agent-context story — the key wedge axis

This is where Parallax claims differentiation, so it must be examined most honestly.

- **Datadog's AI surface (shipped, credit-metered):**
  - **Bits Chat** — natural-language query/analysis, dashboards, monitor inspection (~0.5 credits/msg).
  - **Bits Investigation** — autonomous alert investigation, root-cause correlation, impact summary, remediation (~6.5 credits/run).
  - **Bits Code** — AI-assisted code generation/review/debugging, opens PRs (~5 credits/fix).
  - **Bits Agent Builder** — custom AI agents for ops workflows (~3 credits/run).
  - **Bits Security Analyst** — autonomous Cloud SIEM triage.
  - Priced as **AI Credits** ([live pricing](https://www.datadoghq.com/pricing/?product=ai-credits) + [list](https://www.datadoghq.com/pricing/list/), pass **64** re-confirm): **$500 / 500 credits** annual; **$600 / 500** month-to-month (list); **$1.30/credit** on-demand. Credit table: Chat ~0.5, Agent Builder run ~3, Code ~5, Investigate ~6.5. Credits reset monthly (no rollover).
  - **Datadog Agent Observability** — a dedicated **LLM/agent observability product**: end-to-end LLM tracing (prompts, tool calls, retrieval, decisions), offline/online evals, datasets, experiments, prompt tracking, playground, human annotation; Free 40K LLM spans/mo, Pro $160/mo annual for 100K LLM spans (+$3.5/10K); **Sensitive Data Scanner included**. This is a direct competitor to the LLM-observability wedge (Langfuse/Phoenix/LangSmith) and ships more agent-tracing surface than Parallax has today.

- **Parallax's AI claim (code-shipped, A1 value unproven):** a **read-only, redacted, bounded evidence bundle** served to coding agents (CLI/HTTP first, local-stdio MCP graduated (plan 112 DONE; remote deferred)) — a *context engine for autonomous agents*, not a human chat dashboard. The thesis is that a bounded, validated, redacted dossier beats dumping raw telemetry into an agent.

**Honest verdict:** On every *shipped* AI axis — natural-language query, autonomous investigation, autofix-to-PR, LLM tracing, evals — **Datadog is ahead, and by a lot.** Parallax's only differentiated AI claim is the bounded/redacted/agent-safe bundle, which is **unproven** (this is the A1 gate: does such a bundle actually improve agent fix outcomes vs raw context?). Datadog's Bits Investigation already does much of what a "context for triage" story promises, today, at scale. The burden of proof that Parallax's bundle beats Bits-as-context is on Parallax and has not been met.

A real Datadog weakness here, written plainly: its AI is a **human dashboard plus chat**, gated behind Datadog's SaaS and credit meter, with **write/destructive management-plane capability** — it is not a safe, bounded, self-hosted agent-context projection. That is the genuinely unoccupied cell. But "unoccupied" ≠ "valuable"; it is unproven.

## Architecture & deployment model

- **Datadog:** SaaS-first, multi-region (US/EU/APAC), multi-tenant. **Gov:** **Datadog for Government (US1-FED) achieved FedRAMP® High** (announced **2026-05-06**; FedRAMP Marketplace FR2023864279A, Class D High as of 2026-05-05) — elevates beyond older “Moderate subset” framing. **No production self-host of the observability *backend***: only the OSS Agent + **Observability Pipelines Worker** (runs in your env to aggregate/process/route; still a path into Datadog SaaS, not a self-hosted Datadog store). If the constraint is "data never leaves our network / air-gap," Datadog still largely cannot satisfy it.
- **Parallax:** self-hosted, single-binary target, local-first, air-gapped-capable, three deployment tiers. Designed for the team that cannot or will not use a closed SaaS.

**Verdict:** on **self-host / air-gap / data-sovereignty, Parallax wins (by design); Datadog cannot play here.** On **managed-SaaS scale/multi-region/multi-tenancy, Datadog wins; Parallax has none of that operational machinery.** Again, different axes.

## Operational footprint

- **Datadog:** near-zero backend ops for the customer (it's SaaS); the customer runs the Agent + integrations. Day-2 (upgrades, scaling, HA) is Datadog's problem. The cost is money, not operator burden.
- **Parallax:** the customer operates GreptimeDB + Turso + the Parallax engine. Lower cash cost; nonzero operator burden. Parallax's stated goal is to minimize this (single binary, one-command local run), but production-grade GreptimeDB + Turso operation is real work.

**Verdict:** on **operator burden, Datadog (SaaS) is lower.** On **cash cost and vendor dependency, Parallax (self-host) is lower.** Scoped.

## Scalability & performance

- **Datadog:** demonstrably operates at hyperscale (tens of thousands of enterprise customers, very large ingest). Specific throughput/latency numbers are vendor-marketing and not independently measured here. The fact it scales is proven by its business.
- **Parallax:** unproven at production scale. GreptimeDB and Turso are individually proven technologies, but Parallax's specific throughput/latency/cardinality ceiling is **benchmark-dependent and unmeasured.**

**Verdict:** on **proven-at-scale, Datadog wins conclusively.** Parallax cannot yet make a measured scale claim. (Flagged for the benchmark program.)

## Security

- **Datadog:** SSO/SAML (Enterprise), SCIM, multiple SAML providers, IP/email-domain allowlists, fine-grained RBAC, Data Access Control, **Audit Trail** ($X, billed annually), Secrets Management, audit logging. The Datadog Agent is OSS (reviewable). **Genuinely best-in-class.**
- **Parallax:** SSO/RBAC/audit are planned, not shipped. Redaction (the A6 gate) is designed as a first-class pipeline.

**Verdict:** on **shipped enterprise security posture, Datadog wins decisively.** Parallax has essentially none of it yet. Parallax's only security-relevant edge is *redaction-by-default-before-agent-access* — a narrower, unproven claim.

## Privacy & compliance

- **Datadog:** SOC 2, HIPAA, PCI; data residency US/EU/APAC; **FedRAMP High** on **Datadog for Government (US1-FED)** (2026-05; not “Moderate-only”); **Sensitive Data Scanner** (detection $0.03/GB @ 10% sampling, or $0.30/scanned GB full obfuscation) for PII scrubbing across logs/APM/RUM/S3.
- **Parallax:** no compliance certifications (not yet — pre-release, Apache-2.0). Redaction is a designed pipeline (A6) but unattested. Data ownership is total (self-host).

**Verdict:** on **compliance certifications and attested PII tooling, Datadog wins decisively.** On **data ownership/sovereignty (self-host, air-gap), Parallax wins by design.**

## Openness, licensing & vendor lock-in

- **Datadog:** closed-source proprietary SaaS (Agent is OSS). High vendor lock-in: proprietary data format, proprietary query surface, proprietary dashboards, no portable export of the correlated store. **Migration out of Datadog is a well-documented, expensive undertaking.** This is a real, structural Datadog weakness.
- **Parallax:** Apache-2.0, fully open, self-hostable, OTLP-native (standard format in and out), evidence bundle **code-shipped** as a *portable* artifact (A1 value unproven). Low lock-in by construction.

**Verdict:** on **openness and lock-in cost, Parallax wins decisively; Datadog's closed proprietary model is a genuine liability for buyers who value portability and data ownership.** This is Parallax's strongest *real* (non-thesis) edge and should not be understated.

## Extensibility

- **Datadog:** 1000+ vendor integrations, custom integrations via Agent plugins + DogStatsD + HTTP API, **Workflow Automation** (300+ OOTB actions, $10/100 executions), **App Builder** (1750+ actions), webhooks, Terraform provider, public API. Deepest integration ecosystem in the market.
- **Parallax:** OTel-native (any OTel instrumentation works), pipeline/processor model, CLI/HTTP/MCP surfaces, webhooks (planned). Much smaller ecosystem by design/volume.

**Verdict:** on **integration ecosystem breadth, Datadog wins decisively.**

## Pricing & economics — real numbers

Datadog pricing is **public** and itemized below (annual prices, [datadoghq.com/pricing](https://www.datadoghq.com/pricing/), accessed 2026-07-17):

| Product | Annual price | Unit / notes |
| --- | --- | --- |
| Infrastructure Free | $0 | 5 hosts, 1-day retention |
| Infrastructure Pro | $15 / host / mo ($18 on-demand) | 100 custom metrics/host, 15-mo retention |
| Infrastructure Enterprise | $23 / host / mo ($27 od) | 200 custom metrics/host, ML alerts, Live Processes |
| APM (w/ Infra attached) | $31 / host / mo ($36 standalone) | 150 GB ingested + 1 M indexed spans/host/mo |
| APM Pro | $35 attached ($41 standalone) | + Data Streams Monitoring |
| APM Enterprise | $40 attached ($47 standalone) | + Continuous Profiler |
| Continuous Profiler (standalone) | $19 / profiled host / mo | |
| Log ingest | $0.10 / ingested GB | uncompressed |
| Log Standard Indexing | $1.70 / M events / mo | 15-day |
| Log Flex Storage | $0.05 / M events stored / mo | up to 15 months |
| RUM Measure / Investigate / Replay | $0.15 / $3 / $2.50 per 1K sessions | |
| Database Monitoring | $70 / db host / mo ($84 od) | |
| Synthetic API / Browser / Mobile | $5/10K / $12/1K / $50/100 runs | |
| CI Pipeline Visibility | $8 / committer / mo | |
| Test Optimization | $20 / committer / mo | |
| Code Coverage | $8 / committer / mo | |
| Error Tracking | included w/ APM+RUM; standalone flat $25/mo ≤50k errors | |
| **Bits AI Credits** | **$500 / 500 cr** annual; **$600/500** monthly; **$1.30/cr** on-demand (**pass 64**) | Chat ~0.5; Investigate ~6.5; Code ~5; Agent Builder ~3/run |
| Agent Observability (LLM) | Free 40K LLM spans; Pro $160/mo (100K) + $3.5/10K | bills LLM spans only |
| Managed Platform OTLP | $0.50 / GB ingested spans | pure consumption, no allotments |
| Sensitive Data Scanner | $0.03/GB detect (10%); $0.30/scanned GB obfuscate | |
| Incident Response bundle | $58 / seat / mo | On-Call $20 + Inc Mgmt $30 |
| Observability Pipelines | $0.095 / ingested GB | |

**Parallax pricing:** no public product/pricing yet. Stated monetization shape (from the validation research): Apache-2.0 open core + gated enterprise-ops + managed cloud + outcome-priced fixer. No per-host/per-event consumption tax by design (self-hosted compute is the buyer's).

**The honest cost read:** Datadog's *sticker* entry points look cheap ($15/host), but the **composite bill at scale is widely documented as expensive and unpredictable** — the per-host + per-custom-metric + per-GB-log + per-indexed-span + per-LLM-span + credit model compounds, and high-cardinality Kubernetes metric explosion is a notorious surprise (documented by third parties like [Last9](https://last9.io/blog/datadog-pricing-all-your-questions-answered/), [OneUptime](https://oneuptime.com/blog/post/2026-03-13-how-datadog-pricing-actually-works/view), [Opslyft](https://www.opslyft.com/blog/datadog-pricing), dated 2026). This is a real Datadog weakness. **But** a specific "Parallax is $X cheaper than Datadog at workload Y" claim is **benchmark-dependent and unmeasured** — do not assert a saving that has not been measured. Mark the cost-superiority claim unproven until a benchmark exists.

## Where Datadog plainly wins

- Signal breadth (an order of magnitude more).
- Maturity, scale, proven-at-hyperscale.
- Cross-signal correlation / pivoting (best-in-class).
- SDK fleet + integration ecosystem (1000+).
- Shipped AI: Bits Investigation/Code/Agent Builder + Agent Observability (LLM).
- Enterprise security + compliance (SOC2/HIPAA/PCI, SSO/RBAC/audit).
- Dashboards, alerting, on-call/incident suite.
- Operational simplicity (SaaS = no backend ops for the buyer).

## Where Parallax honestly edges Datadog

- **Openness & vendor lock-in** — Apache-2.0, OTLP-native in/out, portable bundle; Datadog is closed proprietary SaaS with high documented migration cost. *(Real, structural, today — Parallax's strongest non-thesis edge.)*
- **Self-host / air-gap / data sovereignty** — Parallax is designed for it; Datadog cannot satisfy "data never leaves our network."
- **Cost transparency & predictability** — no per-event/per-host consumption tax by design; Datadog's composite metering is documented as expensive/unpredictable. *(Real in shape; specific savings unmeasured.)*
- **Local-first single-binary dev loop** — Parallax's target; Datadog has no local-first story.
- **Bounded, redacted, agent-safe context bundle** — the genuinely unoccupied cell. *(Thesis, **unproven** — A1 gate. Bits Investigation already covers much of the "context for triage" need today, from SaaS.)*

## Open questions / what measurement would settle

- **A1 gate:** does a Parallax evidence bundle beat raw-context (or beat Bits-Investigation-as-context) for agent fix quality, measurably? Unproven. Until shown, Parallax's headline differentiation is a hypothesis.
- **Cost at workload:** a measured Parallax-vs-Datadog TCO at a representative ingestion/cardinality workload. Benchmark-dependent, unmeasured.
- **Datadog self-host reality (2026)** — **RESOLVED pass 42:** still **no self-hosted Datadog backend**. Self-managed pieces = Agent + Observability Pipelines Worker only ([docs](https://docs.datadoghq.com/observability_pipelines/)); OPW processes/routes in-customer infra then to Datadog. **FedRAMP High** on US1-FED **resolved upward** (2026-05-06 blog + FedRAMP Marketplace High).
- ~~Datadog OTLP-in-Agent GA scope~~ → **resolved 2026-07-17 (pass 19): the Agent's OTLP receiver is GA — traces + metrics by default, logs with manual enable** ([docs](https://docs.datadoghq.com/opentelemetry/setup/otlp_ingest_in_the_agent/)). It still transforms OTLP into Datadog's proprietary backend (not OTLP-native storage).

## Sources (accessed 2026-07-17)

- [Datadog Pricing](https://www.datadoghq.com/pricing/) — authoritative live price page (all numbers above).
- [Datadog for Government achieves FedRAMP High (2026-05-06)](https://www.datadoghq.com/blog/datadog-achieves-fedramp-high-certification/); [FedRAMP Marketplace FR2023864279A](https://www.fedramp.gov/marketplace/products/FR2023864279A/); [Observability Pipelines Worker](https://docs.datadoghq.com/observability_pipelines/).
- [Bits Investigation blog](https://www.datadoghq.com/blog/bits-ai-sre/); [Bits Agent Builder](https://www.datadoghq.com/blog/bits-agent-builder/).
- [Datadog Agent Observability](https://www.datadoghq.com/products/ai/agent-observability/); [LLM prompt tracking](https://www.datadoghq.com/blog/llm-prompt-tracking/).
- Third-party pricing analyses (2026): [Opslyft](https://www.opslyft.com/blog/datadog-pricing), [Last9](https://last9.io/blog/datadog-pricing-all-your-questions-answered/), [OneUptime](https://oneuptime.com/blog/post/2026-03-13-how-datadog-pricing-actually-works/view).
- Parallax side: [docs/research/00-vision/](../../00-vision/), [architecture/v1-implementation-spec.md](../../architecture/v1-implementation-spec.md), [validation/a1-bundle-value/](../../validation/a1-bundle-value/).
- Legacy internal note: [competitive-comparison-matrix.md](../competitive-comparison-matrix.md) (source, dated 2026-05-31).
