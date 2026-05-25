# Coroot MCP and AI RCA Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check Coroot as a direct open/self-hosted competitor because it is the
strongest zero-instrumentation route into agent-ready observability:

- eBPF-based metrics, logs, traces, and profiles;
- Community Edition MCP;
- Enterprise/Cloud AI root-cause analysis;
- low-friction self-hosted deployment.

This pass specifically tests whether Coroot now closes Parallax's wedge on
Sentry-compatible ingest, portable evidence bundles, read-only agent context,
local/open AI RCA, or coding-agent action/outcome audit.

## Verdict

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

## Current Source Snapshot

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

## Product Impact

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

## Falsification Criteria

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
