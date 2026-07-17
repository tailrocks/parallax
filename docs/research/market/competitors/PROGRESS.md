# PROGRESS — Comparison Status Board

> The living work queue for this folder. Tracks verification state, last-verified
> dates, source links, open uncertainties, and the next highest-value gap. Update
> on **every pass**. State legend:
>
> - ✅ verified — checked against current primary sources this pass.
> - 🟡 inherited — carried from legacy market notes; not yet re-verified against
>   current sources. Treat as a hypothesis.
> - 🔴 stale / missing — aged number, dead source, or absent deep-dive. Highest priority.
> - ⚪ benchmark-dependent — needs measurement before any number is trusted.
>
> Today: **2026-07-17** (pass 1 — bootstrap).

## Pass log

| Pass | Date | What landed | Commit |
| --- | --- | --- | --- |
| 1 | 2026-07-17 | Bootstrap: `README.md` matrix, `comparison-set.md`, `PROGRESS.md`; first no-bias deep-dive **Datadog** (pricing + OTLP + AI/Agent Observability verified against live `datadoghq.com/pricing` + docs). Inherited matrix cells marked 🟡. Legacy matrix/feature-matrix notes left as sources with pointers. | _pending_ |
| 2 | 2026-07-17 | **SigNoz** deep-dive ([parallax-vs-signoz.md](parallax-vs-signoz.md)): version drift re-verified (v0.132.2, ClickHouse 25.12.5), "open investigation format" confirmed still no published schema, pricing re-cited. Legacy signoz-deep-research.md left as lead w/ pointer. | _pending_ |
| 3 | 2026-07-17 | **Sentry** deep-dive ([parallax-vs-sentry.md](parallax-vs-sentry.md)): pricing re-verified against live sentry.io/pricing (Dev free / Team $26 / Business $80 / Enterprise custom; per-error overage tiers); OTLP **open-beta** (HTTP-only traces+logs, **no metrics, no gRPC**, self-hosted OTLP since ~v25.8.0 #3830); Seer confirmed **$40/active contributor/mo, unlimited usage** (BusinessWire 2026-01 + sentry.io/product/seer); self-host latest **26.4.2** (live GitHub releases; legacy 26.6.0 claim unresolved). Two peer-pass errors corrected: "OTLP GA"→open-beta, "self-host 25.x"→26.4.2. Operator directive folded in: "always compare to latest versions" (now a prompt section). | _pending_ |
| 4 | 2026-07-17 | **Langfuse** deep-dive ([parallax-vs-langfuse.md](parallax-vs-langfuse.md)): the archetypal OSS LLM/agent-obs platform — most direct AI-wedge competitor. Pricing verified (self-host MIT **free unlimited**; Cloud Hobby free / Pro $199/mo + $8/100k units / Enterprise $2499/mo; self-host EE ~$500/mo). OTLP backend at `/api/public/otel` confirmed. No-bias: Langfuse wins decisively on LLM-tracing/evals/prompts/datasets/community/MIT-free; Parallax edges (prod telemetry breadth, prod error+outcome loop, bounded agent bundle) all unproven (A1). Open: pin exact Langfuse release tag + self-host backing store. | _pending_ |
| 5 | 2026-07-17 | **Grafana Cloud/LGTM** deep-dive ([parallax-vs-grafana.md](parallax-vs-grafana.md)): largest OSS-origin full-stack competitor. Cloud pricing verified (Free / Pro $195/mo; metrics ~$6.50/1k series, logs/traces/profiles ~$0.45-0.50/GB; app-obs $0.04/host-hour); Tempo v3 cut from in-repo reference note. No-bias: Grafana wins decisively on breadth/dashboards(OSS standard)/OTLP-native-at-parity/ecosystem/scale/compliance; Parallax edges scoped to self-host simplicity (vs distributed Mimir+Loki+Tempo+Pyroscope), Apache vs AGPLv3, native error-workflow (Grafana has none), bundle thesis (unproven A1). | _pending_ |
| 6 | 2026-07-17 | **Honeycomb** deep-dive ([parallax-vs-honeycomb.md](parallax-vs-honeycomb.md)): defining high-cardinality wide-event platform. Pricing verified (Free 20M events/mo; Pro from $150/50M events; Enterprise custom; cardinality NOT priced separately). AI: stale "Bubbleuppy" codename corrected → **Query Assistant (NLQ) + Canvas + MCP**. No-bias: Honeycomb wins decisively on high-cardinality interactive exploration/event-model-maturity/NLQ-Canvas-MCP/SaaS-scale; Parallax edges scoped to self-host (Honeycomb store is SaaS-only), Apache vs closed, native error-workflow (Honeycomb has none), bundle thesis (unproven A1). High-cardinality query is the riskiest regime for GreptimeDB — benchmark-flagged. | _pending_ |
| 7 | 2026-07-17 | **Arize Phoenix** deep-dive ([parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md)): completes the AI-obs trio (Langfuse/LangSmith/Phoenix). Latest pinned **arize-phoenix-v18.1.0** (2026-07-17). **License corrected: ELv2, not Apache** (self-host free/unlimited, not OSI-open, managed-service restriction — less permissive than Langfuse MIT / Parallax Apache-2.0). OTLP-native + OpenInference (Arize drives the AI-span semantic standard) confirmed. Pricing (self-host free; Cloud Free/Core $29/Pro $199; AX from $50/mo). No-bias: Phoenix wins on LLM-tracing/evals/OpenInference/self-host-free; Parallax edges (prod telemetry breadth, error+outcome loop, bounded bundle) all unproven (A1). Open: self-host backing store. | _pending_ |
| 8 | 2026-07-17 | **New Relic** deep-dive ([parallax-vs-new-relic.md](parallax-vs-new-relic.md)): last big closed-source incumbent without a deep-dive. Pricing verified (100GB free + **$0.40/GB Original / $0.60 Data Plus**, $49/user). **OTLP-native ingest GA since 2021** (traces; metrics+logs after). **AI is the headline: AI Agent Platform (Feb 2026) + AI Coding Observability (June 2026: Claude Code/Cursor/Copilot/Windsurf/Amazon Q) + AIM + MCP** — shipped, direct overlap w/ Parallax's agent wedge, New Relic ahead today. No-bias: New Relic wins on breadth/entity-model/OTLP-maturity/AI/scale/compliance; Parallax edges scoped to self-host/data-ownership (New Relic SaaS-only), Apache vs closed, bundle+outcome thesis (unproven A1). | _pending_ |
| 9 | 2026-07-17 | **OpenObserve** deep-dive ([parallax-vs-openobserve.md](parallax-vs-openobserve.md)): nearest Rust single-binary OSS competitor — overlaps Parallax's OWN axes (Rust/single-binary/self-host/OTLP-native/Parquet-DataFusion). Latest v0.91.1, ~20k★; AGPL-3.0; pricing (OSS free unlimited; self-host Enterprise free ≤50GB/day w/ SSO/RBAC/audit; Cloud usage-based). **Hardest no-bias test**: written plainly that OpenObserve has SHIPPED the Rust-single-binary-self-host-OTLP architecture Parallax is building — OpenObserve is ahead on Parallax's own architectural claim. Parallax wedge narrows to Apache-vs-AGPL, read-only-safe+free-redaction agent posture (vs Enterprise-gated write MCP), Sentry-envelope, prod-error+outcome loop, GreptimeDB choice (unproven), bounded bundle (A1 unproven). | _pending_ |

## Deep-dive status (per product)

| Product | Deep-dive file | State | Last verified | Next gap |
| --- | --- | --- | --- | --- |
| Datadog | [parallax-vs-datadog.md](parallax-vs-datadog.md) | ✅ pass 1 | 2026-07-17 | self-host/agentless reality; gov/FedRAMP posture; exact 2026 OTLP-in-Agent GA scope |
| Sentry | [parallax-vs-sentry.md](parallax-vs-sentry.md) | ✅ pass 3 | 2026-07-17 | track OTLP-metrics GA; A1-vs-Seer measurement; self-host cost/ops benchmark |
| Grafana Cloud/LGTM | [parallax-vs-grafana.md](parallax-vs-grafana.md) | ✅ pass 5 | 2026-07-17 | **versions pinned pass 5b**: Grafana v13.1.0 / Mimir mimir-3.1.3 / Loki v3.7.3 / Tempo v2.10.7 / Pyroscope v2.1.1; corrected "Grafana 12.x"→13.1.0 + "Tempo v3 GA"→v3-not-yet-GA; open: A1-vs-Grafana measurement; self-host cost/ops benchmark |
| Honeycomb | [parallax-vs-honeycomb.md](parallax-vs-honeycomb.md) | ✅ pass 6 | 2026-07-17 | Pro exact unit ($150/50M vs $130/100M); A1-vs-Honeycomb measurement; high-cardinality query parity benchmark (riskiest regime for GreptimeDB) |
| New Relic | [parallax-vs-new-relic.md](parallax-vs-new-relic.md) | ✅ pass 8 | 2026-07-17 | pricing verified (100GB free + $0.40/GB Original / $0.60 Data Plus, ~$49/user); OTLP-native GA since 2021; **AI Coding Obs (June 2026: Claude Code/Cursor/Copilot/Windsurf/Q) = direct overlap w/ Parallax agent wedge — New Relic ahead**; open: NRAI-vs-bundle A1 measurement; self-host never (SaaS-only) |
| SigNoz | [parallax-vs-signoz.md](parallax-vs-signoz.md) | ✅ pass 2 | 2026-07-17 | outstanding: exact current star count + MCP server version (v0.5.1 last confirmed 2026-06-17); current trace/metric throughput (no public number) |
| OpenObserve | [parallax-vs-openobserve.md](parallax-vs-openobserve.md) | ✅ pass 9 | 2026-07-17 | A1-vs-OpenObserve-AI-SRE measurement; GreptimeDB-vs-Parquet/DataFusion cost+perf benchmark (ties to greptimedb-vs-clickhouse study); track read-only MCP mode + license posture |
| Coroot | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-coroot.md`; eBPF partial-span limit + MCP RBAC |
| Maple | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-maple.md`; Tinybird coupling + local UX |
| TMA1 | — | 🟡 inherited | 2026-06 (legacy) | write `parallax-vs-tma1.md`; bundle artifact drift check |
| Highlight.io | — | 🔴 missing | n/a | write deep-dive; closest SaaS session-replay+errors peer |
| Langfuse | [parallax-vs-langfuse.md](parallax-vs-langfuse.md) | ✅ pass 4 | 2026-07-17 | **latest pinned: v3.219.0 (2026-07-17)** + SDK-v4 OTLP-native/MCP-tracing + self-host-SSO-free added pass 4b; open: self-host backing store (Postgres/ClickHouse); A1-vs-Langfuse measurement; Langfuse changelog (production-error extension risk) |
| Arize Phoenix | [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md) | ✅ pass 7 | 2026-07-17 | **latest pinned: arize-phoenix-v18.1.0 (2026-07-17)**; license corrected Apache→**ELv2**; open: self-host backing store; A1-vs-Phoenix measurement; watch triggers in [parallax-vs-arize-phoenix.md](parallax-vs-arize-phoenix.md) |
| LangSmith | — | 🔴 missing | n/a | write deep-dive (AI wedge) |
| Dynatrace / Splunk Obs / Elastic Obs / Sumo / Chronosphere / Observe / Axiom | — | 🔴 missing | n/a | tier 2: one per later pass |
| Uptrace / HyperDX / Odigos | — | 🔴 watch | n/a | brief coverage; verify relevance first |
| PostHog / Helicone / Braintrust | — | 🔴 watch | n/a | brief coverage; AI wedge |

## Overview-matrix cells (from `README.md`)

The wide feature-presence matrix is bootstrapped from legacy notes. **Most cells
are 🟡 inherited** (sources dated 2026-05/06). Priority for re-verification:

1. Every Parallax-column cell — these are most bias-prone (self-assessed). Re-check against shipped code/spec, mark clearly whether the cell reflects *shipped* vs *planned*. (🔴 priority)
2. The AI-native/agent-context column across all competitors — fastest-moving axis, stale within weeks. (🔴 priority)
3. Pricing column — verify each against the live pricing page; mark "no public number" where absent. (🟡→✅ per product)
4. OTLP-native column — was true/false at 2026-06; several products shipped OTLP GA since. (🟡→✅ per product)

## Open uncertainties / questions

- **Datadog self-host posture**: marketing says SaaS-only; clarify whether any production self-host path beyond the Agent + Observability Pipelines Worker exists in 2026. Flagged in Datadog deep-dive.
- **Datadog OTLP-in-Agent GA scope**: confirm which signals (logs/metrics/traces) the Agent OTLP receiver ingests natively vs routes through transform. Flagged.
- **Parallax-column honesty**: several Parallax cells (fix-outcome loop, redacted bundle, MCP) are *planned*, not shipped. Keep the planned/shipped split explicit in every row — do not let "planned" read as "has it".

## Next highest-value gaps (ranked)

1. **Coroot deep-dive** — nearest eBPF/RCA OSS; best MCP RBAC safety model (legacy note aging). Direct overlap with Parallax's safe-agent-projection thesis.
2. **Maple deep-dive** — nearest local-UX OSS; same Turso metadata choice as Parallax.
3. **TMA1 deep-dive** — nearest architectural mirror (embedded GreptimeDB + read-only MCP); #1 watch target.
4. **LangSmith deep-dive** — AI-wedge trio completion (Langfuse + Phoenix done); closed LLM tracing/eval.
5. **Highlight.io deep-dive** — closest SaaS session-replay+errors peer.
6. **SigNoz cell re-verification** — exact current star count + MCP server version; trace/metric throughput (no public number).
7. **Drift watch (Phoenix/Honeycomb/Sentry/NewRelic/OpenObserve)** — version + AI-extension risk; A1-vs-shipped-AI measurements.

## Bias audit (this pass)

- ✅ Datadog deep-dive defaults to "Datadog may be better"; written that Parallax is behind on breadth/maturity/scale/enterprise/AI, with evidence.
- ✅ Parallax's only edges (open-source/self-host, cost predictability, data ownership, bundle thesis) are scoped to named axes; bundle thesis flagged unproven (A1 gate).
- ⚠️ The inherited overview matrix is Parallax-framed from legacy notes (columns = "Parallax's wedge"). Acceptable as bootstrap; must re-verify cells product-by-product and reframe verdicts as no-bias in later passes.
- ✅ Sentry deep-dive (pass 3) defaults to "Sentry may be better"; written plainly that Sentry wins on error-workflow/SDKs/maturity/AI(Seer)/profiling/replay/compliance, and Parallax's edges (OTLP-native incl. metrics, single-binary self-host simplicity, Apache vs FSL, bundle/outcome) are scoped + the bundle/outcome thesis flagged unproven (A1).
- ✅ SigNoz deep-dive (pass 2) defaults to "SigNoz may be better"; written plainly that SigNoz wins on maturity/breadth/MCP/scale/pricing-transparency, and that Parallax's bundle/outcome/Sentry edges are planned or unproven (A1 gate), not parity.
- ✅ Langfuse deep-dive (pass 4) defaults to "Langfuse may be better"; written plainly that Langfuse wins decisively on LLM-tracing/evals/prompts/datasets/community/MIT-free-self-host, and that Parallax's edges (prod telemetry breadth, prod error+outcome loop, bounded agent bundle) are scoped + unproven (A1) — with the honest note that the two serve different loops (LLMOps dev loop vs prod-incident evidence).
- ✅ Grafana deep-dive (pass 5) defaults to "Grafana may be better"; written plainly that Grafana wins decisively on OSS-stack breadth/dashboards(market standard)/OTLP-native-at-parity/ecosystem/scale/compliance, and that Parallax's edges (single-binary self-host simplicity vs distributed Mimir+Loki+Tempo+Pyroscope, Apache vs AGPLv3, native error-workflow Grafana lacks, bundle thesis) are scoped + unproven (A1).
- ✅ Honeycomb deep-dive (pass 6) defaults to "Honeycomb may be better"; written plainly that Honeycomb wins decisively on high-cardinality interactive exploration, event-model maturity, NLQ/Canvas/MCP AI, SaaS scale; Parallax edges (self-host vs SaaS-only, Apache vs closed, native error-workflow Honeycomb lacks, bundle thesis) scoped + unproven (A1). Stale "Bubbleuppy AI" codename corrected → Query Assistant/Canvas/MCP.
- ✅ OpenObserve deep-dive (pass 9) — **the hardest no-bias test**: written plainly that OpenObserve has SHIPPED Parallax's own Rust-single-binary-self-host-OTLP-native-Parquet architecture (v0.91.1, ~20k★, ≥512MB RAM) and is therefore AHEAD on Parallax's own architectural claim; Parallax wedge narrowed honestly to Apache-vs-AGPL, read-only-safe+free-redaction agent posture vs Enterprise-gated-write MCP, Sentry-envelope, prod-error+outcome loop, GreptimeDB choice (unproven), bounded bundle (A1 unproven). No Parallax-favoritism on the shared axes.
