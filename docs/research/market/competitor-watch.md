# Competitor Watch

> **2026-06-02 update:** Syncause, AgentRx, Notrix Trax, AgentReplay, and OpenTelemetry MCP/replay/crash convention drift now have a focused recheck at
> [`agent-debugging-competitor-drift-2026-06-02.md`](agent-debugging-competitor-drift-2026-06-02.md).
> Verdict: agent-debugging tools validate the runtime-facts thesis but narrow the
> Parallax wedge. The defensible claim is no longer "AI agents need runtime
> context"; it is cross-system, redacted, durable evidence bundles that join
> production errors, OTLP telemetry, frontend/session evidence, CLI/CI/coding-agent
> action traces, and fix outcomes. AgentReplay is the highest new agent-capture
> watch item because it is local-first, Rust-core, OTLP-aware, and explicitly
> observes coding tools; Syncause is the closest language/positioning threat and
> a safety warning for skill/MCP install scope. Current OTel issue checks confirm
> replay-adjacent issue #3592 and Android crash-documentation issue #2473, not the
> older unverified #3448 crash reference.

> **2026-05-31 update:** Maple (maple.dev, github.com/Makisuo/maple) has been added as a broad-platform competitor. It is the most polished open-source OTel observability platform in the market — TypeScript/Bun, ClickHouse via Tinybird, 10+ MCP tools, browser session replay, K8s monitoring, and an outstanding single-binary local mode. It pressures "open self-hosted OTel observability" positioning but does NOT pressure Parallax's wedge: no Sentry ingestion, no evidence bundles, no outcome tracking, no CLI/agent/CI session capture, no redaction layer. Deep research at [`maple-deep-research.md`](maple-deep-research.md).
>
> Across all merged notes, no competitor closes the Parallax wedge today, but the wedge is under pressure, not closed: the broad open self-hosted platforms (OpenObserve, SigNoz, Coroot, **Maple**) already own open + self-hosted + agent-native observability, and the lightweight Sentry-compatible/OTLP-native challengers (Bugsink, Rustrak, Traceway, GoSnag, Urgentry) make Sentry-compatible migration, low-ops self-hosting, Rust implementation, and bare MCP availability table stakes rather than differentiators. OpenObserve pressures the Rust/object-storage runtime fit and now uses evidence-chain/audit-trail language, but its AI/MCP are Enterprise-tier with a source-conflicted free Self-Hosted Enterprise allowance (`50 GB/day` in pricing/license docs versus `200 GB/day` on the homepage FAQ) and a broad write-capable MCP; SigNoz pressures agent-native observability and claims an "open investigation format" plus a postmortem evidence-pack workflow and official alert-RCA skills, but publishes no versioned, validator-backed, portable artifact; Coroot pressures adoption friction with eBPF zero-instrumentation and a Community MCP that is OAuth/RBAC-scoped and mostly read-only except the mutating `resolve_alerts`, while AI RCA stays Enterprise or Coroot-Cloud-credit based. Sentry remains the strongest incumbent pressure — hosted Seer/Autofix proves the issue-to-fix workflow and Sentry MCP makes agent access table stakes — yet current self-hosted docs explicitly exclude Seer and other AI/ML features as closed source, the issue-fix API is a privileged `event:admin`/`event:write` surface, and self-hosted `26.5.0` still declares 72 Docker Compose services. The MCP-power boundary work confirms the central rule that MCP is table stakes and the defensible differentiator is the bounded, redacted, citable evidence bundle plus the execution/action/outcome graph, with Parallax's first MCP surface staying read-only and projection-equivalent and all mutation/automation deferred to a later control plane. The GO verdict reopens only if a competitor combines Sentry-compatible migration, OTLP traces/logs/metrics, low-resource self-hosting, a portable versioned evidence-bundle schema with redaction/raw-ref/provenance, read-only agent-safe context, coding-agent/CLI action audit, and an accepted/rejected/reverted fix-outcome loop.

This note consolidates the following previously-separate research files, each preserved in full below:

- `open-self-hosted-competitor-watch.md`
- `lightweight-sentry-compatible-competitor-watch.md`
- `agentic-observability-competitor-drift-ledger.md`
- `openobserve-ai-mcp-enterprise-recheck.md`
- `bugsink-sentry-compatible-simplicity-recheck.md`
- `rustrak-sentry-mcp-protocol-recheck.md`
- `traceway-otlp-ai-replay-recheck.md`
- `gosnag-sentry-ai-mcp-recheck.md`
- `urgentry-sentry-tiny-benchmark-recheck.md`
- `coroot-mcp-ai-rca-recheck.md`
- `signoz-open-investigation-format-check.md`
- `sentry-mcp-seer-self-hosted-recheck.md`
- `lightweight-error-tracker-mcp-boundary-check.md`
- `mcp-power-boundary-competitor-check.md`

## Open Self-Hosted Competitor Watch
_Provenance: merged verbatim from `open-self-hosted-competitor-watch.md` (2026-05-29 restructure)._

### Open Self-Hosted Competitor Watch

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

#### Purpose

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
[Lightweight Sentry-compatible competitor watch](competitor-watch.md).
That note tracks Bugsink, Rustrak, Traceway, GoSnag, and Urgentry because they
pressure the Sentry-compatible migration and low-ops claims from below.

The current trigger-hit status for both watchlists lives in the
[Agentic observability competitor drift ledger](competitor-watch.md).
OpenObserve's current AI/MCP Enterprise posture now has a focused source check:
[OpenObserve AI/MCP Enterprise recheck](competitor-watch.md).
Coroot's Community MCP and Enterprise/Cloud AI RCA posture now has a focused
source check:
[Coroot MCP and AI RCA recheck](competitor-watch.md).
The SigNoz open-investigation-format claim now has a focused falsification note:
[SigNoz open investigation format check](competitor-watch.md).

#### Current Verdict

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

#### Competitor Matrix

| Competitor | Current strongest fit | Current Parallax gap | Threat level |
| --- | --- | --- | --- |
| OpenObserve | Rust, object-storage-oriented, self-hostable observability platform with OTLP ingest, RUM/source maps, Enterprise AI SRE Agent, AI Assistant, incident/RCA workflow, evidence-chain/audit-trail product language, and Enterprise MCP with broad query/admin tools. Latest checked release redirect/tag ref still resolves to `v0.90.2`. | AI/MCP features are Enterprise-tier rather than plain AGPL Community; public pages conflict on the free Self-Hosted Enterprise allowance (`50 GB/day` in pricing/license docs, `200 GB/day` on the homepage FAQ); current ingestion docs emphasize OTLP/log APIs/Prometheus/etc., not Sentry envelopes; MCP is not a narrow read-only bundle surface; docs-search recheck found no exact Sentry or portable evidence-bundle/export terms; no checked coding-agent action audit. | Very high. |
| SigNoz | Open self-hostable MCP server, agent-native positioning, Claude Code/Codex/Cursor/Gemini integration, traces/logs/metrics/topology/deploy history through agent clients, a landing-page claim around an "open investigation format," documented postmortem evidence-pack and on-call lifecycle workflows, and official agent-skills material including read-only alert RCA with evals. | Go + ClickHouse stack; the 2026-05-25 focused check found evidence-pack workflows and agent playbooks but not a published schema, validator, replayable export, or portable artifact; query/management interface rather than deterministic evidence bundle; no Sentry envelope ingestion in current docs; no Parallax-style CLI/agent side-effect audit. | High. |
| Coroot | Apache-2.0 OSS, eBPF zero-instrumentation, metrics/logs/traces/profiles, service map, deployment tracking, current `1.20.2` release, Community MCP with read-only annotations on most telemetry tools, Community `resolve_alerts`, and Enterprise/Cloud AI RCA. | eBPF spans can be incomplete and lack app-level Rust panic/error-chain semantics; AI RCA is not purely OSS/local in Community; MCP is OAuth/RBAC-scoped but still not purely read-only because alert resolution mutates state; no Sentry migration path, portable evidence bundle, or coding-agent action audit in official docs/source. | High. |

#### OpenObserve

##### What It Has

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
- AI SRE product language around verifiable evidence, complete evidence chains,
  and audit trails over the logs, metrics, traces, and dependency maps used in
  an incident investigation.
- Enterprise MCP that can query logs, metrics, and traces, but also exposes
  broad create/update/delete/admin tools for alerts, dashboards, roles, streams,
  functions, pipelines, users, KV, ingestion, and search jobs.
- Same-day release-resolution check that still resolves the latest GitHub
  release to `v0.90.2`, with tag ref
  `308208f35c0a5d42da9f0e1798188cbbf46373fb`.

This overlaps with Parallax's self-hosted, Rust-first, object-storage direction
more than any broad incumbent does.

##### Current Gaps That Keep Parallax Alive

1. **Agent gating.** The SRE Agent docs say it powers AI-driven features in
   OpenObserve Enterprise and list an Enterprise license as a prerequisite.
   Parallax should not gate the evidence/MCP layer if the open schema is meant
   to become the moat. Treat the exact free Self-Hosted Enterprise allowance as
   unresolved until OpenObserve reconciles the checked `50 GB/day` pricing and
   license docs with the `200 GB/day` homepage FAQ. Also avoid overstating this
   as a simple paywall: current sources describe a free Self-Hosted Enterprise
   tier at some allowance.
2. **Sentry migration.** Current ingestion docs show OTLP, log APIs, Prometheus,
   Telegraf, syslog, forwarders, and language examples. They do not show a
   Sentry envelope compatibility path.
3. **Evidence bundle.** OpenObserve now directly pressures this wording with
   product-level evidence-chain and audit-trail claims, but the checked sources
   still do not publish a stable, portable evidence bundle with redaction
   report, edge strengths, missing-evidence flags, query manifest, and raw refs.
   The 2026-05-25 docs search-index recheck returned zero exact matches for
   Parallax-style portable bundle/export terms.
4. **Agent/CLI side-effect audit.** The threat is observability-agent RCA, not
   the full "what did Codex/Claude/Amp run, edit, test, and open as a PR?"
   audit graph.
5. **MCP safety shape.** The public MCP catalog is broad and write-capable.
   Parallax's first MCP surface should stay narrower: read-only bundles,
   redaction reports, raw-ref controls, and audit rows.

##### Watch Triggers

Reopen the Parallax competitive read if OpenObserve:

- moves SRE Agent / AI Assistant / RCA / MCP into the free AGPL tier;
- adds Sentry envelope ingestion or Sentry SDK drop-in migration;
- exports portable investigation bundles with redaction and query manifests;
- turns its AI SRE evidence-chain/audit-trail view into a versioned, exportable,
  machine-readable artifact;
- adds coding-agent session or shell/CLI action audit;
- publishes measured fixer outcome loops or PR-generating workflows.

#### SigNoz

##### What It Has

SigNoz is the closest open agent-native threat:

- A hosted MCP server for cloud users and a self-hosted MCP server for
  self-hosted SigNoz.
- Documented setup for Claude Desktop, Claude Code, OpenAI Codex, Cursor,
  Gemini CLI, Windsurf, Zed, VS Code / GitHub Copilot, and others.
- Agent-native product language: telemetry inside the coding-agent workflow,
  not only dashboards.
- A public landing-page claim that the "open investigation format" SigNoz uses
  can become a team standard.
- A documented postmortem evidence-pack use case where an AI assistant uses MCP
  to compile alert transitions, metric inflection points, representative logs,
  trace search, and trace details into an incident timeline.
- A 2026-05-20 on-call lifecycle blog positioning SigNoz MCP as alert creation,
  handoff brief, alert-fatigue audit, and postmortem evidence-pack automation.
- Official `agent-skills` material for Claude Code, Codex, Cursor, and similar
  tools, including `signoz-investigating-alerts`: a read-only three-tier alert
  RCA playbook with structured output sections, query-citation guardrails, and
  evals for full RCA, fuzzy matching, flapping/marginal fires, never-fired
  stops, and trace-formula fires.
- The focused 2026-05-25 re-check found the open-format claim and evidence-pack
  workflow plus agent investigation skills as product/workflow material, not as
  a published schema, validator, replayable export, or portable artifact.
- Architecture centered on OpenTelemetry and ClickHouse.
- Recent docs for observing Claude Code itself with OpenTelemetry logs and
  metrics, including terminal/MCP connection/cost fields.
- MCP tools that query metrics/traces/logs and also create, update, or delete
  alert rules, dashboards, saved views, and notification channels.

SigNoz validates the Parallax thesis that agents want observability through MCP
and structured APIs.

##### Current Gaps That Keep Parallax Alive

1. **Stack fit.** SigNoz is Go with ClickHouse. That is in-scope by the prompt's
   runtime filter but less aligned with the Rust-first, tiny-self-hosted
   operational target.
2. **No Sentry envelope path.** Current docs center on OpenTelemetry/ClickHouse.
   There is no clear Sentry SDK envelope migration path in the public docs
   checked for this pass.
3. **No checked evidence-bundle contract.** SigNoz MCP gives agents query access
   to observability data, the landing page claims an open investigation format,
   docs now show postmortem evidence-pack/on-call workflows, and official skills
   now prescribe alert-RCA playbooks and evals. This pass still did not find a
   source-linked schema, canonical artifact, replayable export, validator,
   redaction report, query manifest, raw-ref policy, or outcome-row contract.
   Parallax's bet is that a pre-correlated, citable, redacted bundle beats asking
   the agent to assemble context itself. See
   [SigNoz open investigation format check](competitor-watch.md).
4. **MCP power boundary.** SigNoz MCP includes management tools. That is useful,
   but it is not the same product surface as a least-privilege, read-only,
   citable evidence bundle.
5. **Action audit gap.** SigNoz can observe Claude Code activity, which narrows
   Parallax's agent-observability gap, but the current public story is still
   telemetry over agent activity, not a full outcome graph tying context,
   commands, patches, tests, PRs, reviews, reverts, and recurrence.

##### Watch Triggers

Reopen the Parallax competitive read if SigNoz:

- adds Sentry-compatible ingestion or a Sentry migration guide;
- introduces portable incident/evidence bundles;
- publishes the claimed open investigation format as a versioned schema or
  portable artifact;
- turns the postmortem evidence-pack workflow into a validator-backed,
  replayable export format;
- adds deterministic evidence edges and missing-evidence reports;
- connects MCP queries to PR/fix outcome feedback;
- reduces ClickHouse/self-hosting operational weight enough that Parallax's
  "simpler than Sentry" claim weakens.

#### Coroot

##### What It Has

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
- Community Edition is listed as free forever, self-hosted, and without a
  monitored-infrastructure limit; current Community AI RCA still goes through
  Coroot Cloud credits rather than a fully local/open default.
- MCP uses streamable HTTP transport, OAuth 2.0, and server-side authorization
  with the user's Coroot permissions. This is a stronger safety posture than an
  unauthenticated local tool, but it is still a live-production query surface
  and Community includes the mutating `resolve_alerts` tool. Source now confirms
  most telemetry tools are annotated read-only, while `resolve_alerts` is
  non-read-only, requires alert-edit permission, and sends notifications.
- Enterprise `investigate_anomaly` follows the dependency graph, checks
  saturation, deploys, downstream errors, slow databases, log spikes, and
  profile shifts, then hands a focused diagnosis to an LLM; with an incident
  key, Coroot persists that RCA onto the incident.
- AI-powered RCA in Enterprise, or through Coroot Cloud integration for
  Community users. Source shows the Cloud path posts a compressed RCA request
  containing metrics, Kubernetes events, deployments, and selected traces to the
  cloud integration endpoint.

Coroot is dangerous because it attacks adoption friction: install an agent and
get a service map and RCA without changing code.

##### Current Gaps That Keep Parallax Alive

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
   contract.** OAuth, server-side authorization, and read-only annotations on
   most telemetry tools help, but the tool set still lets an agent query raw
   production telemetry and resolve alerts. Parallax's first MCP surface should
   instead expose bounded, redacted, citable bundles and write only audit/outcome
   records.
5. **Action audit is still missing.** Coroot exposes observability data to
   agents through MCP and can resolve alerts, but it still does not appear, in
   official docs checked here, to reconstruct coding-agent file, command, test,
   patch, PR, and outcome chains.

##### Watch Triggers

Reopen the Parallax competitive read if Coroot:

- makes AI RCA fully local/open in Community Edition;
- adds Sentry-compatible error ingestion and grouping;
- adds agent action audit or fix outcome feedback;
- makes MCP outputs citable as portable evidence bundles with redaction reports;
- publishes a Cloud/Enterprise RCA data-boundary contract with replayable local
  artifacts, redaction reports, and raw refs;
- proves eBPF plus OTEL can cover enough app-level error semantics to weaken the
  need for Rust-first SDK capture.

#### Strategic Implication

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
   [lightweight Sentry-compatible watchlist](competitor-watch.md)
   before making any "simpler than Sentry" or "drop-in Sentry-compatible" claim.

#### Sources

OpenObserve:

- [OpenObserve AI/MCP Enterprise recheck](competitor-watch.md)
- [OpenObserve homepage](https://openobserve.ai/)
- [OpenObserve pricing](https://openobserve.ai/pricing/)
- [OpenObserve enterprise features](https://openobserve.ai/docs/features/enterprise/)
- [OpenObserve license and pricing docs](https://openobserve.ai/docs/enterprise-setup/license-and-pricing/)
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
- [SigNoz Postmortem Evidence Pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/)
- [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/)
- [SigNoz open investigation format check](competitor-watch.md)
- [SigNoz AI tools and skills](https://signoz.io/docs/ai/overview/)
- [SigNoz agent skills repository](https://github.com/SigNoz/agent-skills)
- [SigNoz architecture docs](https://signoz.io/docs/architecture/)
- [SigNoz Claude Code monitoring](https://signoz.io/docs/claude-code-monitoring/)

Coroot:

- [Coroot MCP and AI RCA recheck](competitor-watch.md)
- [Coroot GitHub repository](https://github.com/coroot/coroot)
- [Coroot 1.20.2 release](https://github.com/coroot/coroot/releases/tag/v1.20.2)
- [Coroot MCP source](https://github.com/coroot/coroot/blob/main/api/mcp.go)
- [Coroot RCA source](https://github.com/coroot/coroot/blob/main/api/rca.go)
- [Coroot Cloud RCA source](https://github.com/coroot/coroot/blob/main/cloud/rca.go)
- [Coroot product site](https://coroot.com/)
- [Coroot compare editions](https://coroot.com/editions)
- [Coroot AI RCA overview](https://docs.coroot.com/ai/overview/)
- [Coroot AI RCA configuration](https://docs.coroot.com/ai/configuration/)
- [Coroot Cloud integration](https://docs.coroot.com/ai/coroot-cloud/)
- [Coroot MCP server](https://docs.coroot.com/mcp/overview/)
- [Coroot architecture](https://docs.coroot.com/installation/architecture/)
- [Coroot Docker installation](https://docs.coroot.com/installation/docker/)
- [Coroot requirements](https://docs.coroot.com/installation/requirements/)
- [Coroot eBPF-based tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/)
- [Coroot node-agent configuration](https://docs.coroot.com/configuration/coroot-node-agent)

## Lightweight Sentry-Compatible Competitor Watch
_Provenance: merged verbatim from `lightweight-sentry-compatible-competitor-watch.md` (2026-05-29 restructure)._

### Lightweight Sentry-Compatible Competitor Watch

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Purpose

The existing [open self-hosted competitor watch](competitor-watch.md)
tracks broad observability platforms that could close the open + self-hosted +
agent-native wedge. This note tracks a narrower and more immediate competitor
class:

```text
small self-hosted error tracking or OTel platforms
+ Sentry SDK or Sentry-like migration
+ low operational footprint
+ optional AI/debugging features
```

These projects pressure Parallax's migration and simplicity claims even if they
do not yet close the evidence-bundle or agent-action-audit gap.

#### Short Verdict

The Parallax wedge is narrower than the previous watchlist implied.

OpenObserve, SigNoz, and Coroot are broad-platform threats. But Bugsink,
Rustrak, GoSnag, Urgentry, and Traceway show that the market is also attacking
self-hosted Sentry from below: smaller, simpler, Sentry-compatible or
OTel-native systems that avoid the self-hosted Sentry service graph.

None currently closes the full Parallax wedge:

```text
Sentry-compatible error migration
+ OTLP logs/traces/metrics
+ low-resource self-hosted operation
+ portable evidence bundles
+ deterministic evidence graph
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fixer outcome loop
```

But they reduce the value of "simpler than self-hosted Sentry" as a standalone
claim. Parallax must lead with evidence bundles and agent-safe context, not only
with a lighter Sentry replacement.

The focused
[lightweight error-tracker MCP boundary check](competitor-watch.md)
adds the agent-surface detail: Rustrak and GoSnag now prove that MCP can appear
inside small error trackers, but their checked tools are management/write/raw
event surfaces rather than read-only, redacted evidence bundles.
The Bugsink low-ops/Sentry-compatibility claim now has a focused recheck:
[Bugsink Sentry-compatible simplicity recheck](competitor-watch.md).
The Rustrak Rust/Sentry/MCP/protocol claim now has a focused recheck:
[Rustrak Sentry MCP protocol recheck](competitor-watch.md).
The Traceway OTLP/AI/session-replay claim now has a focused recheck:
[Traceway OTLP AI Replay Recheck](competitor-watch.md).
The GoSnag Sentry/AI/MCP claim now has a focused source-level recheck:
[GoSnag Sentry AI MCP Recheck](competitor-watch.md).
The Urgentry Sentry/Tiny/benchmark claim now has a focused source-level recheck:
[Urgentry Sentry Tiny Benchmark Recheck](competitor-watch.md).

#### Current Matrix

| Project | Strongest current fit | Current Parallax gap | Threat |
| --- | --- | --- | --- |
| Bugsink | Source-available self-hosted error tracking, Sentry SDK compatible, DSN migration, one-container throwaway Docker path, SQLite default outside the Docker-volume caveat, MySQL/PostgreSQL support, and small third-party MCP adapters over Bugsink's API. | PolyForm Shield rather than OSI-open; Python/runtime mismatch; persistent Docker setup needs external database care; official product is error tracking rather than OTLP-native evidence graph, first-party read-only agent bundle, CLI/agent audit, or fix-outcome corpus. | High for Sentry-compatible simplicity. |
| Rustrak | Rust/Actix server, Sentry SDK compatible for modern envelope error events, SQLite default or Postgres production mode, small Docker server image, GPL-3.0, `@rustrak/mcp` for AI assistant management, and a maintainer-side Sentry protocol drift workflow. | Early project; UI is a separate Next.js service; MCP exposes project/issue/event/token/alert tools including destructive issue/token actions and raw Sentry-envelope event access, not a read-only citable evidence-bundle contract; current ingest stores event items while its own drift report says sessions, transactions, client reports, attachments, and spans are not stored; no clear OTLP-native logs/traces/metrics or fix-outcome corpus. | Very high for product-shape pressure, lower for maturity. |
| Traceway | MIT, OpenTelemetry-native, self-hostable, direct OTLP/HTTP traces/metrics/logs, OTel exceptions/issues, trace-linked logs, session replay/RUM through native `/api/report`, AI trace promotion from `gen_ai.*`, SQLite/all-in-one/minimal/embedded deployment modes, and integration skills for adding instrumentation. | OTel-first rather than Sentry-envelope-first; Go, not Rust; no checked MCP/CLI evidence access, Parallax-style evidence bundle, redaction manifest, projection-equivalence contract, or coding-agent side-effect/outcome audit. | Very high for OTLP-native unified observability and local/self-hosted simplicity. |
| GoSnag | MIT Go/React tracker with embedded UI/migrations, Sentry `/store/` and `/envelope/` error-event ingest, issue lifecycle, GitHub/Jira/ticket workflows, AI RCA/merge/deploy/ticket/priority/tag/alert features, and a TypeScript MCP server over its management API. | Requires Postgres for normal deployment; early project with low visible traction and no tagged release; not Rust-first; source ignores Sentry `transaction`, `session(s)`, and `client_report` items; MCP uses Bearer-token API calls for broad project/issue/alert/tag/ticket/user management, not a Parallax-style read-only bundle contract or fix-outcome graph. | Medium-high: important capability shape, weak maturity signal. |
| Urgentry | FSL source-available Sentry-compatible replacement with one-binary Tiny SQLite mode, split self-hosted mode, source-confirmed store/envelope/minidump/security/OTLP HTTP JSON routes, broad envelope side effects, and vendor benchmark claims against self-hosted Sentry. | Not OSI-open; OTLP protobuf/gRPC rejected or absent in checked source; benchmark claims are unreproduced; no checked MCP; Autofix is deterministic/stub-like and stops before PRs; no portable evidence schema, redaction/source-policy manifest, projection hashes, missing-evidence model, coding-agent audit, or outcome loop. | Very high for Sentry-compatible breadth and self-hosted simplicity; lower for the open evidence-engine thesis. |

#### Current Version And Maturity Snapshot

Checked on 2026-05-25 with primary project docs, npm, and GitHub metadata:

| Project | Freshness signal | Maturity read |
| --- | --- | --- |
| Bugsink | GitHub latest release `2.2.1` on 2026-05-22; roughly 1.8k stars and 105 forks at check time; release adds canonical API issue actions/comments and OpenAPI docs; docs continue to claim SDK compatibility and low-ops self-hosting; third-party `bugsink-mcp` is now visible as an npm `1.0.0` MIT package. | Mature enough to be a real low-ops Sentry-compatible baseline; API/MCP ecosystem pressure means "no agent access" is no longer a durable ecosystem-level claim. |
| Rustrak | GitHub pushed on 2026-05-25; latest visible release `docs@0.1.16`; server package release `@rustrak/server@0.2.5`; npm `@rustrak/mcp` is `0.1.2`; Docker Hub server image `v0.2.5` was last updated 2026-05-21; roughly 43 stars at check time. | Product shape is very close, but maturity is still early and component release streams must be pinned separately. |
| Traceway | GitHub latest backend release `backend/v1.7.27` on 2026-05-22; MIT license; roughly 817 stars and 23 forks; repo pushed 2026-05-25; source/docs show `/api/otel/v1/{traces,metrics,logs}`, `/api/report`, AI trace promotion, SQLite single-container mode, and integration skills. | Strong active open-source pressure on the OTLP + unified context + replay side. |
| GoSnag | GitHub has no tagged release in the checked metadata, roughly 8 stars and 4 forks, and last push on 2026-04-17; latest checked `main` commit is `418b8b1`. | Treat as a capability warning, not a proven market baseline; source-level recheck supports error-event ingest, AI workflows, and management-shaped MCP, but not OTLP/evidence-bundle parity. |
| Urgentry | GitHub latest release `v0.2.12` on 2026-05-22; latest checked `main` commit `ccc0ff8`; roughly 55 stars and 5 forks; source confirms Tiny mode, DSN migration posture, traces/replay/profiling/logs surfaces, broad envelope side effects, OTLP HTTP JSON handlers, and vendor benchmark deltas versus self-hosted Sentry. | Fresh and strategically relevant. Treat as the strongest lightweight Sentry-replacement breadth warning, but keep performance numbers unmeasured and license gap explicit. |

#### Per-Project Notes

##### Bugsink

Bugsink is the cleanest "Sentry SDK compatibility plus self-hosting simplicity"
reference. Its docs say existing Sentry SDKs can be kept, the DSN changed, and
errors sent to a self-hosted backend. The current recheck narrows the deployment
claim: throwaway Docker is one container with SQLite and no persistence; Docker
with retained data should use an external database; SQLite remains the default
production-ready database in non-containerized setups, while Docker volumes are
not recommended for SQLite WAL mode. Bugsink's license is PolyForm Shield for
most repository content, so it is source-available rather than OSI-open.

The official Bugsink docs and repository still do not present first-party MCP or
AI agent features, but a small third-party `bugsink-mcp` package now exists on
npm at `1.0.0` under MIT. Treat that as ecosystem pressure, not as Bugsink
first-party agent-surface closure. See
[Bugsink Sentry-compatible simplicity recheck](competitor-watch.md).

Implication: Parallax cannot treat Sentry compatibility plus low ops as a unique
position. Bugsink already owns much of that error-tracking-only story and is
active enough to be used in the self-hosted simplicity baseline.

Watch triggers:

- Bugsink adds OTLP logs/traces/metrics correlation;
- Bugsink exports portable evidence bundles or query manifests;
- Bugsink adds first-party agent/MCP context tools or PR/fix outcome feedback;
- third-party Bugsink MCP becomes mature enough to pressure Parallax's
  read-only bundle boundary.

##### Rustrak

Rustrak is the closest language/runtime warning. Its README says the server is
Rust + Actix, Sentry SDK compatible, and can run with SQLite by default or
PostgreSQL for production. It also claims small memory/image footprint and no
Redis or complex infrastructure.

Update: the README now lists official packages for programmatic access and AI
assistant integration, including `@rustrak/mcp`, described as an MCP server that
lets Claude, Cursor, and Continue manage a Rustrak instance. This crosses the
old "adds MCP" watch trigger. The npm package is currently `0.1.2`. MCP
presence is no longer a sufficient Parallax differentiator in lightweight error
tracking.

The focused recheck also found two important caveats. First, current source
stores only Sentry envelope `event` items; Rustrak's own protocol drift report
says sessions, transactions, client reports, attachments, and spans are not
stored. Second, current `main` contains an unreleased `.claude`/BMad Sentry
protocol agent workflow. That is repo-maintenance tooling rather than a
product-facing runtime feature, but it is a warning that Rustrak is
operationalizing compatibility research. See
[Rustrak Sentry MCP protocol recheck](competitor-watch.md).

Implication: Rust-first lightweight Sentry-compatible error tracking now exists
as a live open project. Parallax should not frame itself as "Rustrak plus more
charts." It must be "Rustrak-like migration path plus OTLP context plus evidence
bundles plus agent audit."

Watch triggers:

- Rustrak adds OTLP trace/log/metric ingestion;
- Rustrak's MCP gains read-only, citable evidence bundles and redaction reports;
- Rustrak adds source/release/trace-aware evidence bundles;
- Rustrak proves broader Sentry SDK compatibility through fixture tests.

##### Traceway

Traceway is not Sentry-envelope-first in the checked public docs, but it is
dangerous because it is exactly the kind of low-friction OTel-native product
Parallax wants to be above. The focused recheck found source-level support for
this, not only README language: Traceway registers OTLP/HTTP endpoints for
traces, metrics, and logs; converts spans into endpoints, tasks, exceptions,
generic spans, and AI traces; links OTLP logs to traces/spans; and stores AI
conversation content in local filesystem or S3-backed blob storage. Its native
`/api/report` protocol covers traces, exceptions, metrics, sessions, and
session recordings for SDK surfaces where OTel does not yet cover replay.

Traceway's deployment story also has to be counted by mode. Root Docker Compose
is three services (`traceway`, `clickhouse`, `postgres`); all-in-one hides
ClickHouse and Postgres inside one container; SQLite mode is a single Alpine
container with two SQLite files plus blobs under `/data`; embedded mode runs a
development server inside a Go process. See
[Traceway OTLP AI Replay Recheck](competitor-watch.md).

Implication: Traceway pressures the OTLP-native and frontend/session-replay
parts of the roadmap. If it adds Sentry SDK migration, read-only evidence-bundle
export, or agent action audit, it becomes a direct wedge threat.

Watch triggers:

- Traceway adds Sentry envelope/DSN compatibility;
- Traceway adds evidence-bundle export with redaction reports;
- Traceway adds coding-agent or CLI action tracing;
- Traceway adds accepted/reverted fix feedback or PR workflow integration.

##### GoSnag

GoSnag is a focused Sentry-compatible error tracker with a surprisingly broad
feature list. The focused recheck confirms more than README language:
source registers legacy `/store/` and modern `/envelope/` endpoints, parses and
stores Sentry error-event JSON, runs AI-backed RCA/merge/deploy/ticket/priority/
tag/alert workflows, and ships a TypeScript MCP server. The important caveats
are equally concrete: envelope `transaction`, `session(s)`, and `client_report`
items are ignored; the implemented AI provider switch covers OpenAI-compatible
OpenAI/Groq and Bedrock, while direct Anthropic/Ollama support was not found in
the checked switch; `.env.example` omits the README's AI variables; and the MCP
server mutates project, issue, alert, tag, and ticket state through Bearer-token
API calls. See
[GoSnag Sentry AI MCP Recheck](competitor-watch.md).

Implication: "AI over Sentry-compatible self-hosted errors" is not enough. If
Parallax does not own the runtime/CI/CLI/agent evidence graph and citable bundle
contract, GoSnag-like tools can cover the visible issue-triage layer first. The
checked repository metadata still looks early, so GoSnag should be treated as a
feature-vector warning rather than a mature incumbent.

Watch triggers:

- GoSnag's MCP becomes read-only/citable where needed and writes fix/outcome
  records;
- GoSnag adds OTLP correlation;
- GoSnag adds deterministic bundle export and missing-evidence reporting;
- GoSnag's AI RCA becomes local/open and evidence-citing by default.

##### Urgentry

Urgentry is strategically useful even though it is not open source in the way
Parallax wants. The focused recheck found more than public-site positioning:
source registers Sentry `/store/`, `/envelope/`, minidump, security, CSP, NEL,
Unreal, and OTLP HTTP/JSON trace/log/metric routes. Envelope side effects cover
transactions, sessions, replay, profiles, client reports, check-ins,
attachments, and metric buckets. Tiny mode is one process with a SQLite data
directory; split self-hosted mode runs `api`, `ingest`, `worker`, and
`scheduler` roles over PostgreSQL, MinIO, Valkey, and NATS. See
[Urgentry Sentry Tiny Benchmark Recheck](competitor-watch.md).

The limits are equally important. Checked OTLP handlers reject protobuf and no
gRPC receiver was found. No MCP surface was found in checked README/docs/source.
The `autofix` API builds deterministic summaries, empty codebase/repository
rows, and skipped PR output rather than running a coding agent. The benchmark
docs explicitly exercise a narrow error-envelope workload, so their published
Sentry comparison numbers remain vendor claims until benchmark-agent artifacts
reproduce or reject them.

Implication: Urgentry should be included in the
[self-hosted simplicity gate](../validation/self-hosted-simplicity.md) comparison if
Parallax makes public low-ops claims. It can beat Parallax's "simpler Sentry"
story even if it does not beat the open/evidence/agent story. It also raises the
bar for Sentry item handling: Parallax can still target a narrower MVP, but it
must emit explicit unsupported-item outcomes rather than relying on vague
compatibility language.

Watch triggers:

- Urgentry open-sources under an OSI license;
- Urgentry adds portable evidence bundles, redaction/source-policy reports, or
  read-only MCP/CLI/API evidence tools;
- Urgentry benchmark methodology becomes independently reproducible;
- Urgentry adds OTLP protobuf/gRPC and Collector-equivalence evidence;
- Urgentry adds real CLI/coding-agent action audit and fix outcome rows.

#### Strategic Consequences

1. **Sentry-compatible migration is a requirement, not a moat.** Bugsink,
   Rustrak, GoSnag, and Urgentry all attack that path.
2. **Low-ops setup is a gate, not a differentiator.** The
   [self-hosted simplicity gate](../validation/self-hosted-simplicity.md) must compare
   Parallax against lightweight alternatives, not only self-hosted Sentry.
3. **Rust helps, but does not decide the market.** Rustrak proves Rust is
   available for lightweight error tracking. Traceway and GoSnag prove Go
   projects can still be operationally simple enough to matter.
4. **MCP is not a moat by itself.** Sentry has its own MCP server, Rustrak ships
   `@rustrak/mcp`, and GoSnag documents an MCP server. Parallax's agent surface
   has to be a bounded, redacted, read-only evidence contract with outcome
   writeback, not just tool exposure.
5. **Capability shape and maturity must be separated.** Bugsink and Traceway
   are active enough to be baseline competitors; Rustrak and Urgentry are fresh
   enough to watch closely; GoSnag is currently a feature-vector warning.
6. **Evidence bundles become more important.** The durable Parallax contract is
   the typed, redacted, citable failure dossier plus agent/action outcome graph.
7. **Frontend/session replay is no longer distant.** Traceway and Urgentry both
   pressure the frontend replay/error context direction; Parallax should keep
   frontend collection scoped but real.

#### Update To The Watchlist

The ongoing competitor watch now has two layers:

1. Broad observability platforms: OpenObserve, SigNoz, Coroot.
2. Lightweight Sentry-compatible or OTel-native challengers: Bugsink, Rustrak,
   Traceway, GoSnag, Urgentry.

Current trigger-hit and drift statuses across both layers live in the
[Agentic observability competitor drift ledger](competitor-watch.md).
Agent-surface detail for the lightweight layer lives in the
[lightweight error-tracker MCP boundary check](competitor-watch.md).

Reopen the Parallax wedge if any lightweight challenger combines:

- Sentry SDK/envelope migration;
- OTLP logs/traces/metrics correlation;
- low-resource self-hosting;
- portable evidence bundle/schema;
- read-only agent/CLI/MCP context access;
- coding-agent or CLI side-effect audit;
- accepted/rejected/reverted fix outcome loop.

#### Sources

- [Bugsink Sentry SDK compatibility](https://www.bugsink.com/sentry-sdk-compatible/)
- [Bugsink built to self-host](https://www.bugsink.com/built-to-self-host/)
- [Bugsink Docker install](https://www.bugsink.com/docs/docker-install/)
- [Bugsink settings](https://www.bugsink.com/docs/settings/)
- [Bugsink 2.2.1 release](https://github.com/bugsink/bugsink/releases/tag/2.2.1)
- [Bugsink GitHub repository](https://github.com/bugsink/bugsink)
- [Bugsink Sentry-compatible simplicity recheck](competitor-watch.md)
- [`bugsink-mcp` package](https://www.npmjs.com/package/bugsink-mcp)
- [`j-shelfwood/bugsink-mcp`](https://github.com/j-shelfwood/bugsink-mcp)
- [Rustrak GitHub repository](https://github.com/AbianS/rustrak)
- [Rustrak MCP package](https://www.npmjs.com/package/@rustrak/mcp)
- [Rustrak Docker Hub](https://hub.docker.com/r/abians7/rustrak-server)
- [Rustrak Sentry MCP protocol recheck](competitor-watch.md)
- [Traceway GitHub repository](https://github.com/tracewayapp/traceway)
- [Traceway embedded mode](https://docs.tracewayapp.com/learn/embedded-mode)
- [Traceway OTLP AI Replay Recheck](competitor-watch.md)
- [GoSnag GitHub repository](https://github.com/darkspock/gosnag)
- [GoSnag Sentry AI MCP Recheck](competitor-watch.md)
- [Sentry MCP repository](https://github.com/getsentry/sentry-mcp)
- [Urgentry product site](https://urgentry.com/)
- [Urgentry GitHub repository](https://github.com/urgentry/urgentry)
- [Urgentry Sentry Tiny Benchmark Recheck](competitor-watch.md)

#### Bottom Line

The simplest version of Parallax's pitch is now crowded:

> open/self-hosted, Sentry-compatible, easier than self-hosted Sentry.

The defensible version is still open:

> a Rust-first evidence context engine that starts with Sentry-compatible errors
> and OTLP telemetry, then produces portable redacted bundles and audit trails
> that coding agents can safely use to diagnose and fix software.

## Agentic Observability Competitor Drift Ledger
_Provenance: merged verbatim from `agentic-observability-competitor-drift-ledger.md` (2026-05-29 restructure)._

### Agentic Observability Competitor Drift Ledger

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Purpose

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
[MCP power boundary competitor check](competitor-watch.md).
The lightweight error-tracker MCP subset now has a focused note too:
[Lightweight error-tracker MCP boundary check](competitor-watch.md).
The Sentry-specific Seer/MCP/self-hosted posture has its own recheck:
[Sentry MCP and Seer self-hosted recheck](competitor-watch.md).

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

Same-day release-drift follow-up on 2026-05-25 found no newer latest-release
tags for Sentry MCP (`0.35.0`), self-hosted Sentry (`26.5.0`), OpenObserve
(`v0.90.2`), SigNoz MCP (`v0.4.1`), SigNoz (`v0.125.1`), Coroot (`v1.20.2`),
Bugsink (`2.2.1`), Traceway (`backend/v1.7.27`), Urgentry (`v0.2.12`), or
Rustrak (`docs@0.1.16`). Moving `main` branches are still active watch signals:
OpenObserve, Traceway, and Rustrak all had same-day pushes after the latest
release checks. Treat moving-main activity as drift pressure, not as a tagged
release wedge change until source-level feature evidence or a tagged release
lands.

#### Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer) and [Seer issue-fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | Seer uses Sentry issue context, tracing, logs, profiles, and code context; the issue-fix API can stop at root cause, solution, code changes, or open PR. The API requires `event:admin` or `event:write`. | Sentry owns the production-error agent path for hosted Sentry users, but its issue-fix path is a privileged control surface. Parallax's first agent surface should remain read-only evidence retrieval; PR-opening fix orchestration belongs above it. |
| [Sentry MCP service](https://mcp.sentry.dev/), [sentry-mcp repository](https://github.com/getsentry/sentry-mcp), [sentry-mcp stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/main/docs/testing-stdio.md), and [sentry-mcp `0.35.0` release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0) | Sentry's MCP server is designed for human-in-the-loop coding agents, offers a remote hosted service, a Claude Code plugin/subagent path, and a stdio transport for self-hosted Sentry. The checked README calls stdio work in progress; AI-powered search tools require an OpenAI or Anthropic provider; self-hosted instances may need to disable unsupported Seer skills. The README setup path lists project/team/event write scopes, while the stdio testing guide also documents a read-only testing scope set. Latest checked release: `0.35.0` on 2026-05-21; release redirect returned HTTP `200`, and tag ref `fc04542e24472f00b639f2d591dfc111fa855158` was visible. | Sentry has an agent-facing MCP path in addition to Seer, but self-hosted MCP is not hosted-Seer parity. Read-only testing support narrows the old scope objection; Parallax still needs projection-equivalent, redacted, citable bundles and action/outcome audit. |
| [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/), [self-hosted `26.5.0` release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0), and [`26.5.0` Docker Compose](https://github.com/getsentry/self-hosted/blob/26.5.0/docker-compose.yml) | Current self-hosted docs list feature-complete Sentry features such as traces, profiles, replays, uptime, metrics, feedback, and crons, and explicitly list Seer and other AI/ML features as unavailable because those components are closed source. The sentry-mcp README separately says some features like Seer may not be available on self-hosted instances. Self-hosted Sentry still has a large Docker Compose footprint: `26.5.0` declares 72 services. Latest checked release: `26.5.0` on 2026-05-18; release redirect returned HTTP `200`, and tag ref `aed5b2037e74c771bfe476dbdbeb80420ef4a3d8` was visible. | The self-hosted Seer gap is now explicit in current Sentry docs, not merely unproven. Treat that as an opening, but keep it tied to current docs rather than a permanent impossibility claim. The low-ops benchmark remains relevant. |
| [Datadog Bits AI SRE eval platform](https://www.datadoghq.com/blog/engineering/bits-ai-eval-platform/) and [Datadog Bits AI eval loop note](../validation/a1-bundle-value/datadog-bits-ai-eval-loop.md) | Datadog evaluates agent investigations with reconstructed world snapshots, isolated scenario data layers, representative labels, noisy red-herring environments, segmentation, stored per-scenario scores, `pass@k`, weekly full-set regression runs, feedback-derived labels, and model-refresh checks. | Datadog is industrializing the exact feedback/eval loop Parallax wants for bundle value and corpus moat claims. This raises the A1 bar: Parallax needs frozen noisy world snapshots, open result ledgers, and raw-dump-vs-bundle parity, not only a private "agent seemed better" eval. |
| [Grafana Assistant CLI](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/guides/cli/) and [self-hosted Grafana Assistant](https://grafana.com/docs/grafana/latest/administration/assistant/) | Grafana Assistant CLI is public preview and can connect local projects so Assistant can read local files; terminal access can be enabled with approvals. Grafana v13 supports Assistant on-premise in self-hosted Grafana only by connecting to a Grafana Cloud stack; the Assistant backend, usage limits, and billing stay in Cloud. On-premise currently lacks investigations, investigation memory, infrastructure memory, Grafana Cloud MCP connections, CLI auth tokens, SQL table discovery, automations, sandbox settings, and anonymous Assistant access. | Grafana validates CLI/MCP/local-context agent surfaces but leaves air-gapped/local-first room. Do not describe this as free local Assistant for OSS Grafana. |
| [Grafana Assistant MCP docs](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/) | Grafana Assistant can connect remote MCP servers and skills for incident investigation, issue/code lookups, ticket creation, Slack notifications, and other actions. Local MCP servers are not supported; operators are responsible for server security, reliability, data access, least-privilege tokens, and tool-call review. | Read-only scoping, redaction, and audit are product requirements, not implementation details. Grafana's MCP posture is useful but deliberately broader than Parallax's intended read-only evidence-bundle adapter. |
| [OpenObserve homepage](https://openobserve.ai/), [pricing](https://openobserve.ai/pricing/), [enterprise features](https://openobserve.ai/docs/features/enterprise/), [license/pricing docs](https://openobserve.ai/docs/enterprise-setup/license-and-pricing/), [SRE Agent setup](https://openobserve.ai/docs/enterprise-setup/sre-agent/), [AI SRE product page](https://openobserve.ai/ai-sre/), [MCP docs](https://openobserve.ai/docs/integration/ai/mcp/), [OTLP docs](https://openobserve.ai/docs/ingestion/logs/otlp/), [OpenObserve `v0.90.2` release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2), [docs search index](https://openobserve.ai/docs/search/search_index.json), and [OpenObserve AI/MCP Enterprise recheck](competitor-watch.md) | Pricing, enterprise-feature, and license docs say Self-Hosted Enterprise is free up to `50 GB/day`; license docs also describe a no-key `10 GB/day` threshold; the homepage FAQ instead says Self-Hosted Enterprise is free up to `200 GB/day`. The pricing page says AI features are preview/credit based. The SRE Agent setup requires an OpenObserve Enterprise license, AI provider key, and `O2_AI_ENABLED=true`; setup docs cover direct and gateway provider paths. The AI SRE product page claims OpenAI-compatible/self-hosted endpoint support and now uses evidence-chain/audit-trail language around incident RCA. OpenObserve MCP is Enterprise-only; its MCP catalog includes natural-language queries plus broad create/update/delete/admin tools for alerts, dashboards, roles, streams, functions, KV, org/system settings, pipelines, users, ingestion, and search jobs. OpenObserve also supports OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces. Latest-release redirect still resolves to `v0.90.2`; tag ref `308208f35c0a5d42da9f0e1798188cbbf46373fb`. Current docs-search index returned zero exact matches for `sentry` and Parallax-style portable evidence-bundle/export terms. | OpenObserve is still the closest Rust/object-storage threat. Do not overstate this as a simple paywall because Self-Hosted Enterprise has a free allowance and public pages conflict on the exact limit; the sharper gap is that AI/MCP are Enterprise-tier surfaces and the public MCP shape is broad/write-capable rather than Parallax's intended read-only portable evidence-bundle surface. Treat the AI SRE evidence-chain language as a watch trigger until a versioned/exportable schema appears. |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/), [SigNoz Postmortem Evidence Pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/), [SigNoz on-call lifecycle MCP blog](https://signoz.io/blog/automating-oncall-lifecycle-signoz-mcp/), [SigNoz MCP server](https://signoz.io/docs/ai/signoz-mcp-server/), [`SigNoz/agent-skills`](https://github.com/SigNoz/agent-skills), [`signoz-investigating-alerts`](https://github.com/SigNoz/agent-skills/blob/main/plugins/signoz/skills/signoz-investigating-alerts/SKILL.md), [`signoz-mcp-server` `v0.4.1` release](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.4.1), [SigNoz `v0.125.1` release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1), and [SigNoz open investigation format check](competitor-watch.md) | SigNoz positions observability inside coding agents, claims an "open investigation format," publishes postmortem evidence-pack and on-call lifecycle workflows, supports hosted plus self-hosted MCP, and now packages official agent skills. `signoz-investigating-alerts` is a read-only three-tier alert RCA playbook with required output sections, query-citation guardrails, and evals. The current MCP tool list still covers metrics, traces, logs, docs, alerts, dashboards, saved views, and notification channels, including create/update/delete tools for several resource types. The 2026-05-25 focused refresh found stronger workflow/playbook evidence, but not a versioned schema, validator-backed replayable export, or portable artifact in checked docs/README/source scans/release sources. Latest checked releases remain `signoz-mcp-server` `v0.4.1` on 2026-05-21, tag ref `8a6bb34ea75775bbe678594219bc21a5babd8721`, and `signoz` `v0.125.1` on 2026-05-20, tag ref `fb3e316ce906c36cdb20cd4900e58f2a43804d7a`. SigNoz `main` had a same-day 2026-05-25 push. | SigNoz directly attacks the "agent-native observability" story and is now packaging standardizable investigation workflows, but this remains unproven as A3 closure until the format is source-linked, versioned, and auditable. Parallax must distinguish assistant-generated reports, skills, and query/management MCP from read-only evidence bundles with schema, redaction, raw-ref, and outcome semantics. |
| [SigNoz Claude Code monitoring](https://signoz.io/docs/claude-code-monitoring/) | SigNoz documents Claude Code OpenTelemetry export with logs/metrics and prompt-level correlation fields. | Agent telemetry is no longer a distant niche; Parallax needs the richer action/outcome graph. |
| [Coroot `v1.20.2` release](https://github.com/coroot/coroot/releases/tag/v1.20.2), [Coroot AI RCA](https://docs.coroot.com/ai/overview/), [AI configuration](https://docs.coroot.com/ai/configuration/), [Coroot Cloud integration](https://docs.coroot.com/ai/coroot-cloud/), [Coroot editions](https://coroot.com/editions), [Coroot MCP](https://docs.coroot.com/mcp/overview/), [MCP source](https://github.com/coroot/coroot/blob/main/api/mcp.go), [RCA source](https://github.com/coroot/coroot/blob/main/api/rca.go), [Cloud RCA source](https://github.com/coroot/coroot/blob/main/cloud/rca.go), [Coroot architecture](https://docs.coroot.com/installation/architecture/), [Coroot eBPF tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/), and [Coroot MCP and AI RCA recheck](competitor-watch.md) | Coroot `v1.20.2` remains the latest checked release and added the MCP server on 2026-05-06; GitHub metadata showed a 2026-05-22 push. Coroot Community is listed as free forever, self-hosted, no monitored-infrastructure limit, and includes agentic-ready MCP; Enterprise adds AI RCA and agentic anomaly investigation at $1 per monitored CPU core/month. Community can connect to Coroot Cloud for 10 free RCA investigations/month. The MCP endpoint uses streamable HTTP, OAuth 2.0, and server-side authorization, exposes topology/alerts/incidents/traces/logs/metrics, includes Community `resolve_alerts`, and adds Enterprise `investigate_anomaly`; source confirms most telemetry tools are read-only annotated, `resolve_alerts` is non-read-only and alert-edit gated, and RCA is persisted onto incidents. Cloud RCA source posts a compressed request with metrics/events/deployments/traces to the cloud integration. eBPF traces may not provide complete traces. | Coroot's agent surface is now a serious self-hosted baseline, but still lacks Parallax's Sentry migration, portable evidence-bundle/schema, coding-agent side-effect audit, and fully local/open RCA in Community. |
| [Bugsink docs](https://www.bugsink.com/docs/), [Bugsink repository](https://github.com/bugsink/bugsink), [Bugsink `2.2.1` release](https://github.com/bugsink/bugsink/releases/tag/2.2.1), [`bugsink-mcp` package](https://www.npmjs.com/package/bugsink-mcp), and [Bugsink simplicity recheck](competitor-watch.md) | Bugsink is self-hosted error tracking compatible with the Sentry SDK; current docs claim DSN migration and a low-ops deployment shape. The recheck narrows "single container/SQLite" into throwaway Docker, persistent Docker database, and non-container SQLite cases. GitHub metadata checked on 2026-05-25 shows latest release `2.2.1` on 2026-05-22 with roughly 1.8k stars; the release improves the canonical API. No first-party Bugsink MCP was found in official docs, but third-party `bugsink-mcp` now has an npm `1.0.0` MIT package. | Low-ops Sentry compatibility is not unique, and Bugsink is active enough to be a real simplicity baseline. Ecosystem MCP pressure means Parallax should not rely on "Bugsink data cannot be exposed to agents" as a durable claim. |
| [Rustrak repository](https://github.com/AbianS/rustrak), [`@rustrak/mcp`](https://www.npmjs.com/package/@rustrak/mcp), [Rustrak Sentry MCP protocol recheck](competitor-watch.md), and [lightweight MCP boundary check](competitor-watch.md) | Rustrak is Rust/Actix, Sentry SDK compatible for modern envelope error events, SQLite-by-default, small-footprint, GPL-3.0, and ships `@rustrak/mcp` for AI assistant integration. GitHub metadata checked on 2026-05-25 shows activity the same day, latest visible release `docs@0.1.16`, server release `@rustrak/server@0.2.5`, roughly 43 stars, and npm `@rustrak/mcp` `0.1.2`; the MCP tool surface includes management/destructive/token/raw-event access. A 2026-05-22 unreleased `feat: sentry agent` commit adds repo-maintenance Sentry protocol agent workflow files, not a product-facing runtime feature. | Rust plus Sentry compatibility plus MCP is already a live lightweight competitor shape, though still early maturity. Its MCP posture reinforces Parallax's read-only bundle boundary rather than closing it; its own drift report also confirms non-event Sentry items are not stored. |
| [Traceway repository](https://github.com/tracewayapp/traceway), [OTLP route source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/routes.go), [AI tracing docs](https://docs.tracewayapp.com/learn/ai-tracing), [SQLite docs](https://docs.tracewayapp.com/server/sqlite), [integration skills](https://github.com/tracewayapp/traceway/tree/main/skills), and [Traceway recheck](competitor-watch.md) | Traceway is MIT, OpenTelemetry-native, self-hosted, and combines OTLP/HTTP logs/traces/metrics, OTel exceptions, native `/api/report` session replay/RUM, and AI trace promotion from `gen_ai.*`. GitHub metadata checked on 2026-05-25 shows latest backend release `backend/v1.7.27` on 2026-05-22, repo push on 2026-05-25, latest `main` commit `38b8d385`, and roughly 817 stars. SQLite mode is a single container with two SQLite files plus blobs under `/data`; embedded mode runs a development server inside a Go process. Integration skills exist, but no MCP server or Sentry-compatible ingest path was found in checked sources. | Traceway pressures OTLP/frontend/replay/AI tracing and low-friction local/self-hosted deployment. It narrows Parallax wording around OTLP-native context but does not close Sentry migration, evidence-bundle, read-only agent surface, or coding-agent action/outcome audit. |
| [GoSnag repository](https://github.com/darkspock/gosnag), [MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts), [ingest handler](https://github.com/darkspock/gosnag/blob/main/internal/ingest/handler.go), [AI provider source](https://github.com/darkspock/gosnag/blob/main/internal/ai/provider.go), and [GoSnag recheck](competitor-watch.md) | GoSnag is a self-hosted Sentry-compatible service with source-confirmed `/store/` and `/envelope/` error-event ingest, raw event JSON storage, AI RCA/merge/deploy/ticket/priority/tag/alert workflows, and an MCP server exposing project, issue, alert, tag, ticket, and user management tools. GitHub metadata checked on 2026-05-25 shows no tagged release, roughly 8 stars, last push on 2026-04-17, and latest checked `main` commit `418b8b1`; the MCP implementation uses Bearer-token API calls and includes create/update/delete or workflow-changing tools. Source ignores Sentry transactions, sessions, and client reports; the provider switch confirms OpenAI-compatible OpenAI/Groq and Bedrock, not direct Anthropic/Ollama. | MCP and AI over issue management are already present in small Sentry-compatible tools, but GoSnag should be treated as a capability warning rather than a mature baseline. The remaining gap is OTLP context, citable/redacted evidence bundles, read-only projections, and action/outcome audit. |
| [Urgentry site](https://urgentry.com/), [Urgentry repository](https://github.com/urgentry/urgentry), and [Urgentry recheck](competitor-watch.md) | Urgentry claims DSN-only migration from Sentry, Tiny mode, one-binary startup, traces/replay/profiling/logs, and benchmark comparisons against self-hosted Sentry; source confirms broad store/envelope/minidump/security/OTLP HTTP JSON ingest and envelope side effects for transactions, sessions, replay, profiles, client reports, check-ins, attachments, and metrics. Checked OTLP rejects protobuf; no MCP was found; Autofix is deterministic/stub-like. GitHub metadata checked on 2026-05-25 shows latest release `v0.2.12` on 2026-05-22, latest checked `main` commit `ccc0ff8`, and roughly 55 stars. | Urgentry pressures "simpler than self-hosted Sentry" and broad Sentry-protocol coverage even if it does not satisfy the open-source or agent-evidence thesis; treat performance numbers as vendor claims until reproduced by benchmark artifacts. |

#### Drift Levels

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

#### Current Drift Rows

| Competitor | Drift level | What changed or matters now | Remaining Parallax gap |
| --- | --- | --- | --- |
| Sentry Seer | `wedge_under_pressure` | Hosted Seer has root-cause, solution, code-change, external-agent handoff, and PR API paths; the issue-fix API requires `event:admin` or `event:write`. | Current self-hosted docs explicitly exclude Seer and other AI/ML features because those components are closed source; no open portable evidence-bundle schema; privileged fix APIs reinforce Parallax's read-only context-server boundary. |
| Sentry MCP | `wedge_under_pressure` | Sentry has an official MCP server for coding assistants, a hosted remote endpoint, a Claude Code plugin/subagent path, and a stdio/self-hosted path; current checked release is `0.35.0`. | The self-hosted stdio path is documented as work in progress; README setup lists write scopes while the testing guide documents read-only testing scopes; AI-powered search still needs an external provider; MCP does not erase the self-hosted Seer exclusion, Parallax-style open redacted bundle schema, or coding-agent action/outcome graph gap. |
| Datadog Bits AI SRE | `wedge_under_pressure` | Datadog's eval platform has the feedback, label, world-snapshot, noise, segmentation, score-history, model-refresh, and full-set regression machinery Parallax wants. | Enterprise SaaS data gravity, not an open self-hosted context engine or a public portable evidence-bundle/result-ledger standard. |
| Grafana Assistant | `wedge_under_pressure` | CLI and remote MCP are real agent surfaces; self-managed Grafana works through Grafana Cloud, and CLI can expose local files by tunnel. | Not fully local/air-gapped; on-prem lacks investigations/memory/CLI tokens/Grafana Cloud MCP connections; broad Grafana assistant, not Parallax bundles/outcome graph. |
| OpenObserve | `wedge_under_pressure` | Rust, object-storage-oriented, OTLP, AI SRE, evidence-chain/audit-trail positioning, Enterprise MCP, public free Self-Hosted Enterprise allowance with conflicting `50` versus `200 GB/day` pages, and a large MCP tool catalog with destructive/admin operations. Same-day latest-release redirect/tag-ref and docs-search checks now make the row more auditable. | No Sentry-envelope migration in checked docs; no open read-only bundle/action-audit contract; docs-search recheck found zero exact portable bundle/export terms; AI/MCP are Enterprise-tier rather than plain AGPL Community guarantees; exact free allowance is source-conflicted; evidence-chain language is not yet a versioned/exportable schema. |
| SigNoz | `wedge_under_pressure` | Self-hosted MCP and agent-native observability target Claude Code, Codex, Cursor, and similar workflows; landing page claims an open investigation format; docs now include postmortem evidence-pack/on-call workflows; official `agent-skills` includes a read-only alert-RCA skill with output guardrails and evals; the MCP catalog includes query and management tools. | No Sentry envelope path or documented deterministic evidence bundle/outcome graph in checked docs; first-party MCP is not a Parallax-style bounded read-only bundle; the 2026-05-25 focused refresh found stronger workflows/playbooks but no versioned investigation schema, validator-backed replayable export, portable artifact, or outcome-row contract. |
| Coroot | `trigger_hit` | Focused source refresh confirms Community MCP, OAuth/server-side authorization, read-only annotations on most telemetry tools, mutating alert-edit-gated Community `resolve_alerts`, Enterprise `investigate_anomaly`, Enterprise/local AI provider support, and Community AI RCA only through Coroot Cloud credits. This is stronger than "MCP exists" because Coroot gives agents topology, incidents, traces, logs, metrics, and focused RCA. | eBPF traces can be incomplete; no Sentry migration, portable bundle, coding-agent action/outcome audit, or fully local/open AI RCA in Community; source confirms Cloud RCA is an external request path, not an air-gapped default. |
| Bugsink | `trigger_hit` | Focused refresh confirms strong Sentry SDK/DSN migration and low-ops pressure; `2.2.1` improves the canonical API; no first-party MCP was found in official docs, but third-party Bugsink MCP adapters now exist. | Source-available rather than OSI-open; error-tracking focused; no OTLP evidence graph, first-party read-only evidence-bundle MCP, or agent action/outcome audit in checked docs. |
| Rustrak | `trigger_hit` | Rust, Sentry SDK compatibility, SQLite default, small Docker server image, MCP, and maintainer-side Sentry protocol drift workflow are all present; npm `@rustrak/mcp` is live. | Management-shaped MCP; current ingest stores event items but not session/transaction/client_report/attachment/span context per Rustrak's own drift report; no OTLP traces/logs/metrics, evidence bundles, or outcome loop in checked docs; early maturity. |
| Traceway | `wedge_under_pressure` | Source-level recheck confirms direct OTLP/HTTP ingest, trace/log/metric conversion, OTel exception grouping, AI trace promotion, native session replay protocol, SQLite/all-in-one/minimal/embedded deployment modes, and integration skills. | No checked Sentry-compatible ingest/migration, MCP/CLI read-only evidence bundle, projection-equivalence contract, or coding-agent side-effect/outcome graph. |
| GoSnag | `trigger_hit` | Source-confirmed Sentry error-event ingest, AI RCA/merge/deploy/ticket/priority/tag/alert workflows, and MCP management tools over projects/issues/alerts/tags/tickets/users. | Very early maturity signal; Postgres-backed issue tool; non-event Sentry items ignored; no OTLP evidence graph, bounded read-only bundle contract, projection hashes, or action/outcome audit in checked source. |
| Urgentry | `wedge_under_pressure` | One-binary Tiny mode, DSN migration, broad Sentry envelope side effects, OTLP HTTP/JSON route coverage, Sentry-like traces/replay/profiles/logs, fresh release, and same-host benchmark claims. | Source-available, not OSI-open; OTLP protobuf/gRPC unsupported or absent in checked source; benchmark claims are not independently reproduced; no checked MCP, portable open schema, projection hashes, missing-evidence model, real agent fixer, or action/outcome audit. |

#### Counting Rules

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
- Treat SigNoz's current "open investigation format" phrase, postmortem
  evidence-pack/on-call workflows, and official alert-investigation skills as
  watch triggers, not as A3 closure. The checked landing page, evidence-pack
  docs, MCP docs, MCP README, agent-skills repo, source scans, and release
  metadata did not expose a canonical schema, validator-backed replayable
  export, portable artifact, or outcome-row contract on 2026-05-25.
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

#### Result Row Schema

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
    "docs/research/market/competitor-watch.md"
  ]
}
```

#### Refresh Triggers

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

#### Product Wording Impact

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

#### Strategic Consequence

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

## OpenObserve AI/MCP Enterprise Recheck
_Provenance: merged verbatim from `openobserve-ai-mcp-enterprise-recheck.md` (2026-05-29 restructure)._

> **Re-verified 2026-05-29 (primary sources): no material drift.** Ingestion docs still show OTLP + Elasticsearch-bulk-API + log forwarders and **no Sentry-envelope endpoint** (Parallax's Sentry-migration wedge holds); AI SRE Agent + AI Assistant present; free Self-Hosted Enterprise (50 GB/day) lists SSO/RBAC/audit/redaction but **AI-SRE/MCP free-tier gating remains source-conflicted** (same caveat below); **no versioned/exportable evidence schema** found. The verdict's NO-GO competitive-window trigger has not fired.

### OpenObserve AI/MCP Enterprise Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check the current claim that OpenObserve is the closest open/self-hosted
structural threat to Parallax because it combines Rust, object storage,
self-hosting, observability, AI SRE/RCA, and MCP.

This pass treats every prior OpenObserve finding as provisional and checks the
current public sources most likely to falsify Parallax's wedge:

- free/self-hosted availability for AI and MCP;
- MCP tool scope and safety shape;
- Sentry-compatible ingestion or migration;
- portable evidence-bundle or schema claims;
- current release activity.

#### Verdict

OpenObserve remains the closest broad open/self-hosted competitor on
storage/runtime fit. It is stronger than an observability database comparison:
current public pages position it as a Rust, object-storage-oriented,
self-hostable full-stack observability product with AI Assistant, AI SRE/RCA,
incident workflows, and MCP.

That does **not** close the Parallax wedge today. The current checked sources
keep five important gaps open:

1. **AI/MCP are Enterprise-tier, not plain AGPL Community surfaces.** The SRE
   Agent setup requires an OpenObserve Enterprise license, an AI provider key,
   and `O2_AI_ENABLED=true`. The MCP docs say MCP support is Enterprise-only.
   This is not the same as a simple paywall, because Self-Hosted Enterprise has
   a public free tier; it is a licensing/tier boundary.
2. **The free Self-Hosted Enterprise allowance is source-conflicted.** Pricing
   and Enterprise feature docs say Self-Hosted Enterprise is free up to
   `50 GB/day`; the license/pricing docs also describe a `50 GB/day` free-tier
   license and a no-key `10 GB/day` fresh-install threshold. The homepage FAQ
   says `200 GB/day`. Do not collapse that into a single number until
   OpenObserve reconciles the public pages.
3. **The MCP surface is a broad management plane.** The public MCP catalog
   includes query tools, but also create/update/delete/admin tools for alerts,
   roles, dashboards, folders, functions, KV, org/system settings, pipelines,
   search jobs, service accounts, sourcemaps, streams, users, and log ingestion.
   Some tools are explicitly marked destructive. This is not the same as
   Parallax's intended read-only evidence-bundle projection.
4. **AI SRE now pressures the evidence-bundle story directly.** The AI SRE
   product page uses evidence-chain and audit-trail language around incident
   investigations, and describes context assembly over logs, metrics, traces,
   and dependency maps. That is a serious watch trigger, but the checked sources
   still do not expose a versioned portable artifact, schema, redaction report,
   query manifest, raw-ref policy, or outcome ledger.
5. **The checked ingestion docs still do not show Sentry migration.** Current
   OTLP docs show OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces; this
   pass did not find a Sentry envelope/DSN migration path in the OpenObserve
   docs search index.

Net: keep OpenObserve at `wedge_under_pressure`, not `wedge_closed`.

#### Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [OpenObserve `v0.90.2` release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2) | Latest checked GitHub release was published 2026-05-22. On 2026-05-25, `/releases/latest` returned HTTP `200` and resolved to `v0.90.2`; `git ls-remote` showed tag ref `308208f35c0a5d42da9f0e1798188cbbf46373fb`. | Active release train; do not treat OpenObserve as a static baseline. |
| [OpenObserve homepage](https://openobserve.ai/) | Positions OpenObserve as unified logs/metrics/traces/RUM with object storage, SQL/PromQL, one-binary or Helm deployment, AI SRE Agent, AI Assistant, and MCP. Homepage FAQ says Self-Hosted Enterprise is free up to `200 GB/day`, says the OSS plan has no usage limits, and says AI Assistant is included in Self-Hosted Enterprise and Cloud Enterprise. | Strong threat to broad positioning, but allowance conflicts with pricing/docs. Do not describe the Enterprise boundary as strictly paid at small volumes. |
| [OpenObserve pricing](https://openobserve.ai/pricing/), [Enterprise features](https://openobserve.ai/docs/features/enterprise/), and [license/pricing docs](https://openobserve.ai/docs/enterprise-setup/license-and-pricing/) | Pricing/docs say Self-Hosted Enterprise is free up to `50 GB/day`; the license docs say Enterprise works without a license key up to `10 GB/day`, requires requesting a license above that, and identifies the free-tier license as `<= 50 GB/day`. Pricing says AI-powered features are free during preview with credits. | Legitimate ops-feature comparable; exact free allowance remains unresolved. The right claim is "Enterprise-tier with a source-conflicted free allowance," not simply "paid." |
| [OpenObserve SRE Agent setup](https://openobserve.ai/docs/enterprise-setup/sre-agent/) | SRE Agent powers AI Assistant, incidents, and RCA in OpenObserve Enterprise; requires Enterprise license, AI provider key, and `O2_AI_ENABLED=true`. Setup docs list Anthropic, OpenAI, Gemini, direct, bundled gateway, and external/self-hosted gateway paths. | AI/RCA is real but not plain AGPL Community evidence. |
| [OpenObserve AI SRE product page](https://openobserve.ai/ai-sre/) | Says AI SRE is an OpenObserve Enterprise background service; describes context assembly over logs, metrics, traces, and dependency maps; presents evidence-chain/audit-trail investigation language; says the agent uses MCP to navigate OpenObserve tools; and supports OpenAI, Anthropic Claude, Gemini, AWS Bedrock, DeepSeek, OpenRouter, and OpenAI-compatible/self-hosted endpoints. | Provider flexibility and evidence-chain positioning are stronger than earlier notes; they increase threat but do not prove a portable evidence-bundle schema, export, redaction report, or outcome contract. |
| [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/) | MCP is Enterprise-only, uses `https://your-instance/api/{org_id}/mcp`, supports Claude Code/Cursor/VS Code/ChatGPT connectors and other agents, and exposes query plus broad management/destructive/admin tools. | Strongest current reason Parallax's first MCP surface must stay read-only and bundle-shaped. |
| [OpenObserve OTLP docs](https://openobserve.ai/docs/ingestion/logs/otlp/) | Supports OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces. | OTLP overlap is proven; Sentry-envelope/DSN migration was not found in checked ingestion docs. |
| [OpenObserve docs search index](https://openobserve.ai/docs/search/search_index.json) | Current index shape is an object with `.docs[]` entries containing `title`, `text`, and `location`. On 2026-05-25, exact search across those fields returned `0` matches for `sentry` and `0` matches for `portable evidence`, `evidence bundle`, `investigation bundle`, `query manifest`, `raw refs`, `raw-ref`, `redaction report`, or `outcome ledger`; broader `sre agent` / `ai sre` / `mcp` terms returned `65` matches. | Negative evidence only: it supports keeping Sentry migration and portable evidence-bundle export unproven, but should be rechecked because search indexes can lag source pages. |

#### Source Resolution Notes

This pass tightened the provenance of the highest-risk OpenObserve claims:

- **Release freshness.** The latest-release redirect and tag ref both point to
  `v0.90.2`, so the focused note is not relying only on a manually browsed
  release page.
- **Docs-search method.** Earlier checks assumed the search index was a top-level
  array. The current index is an object with `.docs[]`; the negative Sentry and
  bundle-schema checks now search `title`, `text`, and `location`.
- **MCP power boundary.** The MCP docs explicitly say Enterprise only and list
  query tools beside create/update/delete/admin tools. That keeps the Parallax
  comparison focused on safety shape, not merely whether MCP exists.
- **Evidence-chain pressure.** OpenObserve's AI SRE product page does use
  evidence-chain and reasoning-at-every-step language. The current docs search
  still did not expose a portable schema/export contract, so this remains a
  watch trigger rather than wedge closure.

#### Product Impact

The competitive risk is sharper than "OpenObserve might add AI." It already has
an Enterprise AI/RCA/MCP story, broad provider flexibility including
OpenAI-compatible/self-hosted endpoint language, and product-level evidence-chain
positioning. If it moves that layer into the free AGPL Community tier, adds
Sentry-compatible ingestion, or turns the AI SRE evidence-chain view into a
portable investigation/evidence artifact, Parallax's wedge narrows immediately.

The defensible Parallax boundary remains:

```text
Sentry-compatible error ingest
+ OTLP traces/logs/metrics
+ low-resource self-hosted operation
+ portable evidence bundles
+ deterministic evidence graph
+ CLI/read-only MCP context access
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fixer outcome loop
```

OpenObserve validates that observability systems are becoming agent-addressable.
It does not yet prove that agents should receive broad production-management MCP
tools by default, nor that query-time agent assembly is enough. Parallax should
continue to make the evidence contract the product: citable, redacted,
bounded, portable, and outcome-measurable.

#### Falsification Criteria

Reopen the Parallax verdict if OpenObserve does any of the following in current
primary sources:

- moves SRE Agent, AI Assistant, incident RCA, or MCP into the free AGPL
  Community tier;
- adds Sentry SDK/envelope ingestion or DSN-only Sentry migration;
- publishes a versioned portable investigation/evidence bundle with provenance,
  redaction report, query manifest, missing-evidence flags, and raw refs;
- turns AI SRE evidence-chain/audit-trail UI into an exportable schema or
  machine-readable artifact that coding agents can cite outside OpenObserve;
- adds coding-agent session, shell/CLI action, patch, PR, review, revert, or
  recurrence audit;
- publishes measured fixer/remediation outcome loops tied to evidence and PRs.

Until one of those triggers hits, the right posture is high vigilance, not
thesis failure.

## Bugsink Sentry-Compatible Simplicity Recheck
_Provenance: merged verbatim from `bugsink-sentry-compatible-simplicity-recheck.md` (2026-05-29 restructure)._

### Bugsink Sentry-Compatible Simplicity Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check Bugsink as the most credible lightweight Sentry-compatible simplicity
baseline. This pass tests whether Bugsink weakens Parallax's Phase 1 claim on:

- Sentry SDK / DSN migration;
- low-ops self-hosting;
- license posture;
- first-party or ecosystem MCP/agent access;
- portable evidence bundles, OTLP context, and fixer outcome feedback.

#### Verdict

Bugsink remains a high-pressure baseline for the narrow "Sentry-compatible and
easy to self-host" story. It does **not** close the Parallax wedge.

What strengthened:

1. **Migration is simple.** Bugsink's current Sentry SDK compatibility page says
   teams can keep the same Sentry SDKs and update the DSN; it explicitly
   supports the major official SDKs plus community SDKs such as Rust.
2. **The latest release improved the API.** `2.2.1`, published 2026-05-22,
   added canonical API issue actions, issue comment creation, friendly issue IDs,
   and OpenAPI endpoint documentation. That makes external tooling easier.
3. **Deployment pressure is real.** The Docker path can start a throwaway
   single-container instance with SQLite. The settings page says SQLite is the
   default and production-ready database outside the Docker-volume caveat, while
   MySQL and PostgreSQL are also supported through `DATABASE_URL`.
4. **MCP pressure now exists in the ecosystem.** Bugsink's official docs and
   repository still do not present a first-party MCP/AI agent surface, but two
   small third-party MCP adapters now exist in public sources: `bugsink-mcp` on
   npm and GitHub, and `j-shelfwood/bugsink-mcp`. They expose read/query tools
   over Bugsink issues, events, stack traces, teams, projects, and releases.

What still keeps Parallax distinct:

1. **Bugsink is source-available, not OSI-open.** The repository license is
   PolyForm Shield for most content, with noted third-party exceptions.
2. **Bugsink is intentionally error-tracking-only.** The comparison page says
   traces, performance monitoring, and session replay are not available and
   should generally be disabled in the SDK.
3. **The official product is not an evidence/context engine.** Checked sources
   do not show OTLP logs/traces/metrics correlation, portable evidence-bundle
   schema, redaction report, missing-evidence model, raw-ref policy, coding-agent
   action audit, or accepted/rejected/reverted fixer outcome rows.
4. **The third-party MCP adapters are small and query-shaped.** They prove that
   agents can reach Bugsink, but not that Bugsink has a mature first-party,
   read-only, redacted, citable evidence-bundle surface.

Net: Bugsink makes "change the DSN and self-host" a requirement, not a moat.
Keep it as the mature Sentry-compatible simplicity baseline.

#### Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [Bugsink `2.2.1` release](https://github.com/bugsink/bugsink/releases/tag/2.2.1) | Published 2026-05-22. Adds canonical API issue actions, issue comments, friendly issue IDs, and improved OpenAPI endpoint docs. | API surface is getting friendlier for external tools; do not assume Bugsink is UI-only. |
| [Bugsink docs](https://www.bugsink.com/docs/) and [repository](https://github.com/bugsink/bugsink) | Current docs describe a self-hosted error tracker compatible with the Sentry SDK; GitHub shows `2.2.1` as latest, about 1.8k stars, and Python/Django implementation. | Mature enough to benchmark as a real baseline. |
| [Sentry SDK compatibility](https://www.bugsink.com/sentry-sdk-compatible/) | Supports Sentry SDKs for Python, JavaScript, Ruby, PHP, Java, Go, Rust, and more; migration is keep code, update DSN, done. | Parallax must not claim DSN migration as unique. |
| [Docker install](https://www.bugsink.com/docs/docker-install/) and [settings](https://www.bugsink.com/docs/settings/) | Throwaway Docker path is one container with SQLite and no persistence. Docker docs recommend MySQL for retained data; PostgreSQL can probably work but is not extensively tested. Settings docs say SQLite is the default production-ready database and MySQL/PostgreSQL are supported through `DATABASE_URL`; Docker volumes are not recommended for SQLite WAL mode. | Simplicity claim must separate demo startup, persistent Docker, and non-container SQLite deployment. |
| [Built to self-host](https://www.bugsink.com/built-to-self-host/) | Positions self-hosting as data-control and privacy protection, with minimal setup and no external dependencies. | Same buyer psychology as Parallax; the differentiator must be richer evidence, not data ownership alone. |
| [Sentry vs Bugsink](https://www.bugsink.com/sentry-vs-bugsink/) | Bugsink is a focused crash reporter, not a full observability platform; it says traces, performance monitoring, and session replay are unavailable/ignored. It claims no Redis, queue, or ingestion pipeline and gives vendor scale numbers for a small VPS. | Treat Bugsink performance/scale numbers as vendor claims until measured, but count the product-scope gap as current. |
| [Bugsink license](https://github.com/bugsink/bugsink/blob/main/LICENSE) | Most repository content is under PolyForm Shield; `sentry/` is BSD-3-Clause inherited content and other exceptions are listed. | Good self-hosted baseline, but not proof that Parallax's OSI-open thesis is crowded. |
| [`draded/bugsink-mcp`](https://github.com/draded/bugsink-mcp) and [`bugsink-mcp` on npm](https://www.npmjs.com/package/bugsink-mcp) | npm package `bugsink-mcp` is `1.0.0`, MIT, points at `draded/bugsink-mcp`, and exposes teams/projects/issues/events/stacktraces/releases tools. The GitHub repo has no stars or releases at check time. | Third-party MCP exists but is low-maturity and not first-party Bugsink. |
| [`j-shelfwood/bugsink-mcp`](https://github.com/j-shelfwood/bugsink-mcp) | MIT repository with 6 stars, no releases, last pushed 2026-01-12, exposing Bugsink project/team/issue/event query tools. | Additional evidence that Bugsink data is easy to expose to agents; not a first-party closure. |

#### Product Impact

Bugsink makes the Phase 1 floor explicit:

```text
Parallax must accept Sentry SDK events/envelopes with a DSN-style migration,
and its tiny tier must be close enough to Bugsink's first useful error-capture
workflow that the extra context engine is defensible.
```

The answer is not to copy Bugsink's narrower product. Bugsink deliberately avoids
traces, performance monitoring, and session replay. Parallax should use Bugsink
as the error-only simplicity bar, then justify extra machinery only when it
produces:

- correlated OTLP context;
- portable evidence bundles;
- redaction/source-policy reports;
- read-only agent projections;
- coding-agent action and fixer outcome records.

#### Falsification Criteria

Reopen the Parallax verdict if Bugsink or its ecosystem:

- adds first-party MCP or agent tools with read-only, redacted, citable bundle
  output;
- adds OTLP logs/traces/metrics correlation around Sentry issues;
- publishes a portable evidence-bundle schema, query manifest, or raw-ref
  policy;
- adds coding-agent command/file/patch/test/PR audit or fixer outcome rows;
- changes to an OSI-open license while retaining low-ops Sentry compatibility;
- produces independently reproducible low-resource benchmark artifacts that
  cover a comparable context surface.

Until then, Bugsink is the mature error-only simplicity bar, not the full
Parallax product.

## Rustrak Sentry MCP Protocol Recheck
_Provenance: merged verbatim from `rustrak-sentry-mcp-protocol-recheck.md` (2026-05-29 restructure)._

### Rustrak Sentry MCP Protocol Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Purpose

Re-check Rustrak because it is the closest lightweight product-shape warning for
Parallax:

```text
Rust + Sentry-compatible ingest + low-ops self-hosting + MCP
```

This pass tests whether Rustrak has closed the Parallax wedge or only raised the
minimum bar for Phase 1.

#### Short Verdict

Rustrak is a serious product-shape baseline, but not a wedge closer.

It proves that Rust-first, Sentry-compatible, low-footprint error tracking with
MCP can exist as an open project. Parallax should therefore stop treating any of
these as differentiators on their own:

- Rust server implementation;
- Sentry SDK DSN migration language;
- SQLite-first or low-process self-hosting;
- MCP availability.

The remaining Parallax gap is still meaningful:

```text
Sentry-compatible errors
+ OTLP traces/logs/metrics
+ deterministic evidence bundles
+ read-only, redacted, citable agent projections
+ CLI/coding-agent action audit
+ accepted/rejected/reverted fix outcome loop
```

#### What Changed Or Was Rechecked

| Source | Current evidence | Parallax read |
| --- | --- | --- |
| [Rustrak repository](https://github.com/AbianS/rustrak) | GitHub metadata checked 2026-05-25 shows 43 stars, 6 forks, 30 open issues/PRs combined by the repo API, default branch `main`, and latest repo push on 2026-05-25. The README describes an ultra-lightweight self-hosted tracker compatible with Sentry SDKs. | Treat Rustrak as active and relevant, not as a stale toy project. |
| [Rustrak releases](https://github.com/AbianS/rustrak/releases) | Generic latest release is `docs@0.1.16` on 2026-05-21, while the server package release is [`@rustrak/server@0.2.5`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/server%400.2.5) on 2026-05-21. [`@rustrak/mcp@0.1.2`](https://github.com/AbianS/rustrak/releases/tag/%40rustrak/mcp%400.1.2) was published on 2026-05-17. | Pin component releases, not only `releases/latest`, or the benchmark will accidentally pin docs. |
| [Rustrak installation docs](https://abians.github.io/rustrak/getting-started/installation) and [database docs](https://abians.github.io/rustrak/configuration/database) | SQLite is the default image, data persists through a Docker volume, PostgreSQL uses a separate `:postgres` image, and docs recommend SQLite for personal/low-medium traffic under about 1,000 events/hour. Production can run server-only and put the UI elsewhere. | Low-ops pressure is real, but the benchmark must separate SQLite personal use, Postgres production, server-only, and full UI modes. |
| [Docker Hub server image](https://hub.docker.com/r/abians7/rustrak-server) | Docker Hub API shows `abians7/rustrak-server` last updated 2026-05-21, about 1.6k pulls, `latest`/`v0.2.5` images, and linux amd64/arm64 image sizes around 16-17 MB. | The small-image claim is currently supported by registry metadata. Do not convert that into unmeasured memory or ingestion-throughput proof. |
| [`@rustrak/mcp` docs](https://abians.github.io/rustrak/sdks/mcp), [npm](https://www.npmjs.com/package/@rustrak/mcp), and [package README](https://github.com/AbianS/rustrak/tree/main/packages/mcp) | `@rustrak/mcp` is `0.1.2`, GPL-3.0, Node >=18, stdio, and exposes 18 tools across projects, issues, events, tokens, and alerts. Docs describe it as giving AI assistants control of Rustrak. | MCP presence is table stakes. The checked surface is management/raw-event shaped, not a Parallax-style read-only evidence-bundle surface. |
| [MCP issue tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/issues.ts), [event tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/events.ts), [token tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/tokens.ts), and [alert tools](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/alerts.ts) | Source includes issue state changes, `delete_issue` with `destructiveHint`, raw event detail access, token creation/revocation, and alert test sends. | Parallax's first MCP should stay read-only and bundle/projection based; broad CRUD would weaken the safety distinction. |
| [Ingest route source](https://github.com/AbianS/rustrak/blob/main/apps/server/src/routes/ingest.rs) and [envelope parser source](https://github.com/AbianS/rustrak/blob/main/apps/server/src/ingest/parser.rs) | Current `main` parses Sentry envelopes, accepts `/api/{project_id}/envelope/`, validates event UUIDs, stores only the first item with type `event`, and returns an error for deprecated `/store/`. Parser size limits are present. | "Sentry compatible" is strongest for modern envelope error events, not yet full Sentry protocol coverage. |
| [Rustrak Sentry drift report](https://github.com/AbianS/rustrak/blob/main/docs/sentry-compat/2026-05-11-drift-report.md) | Rustrak's own report says core event-envelope compliance is solid, but session, transaction, client_report, and attachment items are silently discarded; standalone span data is protocol-safe to ignore but not stored. | Rustrak has a credible protocol discipline, but it still lacks broad context capture that Parallax needs for evidence bundles. |
| [`feat: sentry agent` commit](https://github.com/AbianS/rustrak/commit/b29258447523f7cdb0d3fcf763a7313b33c17830) | Current `main` includes an unreleased repo-maintenance agent workflow: `.claude/skills/agent-rusty`, `_bmad/_memory/agent-rusty`, and a `sentry-protocol-drift` skill. This appears to be maintainer workflow/tooling, not a user-facing runtime feature. | Do not count it as product AI closure, but do count it as a warning that Rustrak is actively operationalizing protocol-drift research. |
| [License](https://github.com/AbianS/rustrak/blob/main/LICENSE) | Repository license is GPL-3.0; package metadata for `@rustrak/mcp` and `@rustrak/client` also reports GPL-3.0. | Stronger open-source posture than Bugsink/Urgentry, but GPL may be less compatible with Parallax's likely Apache-2.0 business posture. |

#### Implications For Parallax

1. **Rustrak is the Rust/Sentry/MCP floor.** If Parallax's first artifact only
   offers Rust ingest, DSN migration, and MCP, it is not sufficiently distinct.
2. **Protocol fixture coverage matters.** Rustrak already keeps a Sentry protocol
   drift report and E2E SDK tests. Parallax needs fixture-gated Sentry envelope
   compatibility, not vague "Sentry-compatible" language.
3. **Agent access must be safer than management MCP.** Rustrak's MCP validates
   demand for agent-side issue lookup, but also shows why first Parallax MCP
   should avoid project/token/alert/issue mutation.
4. **Context breadth is still open.** Rustrak's checked sources do not show OTLP
   traces/logs/metrics correlation, session/release health, transaction/span
   storage, evidence bundles, source/redaction policy, CLI action audit, or
   fix-outcome records.
5. **Do not over-weight vendor performance claims.** Registry image size is
   source-checkable; memory, P99 latency, and events/second should stay marked
   as vendor claims until a benchmark artifact measures them.

#### Falsification Triggers

Reopen the Parallax verdict if Rustrak:

- stores transaction/span/session/client_report/attachment data and correlates it
  into issue context;
- adds OTLP logs/traces/metrics ingestion or correlation;
- replaces raw-event MCP with a read-only, redacted, citable evidence-bundle
  schema;
- adds projection-equivalence hashes across CLI/API/MCP outputs;
- records coding-agent file/command/patch/test/deploy actions and outcomes;
- proves broad SDK/envelope compatibility through published fixtures and
  conformance reports;
- gains enough adoption or release maturity that it is no longer only an early
  product-shape warning.

#### Bottom Line

Rustrak narrows Parallax's safe wording:

```text
Not: "open Rust Sentry-compatible error tracking with MCP"
Yes: "open runtime evidence/context engine that starts with Sentry-compatible
errors, adds OTLP context, and gives agents bounded evidence bundles plus
action/outcome audit"
```

Rustrak is now the live Rust-first comparison row for Phase 1. Parallax should
use it as a baseline and beat it on evidence semantics, not on framework choice.

## Traceway OTLP AI Replay Recheck
_Provenance: merged verbatim from `traceway-otlp-ai-replay-recheck.md` (2026-05-29 restructure)._

### Traceway OTLP AI Replay Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check Traceway because it pressures the side of Parallax that is easy to
understate when the research focuses on Sentry-compatible error trackers:

```text
direct OTLP ingest
+ logs/traces/metrics correlation
+ exceptions/issues
+ session replay/RUM
+ AI trace capture
+ low-friction self-hosted and embedded modes
```

This pass tests whether Traceway has closed the Parallax wedge or whether it
raises the minimum bar for OTLP-native context while leaving Sentry migration,
evidence bundles, and agent action audit open.

#### Short Verdict

Traceway is a stronger OTLP/context competitor than the previous watch row
proved. The relevant evidence is not only marketing copy: current source shows
OTLP/HTTP routes for traces, metrics, and logs; code converts spans into
endpoints, tasks, exceptions, generic spans, and AI traces; docs describe direct
SDK export without a required Collector; and the SQLite deployment path is a
single container with local blob storage or optional S3.

Traceway does **not** close the Parallax wedge in the checked sources:

- no Sentry-compatible envelope ingest or DSN migration path was found in the
  checked tree/docs;
- no MCP server or read-only evidence-bundle agent surface was found;
- no canonical, portable, redacted evidence-bundle schema was found;
- no coding-agent command/file/patch/test/action audit or fix-outcome loop was
  found;
- it is Go/Svelte, not Rust-first.

The product implication is narrower and sharper:

```text
Parallax cannot sell "OTLP-native, self-hosted, no Collector required" as a
complete differentiator. Traceway already pressures that shape. Parallax's
defensible gap is Sentry-compatible migration plus OTLP context plus
agent-safe, citable evidence bundles and action/outcome audit.
```

#### Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [Traceway repository](https://github.com/tracewayapp/traceway) and [latest backend release](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27) | GitHub metadata checked 2026-05-25 shows MIT license, 817 stars, 23 forks, 20 open issues/PRs combined by the repo API, latest push on 2026-05-25, latest `main` commit `38b8d385`, and latest release `backend/v1.7.27` published 2026-05-22. The release notes include distributed-trace handling from OTel, a Helm chart, and pre-built Docker image docs. | Treat Traceway as active and strategically relevant, not as a stale README-only signal. |
| [Route source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/routes.go) | `POST /api/report` is the native Traceway client protocol. `POST /api/otel/v1/traces`, `/v1/metrics`, and `/v1/logs` are registered with bearer-token client auth. Dashboard/admin endpoints also expose projects, widgets, metrics query/discovery, endpoint/task/session detail, AI traces, distributed traces, logs, exceptions, auth/OAuth, org/member/invite management, source-map upload, notification channels/rules/history, and archive/unarchive operations. | Traceway has a broad dashboard API and real OTLP ingress. Its HTTP API is not a Parallax-style least-privilege evidence-bundle surface. |
| [OTLP codec source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/codec.go) and [OTel docs](https://docs.tracewayapp.com/client/otel) | OTLP/HTTP accepts protobuf or JSON, supports gzip, has a 10 MB body limit, uses `/api/otel` as the base endpoint, and documents direct SDK export without a required Collector. | Parallax's OTLP receiver must meet this low-friction direct-export bar while adding stronger failure semantics, redaction, and bundle projection. |
| [OTLP controller source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/otel.controller.go) | Trace ingest converts batches and inserts endpoints, tasks, spans, exceptions, and AI traces; metrics ingest inserts metric points and auto-registers metric metadata; log ingest inserts log records; all three record ingest monitoring and enforce report rate limits. | This is real ingest logic, not only a docs promise. Parallax should compete above the conversion layer with evidence semantics. |
| [Trace converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/trace_converter.go) | SERVER/INTERNAL HTTP spans become endpoints, CONSUMER spans become tasks, `exception` span events become issue stack traces, child spans are linked to owning entities, and any span with `gen_ai.*` attributes becomes an AI trace. AI trace conversation content is written to object storage under `ai-traces/<project>/<trace>.json`. | Traceway already turns OTel into product concepts and AI traces. Parallax must preserve broader causality, source policy, redaction, and action/outcome rows rather than only "store spans." |
| [Logs converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/logs_converter.go) and [logs docs](https://docs.tracewayapp.com/client/otel/logs) | OTLP logs preserve trace/span IDs, severity, service/resource/scope/log attributes, and docs say logs emitted inside active spans link to the originating trace/span and appear in trace detail views. | Trace-linked logs are a baseline feature for the Parallax bundle, not a differentiator. |
| [Metrics converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/metric_converter.go) | Gauge, sum, and histogram data points are converted; histogram support stores average/count points; selected process resource attributes are allowlisted into metric tags. | Metrics ingest exists, but this pass did not benchmark storage or query performance. Keep metric-cost and scale claims unmeasured. |
| [AI tracing docs](https://docs.tracewayapp.com/learn/ai-tracing), [OpenRouter guide](https://docs.tracewayapp.com/client/openrouter), and [OpenRouter golden fixture](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/testdata/openrouter_ai_trace.json.golden.json) | Docs state spans with `gen_ai.*` semantic attributes are promoted to AI traces whether root or child spans; conversation content is stored in S3 or local filesystem; OpenRouter can export OTLP traces directly; the checked golden fixture records model, provider, operation, token counts, costs, finish reason, and conversation count. | AI-call observability is not enough for Parallax's agent-session wedge. Parallax needs command/tool/file/approval/test/patch/outcome context around agents, not only LLM call cost and prompt/completion traces. |
| [Protocol spec](https://docs.tracewayapp.com/protocol) | The native `/api/report` protocol accepts traces, exceptions, metrics, sessions, and session recordings. Frontend/mobile SDKs use this protocol where OTel does not yet cover session replay. | Traceway's session replay/RUM path pressures Parallax's frontend roadmap, but it is not Sentry-envelope compatibility. |
| [Embedded mode docs](https://docs.tracewayapp.com/learn/embedded-mode) and [embedded Go example](https://github.com/tracewayapp/traceway/blob/main/examples/embedded-backend-otel/main.go) | A Go app can call `tracewaybackend.Run`, seed a default user/project, and export OTel traces to the embedded server at `/api/otel/v1/traces`. Docs call embedded mode development-only and use SQLite by default, in memory unless `WithSQLitePath` is set. | Parallax local/dev UX must be honest against this bar. Embedded dev mode is strong, but it does not prove production retention, backup, or agent-bundle parity. |
| [Self-host docs](https://docs.tracewayapp.com/server), [SQLite docs](https://docs.tracewayapp.com/server/sqlite), [Docker Compose](https://github.com/tracewayapp/traceway/blob/main/docker-compose.yml), [SQLite Compose](https://github.com/tracewayapp/traceway/blob/main/docker-compose.sqlite.yml), and [Docker signatures](https://github.com/tracewayapp/traceway/blob/main/DOCKER_SIGNATURES.md) | Deployment options include all-in-one, Docker Compose, minimal external-db, SQLite, local setup, and Haloy. Root Compose declares `traceway`, `clickhouse`, and `postgres`. SQLite mode is a single Alpine container with two SQLite files plus local blobs under `/data`, optional S3 for blobs, and retention knobs. Image-size and signed-image claims are documented, not independently measured here. | The simplicity baseline must count both visible services and hidden bundled subsystems. SQLite mode is the relevant low-friction comparison; image size and performance claims remain unmeasured unless benchmark artifacts exist. |
| [Integration skills tree](https://github.com/tracewayapp/traceway/tree/main/skills) and [add Traceway skill](https://github.com/tracewayapp/traceway/blob/main/skills/add-traceway.md) | The tree has integration-instruction files for adding Traceway to apps. The generic skill tells coding assistants how to ensure `http.route`, status codes, exception events, and CONSUMER task spans. It also says Go/frontend SDKs use `/api/report` while generic OTel uses `/api/otel`. Tree-path checks found no `mcp`, `sentry`, `claude`, `cursor`, or `codex` product/tool folders besides `skills/`; content hits for `sentry`, `dsn`, and `envelope` were comparison/design/test/framework references, not a Sentry-compatible ingest surface. | Traceway uses agent-readable integration guidance, but that is not an MCP/data-access/evidence-bundle surface. |
| [Creator note](https://github.com/tracewayapp/traceway/blob/main/HN.md) | Maintainer states goals around simple/cheap hosting, sub-15-developer teams, no paid add-ons, ClickHouse base with SQLite for self-hosting, sessions in S3, no AI SRE upsell, and frontend/mobile custom protocol because current frontend/mobile OTel does not cover session replay. | Useful product-intent evidence, but lower weight than shipped source. It reinforces the small-team simplicity threat. |
| [Traceway OTel Agent docs](https://docs.tracewayapp.com/learn/otel-agent) and [agent repository](https://github.com/tracewayapp/traceway-otel-agent) | Docs describe a small preconfigured OTel Collector service for host metrics every 60 seconds, optional logs and process metrics, checksum-verified installers, and self-hosted endpoint override. GitHub metadata checked 2026-05-25 shows 3 stars, no license in the repo API, and latest push on 2026-04-28. | Host telemetry agent is relevant but much less mature than the core Traceway repo; do not treat it as a mature agent-observability or action-audit surface. |

#### What Changed Or Was Narrowed

1. **Traceway is no longer only a README-level watch item.** Source and docs
   show direct OTLP ingest, conversion logic, trace-linked logs, metrics, AI
   traces, native `/api/report`, session replay, and multiple self-hosted modes.
2. **Traceway's strongest pressure is not Sentry migration.** It pressures the
   OTLP-native unified-context, AI-trace, frontend/session, and embedded/local
   developer experience parts of Parallax.
3. **The "no Collector required" bar is higher.** Traceway documents direct SDK
   export and optional Collector use. Parallax's tiny tier should do the same,
   but with explicit redaction, missing-evidence, and projection-equivalence
   guarantees.
4. **The agent gap remains open.** Integration skills are instructions for
   coding assistants to add instrumentation. They are not an MCP server, CLI
   context surface, or read-only evidence-bundle schema.
5. **Deployment claims need mode separation.** Compose, all-in-one, minimal,
   SQLite, and embedded mode have different hidden dependencies and persistence
   semantics. Benchmark comparisons must not collapse them into "one container."

#### Parallax Impact

Traceway raises the Phase 1 bar in four places:

- OTLP/HTTP traces, metrics, and logs should be accepted directly from SDKs
  without requiring a Collector.
- OTel exception events and trace-linked logs should appear in the first useful
  issue context if available.
- SQLite or similarly low-friction local/dev modes need clear persistence,
  backup, and retention semantics.
- AI traces should be treated as ordinary runtime evidence, but not as a
  substitute for agent-session audit.

The wedge is still not closed because Traceway does not prove:

- Sentry SDK/envelope/DSN migration for existing Sentry users;
- canonical evidence bundles with hashes, raw refs, source policy, redaction
  reports, and missing-evidence rows;
- projection-equivalent CLI/HTTP/MCP/Markdown/JSON output;
- read-only agent context tools;
- coding-agent side-effect audit across commands, files, tests, patches,
  approvals, PRs, deploys, and reversions;
- accepted/rejected/reverted fixer outcome rows.

#### Falsification Triggers

Reopen the Parallax verdict if Traceway:

- adds Sentry envelope or DSN-compatible ingest for current Sentry SDKs;
- publishes a portable evidence-bundle schema with source/raw-ref/redaction
  policy and projection hashes;
- adds a read-only CLI/MCP/HTTP evidence surface that coding agents can consume
  safely;
- adds coding-agent command/file/patch/test/PR/deploy audit and fix-outcome
  writeback;
- makes session replay plus backend traces plus AI traces export as a citable
  incident dossier rather than only dashboard views;
- publishes independently reproducible setup/resource/throughput benchmarks
  that cover the same first-use evidence surface Parallax targets.

#### Bottom Line

Traceway is now the live OTLP-native/self-hosted/context-product comparison row.
It does not make Parallax unnecessary, but it makes weak wording fail:

```text
Not: "self-hosted OTLP observability with AI traces"
Yes: "Sentry-compatible runtime evidence engine that accepts OTLP, correlates
errors/logs/traces/metrics/frontend/agent execution, and returns redacted,
citable bundles with action/outcome history for coding agents"
```

## GoSnag Sentry AI MCP Recheck
_Provenance: merged verbatim from `gosnag-sentry-ai-mcp-recheck.md` (2026-05-29 restructure)._

### GoSnag Sentry AI MCP Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Purpose

Re-check GoSnag because it is the broadest lightweight feature-vector warning in
the Sentry-compatible competitor set:

```text
Sentry SDK ingest + issue workflow + AI RCA/triage + MCP
```

This pass tests whether GoSnag has closed the Parallax wedge or whether it only
proves that "AI over self-hosted Sentry-compatible errors" is becoming a
commodity product shape.

#### Short Verdict

GoSnag is a real capability warning, not a mature wedge closer.

The source-level check supports the core shape: the repository contains Sentry
`/store/` and `/envelope/` ingest paths, stores raw Sentry event JSON, has AI
workers and manual AI endpoints for RCA/merge/deploy/ticket/priority/tag/alert
workflows, and ships a TypeScript MCP server over the management API. But the
checked project has no releases/tags, low visible traction, requires PostgreSQL
in the normal Docker Compose path, ignores Sentry transactions/sessions/client
reports, and exposes MCP as issue/project/ticket management rather than a
bounded read-only evidence-bundle projection.

The durable Parallax distinction remains:

```text
Sentry-compatible errors
+ OTLP logs/traces/metrics
+ deterministic redacted evidence bundles
+ read-only CLI/API/MCP projections with the same bundle hash
+ coding-agent and CLI action audit
+ accepted/rejected/reverted fix outcome loop
```

#### What Changed Or Was Rechecked

| Source | Current evidence | Parallax read |
| --- | --- | --- |
| [GoSnag repository](https://github.com/darkspock/gosnag) | GitHub page and API checks on 2026-05-25 show `darkspock/gosnag`, MIT license, roughly 8 stars and 4 forks, default branch `main`, 136 commits, no published releases, and no tags. The latest checked `main` commit is [`418b8b1`](https://github.com/darkspock/gosnag/commit/418b8b107e274bfaab3f905510ddd274173d216b), dated 2026-04-17. | Treat GoSnag as a moving-target capability warning, not a stable benchmark release. Pin a commit if it is used in comparisons. |
| [README](https://github.com/darkspock/gosnag) and [router source](https://github.com/darkspock/gosnag/blob/main/cmd/gosnag/router.go) | README claims Sentry SDK compatibility, legacy `/store/` and modern `/envelope/`, single Go binary with embedded React UI/migrations, issue workflow, GitHub/Jira, tickets, AI, and MCP. Router source confirms `POST /api/{project_id}/store/` and `POST /api/{project_id}/envelope/`, plus a broad `/api/v1` management API. | The Sentry-compatible issue-tracker posture is real enough to count. The source does not make it an OTLP evidence engine. |
| [Ingest handler](https://github.com/darkspock/gosnag/blob/main/internal/ingest/handler.go), [event parser](https://github.com/darkspock/gosnag/blob/main/internal/ingest/event.go), [envelope parser](https://github.com/darkspock/gosnag/blob/main/internal/ingest/envelope.go), and [auth helper](https://github.com/darkspock/gosnag/blob/main/internal/ingest/auth.go) | `/store/` parses one JSON event; `/envelope/` loops items and stores `event` items. `transaction` is explicitly out of scope; `session`, `sessions`, and `client_report` are silently ignored. Event parsing covers exception, stack frames, tags, extra, user, request, contexts, breadcrumbs, SDK, modules, release, environment, and stores raw JSON. Auth extracts a Sentry public key from `X-Sentry-Auth` or `sentry_key`. Payload reads are gzip/deflate-aware and capped at 1 MiB. | "Sentry-compatible" should be narrowed to error-event ingest. It is not evidence of transaction/span/session coverage or full Sentry protocol parity. |
| [AI provider source](https://github.com/darkspock/gosnag/blob/main/internal/ai/provider.go), [AI service](https://github.com/darkspock/gosnag/blob/main/internal/ai/service.go), [RCA source](https://github.com/darkspock/gosnag/blob/main/internal/ai/rca.go), and [deploy analyzer](https://github.com/darkspock/gosnag/blob/main/internal/ai/deploy.go) | Implemented provider switch covers OpenAI-compatible OpenAI, OpenAI-compatible Groq, and AWS Bedrock. The README/config text lists Claude and Ollama too, but no direct Anthropic or Ollama provider implementation was found in the checked provider switch. AI calls have daily token budgets, calls-per-minute limits, prompt-hash caching, and usage logging. RCA prompts combine issue metadata, latest stack trace, breadcrumbs, tags, similar issues, and recent deploys. Deploy analysis waits 15 minutes and compares pre/post windows. | GoSnag has real AI-assisted triage mechanics, but its "evidence" is model-generated strings over selected DB context, not a canonical, citable evidence bundle with raw references, redaction manifest, missing-evidence fields, and projection hashes. |
| [Merge source](https://github.com/darkspock/gosnag/blob/main/internal/ai/merge.go), [priority evaluator](https://github.com/darkspock/gosnag/blob/main/internal/priority/evaluator.go), and [ticket description source](https://github.com/darkspock/gosnag/blob/main/internal/ai/description.go) | AI can suggest or auto-merge duplicate issues, evaluate custom priority rules once per issue/rule, and generate sanitized HTML ticket descriptions from issue context. Auto-merge can mutate issue/event records when enabled. | This is issue-workflow automation, not the Parallax fixer boundary. It reinforces the need to keep Parallax core read-only and move outcome writes into a separate append-only path. |
| [MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts) and [MCP package](https://github.com/darkspock/gosnag/blob/main/mcp/package.json) | `gosnag-mcp` is version `1.0.0`, TypeScript, stdio, and depends on `@modelcontextprotocol/sdk`. It calls `/api/v1` with `Authorization: Bearer ${GOSNAG_TOKEN}`. Tools include `list_projects`, `get_project`, `create_project`, `update_project`, `delete_project`, `list_issues`, `get_issue`, `update_issue_status`, `get_issue_events`, `get_issue_counts`, `list_alerts`, `create_alert`, `list_issue_tags`, `add_issue_tag`, `list_users`, `create_ticket`, `get_ticket`, `update_ticket`, `list_tickets`, and `get_ticket_counts`. | MCP is table stakes. GoSnag's checked MCP is a management/write surface over issues, alerts, projects, tags, tickets, and users, not a read-only evidence bundle with least-privilege schema and redaction proof. |
| [Docker Compose](https://github.com/darkspock/gosnag/blob/main/docker-compose.yml), [Dockerfile](https://github.com/darkspock/gosnag/blob/main/Dockerfile), [go.mod](https://github.com/darkspock/gosnag/blob/main/go.mod), and [.env example](https://github.com/darkspock/gosnag/blob/main/.env.example) | Compose declares `gosnag` and `db` services, Postgres 16, resource limits, and required `DATABASE_URL`. Dockerfile builds frontend with Node 20 and backend with Go 1.25 into an Alpine runtime. `.env.example` includes core/auth/SMTP/Slack variables but omits the AI variables listed in README/config source. | Low process count is real, but the default persistent path is Postgres-backed. The config/docs gap around AI setup is another maturity warning. |

#### Implications For Parallax

1. **"AI over errors" is not enough.** GoSnag already has AI RCA, deploy
   anomaly analysis, priority/tag/alert suggestions, ticket descriptions, and
   merge suggestions in a lightweight Sentry-compatible tracker.
2. **Sentry compatibility must be fixture-scoped.** GoSnag currently stores
   error events and raw event JSON, while transactions, sessions, and
   client reports are ignored. Parallax should name exactly which Sentry
   envelope items it accepts, stores, correlates, and exposes in bundles.
3. **MCP power must stay bounded.** GoSnag proves MCP can arrive early in a
   small error tracker, but its tool surface mutates projects, statuses, alerts,
   tags, and tickets. Parallax's first context MCP should avoid this shape.
4. **Evidence semantics remain the wedge.** GoSnag RCA is a generated answer
   from issue context. Parallax must produce a portable failure dossier with
   source labels, redaction results, raw references, missing-data warnings, and
   stable hashes before asking an agent to reason over it.
5. **Maturity weighting matters.** No releases/tags and low traction mean
   GoSnag should not be weighted like Bugsink or Traceway in market adoption
   evidence, but it still shows where small competitors can move.

#### Falsification Triggers

Reopen the Parallax verdict if GoSnag:

- publishes stable releases and fixture-backed Sentry protocol compatibility;
- stores transactions, spans, sessions, client reports, and attachments instead
  of ignoring them;
- adds OTLP traces/logs/metrics correlation or an OTel-native ingest path;
- replaces management MCP with read-only, redacted, citable evidence bundles;
- publishes projection-equivalence hashes across API/MCP/CLI/bundle outputs;
- records coding-agent commands/files/patches/tests/approvals and PR outcomes;
- makes AI RCA cite raw evidence fields and missing-evidence records by schema.

#### Bottom Line

GoSnag narrows Parallax wording:

```text
Not: "self-hosted Sentry-compatible errors with AI and MCP"
Yes: "runtime evidence/context engine that uses Sentry-compatible errors as one
input, correlates OTLP and execution traces, and gives agents bounded citable
context plus an action/outcome audit trail"
```

Until GoSnag has releases and broader protocol/context coverage, it remains a
feature-vector warning. Its main value for Parallax is negative space: do not
build a GoSnag-style issue tracker and call it an evidence engine.

## Urgentry Sentry Tiny Benchmark Recheck
_Provenance: merged verbatim from `urgentry-sentry-tiny-benchmark-recheck.md` (2026-05-29 restructure)._

### Urgentry Sentry Tiny Benchmark Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check the existing Urgentry claim from primary sources because prior Parallax
notes treated it mostly as "Tiny mode plus benchmark claims." That was too
thin: Urgentry may be the strongest lightweight pressure on Parallax's
Sentry-compatible simplicity story, and its vendor benchmarks are easy to
over-read without source-level limits.

#### Short Verdict

Urgentry is a stronger Sentry-replacement warning than the previous watchlist
said. Source confirms a broad Sentry-style ingest and product surface: legacy
`/store/`, envelopes, minidumps, security reports, OTLP HTTP/JSON traces, logs,
and metrics, plus envelope side effects for transactions, sessions, replays,
profiles, client reports, check-ins, attachments, and metric buckets.

It does **not** close the Parallax wedge:

- license is FSL-1.1-ALv2 source-available, not OSI-open;
- benchmark numbers are vendor claims and were not reproduced in this pass;
- OTLP is HTTP/JSON only in checked source, with protobuf explicitly rejected;
- no MCP server was found in checked README/docs/source;
- the `autofix` API is a deterministic compatibility/stub-like surface that
  records completed summaries and skipped PR behavior, not a real AI/coding
  agent fixer loop;
- no portable evidence-bundle schema, source-policy manifest, redaction report,
  projection hash, missing-evidence model, or action/outcome audit was found.

Parallax should treat Urgentry as a high-quality simplicity baseline and a
Sentry-protocol coverage challenge, not as proof that the open evidence-context
engine thesis is closed.

#### Version And Source Snapshot

| Field | Current check |
| --- | --- |
| Repository | [urgentry/urgentry](https://github.com/urgentry/urgentry) |
| Latest release | [`v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), published 2026-05-22 |
| Latest checked `main` | [`ccc0ff815ec8b19d3b7c820b95bc3d539414e145`](https://github.com/urgentry/urgentry/commit/ccc0ff815ec8b19d3b7c820b95bc3d539414e145), dated 2026-05-22 |
| Visible traction at check time | Roughly 55 GitHub stars and 5 forks |
| License | [FSL-1.1-ALv2](https://github.com/urgentry/urgentry/blob/main/LICENSE), with Apache-2.0 future-license language |
| Runtime/build note | `go.mod` declares Go `1.26.0`; quickstart says Go 1.26+ |

#### Ingest Surface Checked

The checked HTTP server mounts these ingest endpoints when the ingest role is
enabled:

| Route family | Source-level read |
| --- | --- |
| Sentry event/store | `POST /api/{project_id}/store/` accepts legacy JSON events and returns an event id. |
| Sentry envelope | `POST /api/{project_id}/envelope/` parses envelopes, queues `event` and `transaction` items, applies transaction sampling when configured, then persists side effects. |
| Native/security reports | `minidump`, `unreal`, `security`, `csp-report`, and `nel` routes are registered. |
| OTLP | `POST /api/{project_id}/otlp/v1/{traces,logs,metrics}/` routes exist. Checked handlers accept JSON and reject `application/x-protobuf` with `415`; no gRPC receiver was found in this pass. |

Envelope side effects are materially broader than Rustrak or GoSnag in their
current checked forms. Urgentry handles or stores:

- `event` and `transaction` through the pipeline;
- `user_report`;
- `attachment`;
- `replay_event`, `replay_recording`, `replay_recording_not_chunked`, and
  `replay_video`;
- `profile`;
- `client_report`;
- `session` and `sessions`;
- `check_in`;
- `statsd` and `metric_buckets`;
- unknown item types as logged skips.

This matters for Parallax because "Sentry-compatible" competitors are no longer
only error-event parsers. A Parallax compatibility ledger must distinguish:

```text
error-event ingest
vs
explicit unsupported-item outcomes
vs
broad Sentry replacement behavior
```

#### Deployment Shape

| Mode | Source-level shape | Parallax implication |
| --- | --- | --- |
| Tiny | `docs/tiny/README.md` describes the full product in one process with one SQLite data directory. Backup is copying `URGENTRY_DATA_DIR` or the mounted volume. | This is the low-ops baseline Parallax's first useful bundle must stay near. |
| Self-hosted | `docs/self-hosted/README.md` describes split `api`, `ingest`, `worker`, and `scheduler` roles on PostgreSQL, MinIO, Valkey, and NATS. | This is a reasonable scale-out topology, but not the tiny-tier bar. |
| Compose | The checked Compose file includes PostgreSQL, MinIO, Valkey, NATS, MinIO/bootstrap helpers, four Urgentry roles, and optional ClickHouse profile. | Count helper/init services separately from steady-state roles in the simplicity benchmark. |

#### Benchmark Claim Boundary

Urgentry publishes benchmark tables for Tiny, self-hosted Urgentry, and
self-hosted Sentry. The important source-level caveat is methodological: the
benchmark note says the workload is intentionally narrow and covers envelope
ingest, a 70/30 small/medium error mix, and issue/event query probes after load.

Do not turn these into Parallax evidence without a benchmark artifact:

- Tiny claims: `400 eps`, ingest p95 around `10.08 ms`, query p95 around
  `78.66 ms`, peak memory around `52.3 MB`.
- Self-hosted Urgentry claims: `2200 eps`, ingest p95 around `0.71 ms`, query
  p95 around `48.82 ms`, peak memory around `391.8 MB`.
- Self-hosted Sentry `26.3.1` reference claim: `1000 eps`, query p95 around
  `1400.81 ms`, peak memory around `8191.7 MB`.
- Small-box note says Sentry self-hosted did not complete on that host.

These are useful benchmark-design inputs and positioning pressure. They are not
measured Parallax evidence.

#### Agent And Autofix Boundary

No MCP server was found by checking README, docs, `internal`, `cmd`, or `deploy`
for `MCP`/`Model Context Protocol`/`mcp`.

There is an `autofix` API path under issue routes, but checked source builds a
deterministic payload from issue title/culprit/operator instruction, stores the
run as `COMPLETED`, records empty repositories/codebases, and, for `open_pr`,
sets pull request status to `SKIPPED` because no linked repository integration
is available. That should not be counted as an AI fixer, PR-opening agent, or
outcome loop.

There is also a Sentry-shaped `GET /api/0/seer/models/` stub returning an empty
AI models list. Count this as Sentry API compatibility surface, not agent-native
debugging capability.

#### Parallax Impact

What Urgentry weakens:

- "simpler than self-hosted Sentry" as a standalone public claim;
- "Sentry-compatible replacement" as a unique migration story;
- any fixture plan that only tests error events and ignores transactions,
  sessions, client reports, replay/profile/check-in behavior;
- any OTLP claim that does not say whether protobuf/gRPC is supported.

What Urgentry does not weaken:

- open-source thesis, because FSL is source-available;
- portable evidence-bundle schema;
- source-policy and missing-evidence semantics;
- redacted, hash-equivalent CLI/API/MCP projections;
- coding-agent command/file/approval/patch/test audit;
- accepted/rejected/reverted fix-outcome corpus.

#### Required Parallax Response

1. Keep Urgentry in the self-hosted simplicity baseline, but label vendor
   performance numbers unmeasured until reproduced by benchmark-agent artifacts.
2. Update Sentry compatibility wording to require explicit unsupported-item
   outcomes, not silent drops, because Urgentry shows broader item handling can
   fit a lightweight product.
3. Keep the tiny tier from requiring Postgres, Redis/Valkey, NATS, MinIO, a
   separate UI, a Collector, or MCP before the first bundle works.
4. Do not answer Urgentry by becoming a broad Sentry clone. Answer with the
   open evidence contract and action/outcome audit.
5. If Parallax uses OTLP in public wording, distinguish OTLP HTTP/JSON, OTLP
   HTTP/protobuf, and OTLP/gRPC conformance.

#### Falsification Triggers

Reopen the GO verdict or narrow the Parallax wedge if Urgentry publishes any of
the following:

- OSI-open license change;
- read-only MCP/CLI/API evidence-bundle export with schema and redaction
  manifest;
- independently reproducible benchmark artifacts under a shared protocol;
- OTLP protobuf/gRPC receiver support and Collector-equivalence evidence;
- real AI/coding-agent remediation with patch/PR/outcome audit;
- portable evidence graph or bundle format with missing-evidence semantics.

#### Sources

- [Urgentry repository](https://github.com/urgentry/urgentry)
- [Urgentry release `v0.2.12`](https://github.com/urgentry/urgentry/releases/tag/v0.2.12)
- [Urgentry license](https://github.com/urgentry/urgentry/blob/main/LICENSE)
- [Urgentry benchmark docs](https://github.com/urgentry/urgentry/blob/main/docs/benchmarks.md)
- [Urgentry Tiny mode docs](https://github.com/urgentry/urgentry/blob/main/docs/tiny/README.md)
- [Urgentry self-hosted docs](https://github.com/urgentry/urgentry/blob/main/docs/self-hosted/README.md)
- [Urgentry Compose file](https://github.com/urgentry/urgentry/blob/main/deploy/compose/docker-compose.yml)
- [Urgentry HTTP route source](https://github.com/urgentry/urgentry/blob/main/internal/http/server.go)
- [Urgentry envelope handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/envelope_handler.go)
- [Urgentry envelope side-effect source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/envelope_side_effects.go)
- [Urgentry OTLP handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/otlp_handler.go)
- [Urgentry metrics OTLP handler source](https://github.com/urgentry/urgentry/blob/main/internal/ingest/otlp_metrics_handler.go)
- [Urgentry Autofix API source](https://github.com/urgentry/urgentry/blob/main/internal/api/autofix.go)

#### Bottom Line

Urgentry raises the bar for lightweight Sentry-compatible breadth and setup
simplicity. It does not replace Parallax's research target unless it adds open,
portable, redacted evidence bundles plus real coding-agent action and outcome
audit.

## Coroot MCP and AI RCA Recheck
_Provenance: merged verbatim from `coroot-mcp-ai-rca-recheck.md` (2026-05-29 restructure)._

### Coroot MCP and AI RCA Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check Coroot as a direct open/self-hosted competitor because it is the
strongest zero-instrumentation route into agent-ready observability:

- eBPF-based metrics, logs, traces, and profiles;
- Community Edition MCP;
- Enterprise/Cloud AI root-cause analysis;
- low-friction self-hosted deployment.

This pass specifically tests whether Coroot now closes Parallax's wedge on
Sentry-compatible ingest, portable evidence bundles, read-only agent context,
local/open AI RCA, or coding-agent action/outcome audit.

#### Verdict

Coroot remains a serious self-hosted baseline and should stay on the direct
watchlist. The current source refresh strengthens the threat in one narrow way:
MCP is no longer an Enterprise-only or future feature. Coroot Community exposes
an official MCP endpoint for Claude Code, Cursor, Codex, and other MCP clients.

That does **not** close the Parallax wedge today:

1. **Community MCP is agent-ready but not purely read-only.** The MCP endpoint
   has a stronger security posture than many examples: streamable HTTP, OAuth
   2.0, each user's Coroot account, RBAC, and server-side authorization. Source
   confirms most telemetry tools carry read-only MCP annotations, but
   `resolve_alerts` is explicitly non-read-only, requires alert-edit
   permission, mutates alert state, and reports notifications sent.
2. **AI RCA is not fully local/open in Community Edition.** Current docs say AI
   RCA is available in Enterprise, or to Community users through Coroot Cloud
   integration with 10 free investigations per month. Enterprise configuration
   docs list Anthropic, OpenAI, and OpenAI-compatible APIs such as Gemini and
   DeepSeek. Source for the Community Cloud path sends a compressed RCA request
   with metrics, Kubernetes events, deployments, and selected traces to the
   Coroot Cloud integration endpoint.
3. **eBPF traces remain complementary, not a replacement for app-level capture.**
   Coroot's own tracing docs say eBPF spans may not provide complete traces.
   That keeps room for Parallax's Rust panic/error-chain, stack, release, and
   source-level semantics.
4. **No Sentry or evidence-bundle contract was found.** Checked sources show
   OTLP logs/traces, Prometheus metrics, ClickHouse storage, Prometheus cache,
   MCP tools, and AI RCA. They do not show Sentry envelope/DSN migration,
   portable evidence-bundle schema, redaction report, raw-ref policy,
   coding-agent command/file/test/patch audit, or accepted/rejected/reverted
   fixer outcome rows.

Net: keep Coroot at `trigger_hit` / high threat, not `wedge_closed`.

#### Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [Coroot `v1.20.2` release](https://github.com/coroot/coroot/releases/tag/v1.20.2) | GitHub API check still returned latest release `v1.20.2`, published 2026-05-06, and release context includes the MCP server. | Current agent access is real, not roadmap-only; no newer release changed the posture. |
| [Coroot repository](https://github.com/coroot/coroot) | Apache-2.0 project; README describes metrics, logs, traces, profiles, service map, built-in inspections, deployment tracking, and ClickHouse-backed log/tracing search. GitHub API check showed the repo was pushed on 2026-05-22. | Open-source/self-hosted posture is credible enough to treat Coroot as a direct baseline and active enough to keep on the direct watchlist. |
| [Coroot product page](https://coroot.com/) | Positions Coroot as eBPF-powered, AI-guided full-stack observability with zero code changes. | Strong adoption-friction pressure: install agent first, instrument later. |
| [Coroot editions](https://coroot.com/editions) | Community Edition is free forever, self-hosted, has no monitored-infrastructure limit, and includes agentic-ready MCP; Enterprise adds AI RCA, agentic anomaly detection/investigation, SSO, RBAC, and support at $1 per monitored CPU core/month. | Community MCP is a trigger hit; AI RCA remains paid or cloud-assisted, not fully local/open Community evidence. |
| [Coroot MCP docs](https://docs.coroot.com/mcp/overview/) | MCP exposes topology, alerts, incidents, nodes, application status, traces, PromQL metrics, logs, project switching, Community `resolve_alerts`, and Enterprise `investigate_anomaly`; OAuth 2.0 and server-side RBAC authorize tool calls. | Good auth baseline, but still a live production query/mutation surface rather than a bounded read-only evidence bundle. |
| [Coroot MCP source](https://github.com/coroot/coroot/blob/main/api/mcp.go) | Source registers read-only annotations on most telemetry tools, marks `select_project` and `resolve_alerts` non-read-only, enforces project/RBAC checks, and implements `resolve_alerts` with alert-edit permission plus a `resolvedBy` value ending in "via MCP". | Coroot's MCP safety posture is materially better than broad admin MCP catalogs, but it still is not Parallax's proposed first surface: a read-only, projection-equivalent evidence-bundle adapter. |
| [Coroot AI RCA overview](https://docs.coroot.com/ai/overview/) and [configuration](https://docs.coroot.com/ai/configuration/) | AI RCA is Enterprise or Coroot Cloud-connected for Community users; Coroot runs deterministic/ML correlation first and uses an LLM to summarize findings and fixes. Enterprise docs list Anthropic, OpenAI, and OpenAI-compatible APIs. | Useful pattern: LLM explains a precomputed diagnosis. Gap remains local/open Community availability and portable artifact semantics. |
| [Coroot Cloud integration](https://docs.coroot.com/ai/coroot-cloud/) | Coroot Cloud extends Community Edition with 10 free RCA investigations per month and can automatically investigate incidents. | Community AI path depends on an external Coroot Cloud service, not an air-gapped local default. |
| [Coroot RCA source](https://github.com/coroot/coroot/blob/main/api/rca.go) and [Cloud RCA source](https://github.com/coroot/coroot/blob/main/cloud/rca.go) | Source builds an RCA request from metrics, Kubernetes events, deployments, check config, category settings, optional error/slow traces, and incident time context, then posts an LZ4/msgpack payload to `/integration/rca`; incident RCA is persisted back to Coroot's incident record. | This confirms Coroot's valuable deterministic-precompute pattern, but also confirms Community Cloud RCA is not local-only and does not publish a portable evidence artifact contract. |
| [Coroot architecture](https://docs.coroot.com/installation/architecture/) and [Docker install](https://docs.coroot.com/installation/docker/) | Architecture uses coroot-node-agent, coroot-cluster-agent, OTLP over HTTP for logs/traces, Prometheus-compatible metrics storage, ClickHouse for logs/traces/profiles and optionally metrics; Docker Compose example runs `coroot`, `node-agent`, `cluster-agent`, `prometheus`, and `clickhouse`. | Coroot is broader and heavier than Parallax's intended tiny error/context tier, but it is still a practical self-hosted comparison baseline. |
| [Coroot requirements](https://docs.coroot.com/installation/requirements/) and [eBPF tracing](https://docs.coroot.com/tracing/ebpf-based-tracing/) | Requires Linux kernel 5.1+ and container/systemd coverage; docs state eBPF spans may not provide complete traces. | Validates the "zero-code visibility" strength and the "not enough app semantics" gap at the same time. |
| Coroot repository tree scan | Path scan for Sentry, envelope, DSN, evidence, bundle, artifact, schema, export, RCA, anomaly, and MCP terms found MCP and RCA implementation files, alerting export UI assets, and docs/images, but no obvious Sentry-ingest path or portable investigation/evidence schema. | Negative evidence only within tree-path names, but enough to keep Sentry migration and evidence-bundle claims unproven in current public source. |

#### Product Impact

Coroot is not the closest wedge-killer on Sentry migration or Rust/object-store
fit. It is the closest threat on adoption friction: a team can install it and
quickly give agents topology, health, logs, metrics, traces, incidents, and
alerts without changing application code.

Parallax should not answer Coroot by trying to become a broader infrastructure
dashboard. The defensible response is narrower:

```text
Sentry-compatible application error ingest
+ OTLP telemetry
+ app-level Rust/front-end semantics
+ portable evidence bundles
+ read-only context projection
+ coding-agent action/outcome audit
```

Coroot's most useful lesson is its RCA split: deterministic correlation first,
LLM explanation second. Parallax should use the same principle for bundles:
precompute and cite evidence, then let agents explain or act on it.

#### Falsification Criteria

Reopen the Parallax verdict if Coroot:

- adds Sentry SDK/envelope ingestion or DSN-only migration;
- makes AI RCA fully local/open in Community Edition;
- turns MCP outputs into portable, versioned, citable evidence bundles with
  redaction reports and raw refs;
- removes or cleanly separates mutating MCP tools from the default Community
  agent surface;
- adds coding-agent session, shell/CLI action, patch, PR, review, revert, or
  recurrence audit;
- proves eBPF plus OpenTelemetry covers enough application error semantics to
  weaken the need for Rust-first SDK/error capture.

Until then, Coroot raises the baseline for self-hosted agent access but leaves
Parallax's application-evidence and action-outcome wedge open.

Prompt update: not needed in this pass. The active prompt already names Coroot's
Community MCP, Enterprise/Cloud AI RCA boundary, and the watch triggers for
fully local/open RCA, Sentry ingest, evidence bundles, and action/outcome audit.

## SigNoz Open Investigation Format Check
_Provenance: merged verbatim from `signoz-open-investigation-format-check.md` (2026-05-29 restructure)._

> **Re-verified 2026-05-29 (primary sources): no material drift.** Errors still arrive as **OpenTelemetry exception span-events**, not a Sentry-envelope ingest path — the bear-case "SigNoz adds Sentry ingest" trigger has **not fired**; SigNoz stays OTel-native (Go/ClickHouse). No checked versioned/portable evidence-bundle schema surfaced behind the "open investigation format" framing.

### SigNoz Open Investigation Format Check

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-test the strongest suspicious SigNoz claim in the current research record:
does SigNoz's public "open investigation format" language mean it has published
a versioned, portable investigation or evidence-bundle schema that weakens
Parallax's A3 schema/corpus wedge?

#### Verdict

As of 2026-05-25, this remains **unproven**.

SigNoz has a real agent-native observability surface: hosted and self-hosted
MCP, coding-agent setup docs, live metrics/logs/traces query tools, and
management tools for alerts, dashboards, saved views, and notification channels.
It also has official landing-page language saying an investigation workflow can
become an "open investigation format" for a team. The newest material checked in
this pass strengthens the signal: SigNoz now documents a "Postmortem Evidence
Pack" use case where an AI assistant uses MCP tools to compile alert history,
metric inflections, representative logs, trace search results, and a full trace
breakdown into an incident timeline. A same-day source refresh also found an
official `signoz-investigating-alerts` skill with a read-only, three-tier RCA
workflow, prescribed evidence-trail output, guardrails, and eval cases.

The checked primary sources did **not** expose a source-linked, versioned,
portable investigation schema or artifact with provenance, redaction, raw-ref,
query-manifest, missing-evidence, or outcome semantics. Treat the phrase and the
evidence-pack workflow as competitive pressure signals, not as evidence that
SigNoz has closed Parallax's open evidence-bundle moat. The alert-investigation
skill strengthens the workflow threat, but it is still a playbook over live MCP
queries rather than a canonical portable artifact.

#### Sources Checked

| Source | Current evidence | Interpretation |
| --- | --- | --- |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/) and [page source](https://github.com/SigNoz/signoz.io/blob/main/app/%28site%29/agent-native-observability/AgentNativeObservabilityPage.constants.tsx) | Positions SigNoz inside coding agents, says investigation data stays with the user, and says the "open investigation format" can become a team standard. The source file shows the same language in product-page copy rather than a linked artifact spec. | The phrase is real and official. It validates market direction, but it is still product/workflow language unless paired with a schema, validator, or exportable artifact. |
| [SigNoz Postmortem Evidence Pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/) and [source MDX](https://github.com/SigNoz/signoz.io/blob/main/data/docs/ai/use-cases/postmortem-evidence-pack.mdx) | Docs page dated 2026-04-24 shows an assistant prompt that compiles an incident timeline from alert transitions, metric inflection points, representative errors, and trace details. The "Under the Hood" table maps the workflow to `signoz_get_alert_history`, `signoz_query_metrics`, `signoz_search_logs`, `signoz_search_traces`, and `signoz_get_trace_details`. | This is closer to Parallax's evidence-pack language than the landing page alone. It is still an example LLM response/workflow, not a versioned portable evidence object with conformance tests, provenance fields, redaction report, raw refs, missing-evidence flags, or outcome rows. |
| [SigNoz on-call lifecycle MCP blog](https://signoz.io/blog/automating-oncall-lifecycle-signoz-mcp/) and [source MDX](https://github.com/SigNoz/signoz.io/blob/main/data/blog/automating-oncall-lifecycle-signoz-mcp.mdx) | Blog post dated 2026-05-20 frames MCP as automating alert creation, handoff briefs, alert-fatigue audits, and postmortem evidence packs. The postmortem section describes a single prompt that compiles alert transitions, metric inflection points, representative errors, and a representative trace into a structured incident timeline. | This widens SigNoz from "agent can query telemetry" to "agent can run on-call workflows." It still describes prompt-driven generated output rather than a separately versioned, portable, validator-backed artifact. |
| [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/) | Docs page dated 2026-05-13 covers Claude Desktop, Claude Code, OpenAI Codex, Cursor, GitHub Copilot, Gemini CLI, Windsurf, Zed, hosted MCP, and self-hosted MCP. | Agent access is real and current. The docs describe connection and tools, not a canonical investigation artifact. |
| [`SigNoz/signoz-mcp-server` README](https://github.com/SigNoz/signoz-mcp-server) | README describes natural-language access to metrics, traces, logs, alerts, dashboards, and services; lists tools such as `signoz_query_metrics`, `signoz_search_logs`, `signoz_aggregate_traces`, `signoz_get_trace_details`, `signoz_execute_builder_query`, and create/update/delete tools for alerts, dashboards, saved views, and notification channels. It also says every tool accepts `searchContext` for MCP observability and does not forward that field to SigNoz APIs. | The MCP surface is primarily query plus management with some MCP self-observability metadata. It is not shaped like Parallax's intended read-only bundle projection. |
| [`SigNoz/agent-skills`](https://github.com/SigNoz/agent-skills), [`signoz-investigating-alerts`](https://github.com/SigNoz/agent-skills/blob/main/plugins/signoz/skills/signoz-investigating-alerts/SKILL.md), and [evals](https://github.com/SigNoz/agent-skills/blob/main/plugins/signoz/skills/signoz-investigating-alerts/evals/evals.json) | Repository metadata checked on 2026-05-25 showed latest push on 2026-05-19; shallow clone HEAD was `4321d40f277e24c7b2660559fcb7c1de78ea84ca`. The repo includes official skills for MCP setup, alert creation/explanation/investigation, dashboard work, query generation, ClickHouse query writing, docs search, and saved views. `signoz-investigating-alerts` is read-only, requires SigNoz MCP tools, runs a three-tier alert RCA flow, mandates exact output sections, requires every claim to cite MCP query results, and has evals for full RCA, fuzzy matching, marginal/flapping fires, never-fired stops, and trace-formula fires. `CONTRIBUTING.md` says MCP is the API, skills are the playbook, and tool definitions/input schemas/schema validation belong in MCP tools/resources. | Stronger than a blog use case: SigNoz is packaging agent investigation behavior and eval expectations. It still does not publish a portable investigation artifact schema; the skill is an instruction/playbook over live MCP queries and prose output. |
| [`SigNoz/signoz-mcp-server`, `SigNoz/agent-skills`, and `SigNoz/signoz.io` source scans](https://github.com/SigNoz) | Shallow clones checked on 2026-05-25 at MCP HEAD `8a6bb34ea75775bbe678594219bc21a5babd8721`, skills HEAD `4321d40f277e24c7b2660559fcb7c1de78ea84ca`, and site HEAD `2afebfb8e4212b8db7de0a15fb7a324b5bd53191`. Targeted content search for `investigation artifact`, `evidence bundle`, `query manifest`, `redaction report`, `raw refs`, `raw-ref`, `outcome ledger`, `portable evidence`, `validator-backed`, and `replayable evidence` returned no matches. Path scans still found dashboard schemas, schema compatibility helpers, manifests, raw-data export, skills, and docs/blog files, but no obvious investigation artifact schema. | Stronger than path-only evidence, but still not authenticated GitHub code search across every repo. Bound the negative claim to these public repositories and checked SHAs. |
| [`signoz-mcp-server` `v0.4.1` release](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.4.1) | Latest-release redirect returned HTTP `200` and resolved to `v0.4.1`; tag ref `8a6bb34ea75775bbe678594219bc21a5babd8721`; GitHub API `published_at` `2026-05-21T07:55:27Z`. Release body fixes Query Builder typed round-trip for PromQL and ClickHouse SQL. | The MCP server is active and query semantics are still being refined. The latest release did not publish an investigation schema. |
| [SigNoz `v0.125.1` release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1) | Latest-release redirect returned HTTP `200` and resolved to `v0.125.1`; tag ref `fb3e316ce906c36cdb20cd4900e58f2a43804d7a`; GitHub API `published_at` `2026-05-20T18:04:37Z`. Main branch metadata showed same-day push on 2026-05-25. | The platform is active and same-day main movement raises watch priority, but it does not change the schema finding. |

Unauthenticated GitHub code search for exact phrases was not usable through the
public API during the earlier pass because it requires authentication. This pass
improved the evidence class by using current shallow clones and targeted content
searches for the public MCP, skills, and website repositories. That is still
weaker than authenticated GitHub code search across every SigNoz repository and
not proof that no schema exists anywhere. The claim is therefore bounded to the
checked official landing page, docs/use-case pages, MCP README, on-call blog,
agent-skills repository, public source scans, and release metadata.

#### What Would Count As Closing The Gap

A future SigNoz source should be treated as A3-relevant if it publishes any of
the following:

- a JSON/YAML/Protobuf schema for a canonical investigation object;
- positive and negative fixtures or conformance tests for that object;
- an exported investigation artifact that can be validated without a live SigNoz
  tenant;
- an MCP structured output schema whose canonical payload is an investigation
  artifact rather than free-form text or ad hoc query results;
- a postmortem evidence-pack export that can be validated and replayed outside
  the live SigNoz tenant and the original assistant conversation;
- fields for evidence provenance, redaction report, raw refs, query manifest,
  missing evidence, hypotheses, and outcome rows;
- public outcome feedback tying an investigation to a fix, review, recurrence,
  revert, or rejection.

Until then, do not count "open investigation format" as a published evidence
schema.

#### Parallax Implication

The old research note was right to mark SigNoz as the closest open
agent-native threat, but it should stay precise:

- Do not claim no competitor uses "investigation format" language. SigNoz does.
- Do not claim MCP is unique. SigNoz has a current hosted and self-hosted MCP
  surface.
- Keep the differentiator on the stricter contract: versioned portable bundles,
  schema fixtures, validator, compatibility policy, redaction reports, raw-ref
  controls, missing-evidence flags, and accepted/rejected/reverted outcome rows.
- Do not claim SigNoz lacks agent investigation playbooks. It now has an
  official read-only alert-RCA skill with output discipline and evals. The
  distinction is that this is live-query workflow scaffolding, not a
  source-linked portable artifact.
- Watch SigNoz closely because it could turn the current evidence-pack
  workflow, open-format language, and official skills into a real open schema
  quickly.

Prompt update: needed in this pass because the durable prompt should remember
that SigNoz now has official agent-skills alert-investigation material, not only
a landing-page phrase or postmortem evidence-pack use case.

## Sentry MCP and Seer Self-Hosted Recheck
_Provenance: merged verbatim from `sentry-mcp-seer-self-hosted-recheck.md` (2026-05-29 restructure)._

### Sentry MCP And Seer Self-Hosted Recheck

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-check the most load-bearing Sentry claim in the current research record:
whether Parallax can still treat self-hosted Sentry users as lacking Sentry's
hosted Seer/Autofix experience, and whether Sentry MCP has closed the
agent-access gap for self-hosted Sentry.

#### Verdict

Sentry remains the strongest direct incumbent pressure, but the repository
should narrow the old claim.

Keep:

- hosted Seer/Autofix is a strong proof that production-error AI debugging,
  root-cause analysis, solution planning, code changes, PR creation, and external
  coding-agent handoff are real workflows;
- Sentry MCP makes agent access to Sentry data table stakes;
- current self-hosted docs now explicitly exclude Seer and other AI/ML features
  from self-hosted Sentry because those components are closed source;
- Parallax's sharper gap is the open, portable, redacted evidence bundle plus
  action/outcome ledger, not "Sentry has no AI."

Remove or avoid:

- treating the current self-hosted Seer exclusion as a permanent technical
  impossibility or proof that Sentry will never ship a self-hosted AI path;
- categorical wording that Sentry MCP self-hosted use always requires write
  scopes;
- any implication that "MCP exists" closes or proves the Parallax agent surface.

#### Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| [Sentry Seer docs](https://docs.sentry.io/product/ai-in-sentry/seer/) | Seer is described as Sentry's AI debugging agent using issue details, tracing data, logs, profiles, and code context. Seer includes Autofix, PR creation, external coding-agent handoff, Seer Agent, and code review. Seer is also an add-on to a Sentry subscription. | Hosted Sentry is directly attacking the issue-to-fix workflow. Parallax must not claim PR generation or AI RCA as a moat. |
| [Sentry Autofix docs](https://docs.sentry.io/product/ai-in-sentry/seer/autofix/) | Autofix uses Sentry context and GitHub-integrated codebases; it can stop after root cause, plan, or PR draft. The docs say Seer can only integrate with the cloud version of GitHub, and that cloud GitHub is currently the only SCM supported by Seer. Handoff agents listed are Claude Code and Cursor Cloud Agents. | Sentry's hosted path is operationally strong but cloud-GitHub-oriented. A local/open evidence engine can still win for self-hosted, air-gapped, multi-source, or schema-first users. |
| [Seer Issue Fix API](https://docs.sentry.io/api/seer/start-seer-issue-fix/) | The API can identify root cause, propose a solution, generate code changes, and create a PR. Stop points are `root_cause`, `solution`, `code_changes`, and `open_pr`; runs are asynchronous. The endpoint requires an auth token with `event:admin` or `event:write`. | The separate-fixer workflow is incumbent behavior and it is a write/admin event-scope surface. Parallax must differentiate below the fixer: evidence contract, redaction, provenance, and outcome rows. Keep the first Parallax MCP/context surface read-only; put fix orchestration in a separate control plane. |
| [sentry-mcp README](https://github.com/getsentry/sentry-mcp) and [`0.35.0` release](https://github.com/getsentry/sentry-mcp/releases/tag/0.35.0) | Latest release checked is `0.35.0`, published 2026-05-21; `/releases/latest` returned HTTP `200` at `0.35.0`, and `git ls-remote` shows tag `fc04542e24472f00b639f2d591dfc111fa855158`. The README says Sentry MCP is primarily for human-in-the-loop coding agents. It supports remote MCP, Claude Code plugin/subagent use, and stdio. The README calls stdio a work-in-progress path for self-hosted Sentry; AI-powered search needs OpenAI or Anthropic configuration; self-hosted instances may need unsupported Seer skills disabled; the README setup path lists `project:write`, `team:write`, and `event:write`. | Sentry MCP is real and important. It is not proof of self-hosted hosted-Seer parity, canonical evidence-bundle projection, redaction reports, or read-only-by-default safety. |
| [sentry-mcp stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/main/docs/testing-stdio.md) | The testing guide documents full-function scopes including write scopes, but also states read-only testing can use `org:read`, `project:read`, `team:read`, and `event:read`. The guide still shows stdio self-hosted configs and example output with 20 tools available. | Narrow the old scope claim. Sentry documents a read-only testing path, but Parallax still cannot count this as a read-only-safe agent surface until tool availability, projection equivalence, redaction, and fixture behavior are proven. |
| [Self-hosted Sentry docs](https://develop.sentry.dev/self-hosted/) and [`26.5.0` release](https://github.com/getsentry/self-hosted/releases/tag/26.5.0) | Latest self-hosted release checked is `26.5.0`, published 2026-05-18; `/releases/latest` returned HTTP `200` at `26.5.0`, and `git ls-remote` shows tag `aed5b2037e74c771bfe476dbdbeb80420ef4a3d8`. Current docs describe a Docker Compose/bash setup, minimum and recommended resources, errors-only beta, feature-complete mode, single-node service caveats, FSL licensing, and a feature-complete list including traces, profiles, replays, uptime, metrics, feedback, and crons. They explicitly list Seer and other AI/ML features as unavailable on self-hosted Sentry because those components are closed source. The `26.5.0` compose file declares 72 services. | The self-hosted Seer gap is explicit in current primary docs, not merely unproven. Keep the claim tied to current docs and avoid implying Sentry cannot change this later. |

#### Product Impact

The Sentry wedge should now be worded as:

> Sentry's hosted Seer/Autofix path validates the issue-to-fix workflow. Current
> self-hosted docs explicitly exclude Seer and other AI/ML features from
> self-hosted Sentry, and Sentry MCP does not publish Parallax-style portable,
> redacted, citable evidence bundles or outcome ledgers.

The issue-fix API's `event:admin`/`event:write` requirement also reinforces the
component boundary: Sentry's fix path is a privileged control surface. Parallax's
first agent-facing surface should stay read-only evidence retrieval; any
PR-opening fixer belongs above it.

This is still strategically important. It just keeps the self-hosted claim tied
to current Sentry docs instead of turning it into a future-proof guarantee.

#### Falsification Criteria

Revisit the verdict and drift ledger if Sentry publishes any of the following:

- removal or reversal of the current self-hosted docs exclusion for Seer and
  other AI/ML features;
- self-hosted Seer/Autofix parity with local or customer-selected LLM providers;
- self-hosted Seer Agent support with documented access controls below
  organization-wide telemetry;
- sentry-mcp stdio graduating from work-in-progress status with read-only default
  scopes and a published tool catalog for those scopes;
- structured MCP outputs that are equivalent to a portable evidence bundle with
  redaction reports, raw refs, and source-field provenance;
- a public issue-fix outcome ledger covering accepted, rejected, reverted, and
  recurrent fixes.

Until then, keep Sentry at `wedge_under_pressure`, not `wedge_closed`.

## Lightweight Error-Tracker MCP Boundary Check
_Provenance: merged verbatim from `lightweight-error-tracker-mcp-boundary-check.md` (2026-05-29 restructure)._

### Lightweight Error-Tracker MCP Boundary Check

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Purpose

The [lightweight Sentry-compatible competitor watch](competitor-watch.md)
already shows that "simpler than self-hosted Sentry" is crowded. This note
checks a narrower question:

> Have lightweight Sentry-compatible or OTLP-native challengers closed
> Parallax's agent-access wedge by shipping MCP or agent tools?

Short answer: **no, but the wedge is under more pressure**. Rustrak and GoSnag
now prove that MCP can appear inside very small error-tracking projects. Their
checked MCP surfaces are management and raw-event tools, not Parallax-style
read-only, redacted, canonical evidence-bundle projections.

#### Boundary Verdict

| Project | Agent/MCP posture checked | Boundary read |
| --- | --- | --- |
| Bugsink | No first-party MCP or AI agent surface found in the checked official README, self-hosting page, or Sentry-SDK compatibility page. Current release: `2.2.1` on 2026-05-22 adds canonical API issue actions/comments and OpenAPI docs. Small third-party MCP adapters now exist (`bugsink-mcp` on npm / `draded/bugsink-mcp`, plus `j-shelfwood/bugsink-mcp`) with Bugsink project/team/issue/event/stacktrace/release query tools. License file uses PolyForm Shield for most repository content, with noted third-party exceptions. | Strong low-ops Sentry-compatible baseline, and ecosystem-level MCP pressure exists, but no first-party or mature Parallax-style read-only evidence-bundle surface is proven. |
| Rustrak | `@rustrak/mcp` `0.1.2` is live on npm. Its README says it gives AI assistants "full control" and exposes 18 tools across projects, issues, events, tokens, and alerts. Source/docs include `create_project`, `resolve_issue`, `unresolve_issue`, `mute_issue`, `delete_issue` with `destructiveHint`, `get_event` with full Sentry-envelope data, `create_token`, `revoke_token`, and `test_alert_channel`. A focused recheck also found an unreleased repo-maintenance Sentry protocol agent workflow in `main`, not a product-facing runtime feature. | MCP trigger is hit. The surface is management/write/raw-event shaped, not a bounded evidence-bundle contract. Parallax should not compete by adding more MCP CRUD; it should keep first MCP read-only and bundle/projection based. |
| Traceway | Focused recheck confirms stronger-than-README OTLP/context pressure: source registers `/api/otel/v1/{traces,metrics,logs}` plus native `/api/report`; converters map spans, exceptions, logs, metrics, and `gen_ai.*` AI traces; docs cover SQLite/all-in-one/minimal/embedded modes and integration skills. Current backend release: `backend/v1.7.27` on 2026-05-22. Checks found integration skills but no MCP server or Sentry-compatible ingest path; `sentry`/`dsn`/`envelope` content hits were comparison/design/test/framework references. | Strong OTLP/context/replay pressure, but not an MCP or Sentry-migration closure. Integration skills are assistant guidance for instrumentation, not a read-only data-access evidence surface. |
| GoSnag | Focused source recheck confirms a TypeScript stdio MCP server using `GOSNAG_URL`, `GOSNAG_TOKEN`, and Bearer-token `/api/v1` calls. Tools include `list_projects`, `get_project`, `create_project`, `update_project`, `delete_project`, `list_issues`, `get_issue`, `update_issue_status`, `get_issue_events`, `get_issue_counts`, `list_alerts`, `create_alert`, `list_issue_tags`, `add_issue_tag`, `list_users`, `create_ticket`, `get_ticket`, `update_ticket`, `list_tickets`, and `get_ticket_counts`. GitHub has no tagged release; latest checked push is 2026-04-17. | Capability warning. The AI/MCP feature vector is broad, but maturity is weak and the MCP surface is management/write/raw-event-list shaped, not read-only evidence context. |
| Urgentry | Focused source recheck confirms DSN migration posture, one-binary Tiny mode, split self-hosted mode over PostgreSQL/MinIO/Valkey/NATS, broad Sentry envelope side effects, OTLP HTTP/JSON traces/logs/metrics, and benchmark claims. Current release: `v0.2.12` on 2026-05-22; latest checked `main` commit `ccc0ff8`. License is FSL-1.1-ALv2. No MCP surface was found in checked README/docs/source. The `autofix` API is deterministic/stub-like and records skipped PR behavior, not a real agent fixer loop. | Strong Sentry-compatible simplicity and protocol-breadth pressure, but not an open-source, MCP, evidence-bundle, or agent-action closure in checked sources. |

#### What This Changes

The old comparison "Parallax has MCP and lightweight competitors do not" is no
longer safe. The correct comparison is:

```text
management MCP / raw issue access
vs
read-only, redacted, citable evidence bundle with hashes, source policy,
missing-evidence fields, and outcome writeback outside the read path
```

Rustrak and GoSnag make MCP table stakes even at the lightweight end of the
market. They do **not** remove the Parallax wedge because the checked surfaces
do not publish:

- canonical evidence bundle schema;
- projection-equivalence hashes across CLI/API/MCP/Markdown/JSON;
- redaction manifest and source-field policy rows;
- missing-evidence model;
- read-only-by-default least-privilege bundle access;
- coding-agent action audit;
- accepted/rejected/reverted fix-outcome loop.

#### Source Snapshot

| Source | Evidence checked | Freshness |
| --- | --- | --- |
| [Bugsink release](https://github.com/bugsink/bugsink/releases/tag/2.2.1), [self-hosting page](https://www.bugsink.com/built-to-self-host/), [Sentry-SDK compatibility page](https://www.bugsink.com/sentry-sdk-compatible/), [license](https://github.com/bugsink/bugsink/blob/main/LICENSE), [`bugsink-mcp` package](https://www.npmjs.com/package/bugsink-mcp), [`draded/bugsink-mcp`](https://github.com/draded/bugsink-mcp), and [`j-shelfwood/bugsink-mcp`](https://github.com/j-shelfwood/bugsink-mcp) | Single-container/SQLite/no-queue posture, Sentry SDK compatibility, no first-party MCP/AI agent surface in official docs, PolyForm Shield license posture, and small third-party Bugsink MCP adapters with issue/event/stacktrace query tools. | Bugsink release `2.2.1` published 2026-05-22; `bugsink-mcp` npm package checked at `1.0.0`; `draded/bugsink-mcp` has 0 stars/no releases; `j-shelfwood/bugsink-mcp` has 6 stars/no releases and last push 2026-01-12. |
| [Rustrak repository](https://github.com/AbianS/rustrak), [`@rustrak/mcp` npm package](https://www.npmjs.com/package/@rustrak/mcp), [MCP package README](https://github.com/AbianS/rustrak/tree/main/packages/mcp), [MCP issue tools source](https://github.com/AbianS/rustrak/blob/main/packages/mcp/src/tools/issues.ts), and [Rustrak recheck](competitor-watch.md) | Rust/Actix Sentry-compatible tracker, SQLite/Postgres deployment, `@rustrak/mcp` `0.1.2`, project/issue/event/token/alert tools, destructive issue and token operations, raw Sentry-envelope event access, and unreleased repo-maintenance Sentry protocol agent workflow. | Repo pushed 2026-05-25; generic latest release `docs@0.1.16` and server release `@rustrak/server@0.2.5` published 2026-05-21; npm `@rustrak/mcp` checked at `0.1.2`. |
| [Traceway repository](https://github.com/tracewayapp/traceway), [OTLP route source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/routes.go), [integration skills](https://github.com/tracewayapp/traceway/tree/main/skills), and [Traceway recheck](competitor-watch.md) | MIT, direct OTLP/HTTP logs/traces/metrics, native `/api/report` for sessions/recordings, AI trace conversion from `gen_ai.*`, SQLite/all-in-one/minimal/embedded deployment modes, integration-instruction skills, and no checked MCP or Sentry-compatible ingest path. | Backend release `backend/v1.7.27` published 2026-05-22; repo pushed 2026-05-25; latest checked `main` commit `38b8d385`. |
| [GoSnag repository](https://github.com/darkspock/gosnag), [GoSnag MCP source](https://github.com/darkspock/gosnag/blob/main/mcp/src/index.ts), [GoSnag MCP package file](https://github.com/darkspock/gosnag/blob/main/mcp/package.json), and [GoSnag recheck](competitor-watch.md) | Sentry `/store/` and `/envelope/` error-event ingest, AI RCA/triage source, Bearer-token MCP server, management tools for projects/issues/alerts/tags/tickets/users, no tagged GitHub release, and no Parallax-style evidence-bundle contract. | Repo pushed 2026-04-17; no latest release found in GitHub API; `mcp/package.json` version `1.0.0`; latest checked `main` commit `418b8b1`. |
| [Urgentry repository](https://github.com/urgentry/urgentry), [release](https://github.com/urgentry/urgentry/releases/tag/v0.2.12), [license](https://github.com/urgentry/urgentry/blob/main/LICENSE), and [Urgentry recheck](competitor-watch.md) | DSN migration, Tiny one-binary SQLite mode, split PostgreSQL/MinIO/Valkey/NATS mode, broad envelope side effects, OTLP HTTP/JSON with protobuf rejected, benchmark claims, FSL source-available license, no checked MCP surface, and deterministic/stub-like Autofix output. | Release `v0.2.12` published 2026-05-22; latest checked `main` commit `ccc0ff8` dated 2026-05-22. |

#### Counting Rules

- Count lightweight MCP as a watch trigger, not a moat closure.
- Count write/destructive tools as safety pressure against Parallax's MCP
  design, not as evidence-bundle parity.
- Do not count raw Sentry event access as agent-ready context unless it is
  redacted, source-labeled, bounded, and projected through the same canonical
  bundle contract as CLI/API output.
- Keep license posture separate from deployment simplicity: Bugsink and
  Urgentry are relevant self-hosting baselines even though their checked licenses
  do not satisfy Parallax's open-source thesis.
- Treat no-release projects as capability warnings until release cadence,
  install path, and fixture behavior become reproducible.

#### Parallax Impact

This pass strengthens the current product boundary:

- CLI and HTTP can remain day-one access surfaces.
- MCP should ship only after projection-equivalence and redaction fixtures pass.
- First MCP server should be read-only evidence context, not alert/dashboard/
  user/token/project/ticket CRUD and not issue resolution.
- Outcome records belong in a separate append-only write path after the core
  bundle contract is tested.

#### Falsification Triggers

Reopen this note and the GO verdict if any lightweight challenger publishes:

- Sentry SDK migration plus OTLP traces/logs/metrics correlation;
- a versioned portable evidence-bundle schema with redaction/source policy;
- read-only MCP bundle tools with `structuredContent`/schema validation and
  projection-equivalence hashes;
- coding-agent command/file/approval/patch/test audit;
- accepted/rejected/reverted fix outcome rows;
- reproducible benchmark artifacts that beat Parallax's tiny-tier first-use
  target while covering a comparable evidence surface.

#### Bottom Line

Lightweight competitors and their ecosystems have crossed the "has MCP" threshold. They have not
crossed the "safe evidence contract for agents" threshold. Parallax should use
that distinction aggressively: no broad management MCP in the first context
server, and no "agent-ready" wording without canonical bundle, redaction,
projection, and outcome-ledger proof.

## MCP Power Boundary Competitor Check
_Provenance: merged verbatim from `mcp-power-boundary-competitor-check.md` (2026-05-29 restructure)._

### MCP Power Boundary Competitor Check

_(Shared note — see the Open Self-Hosted Competitor Watch section above.)_

Research date: 2026-05-25

#### Pass Target

Re-test the claim that Parallax's first MCP surface should be a narrow read-only
bundle adapter, not a broad observability management MCP. The suspicious part of
the current research record is that "write-capable competitor MCP" could become
lazy positioning unless the repository records exactly what current primary
sources show.

#### Verdict

Keep the read-only MCP boundary, but state it precisely.

MCP itself is not dangerous or differentiating. It is now a normal agent access
surface. The differentiator Parallax can still defend is a stricter first
contract:

```text
canonical evidence bundle
+ bounded structuredContent
+ redaction/source-field policy
+ raw refs behind explicit scope
+ audit rows
+ no production/project mutation tools in the context server
```

Current observability MCP surfaces often mix investigation with management:
creating or deleting alerts, dashboards, roles, streams, users, incidents,
notification channels, saved views, search jobs, or persisted investigations.
Those features are useful. They are also a different product surface than
Parallax's first read-only evidence context adapter.

#### Source Matrix

| Source | Current power shape | Parallax interpretation |
| --- | --- | --- |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools are model-invoked, can expose input and output schemas, can return `structuredContent`, can notify tool-list changes, and the spec's security section requires validation, access control, rate limiting, sanitization, confirmation for sensitive operations, and audit logging. | Parallax should use MCP structured output for bundles, but protocol support does not by itself prove safety. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | Current guidance calls out token passthrough, local MCP server compromise, session hijack, SSRF, broad scopes, and local server execution risk. Scope minimization recommends small initial scopes, targeted elevation, down-scoping tolerance, and correlation-id logging. | A broad all-in-one observability MCP makes the first Parallax safety claim harder. Start with a low-risk read-only scope and no admin tools. |
| [Sentry MCP repository](https://github.com/getsentry/sentry-mcp), [Sentry MCP stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/master/docs/testing-stdio.md), and [Sentry MCP releases](https://github.com/getsentry/sentry-mcp/releases) | Sentry says its MCP service targets human-in-the-loop coding agents rather than every Sentry API. The checked release page shows `0.35.0` on 2026-05-21. The README describes stdio for self-hosted Sentry as work in progress, lists setup scopes including `project:write`, `team:write`, and `event:write`, and says AI-powered search needs OpenAI or Anthropic provider configuration. The stdio testing guide also documents a read-only testing scope set: `org:read`, `project:read`, `team:read`, and `event:read`. | Sentry is not simply a broad admin MCP, and the old "no documented read-only path" objection is too strong. Parallax should still not claim MCP parity until it proves read-only tool availability, projection equivalence, redaction, and bundle output under those narrower scopes. |
| [Grafana Assistant MCP servers](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/) and [Grafana Incident/Sift MCP guide](https://grafana.com/docs/grafana/latest/developer-resources/mcp/guides/use-grafana-incident-and-sift/) | Grafana Assistant supports only remote MCP servers, requires operators to trust connected servers and review tool calls, and frames MCP around issue/code lookup, ticket creation, Slack notifications, and similar actions. The Incident/Sift guide separates Viewer read operations from Editor write operations such as creating incidents, adding incident activity, and running Sift analyses that create investigations. | Grafana validates a human-approved action model. Parallax's first surface should be a context server, not an incident/ticket/action automation server. |
| [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/) | OpenObserve documents a large MCP catalog with many create/update/delete/admin operations. Checked tool categories include alerts, authorization roles, dashboards, folders, functions, KV, organizations/settings, pipelines, search jobs, service accounts, sourcemaps, streams, and users, with destructive tools marked. It recommends a dedicated MCP user, secret handling, credential rotation, and client-side confirmation when possible. | OpenObserve is the clearest evidence that agentic observability MCP can become a management plane. Parallax must avoid copying that shape for the first context adapter. |
| [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/) and [SigNoz open investigation format check](competitor-watch.md) | SigNoz exposes hosted and self-hosted MCP. The checked README/docs list metrics/logs/traces/query tools plus create/update/delete operations for alerts, dashboards, saved views, and notification channels, and a raw Query Builder tool. | SigNoz proves open self-hosted MCP is table stakes. Its tool shape is query plus management, not Parallax-style read-only bundle projection. |
| [Coroot MCP overview](https://docs.coroot.com/mcp/overview/), [Coroot releases](https://github.com/coroot/coroot/releases), and [Coroot MCP and AI RCA recheck](competitor-watch.md) | Coroot's latest checked release page shows `1.20.2` as latest and includes the MCP server. The MCP docs use OAuth 2.0 and server-side RBAC; Community tools expose topology, alerts, incidents, traces, logs, metrics, raw telemetry, `select_project`, and `resolve_alerts`. Enterprise adds `investigate_anomaly`, which can persist RCA onto an incident when an incident key is supplied. | Coroot has a stronger auth story than many MCP examples, but it still mixes live production query, alert resolution, and persisted RCA. Treat it as a serious baseline, not as proof Parallax should add write tools early. |

GitHub REST release checks for several repositories were rate-limited during this
pass, so this note does not replace the existing release-version rows in the
competitor drift ledger. The claims above are about documented MCP tool power,
not about a new release-recency audit.

The same boundary now applies below the large observability-suite tier. The
[Lightweight error-tracker MCP boundary check](competitor-watch.md)
found Rustrak and GoSnag MCP surfaces in small error trackers, including
project/issue/event/token/alert/ticket/user management and raw Sentry-envelope
event access. That makes MCP availability a table-stakes feature, not a moat,
while strengthening the case for a Parallax first server that is read-only,
redacted, schema-bound, and projection-equivalent.

#### Product Boundary

"Read-only MCP context adapter" should mean all of the following for the first
Parallax MCP server:

- no generic shell, SQL, Kubernetes, SSH, deploy, rollback, or database tools;
- no alert, dashboard, role, user, organization, pipeline, notification-channel,
  incident, ticket, saved-view, stream, sourcemap, or search-job create, update,
  or delete tools;
- no alert resolution, incident suppression, ticket update, Slack send, or
  persisted RCA write in the context server;
- no raw table dumps, raw log dumps, unbounded trace dumps, source-map dumps, or
  full agent transcripts inline;
- read-sensitive raw refs disabled by default and gated behind a narrow audited
  scope;
- outcome/audit writes are separate from production mutation and must be
  append-only, explicitly scoped, and covered by the access-surface ledger.

This does not forbid future automation. It says automation belongs in a separate
fixer/control-plane component after bundle value, redaction, source-field
policy, and outcome ledgers are green.

#### Falsification Criteria

Revisit this boundary if any of these become true:

- a competitor ships a source-linked, versioned, read-only MCP evidence-bundle
  schema with redaction report, raw refs, and output schemas;
- a competitor proves broad management MCP can be least-privilege, redacted,
  auditable, prompt-injection-resistant, and cross-client safe with public
  fixture rows;
- Parallax's A1 bundle-value eval shows agents need mutation tools in the same
  MCP session to realize value;
- operator interviews show buyers reject a read-only context server unless it
  can create/update incidents or tickets immediately.

Until then, the safest and clearest public wording is:

> Parallax plans a read-only MCP context adapter over canonical evidence bundles;
> automation and production mutation are separate, later surfaces.
