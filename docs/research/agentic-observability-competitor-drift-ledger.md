# Agentic Observability Competitor Drift Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This ledger turns the ongoing "watch every run" market requirement into an
auditable drift artifact. It tracks whether direct competitors have closed any
part of the Parallax wedge:

```text
Sentry-compatible error migration
+ OTLP traces/logs/metrics
+ low-resource self-hosting
+ portable evidence bundles
+ deterministic evidence graph
+ read-only CLI/API/MCP context access
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fix outcome loop
```

Current status: **wedge under pressure, not closed**.

The material follow-up in this pass is that the MCP surface has shifted from
"present or absent" to "what kind of tool power does it expose?" Sentry now has
a purpose-built MCP server for coding agents; SigNoz, OpenObserve, and Coroot
all expose real agent MCP surfaces; and several of those catalogs include
create, update, delete, resolve, or administrative tools. That does not close
the Parallax wedge. It changes the safety comparison: Parallax's first agent
surface must be a bounded read-only evidence-bundle projection, not a broad
management MCP.

The central rule:

> MCP is table stakes. The differentiator must be the bounded, redacted,
> citable evidence bundle plus the execution/action/outcome graph.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer) and [Seer issue-fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | Seer uses Sentry issue context, tracing, logs, profiles, and code context; the issue-fix API can stop at root cause, solution, code changes, or open PR. | Sentry owns the production-error agent path for hosted Sentry users. |
| [Sentry MCP service](https://mcp.sentry.dev/) and [sentry-mcp repository](https://github.com/getsentry/sentry-mcp) | Sentry's MCP server is designed for human-in-the-loop coding agents, offers a remote hosted service, a Claude Code plugin/subagent path, and a stdio transport for self-hosted Sentry. AI-powered search tools require an OpenAI or Anthropic provider; self-hosted instances may need to disable unsupported Seer skills; the documented stdio scopes include project/team/event write scopes. | Sentry has an agent-facing MCP path in addition to Seer. MCP availability is not a moat; Parallax must differentiate on read-only, redacted, citable bundles and action/outcome audit. |
| [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/) | Self-hosted Sentry excludes Seer and other AI/ML features, requires at least 4 CPU cores, 16 GB RAM plus swap, and is a Docker Compose service graph with limited support. | Self-hosted AI exclusion remains a Parallax opening, while Sentry's operational footprint keeps the low-ops benchmark relevant. |
| [Datadog Bits AI SRE eval platform](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/) | Datadog evaluates agent investigations with world snapshots, representative labels, trajectory scoring, feedback-derived labels, and deliberately noisy simulated environments. | Datadog is industrializing the exact feedback/eval loop Parallax wants for bundle value and corpus moat claims. |
| [Grafana Assistant CLI](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/) and [self-hosted Grafana Assistant](https://grafana.com/docs/grafana/latest/administration/assistant/) | Grafana Assistant CLI is public preview; Grafana v13 supports Assistant on-premise by connecting self-hosted Grafana to a Grafana Cloud stack, but some investigation and memory features remain unavailable on-premise. | Grafana validates CLI/MCP agent surfaces but leaves air-gapped/local-first room. |
| [Grafana Assistant MCP docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/) | Grafana Assistant can connect MCP servers and skills, with explicit warnings that operators are responsible for MCP security, data access, and tool actions. | Read-only scoping, redaction, and audit are product requirements, not implementation details. |
| [OpenObserve SRE Agent setup](https://openobserve.ai/docs/administration/deployment/sre-agent-setup-guide/), [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/), and [OTLP docs](https://openobserve.ai/docs/ingestion/logs/otlp/) | The SRE Agent requires an OpenObserve Enterprise license and AI provider key; OpenObserve MCP is Enterprise-only; its MCP catalog includes natural-language queries plus broad create/update/delete/admin tools for alerts, dashboards, roles, streams, functions, KV, pipelines, users, and ingestion/search jobs. OpenObserve also supports OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces. | OpenObserve is still the closest Rust/object-storage threat, but its public MCP shape is broad and write-capable rather than Parallax's intended read-only evidence-bundle surface. |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/) and [SigNoz MCP server](https://signoz.io/docs/ai/signoz-mcp-server/) | SigNoz positions observability inside coding agents and supports hosted plus self-hosted MCP. Its current MCP tool list covers metrics, traces, logs, docs, alerts, dashboards, saved views, and notification channels, including create/update/delete tools for several resource types. | SigNoz directly attacks the "agent-native observability" story, but also proves that Parallax must distinguish query/management MCP from read-only evidence bundles. |
| [SigNoz Claude Code monitoring](https://signoz.io/docs/claude-code-monitoring/) | SigNoz documents Claude Code OpenTelemetry export with logs/metrics and prompt-level correlation fields. | Agent telemetry is no longer a distant niche; Parallax needs the richer action/outcome graph. |
| [Coroot 1.20.2 release](https://github.com/coroot/coroot/releases/tag/v1.20.2), [Coroot AI RCA](https://docs.coroot.com/ai/overview/), [Coroot editions](https://coroot.com/editions), [Coroot MCP](https://docs.coroot.com/mcp/overview/), and [Coroot eBPF tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/) | Coroot `1.20.2` added the MCP server. Coroot Community includes agentic-ready MCP, while Enterprise adds AI-powered RCA and agentic anomaly investigation; Community can connect to Coroot Cloud for 10 free RCA investigations/month. The MCP endpoint uses streamable HTTP, OAuth 2.0, and server-side authorization, exposes topology/alerts/incidents/traces/logs/metrics, includes Community `resolve_alerts`, and adds Enterprise `investigate_anomaly`; eBPF traces may not provide complete traces. | Coroot's agent surface is now a serious self-hosted baseline, but still lacks Parallax's Sentry migration, portable evidence-bundle/schema, coding-agent side-effect audit, and local open RCA in Community. |
| [Bugsink docs](https://www.bugsink.com/docs/) | Bugsink is self-hosted error tracking compatible with the Sentry SDK. | Low-ops Sentry compatibility is not unique. |
| [Rustrak repository](https://github.com/AbianS/rustrak) | Rustrak is Rust/Actix, Sentry SDK compatible, SQLite-by-default, small-footprint, and ships `@rustrak/mcp` for AI assistant integration. | Rust plus Sentry compatibility plus MCP is already a live lightweight competitor shape. |
| [Traceway repository](https://github.com/tracewayapp/traceway) | Traceway is MIT, OpenTelemetry-native, self-hosted, and combines logs, traces, metrics, exceptions, session replay/RUM, and AI observability with OTLP/HTTP ingest. | Traceway pressures the OTLP/frontend/replay part of Parallax's roadmap. |
| [GoSnag repository](https://github.com/darkspock/gosnag) | GoSnag is a self-hosted Sentry-compatible service with an MCP server exposing project, issue, alert, tag, ticket, and user management tools. | MCP over issue management is already present in small Sentry-compatible tools. |
| [Urgentry site](https://urgentry.com/) | Urgentry claims DSN-only migration from Sentry, Tiny mode, one-binary startup, traces/replay/profiling/logs, and benchmark comparisons against self-hosted Sentry; it is source-available under FSL. | Urgentry pressures the "simpler than self-hosted Sentry" claim even if it does not satisfy the open-source thesis. |

## Drift Levels

| Level | Meaning | Required response |
| --- | --- | --- |
| `not_checked` | No current source check exists. | Do not rely on the prior competitor conclusion. |
| `no_material_drift` | Current sources match the previous watchlist posture. | Keep watching; no product-position change. |
| `trigger_hit` | A competitor crossed a named watch trigger but still misses the full Parallax combination. | Update the relevant watchlist and narrow the differentiator. |
| `wedge_under_pressure` | A competitor now covers multiple Parallax ingredients or directly weakens public positioning. | Revisit roadmap priority and public wording. |
| `wedge_partially_closed` | A competitor covers a majority of the wedge, leaving only one or two defensible gaps. | Reopen the GO verdict and narrow the build target. |
| `wedge_closed` | A competitor covers Sentry migration, OTLP, low-ops self-hosting, bundles/schema, agent-safe context, action audit, and outcome loop. | Treat the current Parallax thesis as failed or requiring a new wedge. |
| `source_stale` | A watched source is older than the refresh rule or could not be rechecked. | Mark the row stale and recheck before citing. |

Current aggregate level: `wedge_under_pressure`.

## Current Drift Rows

| Competitor | Drift level | What changed or matters now | Remaining Parallax gap |
| --- | --- | --- | --- |
| Sentry Seer | `wedge_under_pressure` | Hosted Seer has root-cause, solution, code-change, and PR API paths. | Seer is excluded from self-hosted Sentry in the checked docs; no open portable evidence-bundle schema. |
| Sentry MCP | `wedge_under_pressure` | Sentry has an official MCP server for coding assistants, a hosted remote endpoint, a Claude Code plugin/subagent path, and a stdio/self-hosted path. | The surface is Sentry-data-centric and may need write scopes or external LLM providers for some tools; it does not provide Parallax's open, redacted bundle schema or coding-agent action/outcome graph. |
| Datadog Bits AI SRE | `wedge_under_pressure` | Datadog's eval platform has the feedback, label, world-snapshot, noise, and trajectory machinery Parallax wants. | Enterprise SaaS data gravity, not an open self-hosted context engine. |
| Grafana Assistant | `wedge_under_pressure` | CLI and MCP are real agent surfaces; on-prem works through Grafana Cloud. | Not fully local/air-gapped; broad Grafana assistant, not Parallax bundles/outcome graph. |
| OpenObserve | `wedge_under_pressure` | Rust, object-storage-oriented, OTLP, AI SRE, Enterprise MCP, and a large MCP tool catalog with destructive/admin operations. | No Sentry-envelope migration in checked docs; no open read-only bundle/action-audit contract. |
| SigNoz | `wedge_under_pressure` | Self-hosted MCP and agent-native observability target Claude Code, Codex, Cursor, and similar workflows; the MCP catalog includes query and management tools. | No Sentry envelope path or deterministic evidence bundle/outcome graph in checked docs; first-party MCP is not a Parallax-style bounded read-only bundle. |
| Coroot | `trigger_hit` | Source refresh confirms Community MCP, OAuth/server-side authorization, Community `resolve_alerts`, Enterprise `investigate_anomaly`, and Enterprise/Cloud-gated AI RCA. This is stronger than "MCP exists" because Coroot gives agents topology, incidents, traces, logs, metrics, and focused RCA. | eBPF traces can be incomplete; no Sentry migration, portable bundle, coding-agent action/outcome audit, or fully local/open AI RCA in Community. |
| Bugsink | `no_material_drift` | Sentry SDK compatible self-hosted error tracking remains a strong low-ops baseline. | Error-tracking focused; no OTLP evidence graph or agent action audit in checked docs. |
| Rustrak | `trigger_hit` | Rust, Sentry SDK compatibility, SQLite default, and MCP are all present. | Management-shaped MCP; no OTLP traces/logs/metrics, evidence bundles, or outcome loop in checked docs. |
| Traceway | `wedge_under_pressure` | OpenTelemetry-native logs/traces/metrics/session replay/exceptions/AI observability, MIT, self-hosted. | No Sentry-envelope migration or coding-agent side-effect/outcome graph in checked docs. |
| GoSnag | `trigger_hit` | Sentry-compatible issue tracker with MCP management tools. | Postgres-backed issue tool; no OTLP evidence graph or bounded read-only bundle contract in checked docs. |
| Urgentry | `wedge_under_pressure` | One-binary Tiny mode, DSN migration, Sentry-like traces/replay/profiles/logs, and same-host benchmark claims. | Source-available, not OSI open; no Parallax-style open schema or agent action audit in checked docs. |

## Counting Rules

- Do not use MCP as a moat claim. Treat MCP as an expected access surface.
- Classify competitor MCP by power: read-only evidence, query, management/write,
  RCA/agent, and code/fix. A write-capable MCP surface does not equal Parallax's
  intended first read-only bundle adapter.
- Do not claim "open self-hosted AI observability" as unique. SigNoz, Coroot,
  OpenObserve, Traceway, and lightweight Sentry-compatible tools all pressure
  that phrase from different directions.
- Do not claim "simpler than Sentry" without comparing against Bugsink, Rustrak,
  GoSnag, Traceway, Urgentry, and self-hosted Sentry on the same baseline.
- Do not claim "agent-native" unless the surface is read-only by default,
  redacted, citable, auditable, and safe for least-privilege operation.
- Do not claim an evidence-bundle moat until A1 proves bundles beat raw context
  and A3 proves the schema/corpus loop has adoption or unique data.
- Treat a competitor's management MCP as different from Parallax's intended
  read-only evidence-bundle MCP. Management tools can create, update, or resolve
  things; Parallax's first agent surface should expose bounded evidence and write
  only outcome records.
- Treat agent observability as table stakes once SigNoz can ingest Claude Code
  telemetry. Parallax must capture decisions, files, commands, approvals, tests,
  patches, PRs, recurrence, and reversions, not only token/cost/tool metrics.
- Treat eBPF RCA as complementary, not sufficient, when app-level Rust panic,
  stack, source chain, trace field, and release semantics are required.

## Result Row Schema

Each future drift update should add a row in this document or in a dedicated
results file with this shape:

```json
{
  "check_id": "competitor-drift-YYYYMMDD-competitor",
  "research_date": "YYYY-MM-DD",
  "competitor": "Coroot",
  "source_urls": ["https://docs.coroot.com/mcp/overview/"],
  "source_freshness": "current|stale|unreachable",
  "capabilities": {
    "sentry_migration": false,
    "otlp_three_signal": true,
    "low_ops_self_hosted": true,
    "portable_bundle": false,
    "deterministic_evidence_graph": true,
    "mcp": true,
    "management_or_write_mcp": true,
    "read_only_agent_surface": false,
    "agent_action_audit": false,
    "fix_outcome_loop": false
  },
  "drift_level": "trigger_hit",
  "parallax_impact": "MCP is no longer a Coroot gap; retain Sentry migration, bundle, and action audit as differentiators.",
  "required_doc_updates": [
    "docs/research/open-self-hosted-competitor-watch.md"
  ]
}
```

## Refresh Triggers

Mark affected rows stale and recheck when any watched competitor:

- ships or removes MCP, CLI, API, or agent-tool access;
- changes AI feature license tier or self-hosting availability;
- adds Sentry SDK/envelope ingestion or DSN migration;
- adds OTLP traces/logs/metrics or frontend session replay;
- exports portable evidence bundles, query manifests, or redaction reports;
- adds code-change, PR, ticket, or remediation workflows;
- adds agent/CLI command, file, approval, patch, test, or outcome audit;
- publishes benchmark claims against self-hosted Sentry or open observability
  stacks;
- changes license from open to source-available, or vice versa;
- passes 30 days without recheck during active market-position research.

## Product Wording Impact

Still allowed:

> Parallax is exploring an open, self-hosted evidence context engine that starts
> with Sentry-compatible errors and OTLP telemetry, then produces portable,
> redacted bundles and execution/action outcome records for coding agents.

Avoid:

- "MCP-native observability" as a differentiator;
- "open AI SRE" as the product category;
- "lighter Sentry" as the standalone pitch;
- "unique Sentry-compatible migration" without fixture results and lightweight
  competitor comparison;
- "agent observability" without side-effect and outcome audit.

## Strategic Consequence

The market has converged on agents as observability users. The remaining
Parallax wedge is not agent access; it is the evidence contract:

```text
what happened
+ why this evidence supports the claim
+ what is missing
+ what the agent did with it
+ whether the result fixed, failed, regressed, or recurred
```

The next build and research priority should protect that contract before adding
dashboard breadth.
