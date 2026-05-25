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

The MCP tool-power comparison behind this ledger now has a dedicated note:
[MCP power boundary competitor check](mcp-power-boundary-competitor-check.md).
The lightweight error-tracker MCP subset now has a focused note too:
[Lightweight error-tracker MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md).
The Sentry-specific Seer/MCP/self-hosted posture has its own recheck:
[Sentry MCP and Seer self-hosted recheck](sentry-mcp-seer-self-hosted-recheck.md).

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
| [Sentry MCP service](https://mcp.sentry.dev/), [sentry-mcp repository](https://github.com/getsentry/sentry-mcp), [sentry-mcp stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/master/docs/testing-stdio.md), and [sentry-mcp `0.35.0` release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0) | Sentry's MCP server is designed for human-in-the-loop coding agents, offers a remote hosted service, a Claude Code plugin/subagent path, and a stdio transport for self-hosted Sentry. The checked README calls stdio work in progress; AI-powered search tools require an OpenAI or Anthropic provider; self-hosted instances may need to disable unsupported Seer skills. The README setup path lists project/team/event write scopes, while the stdio testing guide also documents a read-only testing scope set. Latest checked release: `0.35.0` on 2026-05-21. | Sentry has an agent-facing MCP path in addition to Seer, but self-hosted MCP is not hosted-Seer parity. Read-only testing support narrows the old scope objection; Parallax still needs projection-equivalent, redacted, citable bundles and action/outcome audit. |
| [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/), [self-hosted `26.5.0` release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0), and [`26.5.0` Docker Compose](https://github.com/getsentry/self-hosted/blob/26.5.0/docker-compose.yml) | Current self-hosted docs list feature-complete Sentry features such as traces, profiles, replays, uptime, metrics, feedback, and crons, but do not list Seer/AI. The sentry-mcp README separately says some features like Seer may not be available on self-hosted instances. Self-hosted Sentry still has a large Docker Compose footprint: `26.5.0` declares 72 services. Latest checked release: `26.5.0` on 2026-05-18. | Current sources do not prove hosted-Seer parity for self-hosted Sentry. Treat that as an opening, but do not overclaim an explicit blanket AI exclusion from the current self-hosted docs. The low-ops benchmark remains relevant. |
| [Datadog Bits AI SRE eval platform](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/) and [Datadog Bits AI eval loop note](datadog-bits-ai-eval-loop.md) | Datadog evaluates agent investigations with reconstructed world snapshots, isolated scenario data layers, representative labels, noisy red-herring environments, segmentation, stored per-scenario scores, `pass@k`, weekly full-set regression runs, feedback-derived labels, and model-refresh checks. | Datadog is industrializing the exact feedback/eval loop Parallax wants for bundle value and corpus moat claims. This raises the A1 bar: Parallax needs frozen noisy world snapshots, open result ledgers, and raw-dump-vs-bundle parity, not only a private "agent seemed better" eval. |
| [Grafana Assistant CLI](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/) and [self-hosted Grafana Assistant](https://grafana.com/docs/grafana/latest/administration/assistant/) | Grafana Assistant CLI is public preview and can connect local projects so Assistant can read local files; terminal access can be enabled with approvals. Grafana v13 supports Assistant on-premise in self-hosted Grafana only by connecting to a Grafana Cloud stack; the Assistant backend, usage limits, and billing stay in Cloud. On-premise currently lacks investigations, investigation memory, infrastructure memory, Grafana Cloud MCP connections, CLI auth tokens, SQL table discovery, automations, sandbox settings, and anonymous Assistant access. | Grafana validates CLI/MCP/local-context agent surfaces but leaves air-gapped/local-first room. Do not describe this as free local Assistant for OSS Grafana. |
| [Grafana Assistant MCP docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/) | Grafana Assistant can connect remote MCP servers and skills for incident investigation, issue/code lookups, ticket creation, Slack notifications, and other actions. Local MCP servers are not supported; operators are responsible for server security, reliability, data access, least-privilege tokens, and tool-call review. | Read-only scoping, redaction, and audit are product requirements, not implementation details. Grafana's MCP posture is useful but deliberately broader than Parallax's intended read-only evidence-bundle adapter. |
| [OpenObserve homepage](https://openobserve.ai/), [pricing](https://openobserve.ai/pricing/), [enterprise features](https://openobserve.ai/docs/features/enterprise/), [SRE Agent setup](https://openobserve.ai/docs/enterprise-setup/sre-agent/), [AI SRE product page](https://openobserve.ai/ai-sre/), [MCP docs](https://openobserve.ai/docs/integration/ai/mcp/), [OTLP docs](https://openobserve.ai/docs/ingestion/logs/otlp/), [OpenObserve `v0.90.2` release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2), and [OpenObserve AI/MCP Enterprise recheck](openobserve-ai-mcp-enterprise-recheck.md) | The pricing page lists AI-powered observability, Incident Management & AI SRE Agent, and AI Assistant in Enterprise, says Self-Hosted Enterprise is free up to `50 GB/day`, and says AI features are preview/credit based; the homepage FAQ instead says Self-Hosted Enterprise is free up to `200 GB/day`. The SRE Agent setup requires an OpenObserve Enterprise license, AI provider key, and `O2_AI_ENABLED=true`; setup docs cover direct and gateway provider paths, while the AI SRE product page also claims OpenAI-compatible/self-hosted endpoint support. OpenObserve MCP is Enterprise-only; its MCP catalog includes natural-language queries plus broad create/update/delete/admin tools for alerts, dashboards, roles, streams, functions, KV, org/system settings, pipelines, users, ingestion, and search jobs. OpenObserve also supports OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces. Latest checked GitHub release: `v0.90.2` on 2026-05-22. | OpenObserve is still the closest Rust/object-storage threat. Do not overstate this as a simple paywall because Self-Hosted Enterprise has a free allowance and public pages conflict on the exact limit; the sharper gap is that AI/MCP are Enterprise-tier surfaces and the public MCP shape is broad/write-capable rather than Parallax's intended read-only evidence-bundle surface. |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/), [SigNoz MCP server](https://signoz.io/docs/ai/signoz-mcp-server/), [`signoz-mcp-server` `v0.4.1` release](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.4.1), [SigNoz `v0.125.1` release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1), and [SigNoz open investigation format check](signoz-open-investigation-format-check.md) | SigNoz positions observability inside coding agents, claims an "open investigation format," and supports hosted plus self-hosted MCP. Its current MCP tool list covers metrics, traces, logs, docs, alerts, dashboards, saved views, and notification channels, including create/update/delete tools for several resource types. The 2026-05-25 focused check found the claim only on the landing page, not a versioned schema or portable artifact in the checked docs/README/release sources. Latest checked GitHub releases: `signoz-mcp-server` `v0.4.1` on 2026-05-21 and `signoz` `v0.125.1` on 2026-05-20. | SigNoz directly attacks the "agent-native observability" story and is now gesturing at a standardizable investigation format, but this remains unproven until the format is source-linked, versioned, and auditable. Parallax must distinguish query/management MCP from read-only evidence bundles with schema, redaction, raw-ref, and outcome semantics. |
| [SigNoz Claude Code monitoring](https://signoz.io/docs/claude-code-monitoring/) | SigNoz documents Claude Code OpenTelemetry export with logs/metrics and prompt-level correlation fields. | Agent telemetry is no longer a distant niche; Parallax needs the richer action/outcome graph. |
| [Coroot `v1.20.2` release](https://github.com/coroot/coroot/releases/tag/v1.20.2), [Coroot AI RCA](https://docs.coroot.com/ai/overview/), [AI configuration](https://docs.coroot.com/ai/configuration/), [Coroot Cloud integration](https://docs.coroot.com/ai/coroot-cloud/), [Coroot editions](https://coroot.com/editions), [Coroot MCP](https://docs.coroot.com/mcp/overview/), [Coroot architecture](https://docs.coroot.com/installation/architecture/), [Coroot eBPF tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/), and [Coroot MCP and AI RCA recheck](coroot-mcp-ai-rca-recheck.md) | Coroot `v1.20.2` added the MCP server on 2026-05-06. Coroot Community is listed as free forever, self-hosted, no monitored-infrastructure limit, and includes agentic-ready MCP; Enterprise adds AI RCA and agentic anomaly investigation at $1 per monitored CPU core/month. Community can connect to Coroot Cloud for 10 free RCA investigations/month. The MCP endpoint uses streamable HTTP, OAuth 2.0, and server-side authorization, exposes topology/alerts/incidents/traces/logs/metrics, includes Community `resolve_alerts`, and adds Enterprise `investigate_anomaly`; eBPF traces may not provide complete traces. | Coroot's agent surface is now a serious self-hosted baseline, but still lacks Parallax's Sentry migration, portable evidence-bundle/schema, coding-agent side-effect audit, and fully local/open RCA in Community. |
| [Bugsink docs](https://www.bugsink.com/docs/), [Bugsink repository](https://github.com/bugsink/bugsink), [Bugsink `2.2.1` release](https://github.com/bugsink/bugsink/releases/tag/2.2.1), [`bugsink-mcp` package](https://www.npmjs.com/package/bugsink-mcp), and [Bugsink simplicity recheck](bugsink-sentry-compatible-simplicity-recheck.md) | Bugsink is self-hosted error tracking compatible with the Sentry SDK; current docs claim DSN migration and a low-ops deployment shape. The recheck narrows "single container/SQLite" into throwaway Docker, persistent Docker database, and non-container SQLite cases. GitHub metadata checked on 2026-05-25 shows latest release `2.2.1` on 2026-05-22 with roughly 1.8k stars; the release improves the canonical API. No first-party Bugsink MCP was found in official docs, but small third-party Bugsink MCP adapters now exist. | Low-ops Sentry compatibility is not unique, and Bugsink is active enough to be a real simplicity baseline. Ecosystem MCP pressure means Parallax should not rely on "Bugsink data cannot be exposed to agents" as a durable claim. |
| [Rustrak repository](https://github.com/AbianS/rustrak), [`@rustrak/mcp`](https://www.npmjs.com/package/@rustrak/mcp), and [lightweight MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md) | Rustrak is Rust/Actix, Sentry SDK compatible, SQLite-by-default, small-footprint, and ships `@rustrak/mcp` for AI assistant integration. GitHub metadata checked on 2026-05-25 shows activity the same day, latest visible release `docs@0.1.16`, roughly 43 stars, and npm `@rustrak/mcp` `0.1.2`; the MCP tool surface includes management/destructive/token/raw-event access. | Rust plus Sentry compatibility plus MCP is already a live lightweight competitor shape, though still early maturity. Its MCP posture reinforces Parallax's read-only bundle boundary rather than closing it. |
| [Traceway repository](https://github.com/tracewayapp/traceway) and [embedded mode docs](https://docs.tracewayapp.com/learn/embedded-mode) | Traceway is MIT, OpenTelemetry-native, self-hosted, and combines logs, traces, metrics, exceptions, session replay/RUM, and AI observability with OTLP/HTTP ingest. GitHub metadata checked on 2026-05-25 shows latest backend release `backend/v1.7.27` on 2026-05-22 and roughly 817 stars; embedded mode runs a pure-SQLite server in a Go process. | Traceway pressures the OTLP/frontend/replay and low-friction embedded parts of Parallax's roadmap. |
| [GoSnag repository](https://github.com/darkspock/gosnag), [MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts), and [lightweight MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md) | GoSnag is a self-hosted Sentry-compatible service with an MCP server exposing project, issue, alert, tag, ticket, and user management tools plus broad AI RCA and ticket-generation claims. GitHub metadata checked on 2026-05-25 shows no tagged release, roughly 8 stars, and last push on 2026-04-17; the MCP implementation uses Bearer-token API calls and includes create/update/delete or workflow-changing tools. | MCP over issue management is already present in small Sentry-compatible tools, but GoSnag should be treated as a capability warning rather than a mature baseline. |
| [Urgentry site](https://urgentry.com/) and [Urgentry repository](https://github.com/urgentry/urgentry) | Urgentry claims DSN-only migration from Sentry, Tiny mode, one-binary startup, traces/replay/profiling/logs, and benchmark comparisons against self-hosted Sentry; it is source-available under FSL. GitHub metadata checked on 2026-05-25 shows latest release `v0.2.12` on 2026-05-22 with roughly 55 stars. | Urgentry pressures the "simpler than self-hosted Sentry" claim even if it does not satisfy the open-source thesis; treat its performance numbers as vendor claims until reproduced by benchmark artifacts. |

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
| Sentry Seer | `wedge_under_pressure` | Hosted Seer has root-cause, solution, code-change, external-agent handoff, and PR API paths. | Current self-hosted docs and sentry-mcp docs do not prove hosted-Seer parity for self-hosted Sentry; no open portable evidence-bundle schema. |
| Sentry MCP | `wedge_under_pressure` | Sentry has an official MCP server for coding assistants, a hosted remote endpoint, a Claude Code plugin/subagent path, and a stdio/self-hosted path; current checked release is `0.35.0`. | The self-hosted stdio path is documented as work in progress; README setup lists write scopes while the testing guide documents read-only testing scopes; AI-powered search still needs an external provider; no hosted-Seer parity, Parallax-style open redacted bundle schema, or coding-agent action/outcome graph is proven. |
| Datadog Bits AI SRE | `wedge_under_pressure` | Datadog's eval platform has the feedback, label, world-snapshot, noise, segmentation, score-history, model-refresh, and full-set regression machinery Parallax wants. | Enterprise SaaS data gravity, not an open self-hosted context engine or a public portable evidence-bundle/result-ledger standard. |
| Grafana Assistant | `wedge_under_pressure` | CLI and remote MCP are real agent surfaces; self-managed Grafana works through Grafana Cloud, and CLI can expose local files by tunnel. | Not fully local/air-gapped; on-prem lacks investigations/memory/CLI tokens/Grafana Cloud MCP connections; broad Grafana assistant, not Parallax bundles/outcome graph. |
| OpenObserve | `wedge_under_pressure` | Rust, object-storage-oriented, OTLP, AI SRE, Enterprise MCP, public free Self-Hosted Enterprise allowance with conflicting `50` versus `200 GB/day` pages, and a large MCP tool catalog with destructive/admin operations. | No Sentry-envelope migration in checked docs; no open read-only bundle/action-audit contract; AI/MCP are Enterprise-tier rather than plain AGPL Community guarantees; exact free allowance is source-conflicted. |
| SigNoz | `wedge_under_pressure` | Self-hosted MCP and agent-native observability target Claude Code, Codex, Cursor, and similar workflows; landing page claims an open investigation format; the MCP catalog includes query and management tools. | No Sentry envelope path or documented deterministic evidence bundle/outcome graph in checked docs; first-party MCP is not a Parallax-style bounded read-only bundle; the 2026-05-25 focused check still found no versioned investigation schema or portable artifact. |
| Coroot | `trigger_hit` | Focused source refresh confirms Community MCP, OAuth/server-side authorization, Community `resolve_alerts`, Enterprise `investigate_anomaly`, Enterprise/local AI provider support, and Community AI RCA only through Coroot Cloud credits. This is stronger than "MCP exists" because Coroot gives agents topology, incidents, traces, logs, metrics, and focused RCA. | eBPF traces can be incomplete; no Sentry migration, portable bundle, coding-agent action/outcome audit, or fully local/open AI RCA in Community. |
| Bugsink | `trigger_hit` | Focused refresh confirms strong Sentry SDK/DSN migration and low-ops pressure; `2.2.1` improves the canonical API; no first-party MCP was found in official docs, but third-party Bugsink MCP adapters now exist. | Source-available rather than OSI-open; error-tracking focused; no OTLP evidence graph, first-party read-only evidence-bundle MCP, or agent action/outcome audit in checked docs. |
| Rustrak | `trigger_hit` | Rust, Sentry SDK compatibility, SQLite default, and MCP are all present; npm `@rustrak/mcp` is live. | Management-shaped MCP; no OTLP traces/logs/metrics, evidence bundles, or outcome loop in checked docs; early maturity. |
| Traceway | `wedge_under_pressure` | OpenTelemetry-native logs/traces/metrics/session replay/exceptions/AI observability, MIT, self-hosted, active release cadence, and SQLite embedded mode. | No Sentry-envelope migration or coding-agent side-effect/outcome graph in checked docs. |
| GoSnag | `trigger_hit` | Sentry-compatible issue tracker with MCP management tools and AI RCA claims. | Very early maturity signal; Postgres-backed issue tool; no OTLP evidence graph or bounded read-only bundle contract in checked docs. |
| Urgentry | `wedge_under_pressure` | One-binary Tiny mode, DSN migration, Sentry-like traces/replay/profiles/logs, fresh release, and same-host benchmark claims. | Source-available, not OSI open; benchmark claims are not independently reproduced; no Parallax-style open schema or agent action audit in checked docs. |

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
- Separate capability shape from maturity. A no-release, low-traction project can
  still warn about market direction, but it should not be weighted like an
  active release train in Phase 1 go/no-go evidence.
- Do not claim "agent-native" unless the surface is read-only by default,
  redacted, citable, auditable, and safe for least-privilege operation.
- Do not accept "open investigation format" as an evidence-bundle moat claim
  unless the competitor publishes a versioned schema or portable artifact with
  provenance, redaction, raw-ref, and outcome semantics.
- Treat SigNoz's current "open investigation format" phrase as a watch trigger,
  not as A3 closure. The checked landing page, MCP docs, MCP README, and release
  metadata did not expose a canonical schema or artifact on 2026-05-25.
- Do not claim an evidence-bundle moat until A1 proves bundles beat raw context
  and A3 proves the schema/corpus loop has adoption or unique data.
- Treat a competitor's management MCP as different from Parallax's intended
  read-only evidence-bundle MCP. Management tools can create, update, or resolve
  things; Parallax's first agent surface should expose bounded evidence and write
  only outcome records.
- Keep the first Parallax context server free of alert/dashboard/user/role/
  pipeline/notification/incident/ticket CRUD, alert resolution, notification
  sends, and persisted RCA writes. Those belong in later automation/control
  surfaces if fixture results justify them.
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
