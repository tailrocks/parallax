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
| **Grafana Cloud / LGTM** | Managed stack on Prometheus/Mimir, Loki, Tempo, Pyroscope, Grafana; OTLP-native. | Mixed OSS + Cloud SaaS (Grafana Labs). | Metrics + logs + traces + profiles. | stub |
| **Honeycomb** | High-cardinality event-pipeline observability; exploratory query; Bubbleuppy AI. | Closed SaaS (source-available pieces). | Events / traces (high cardinality). | stub |
| **New Relic** | Full-platform SaaS; entity-centric; AI (NRAI). | Closed SaaS. | All signals. | stub |
| **Dynatrace** | AI-driven (Davis) full-stack; deep auto-instrumentation via OneAgent. | Closed SaaS. | All signals + topology. | stub |
| **Splunk Observability Cloud** | Observability on top of Splunk (post-Cisco); OTel-native metrics/traces + logs. | Closed SaaS. | Logs + metrics + traces. | stub |
| **Elastic Observability** | ES/Kibana stack (search + observability + security); ES|QL. | Elastic License v2 (source-available). | Logs + metrics + traces + security. | stub |
| **Sumo Logic** | Cloud log/SIEM/observability SaaS. | Closed SaaS. | Logs + metrics + security. | stub |
| **Chronosphere** | Scale metrics platform on M3/Cube; controlled-cost metrics. | Closed SaaS. | Metrics (high scale). | stub |
| **Observe** | Data-/SQL-centric observability on Snowflake; relationship graph. | Closed SaaS. | All signals (relational). | stub |
| **Axiom** | Serverless log/event analytics; cheap ingest. | Closed SaaS (OSS SDKs). | Logs + events. | stub |
| **Mezmo** | Log/data pipeline + telemetry routing. | Closed SaaS (ex-LogDNA). | Logs + pipelines. | watch |
| **Tracelo** | AI-agent tracing / debugging. | Closed. | Agent/LLM traces. | watch |

## C. Open-source / self-hosted observability platforms

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **SigNoz** | OTLP-native full obs on ClickHouse; most mature MCP. | MIT-Expat core + proprietary `ee/`. | All signals. | [deep-dive](parallax-vs-signoz.md) |
| **OpenObserve** | Rust single-binary, object-storage-native (Parquet/DataFusion); AI SRE + 140+ MCP. | AGPL-3.0 + commercial EE. | All signals. | stub (legacy [openobserve-deep-research.md](../openobserve-deep-research.md)) |
| **Coroot** | eBPF zero-instrumentation obs + 2-stage AI RCA; safest MCP (OAuth+RBAC). | Apache-2.0 + commercial EE. | Traces/logs/profiles (eBPF). | stub (legacy [coroot-deep-research.md](../coroot-deep-research.md)) |
| **Highlight.io** | Session replay + error tracking + logs + traces; OSS SaaS. | Apache-2.0 (self-host) + Cloud. | Errors + RUM + logs. | stub |
| **Uptrace** | OTLP on ClickHouse/Postgres; tracing-first. | Open core (BSL-adjacent). | Traces + metrics. | watch |
| **HyperDX** | OTLP on ClickHouse; single-pane logs/metrics/traces. | Apache-2.0 + Cloud. | All signals. | watch |
| **Odigos** | eBPF auto-instrumentation to OTLP (collector, not a backend). | Apache-2.0. | Auto-instrumentation. | watch |
| **Maple** | OTLP single-binary best local UX; Turso metadata sibling choice. | FSL-1.1 (TS/Bun). | All signals. | stub (legacy [maple-deep-research.md](../maple-deep-research.md)) |
| **TMA1** | Nearest architectural mirror: Go single binary + embedded GreptimeDB + read-only MCP context-bundle for coding agents. | Apache-2.0. | AI-agent cost/sessions/traces. | stub (legacy [tma1-deep-research.md](../tma1-deep-research.md)) |

Component-level (the "stack it yourself" pieces, referenced not deep-dived):

- **Prometheus / Grafana Mimir** — metrics. **Loki** — logs. **Tempo / Jaeger** — traces. **Pyroscope / Parca** — profiling. **Vector / Fluent Bit** — collection/pipelines.

## D. AI / LLM-agent observability (direct relevance to Parallax's wedge)

| Product | What it is | License / model | Primary signal focus | State |
| --- | --- | --- | --- | --- |
| **Langfuse** | OSS LLM/agent tracing + evals + prompt mgmt (self-host or cloud). | MIT (self-host) + Cloud. | LLM/agent traces + evals. | stub |
| **LangSmith** | LangChain's closed tracing/eval/prompt platform. | Closed SaaS. | LLM/agent traces + evals. | watch |
| **Arize Phoenix** | OSS LLM/agent tracing + evals. | Apache-2.0 (ELv2 parts). | LLM/agent traces + evals. | stub |
| **PostHog** | OSS product analytics + session replay + (now) LLM/agent tracing. | OSS + Cloud. | Product analytics + LLM. | watch |
| **Helicone** | LLM gateway/proxy + observability. | MIT + Cloud. | LLM proxies + traces. | watch |
| **Braintrust** | Eval/experiment platform (LLM). | Open core + Cloud. | LLM evals + experiments. | watch |

## Maintenance notes

- When a product enters **deep-dive**, move its row's State to `[deep-dive](parallax-vs-<product>.md)` and flip the PROGRESS row.
- When a product is retired from scope, delete the row and note why in `PROGRESS.md` (acquired, dead, out-of-scope).
- Stars, versions, and funding figures are **snapshots** — never treat them as current; re-verify on the deep-dive pass and date them there.
