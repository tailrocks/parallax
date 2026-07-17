# Comparison Set — What Is In Scope

> The authoritative roster of products compared in this folder. Each entry is a
> one-line definition: what it is, license model, primary signal focus. Kept
> current as the market shifts — products are added, merged, or retired on every
> pass. Verify each still exists and still matters before relying on a row.
>
> Last reviewed: 2026-07-17.

Legend for the **State** column:

- **deep-dive** — a `parallax-vs-<product>.md` exists and is verified.
- **stub / stale** — product is in scope but the deep-dive is missing or aged.
- **watch** — tracked for drift but not yet a priority deep-dive.

## A. Parallax (the reference, not a competitor)

- **Parallax** — open-source (Apache-2.0), Rust-first, self-hosted **execution-context engine**: ingests OTLP traces/logs/metrics + CLI/agent execution traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, and serves bounded, redacted, schema-valid **evidence bundles** to humans and coding agents (CLI/HTTP first, read-only local-stdio MCP graduated (plan 112 DONE; remote deferred)). Storage: GreptimeDB (telemetry native OTLP tables) + Turso (metadata). Pre-release. **It is the reference design the rest are measured against, not the assumed winner.**

## B. Closed-source / commercial observability platforms

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **Datadog** | Full-stack SaaS observability + security; broadest commercial surface (infra, APM, logs, RUM, profiling, LLM/agent obs, CI/test, incident, security). | Closed SaaS; OSS agent, proprietary backend. | All signals. | [deep-dive](parallax-vs-datadog.md) |
| **Sentry** | Error tracking + tracing + logs + metrics + profiling + replay + Seer AI; OTLP traces+logs (no OTLP metrics); best-in-class issue lifecycle. | Source-available FSL (→Apache/MIT @2yr). | Errors + perf + replay. | [deep-dive](parallax-vs-sentry.md) |
| **Grafana Cloud / LGTM** | Managed Mimir/Loki/Tempo/Pyroscope/Grafana; OTLP-native. Cloud Free / Pro **$19+usage** / Enterprise from **$25k/yr**. | Mixed OSS + Cloud SaaS (Grafana Labs). | Metrics + logs + traces + profiles + AI Assistant. | [deep-dive](parallax-vs-grafana.md) |
| **Honeycomb** | High-cardinality event-pipeline observability; exploratory query; **Agent Observability (2026-05-12): Agent Timeline + autonomous Auto-investigations + Canvas-agent + GenAI semconv** (ships agent-obs AND autonomous RCA). | Closed SaaS (Refinery OSS). | Events / traces (high cardinality) + agent. | [deep-dive](parallax-vs-honeycomb.md) |
| **New Relic** | Full-platform SaaS; entity-centric; AI (NRAI + **AI Coding Obs** for Claude Code/Cursor/Copilot/Windsurf/Q); OTLP-native. | Closed SaaS (no self-host). | All signals. | [deep-dive](parallax-vs-new-relic.md) |
| **Dynatrace** | AI-driven (Davis) full-stack; deep auto-instrumentation via OneAgent; **Perform-2026 agentic-operations platform: Dynatrace Intelligence + Smartscape truth-graph + Intelligence Agents + MCP Server = "bounded agent context" (DIRECT collision with Parallax's thesis).** | Closed SaaS. | All signals + topology + agent-context. | [deep-dive](parallax-vs-dynatrace.md) |
| **Splunk Observability Cloud** | Observability on top of Splunk (post-Cisco); OTel-native metrics/traces + logs + NoSample; **AI Agent Monitoring (OTel + Cisco AGNTCY) + Agentic Observability + Cisco AI Defense** (2026). | Closed SaaS. | Logs + metrics + traces + AI/agent. | [deep-dive](parallax-vs-splunk.md) |
| **Elastic Observability** | ES/Kibana stack (search + observability + security); ES|QL. | Elastic License v2 (source-available). | Logs + metrics + traces + security. | [deep-dive](parallax-vs-elastic.md) |
| **Sumo Logic** | Cloud log/SIEM/observability SaaS; Flex ($0 ingest, pay scan+storage credits); **Dojo AI** (Mobot + agents); Francisco Partners-owned. | Closed SaaS. | Logs + metrics + security + AI agents. | [deep-dive](parallax-vs-sumo.md) |
| **Chronosphere** | Scale metrics on M3/Cube + Control Plane + Telemetry Pipeline; Gartner #1 cost control. **Palo Alto Networks–owned (acq. closed 2026-01-29, ~$3.35B).** Quote-based retained-data pricing (no public rate card). | Closed SaaS (PANW). | Metrics (high scale) + pipeline. | [deep-dive](parallax-vs-chronosphere.md) |
| **Observe** | Data-/SQL-centric observability on Snowflake (acquired ~$1B Jan 2026); O11y Knowledge Graph + AI SRE/o11y.ai agents. | Closed SaaS (Snowflake). | All signals (relational). | [deep-dive](parallax-vs-observe.md) |
| **Axiom** | Serverless **full-stack** observability (logs/traces/**metrics GA**/events) **+ AI Engineering** (agent-workflow tracing, evals, cost/latency); OTel-native; **4-part usage pricing** ($25 platform + data-loading + query + storage + add-ons; perpetual 1 TB Always-Free; no egress/seat). | Closed SaaS (OSS SDKs). | Full signals + AI/agent (was logs+events). | [deep-dive](parallax-vs-axiom.md) |
| **Mezmo** | Telemetry data pipeline + log analysis (ex-LogDNA); Mezmo Flow; route/optimize/govern in flight. | Closed SaaS. | Logs + pipelines (cost-governance layer). | [deep-dive](parallax-vs-mezmo.md) |

> **Roster correction (pass 31):** the legacy "**Tracelo**" row was removed —
> `tracelo.com` is a phone-geolocation service, not an observability product. The
> intended LLM-instrumentation tool is **Traceloop / OpenLLMetry**, deep-dived in
> [section C](parallax-vs-traceloop.md) as the LLM-instrumentation sibling of Odigos.

## C. Open-source / self-hosted observability platforms

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **SigNoz** | OTLP-native full obs on ClickHouse; most mature MCP. | MIT-Expat core + proprietary `ee/`. | All signals. | [deep-dive](parallax-vs-signoz.md) |
| **OpenObserve** | Rust single-binary, object-storage-native (Parquet/DataFusion); AI SRE + 140+ MCP. | AGPL-3.0 + commercial EE. | All signals. | [deep-dive](parallax-vs-openobserve.md) |
| **Coroot** | eBPF zero-instrumentation obs + 2-stage AI RCA; safest MCP (OAuth+RBAC). | Apache-2.0 + commercial EE. | Traces/logs/profiles (eBPF). | [deep-dive](parallax-vs-coroot.md) |
| **Highlight.io** | Session replay + error tracking + logs + traces; OTLP-native; Apache-2.0 OSS self-host. **🛑 Wound down (pass 33): acquired by LaunchDarkly; standalone SaaS shut down 2026-02-28; OSS repo unmaintained (no release since 2025-08).** Historical/reference only. | Apache-2.0 (self-host) + Cloud. | Errors + RUM + logs. | [deep-dive](parallax-vs-highlight.md) |
| **Bugsink** | Focused **self-hosted Sentry-SDK-compatible error-tracking server** (Python/Django; full issue lifecycle); **1,940★, v2.4.0**; Hosted public EUR event tiers (free 15K → €1,288/50M); self-host free. Error-only (no OTLP). Cleanest "run your own Sentry." | Open-core (`ee/` + BSD-3 `sentry/`; `NOASSERTION`) + Cloud. | Errors only (Sentry-alternative). | [deep-dive](parallax-vs-bugsink.md) |
| **Uptrace** | OTLP tracing-first APM on ClickHouse+Postgres; Bun-author lineage. | **AGPL** (Community free) + paid editions + Cloud. | Traces + metrics + logs. | [deep-dive](parallax-vs-uptrace.md) |
| **HyperDX** | OTLP + multi-protocol on **ClickHouse**; full-stack incl. **session replay**; = ClickHouse Inc.'s **ClickStack**. Cloud: Free 3GB / Starter **$20 + $0.40/GB**. | **MIT** + Cloud + Managed ClickStack. | All signals + RUM/replay. | [deep-dive](parallax-vs-hyperdx.md) |
| **Odigos** | eBPF + OTel auto-instrumentation control plane (→ any backend); marketing **“Ask Production Anything” / AI SRE**; GenAI auto-instrument. OSS free; Enterprise trial then custom (**no public $/unit**). **v1.31.2, ~3.7k★.** | Apache-2.0 + Enterprise. | Instrumentation layer (complementary). | [deep-dive](parallax-vs-odigos.md) |
| **Traceloop** (OpenLLMetry) | OSS Apache-2.0 OTel **LLM-instrumentation SDK** (auto-instrument providers/frameworks/vector-DBs/MCP → OTLP GenAI spans to any backend); drove GenAI semantic conventions into upstream OTel; **ServiceNow-acquired (~$60–80M) → AI Control Tower** (OSS project stays Apache-2.0, active v0.62.1). The LLM-instrumentation sibling of Odigos. | Apache-2.0 + Cloud (now ServiceNow). | LLM instrumentation layer (complementary). | [deep-dive](parallax-vs-traceloop.md) |
| **Maple** | OTLP single-binary best local UX; Turso metadata sibling choice. | FSL-1.1 (TS/Bun). | All signals. | [deep-dive](parallax-vs-maple.md) |
| **TMA1** | Nearest architectural mirror: Go single binary + embedded GreptimeDB + read-only MCP context-bundle for coding agents. | Apache-2.0. | AI-agent cost/sessions/traces. | [deep-dive](parallax-vs-tma1.md) |
| **Traceway** | MIT OTel-native full-stack self-host (logs/traces/metrics/exceptions/RUM/AI traces); **agent-first CLI + skills + local/remote MCP** (mostly read-only). ClickHouse+Postgres or SQLite/DuckDB. **No Sentry; no portable redacted bundle.** ~1k★, backend v1.9.1. | MIT + self-host (+ cloud TBD). | All signals + agent investigation. | [deep-dive](parallax-vs-traceway.md) |

Component-level (the "stack it yourself" pieces, referenced not deep-dived):

- **Prometheus / Grafana Mimir** — metrics. **Loki** — logs. **Tempo / Jaeger** — traces. **Pyroscope / Parca** — profiling. **Vector / Fluent Bit** — collection/pipelines.

## D. AI / LLM-agent observability (direct relevance to Parallax's wedge)

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **Langfuse** | OSS LLM/agent tracing + evals + prompt mgmt (self-host or cloud). | MIT (self-host) + Cloud. | LLM/agent traces + evals. | [deep-dive](parallax-vs-langfuse.md) |
| **LangSmith** | LangChain closed tracing/eval/prompt platform + **Engine** (autonomous agent failure diagnosis/fix recs, LCU-metered) + Fleet/Sandboxes. LCU $1.50 / LSU $1.00. | Closed SaaS (+ Enterprise self-host). | LLM/agent traces + evals + agent Engine. | [deep-dive](parallax-vs-langsmith.md) |
| **Arize Phoenix** | OSS LLM/agent tracing + evals (drives **OpenInference**); OTLP-native. | **ELv2** (self-host free + unlimited, but not OSI-open; managed-service restriction). | LLM/agent traces + evals. | [deep-dive](parallax-vs-arize-phoenix.md) |
| **PostHog** | OSS product analytics + session replay + feature flags/experiments + LLM/agent tracing. **MIT Expat core + proprietary `ee/`** (~36k★). | MIT core + EE proprietary + Cloud. | Product analytics + LLM. | [deep-dive](parallax-vs-posthog.md) |
| **Helicone** | LLM gateway/proxy + observability; caching + zero LLM-markup. **🛑 Acquired by Mintlify 2026-03-03 → Cloud maintenance mode**; OSS Apache-2.0 (~6k★). | Apache-2.0 + Cloud (Mintlify). | LLM proxies + traces (maintenance). | [deep-dive](parallax-vs-helicone.md) |
| **Braintrust** | Eval-first LLM eval/experiment platform (datasets/scorers/playground); OSS SDK + closed core. | OSS SDK + SaaS. | LLM evals + experiments. | [deep-dive](parallax-vs-braintrust.md) |

## E. AI investigation / causal-context layers (different layer — consume telemetry, not stores)

These sit *on top of* your telemetry (metrics/logs/traces/K8s) as reasoning/grounding layers for agents. They overlap Parallax's "context for production agents" thesis but **do not own the telemetry** — a different layer than the stores above. Referenced for completeness; only **Causely** is deep-dived (it is the clearest shipped "agent-context layer").

| Product | What it is | License / model | Layer | State |
| --- | --- | --- | --- | --- |
| **Causely** | Causal-intelligence layer + MCP server — "live causal model via MCP" so agents stop guessing / burn fewer tokens / act before break; BYO telemetry. | Closed commercial (verify self-host). | Causal-context MCP over BYO telemetry. | [deep-dive](parallax-vs-causely.md) |
| **HolmesGPT** (CNCF Sandbox) | Open AI SRE over Prometheus/Loki/Tempo/K8s; no own store; MCP-extensible. **v0.36.0, 2,873★.** **= the shipped "fixer agent"** Parallax's "context engine, not the fixer" framing positions against. | Apache-2.0 (+ Robusta commercial). | AI-investigation query layer (complementary). | [deep-dive](parallax-vs-holmesgpt.md) |

## Maintenance notes

- When a product enters **deep-dive**, move its row's State to `[deep-dive](parallax-vs-<product>.md)` and flip the PROGRESS row.
- When a product is retired from scope, delete the row and note why in `PROGRESS.md` (acquired, dead, out-of-scope).
- Stars, versions, and funding figures are **snapshots** — never treat them as current; re-verify on the deep-dive pass and date them there.
