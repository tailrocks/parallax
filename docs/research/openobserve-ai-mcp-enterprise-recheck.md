# OpenObserve AI/MCP Enterprise Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

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

## Verdict

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

## Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [OpenObserve `v0.90.2` release](https://github.com/openobserve/openobserve/releases/tag/v0.90.2) | Latest checked GitHub release was published 2026-05-22. | Active release train; do not treat OpenObserve as a static baseline. |
| [OpenObserve homepage](https://openobserve.ai/) | Positions OpenObserve as unified logs/metrics/traces/RUM with object storage, SQL/PromQL, one-binary or Helm deployment, AI SRE Agent, AI Assistant, and MCP. Homepage FAQ says Self-Hosted Enterprise is free up to `200 GB/day`, says the OSS plan has no usage limits, and says AI Assistant is included in Self-Hosted Enterprise and Cloud Enterprise. | Strong threat to broad positioning, but allowance conflicts with pricing/docs. Do not describe the Enterprise boundary as strictly paid at small volumes. |
| [OpenObserve pricing](https://openobserve.ai/pricing/), [Enterprise features](https://openobserve.ai/docs/features/enterprise/), and [license/pricing docs](https://openobserve.ai/docs/enterprise-setup/license-and-pricing/) | Pricing/docs say Self-Hosted Enterprise is free up to `50 GB/day`; the license docs say Enterprise works without a license key up to `10 GB/day`, requires requesting a license above that, and identifies the free-tier license as `<= 50 GB/day`. Pricing says AI-powered features are free during preview with credits. | Legitimate ops-feature comparable; exact free allowance remains unresolved. The right claim is "Enterprise-tier with a source-conflicted free allowance," not simply "paid." |
| [OpenObserve SRE Agent setup](https://openobserve.ai/docs/enterprise-setup/sre-agent/) | SRE Agent powers AI Assistant, incidents, and RCA in OpenObserve Enterprise; requires Enterprise license, AI provider key, and `O2_AI_ENABLED=true`. Setup docs list Anthropic, OpenAI, Gemini, direct, bundled gateway, and external/self-hosted gateway paths. | AI/RCA is real but not plain AGPL Community evidence. |
| [OpenObserve AI SRE product page](https://openobserve.ai/ai-sre/) | Says AI SRE is an OpenObserve Enterprise background service; describes context assembly over logs, metrics, traces, and dependency maps; presents evidence-chain/audit-trail investigation language; says the agent uses MCP to navigate OpenObserve tools; and supports OpenAI, Anthropic Claude, Gemini, AWS Bedrock, DeepSeek, OpenRouter, and OpenAI-compatible/self-hosted endpoints. | Provider flexibility and evidence-chain positioning are stronger than earlier notes; they increase threat but do not prove a portable evidence-bundle schema, export, redaction report, or outcome contract. |
| [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/) | MCP is Enterprise-only, uses `https://your-instance/api/{org_id}/mcp`, supports Claude Code/Cursor/VS Code/ChatGPT connectors and other agents, and exposes query plus broad management/destructive/admin tools. | Strongest current reason Parallax's first MCP surface must stay read-only and bundle-shaped. |
| [OpenObserve OTLP docs](https://openobserve.ai/docs/ingestion/logs/otlp/) | Supports OTLP/HTTP and OTLP/gRPC for logs, metrics, and traces. | OTLP overlap is proven; Sentry-envelope/DSN migration was not found in checked ingestion docs. |
| [OpenObserve docs search index](https://openobserve.ai/docs/search/search_index.json) | Exact search for `sentry` returned no matches in the current docs index. | Negative evidence only: it supports keeping Sentry migration unproven, but should be rechecked because search indexes can lag source pages. |

## Product Impact

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

## Falsification Criteria

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
