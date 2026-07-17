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

- **Parallax** — open-source (Apache-2.0), Rust-first, self-hosted **execution-context engine**: ingests OTLP traces/logs/metrics + CLI/agent execution traces, derives owned `error_event`s, fingerprints, correlates into a typed evidence graph, and serves bounded, redacted, schema-valid **evidence bundles** to humans and coding agents (CLI/HTTP first, read-only MCP after safety gates). Storage: GreptimeDB (telemetry native OTLP tables) + Turso (metadata). Pre-release. **It is the reference design the rest are measured against, not the assumed winner.**

## B. Closed-source / commercial observability platforms

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **Datadog** | Full-stack SaaS observability + security; broadest commercial surface (infra, APM, logs, RUM, profiling, LLM/agent obs, CI/test, incident, security). | Closed SaaS; OSS agent, proprietary backend. | All signals. | [deep-dive](parallax-vs-datadog.md) |
| **Sentry** | Error tracking + tracing + logs + metrics + profiling + replay + Seer AI; OTLP traces+logs (no OTLP metrics); best-in-class issue lifecycle. | Source-available FSL (→Apache/MIT @2yr). | Errors + perf + replay. | [deep-dive](parallax-vs-sentry.md) |
| **Grafana Cloud / LGTM** | Managed stack on Prometheus/Mimir, Loki, Tempo, Pyroscope, Grafana; OTLP-native. | Mixed OSS + Cloud SaaS (Grafana Labs). | Metrics + logs + traces + profiles. | [deep-dive](parallax-vs-grafana.md) |
| **Honeycomb** | High-cardinality event-pipeline observability; exploratory query; Query Assistant/Canvas AI + MCP. | Closed SaaS (Refinery OSS). | Events / traces (high cardinality). | [deep-dive](parallax-vs-honeycomb.md) |
| **New Relic** | Full-platform SaaS; entity-centric; AI (NRAI + **AI Coding Obs** for Claude Code/Cursor/Copilot/Windsurf/Q); OTLP-native. | Closed SaaS (no self-host). | All signals. | [deep-dive](parallax-vs-new-relic.md) |
| **Dynatrace** | AI-driven (Davis) full-stack; deep auto-instrumentation via OneAgent. | Closed SaaS. | All signals + topology. | [deep-dive](parallax-vs-dynatrace.md) |
| **Splunk Observability Cloud** | Observability on top of Splunk (post-Cisco); OTel-native metrics/traces + logs. | Closed SaaS. | Logs + metrics + traces. | [deep-dive](parallax-vs-splunk.md) |
| **Elastic Observability** | ES/Kibana stack (search + observability + security); ES|QL. | Elastic License v2 (source-available). | Logs + metrics + traces + security. | [deep-dive](parallax-vs-elastic.md) |
| **Sumo Logic** | Cloud log/SIEM/observability SaaS; Flex scan-pricing; Francisco Partners-owned. | Closed SaaS. | Logs + metrics + security. | [deep-dive](parallax-vs-sumo.md) |
| **Chronosphere** | Scale metrics platform on M3/Cube; controlled-cost metrics + Telemetry Pipeline. | Closed SaaS. | Metrics (high scale). | [deep-dive](parallax-vs-chronosphere.md) |
| **Observe** | Data-/SQL-centric observability on Snowflake (acquired ~$1B Jan 2026); O11y Knowledge Graph + AI SRE/o11y.ai agents. | Closed SaaS (Snowflake). | All signals (relational). | [deep-dive](parallax-vs-observe.md) |
| **Axiom** | Serverless log/event analytics; 3-part usage pricing (ingest+query+storage); OTel-native. | Closed SaaS (OSS SDKs). | Logs + events. | [deep-dive](parallax-vs-axiom.md) |
| **Mezmo** | Log/data pipeline + telemetry routing. | Closed SaaS (ex-LogDNA). | Logs + pipelines. | watch |
| **Tracelo** | AI-agent tracing / debugging. | Closed. | Agent/LLM traces. | watch |

## C. Open-source / self-hosted observability platforms

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **SigNoz** | OTLP-native full obs on ClickHouse; most mature MCP. | MIT-Expat core + proprietary `ee/`. | All signals. | [deep-dive](parallax-vs-signoz.md) |
| **OpenObserve** | Rust single-binary, object-storage-native (Parquet/DataFusion); AI SRE + 140+ MCP. | AGPL-3.0 + commercial EE. | All signals. | [deep-dive](parallax-vs-openobserve.md) |
| **Coroot** | eBPF zero-instrumentation obs + 2-stage AI RCA; safest MCP (OAuth+RBAC). | Apache-2.0 + commercial EE. | Traces/logs/profiles (eBPF). | [deep-dive](parallax-vs-coroot.md) |
| **Highlight.io** | Session replay + error tracking + logs + traces; OTLP-native; Apache-2.0 OSS self-host. | Apache-2.0 (self-host) + Cloud. | Errors + RUM + logs. | [deep-dive](parallax-vs-highlight.md) |
| **Uptrace** | OTLP on ClickHouse/Postgres; tracing-first. | Open core (BSL-adjacent). | Traces + metrics. | watch |
| **HyperDX** | OTLP on ClickHouse; single-pane logs/metrics/traces. | Apache-2.0 + Cloud. | All signals. | watch |
| **Odigos** | eBPF auto-instrumentation to OTLP (collector, not a backend). | Apache-2.0. | Auto-instrumentation. | watch |
| **Maple** | OTLP single-binary best local UX; Turso metadata sibling choice. | FSL-1.1 (TS/Bun). | All signals. | [deep-dive](parallax-vs-maple.md) |
| **TMA1** | Nearest architectural mirror: Go single binary + embedded GreptimeDB + read-only MCP context-bundle for coding agents. | Apache-2.0. | AI-agent cost/sessions/traces. | [deep-dive](parallax-vs-tma1.md) |

Component-level (the "stack it yourself" pieces, referenced not deep-dived):

- **Prometheus / Grafana Mimir** — metrics. **Loki** — logs. **Tempo / Jaeger** — traces. **Pyroscope / Parca** — profiling. **Vector / Fluent Bit** — collection/pipelines.

## D. AI / LLM-agent observability (direct relevance to Parallax's wedge)

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **Langfuse** | OSS LLM/agent tracing + evals + prompt mgmt (self-host or cloud). | MIT (self-host) + Cloud. | LLM/agent traces + evals. | [deep-dive](parallax-vs-langfuse.md) |
| **LangSmith** | LangChain's closed tracing/eval/prompt platform. | Closed SaaS. | LLM/agent traces + evals. | [deep-dive](parallax-vs-langsmith.md) |
| **Arize Phoenix** | OSS LLM/agent tracing + evals (drives **OpenInference**); OTLP-native. | **ELv2** (self-host free + unlimited, but not OSI-open; managed-service restriction). | LLM/agent traces + evals. | [deep-dive](parallax-vs-arize-phoenix.md) |
| **PostHog** | OSS product analytics + session replay + feature flags/experiments + (now) LLM/agent tracing. | OSS (own license) + Cloud. | Product analytics + LLM. | [deep-dive](parallax-vs-posthog.md) |
| **Helicone** | LLM gateway/proxy + observability; caching + cost analytics; zero LLM-cost markup. | MIT + Cloud. | LLM proxies + traces. | [deep-dive](parallax-vs-helicone.md) |
| **Braintrust** | Eval-first LLM eval/experiment platform (datasets/scorers/playground); OSS SDK + closed core. | OSS SDK + SaaS. | LLM evals + experiments. | [deep-dive](parallax-vs-braintrust.md) |

## Maintenance notes

- When a product enters **deep-dive**, move its row's State to `[deep-dive](parallax-vs-<product>.md)` and flip the PROGRESS row.
- When a product is retired from scope, delete the row and note why in `PROGRESS.md` (acquired, dead, out-of-scope).
- Stars, versions, and funding figures are **snapshots** — never treat them as current; re-verify on the deep-dive pass and date them there.
