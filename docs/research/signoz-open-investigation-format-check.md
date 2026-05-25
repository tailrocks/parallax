# SigNoz Open Investigation Format Check

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-test the strongest suspicious SigNoz claim in the current research record:
does SigNoz's public "open investigation format" language mean it has published
a versioned, portable investigation or evidence-bundle schema that weakens
Parallax's A3 schema/corpus wedge?

## Verdict

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

## Sources Checked

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

## What Would Count As Closing The Gap

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

## Parallax Implication

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
