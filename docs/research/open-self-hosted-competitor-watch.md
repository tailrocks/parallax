# Open Self-Hosted Competitor Watch

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The prompt now requires every ongoing research pass to track the direct open /
self-hosted competitors closest to Parallax's wedge:

- OpenObserve;
- SigNoz;
- Coroot.

These are more dangerous than Datadog/Sentry/Grafana in one specific way: they
can close the "open, self-hosted, agent-native" part of the market without
needing incumbent cloud data gravity. This note is the current watchlist and the
trigger list for reopening the GO verdict.

A second, narrower competitor class now has its own watchlist:
[Lightweight Sentry-compatible competitor watch](lightweight-sentry-compatible-competitor-watch.md).
That note tracks Bugsink, Rustrak, Traceway, GoSnag, and Urgentry because they
pressure the Sentry-compatible migration and low-ops claims from below.

The current trigger-hit status for both watchlists lives in the
[Agentic observability competitor drift ledger](agentic-observability-competitor-drift-ledger.md).
OpenObserve's current AI/MCP Enterprise posture now has a focused source check:
[OpenObserve AI/MCP Enterprise recheck](openobserve-ai-mcp-enterprise-recheck.md).
The SigNoz open-investigation-format claim now has a focused falsification note:
[SigNoz open investigation format check](signoz-open-investigation-format-check.md).

## Current Verdict

None of the three closes the Parallax wedge today, but the window is narrow.

Parallax is still differentiated only if it ships the combination:

```text
Sentry-compatible error ingest
+ OTLP telemetry
+ low-resource self-hosted operation
+ portable evidence bundles
+ deterministic evidence graph
+ CLI and read-only MCP
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fixer outcome loop
```

OpenObserve is closest on storage/runtime fit. SigNoz is closest on open
agent-native MCP. Coroot is closest on zero-code infrastructure visibility and
now has an official MCP endpoint. None currently combines Sentry-envelope
migration, portable evidence bundle/schema, and coding-agent/CLI side-effect
audit.

## Competitor Matrix

| Competitor | Current strongest fit | Current Parallax gap | Threat level |
| --- | --- | --- | --- |
| OpenObserve | Rust, object-storage-oriented, self-hostable observability platform with OTLP ingest, RUM/source maps, Enterprise AI SRE Agent, AI Assistant, incident/RCA workflow, and Enterprise MCP with broad query/admin tools. | AI/MCP features are Enterprise-gated; public pages conflict on the free Self-Hosted Enterprise allowance (`50 GB/day` on pricing, `200 GB/day` on the homepage FAQ); current ingestion docs emphasize OTLP/log APIs/Prometheus/etc., not Sentry envelopes; MCP is not a narrow read-only bundle surface; no portable evidence-bundle schema or coding-agent action audit. | Very high. |
| SigNoz | Open self-hostable MCP server, agent-native positioning, Claude Code/Codex/Cursor/Gemini integration, traces/logs/metrics/topology/deploy history through agent clients, a landing-page claim around an "open investigation format," and MCP tools for alerts/dashboards/views/channels. | Go + ClickHouse stack; the 2026-05-25 focused check found the claim only as workflow/product language, not as a published schema or portable artifact; query/management interface rather than deterministic evidence bundle; no Sentry envelope ingestion in current docs; no Parallax-style CLI/agent side-effect audit. | High. |
| Coroot | Apache-2.0 OSS, eBPF zero-instrumentation, metrics/logs/traces/profiles, service map, deployment tracking, current `1.20.2` release, Community MCP, Community `resolve_alerts`, and Enterprise/Cloud AI RCA. | eBPF spans can be incomplete and lack app-level Rust panic/error-chain semantics; AI RCA is not purely OSS/local in Community; MCP is OAuth/RBAC-scoped but still not purely read-only; no Sentry migration path, portable evidence bundle, or coding-agent action audit in official docs. | High. |

## OpenObserve

### What It Has

OpenObserve is the closest structural threat:

- Rust implementation and self-hostable posture.
- Unified logs, metrics, traces, RUM, source maps, pipelines, dashboards,
  alerts, IAM, and storage management.
- OTLP HTTP and OTLP gRPC ingestion for logs, metrics, and traces.
- Object-storage-oriented architecture already aligned with cheap retention.
- SRE Agent setup that powers AI Assistant, incident management, and RCA in
  OpenObserve Enterprise, with direct, bundled-gateway, and external/self-hosted
  gateway provider paths; the AI SRE product page also claims support for
  OpenAI-compatible/self-hosted endpoints.
- Enterprise MCP that can query logs, metrics, and traces, but also exposes
  broad create/update/delete/admin tools for alerts, dashboards, roles, streams,
  functions, pipelines, users, KV, ingestion, and search jobs.

This overlaps with Parallax's self-hosted, Rust-first, object-storage direction
more than any broad incumbent does.

### Current Gaps That Keep Parallax Alive

1. **Agent gating.** The SRE Agent docs say it powers AI-driven features in
   OpenObserve Enterprise and list an Enterprise license as a prerequisite.
   Parallax should not gate the evidence/MCP layer if the open schema is meant
   to become the moat. Treat the exact free Self-Hosted Enterprise allowance as
   unresolved until OpenObserve reconciles the checked `50 GB/day` pricing page
   with the `200 GB/day` homepage FAQ.
2. **Sentry migration.** Current ingestion docs show OTLP, log APIs, Prometheus,
   Telegraf, syslog, forwarders, and language examples. They do not show a
   Sentry envelope compatibility path.
3. **Evidence bundle.** OpenObserve exposes observability data and AI/RCA
   workflows, but not a stable, portable evidence bundle with redaction report,
   edge strengths, missing-evidence flags, query manifest, and raw refs.
4. **Agent/CLI side-effect audit.** The threat is observability-agent RCA, not
   the full "what did Codex/Claude/Amp run, edit, test, and open as a PR?"
   audit graph.
5. **MCP safety shape.** The public MCP catalog is broad and write-capable.
   Parallax's first MCP surface should stay narrower: read-only bundles,
   redaction reports, raw-ref controls, and audit rows.

### Watch Triggers

Reopen the Parallax competitive read if OpenObserve:

- moves SRE Agent / AI Assistant / RCA / MCP into the free AGPL tier;
- adds Sentry envelope ingestion or Sentry SDK drop-in migration;
- exports portable investigation bundles with redaction and query manifests;
- adds coding-agent session or shell/CLI action audit;
- publishes measured fixer outcome loops or PR-generating workflows.

## SigNoz

### What It Has

SigNoz is the closest open agent-native threat:

- A hosted MCP server for cloud users and a self-hosted MCP server for
  self-hosted SigNoz.
- Documented setup for Claude Desktop, Claude Code, OpenAI Codex, Cursor,
  Gemini CLI, Windsurf, Zed, VS Code / GitHub Copilot, and others.
- Agent-native product language: telemetry inside the coding-agent workflow,
  not only dashboards.
- A public landing-page claim that the "open investigation format" SigNoz uses
  can become a team standard. The focused 2026-05-25 re-check found this as
  product/workflow language, not as a published schema or artifact.
- Architecture centered on OpenTelemetry and ClickHouse.
- Recent docs for observing Claude Code itself with OpenTelemetry logs and
  metrics, including terminal/MCP connection/cost fields.
- MCP tools that query metrics/traces/logs and also create, update, or delete
  alert rules, dashboards, saved views, and notification channels.

SigNoz validates the Parallax thesis that agents want observability through MCP
and structured APIs.

### Current Gaps That Keep Parallax Alive

1. **Stack fit.** SigNoz is Go with ClickHouse. That is in-scope by the prompt's
   runtime filter but less aligned with the Rust-first, tiny-self-hosted
   operational target.
2. **No Sentry envelope path.** Current docs center on OpenTelemetry/ClickHouse.
   There is no clear Sentry SDK envelope migration path in the public docs
   checked for this pass.
3. **No checked evidence-bundle contract.** SigNoz MCP gives agents query access
   to observability data and the landing page now claims an open investigation
   format, but this pass did not find a source-linked schema, canonical artifact,
   redaction report, query manifest, raw-ref policy, or outcome-row contract.
   Parallax's bet is that a pre-correlated, citable, redacted bundle beats asking
   the agent to assemble context itself. See
   [SigNoz open investigation format check](signoz-open-investigation-format-check.md).
4. **MCP power boundary.** SigNoz MCP includes management tools. That is useful,
   but it is not the same product surface as a least-privilege, read-only,
   citable evidence bundle.
5. **Action audit gap.** SigNoz can observe Claude Code activity, which narrows
   Parallax's agent-observability gap, but the current public story is still
   telemetry over agent activity, not a full outcome graph tying context,
   commands, patches, tests, PRs, reviews, reverts, and recurrence.

### Watch Triggers

Reopen the Parallax competitive read if SigNoz:

- adds Sentry-compatible ingestion or a Sentry migration guide;
- introduces portable incident/evidence bundles;
- publishes the claimed open investigation format as a versioned schema or
  portable artifact;
- adds deterministic evidence edges and missing-evidence reports;
- connects MCP queries to PR/fix outcome feedback;
- reduces ClickHouse/self-hosting operational weight enough that Parallax's
  "simpler than Sentry" claim weakens.

## Coroot

### What It Has

Coroot is the strongest zero-instrumentation competitor:

- Apache-2.0 OSS core.
- Latest checked GitHub release: `1.20.2` on 2026-05-06, with roughly 7.7k
  GitHub stars at source-check time.
- eBPF-based metrics, logs, traces, profiles, and service map.
- OpenTelemetry-compatible eBPF spans for uninstrumented services.
- Built-in inspections, SLOs, deployment tracking, cost monitoring, and
  predefined operational knowledge.
- Official MCP endpoint that exposes topology, alerts, incidents, traces, logs,
  metrics, and project selection to Claude Code, Cursor, Codex, and other MCP
  clients.
- Community Edition includes the agentic-ready MCP surface; Enterprise adds
  AI-powered RCA and agentic anomaly investigation at the listed $1 per
  monitored CPU core/month price point.
- MCP uses streamable HTTP transport, OAuth 2.0, and server-side authorization
  with the user's Coroot permissions. This is a stronger safety posture than an
  unauthenticated local tool, but it is still a live-production query surface
  and Community includes the mutating `resolve_alerts` tool.
- Enterprise `investigate_anomaly` follows the dependency graph, checks
  saturation, deploys, downstream errors, slow databases, log spikes, and
  profile shifts, then hands a focused diagnosis to an LLM; with an incident
  key, Coroot persists that RCA onto the incident.
- AI-powered RCA in Enterprise, or through Coroot Cloud integration for
  Community users.

Coroot is dangerous because it attacks adoption friction: install an agent and
get a service map and RCA without changing code.

### Current Gaps That Keep Parallax Alive

1. **App semantics.** Coroot's own eBPF tracing docs say eBPF spans may not
   provide complete traces. That matches Parallax's prior conclusion: eBPF is
   useful for infrastructure and dependency visibility, but cannot replace
   in-process Rust panic messages, typed error chains, `tracing` fields, release
   context, and source-level stack semantics.
2. **AI availability.** Official docs say AI RCA is Enterprise or available to
   Community through Coroot Cloud integration with 10 free investigations per
   month. That is not the same as fully local, open, agent-consumable evidence.
3. **Migration and bundle gap.** Coroot is not a Sentry-compatible error
   migration path and does not expose a Parallax-style portable evidence bundle.
4. **MCP safety is better than broad admin MCP, but still not Parallax's bundle
   contract.** OAuth and server-side authorization help, but the tool set still
   lets an agent query raw production telemetry and resolve alerts. Parallax's
   first MCP surface should instead expose bounded, redacted, citable bundles
   and write only audit/outcome records.
5. **Action audit is still missing.** Coroot exposes observability data to
   agents through MCP and can resolve alerts, but it still does not appear, in
   official docs checked here, to reconstruct coding-agent file, command, test,
   patch, PR, and outcome chains.

### Watch Triggers

Reopen the Parallax competitive read if Coroot:

- makes AI RCA fully local/open in Community Edition;
- adds Sentry-compatible error ingestion and grouping;
- adds agent action audit or fix outcome feedback;
- makes MCP outputs citable as portable evidence bundles with redaction reports;
- proves eBPF plus OTEL can cover enough app-level error semantics to weaken the
  need for Rust-first SDK capture.

## Strategic Implication

The wedge is not "open observability with AI." That wedge is already closing.

The wedge is:

> open self-hosted evidence bundles for agents, starting from Sentry-compatible
> errors and OTLP traces/logs/metrics, with CLI/coding-agent action audit.

The watchlist changes the product priority:

1. Do not spend early cycles on dashboards. OpenObserve, SigNoz, and Coroot
   already cover that market well.
2. Ship Sentry-compatible error ingestion and deterministic grouping early.
   That is the cleanest gap in all three.
3. Ship the evidence bundle/schema as the first open contract, not as a later
   export feature.
4. Keep CLI and coding-agent action audit first-class; this is the gap that
   generic observability platforms are least likely to model correctly.
5. Re-check this watchlist every ongoing research pass. If OpenObserve or SigNoz
   closes Sentry migration plus bundle export, the Parallax wedge must narrow
   further to agent action audit and measured fixer-outcome corpus.
6. Re-check the
   [lightweight Sentry-compatible watchlist](lightweight-sentry-compatible-competitor-watch.md)
   before making any "simpler than Sentry" or "drop-in Sentry-compatible" claim.

## Sources

OpenObserve:

- [OpenObserve AI/MCP Enterprise recheck](openobserve-ai-mcp-enterprise-recheck.md)
- [OpenObserve homepage](https://openobserve.ai/)
- [OpenObserve pricing](https://openobserve.ai/pricing/)
- [OpenObserve enterprise features](https://openobserve.ai/docs/features/enterprise/)
- [OpenObserve SRE Agent setup guide](https://openobserve.ai/docs/enterprise-setup/sre-agent/)
- [OpenObserve AI SRE product page](https://openobserve.ai/ai-sre/)
- [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/)
- [OpenObserve OTLP ingestion](https://openobserve.ai/docs/ingestion/logs/otlp/)
- [OpenObserve data ingestion guide](https://openobserve.ai/docs/ingestion/)
- [OpenObserve Enterprise license](https://openobserve.ai/enterprise-license/)
- [OpenObserve v0.90.2 release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2)
- [OpenObserve RUM source maps](https://openobserve.ai/blog/rum-source-map/)

SigNoz:

- [SigNoz agent-native observability](https://signoz.io/agent-native-observability/)
- [SigNoz agent-native blog](https://signoz.io/blog/introducing-agent-native-observability/)
- [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/)
- [SigNoz open investigation format check](signoz-open-investigation-format-check.md)
- [SigNoz AI tools and skills](https://signoz.io/docs/ai/overview/)
- [SigNoz architecture docs](https://signoz.io/docs/architecture/)
- [SigNoz Claude Code monitoring](https://signoz.io/docs/claude-code-monitoring/)

Coroot:

- [Coroot GitHub repository](https://github.com/coroot/coroot)
- [Coroot 1.20.2 release](https://github.com/coroot/coroot/releases/tag/v1.20.2)
- [Coroot product site](https://coroot.com/)
- [Coroot compare editions](https://coroot.com/editions)
- [Coroot AI RCA overview](https://docs.coroot.com/ai/overview/)
- [Coroot AI RCA configuration](https://docs.coroot.com/ai/configuration/)
- [Coroot Cloud integration](https://docs.coroot.com/ai/coroot-cloud/)
- [Coroot MCP server](https://docs.coroot.com/mcp/overview/)
- [Coroot eBPF-based tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/)
- [Coroot node-agent configuration](https://docs.coroot.com/configuration/coroot-node-agent)
