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
breakdown into an incident timeline.

The checked primary sources did **not** expose a source-linked, versioned,
portable investigation schema or artifact with provenance, redaction, raw-ref,
query-manifest, missing-evidence, or outcome semantics. Treat the phrase and the
evidence-pack workflow as competitive pressure signals, not as evidence that
SigNoz has closed Parallax's open evidence-bundle moat.

## Sources Checked

| Source | Current evidence | Interpretation |
| --- | --- | --- |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/) and [page source](https://github.com/SigNoz/signoz.io/blob/main/app/%28site%29/agent-native-observability/AgentNativeObservabilityPage.constants.tsx) | Positions SigNoz inside coding agents, says investigation data stays with the user, and says the "open investigation format" can become a team standard. The source file shows the same language in product-page copy rather than a linked artifact spec. | The phrase is real and official. It validates market direction, but it is still product/workflow language unless paired with a schema, validator, or exportable artifact. |
| [SigNoz Postmortem Evidence Pack](https://signoz.io/docs/ai/use-cases/postmortem-evidence-pack/) and [source MDX](https://github.com/SigNoz/signoz.io/blob/main/data/docs/ai/use-cases/postmortem-evidence-pack.mdx) | Docs page dated 2026-04-24 shows an assistant prompt that compiles an incident timeline from alert transitions, metric inflection points, representative errors, and trace details. The "Under the Hood" table maps the workflow to `signoz_get_alert_history`, `signoz_query_metrics`, `signoz_search_logs`, `signoz_search_traces`, and `signoz_get_trace_details`. | This is closer to Parallax's evidence-pack language than the landing page alone. It is still an example LLM response/workflow, not a versioned portable evidence object with conformance tests, provenance fields, redaction report, raw refs, missing-evidence flags, or outcome rows. |
| [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/) | Docs page dated 2026-05-13 covers Claude Desktop, Claude Code, OpenAI Codex, Cursor, GitHub Copilot, Gemini CLI, Windsurf, Zed, hosted MCP, and self-hosted MCP. | Agent access is real and current. The docs describe connection and tools, not a canonical investigation artifact. |
| [`SigNoz/signoz-mcp-server` README](https://github.com/SigNoz/signoz-mcp-server) | README describes natural-language access to metrics, traces, logs, alerts, dashboards, and services; lists tools such as `signoz_query_metrics`, `signoz_search_logs`, `signoz_aggregate_traces`, `signoz_get_trace_details`, `signoz_execute_builder_query`, and create/update/delete tools for alerts, dashboards, saved views, and notification channels. It also says every tool accepts `searchContext` for MCP observability and does not forward that field to SigNoz APIs. | The MCP surface is primarily query plus management with some MCP self-observability metadata. It is not shaped like Parallax's intended read-only bundle projection. |
| [`SigNoz/agent-skills`](https://github.com/SigNoz/agent-skills) | Repository metadata checked on 2026-05-25 showed latest push on 2026-05-19. README describes official SigNoz skills/plugins for Claude Code, Codex, Cursor, and the skills.sh ecosystem; available skills include MCP setup, creating/explaining alerts, investigating alerts, dashboards, query generation, ClickHouse queries, docs search, and saved views. | SigNoz is turning agent workflows into distributable Markdown skills. That is relevant adoption pressure, but the checked tree exposes workflow skills rather than a portable investigation/evidence artifact schema. |
| [`SigNoz/signoz-mcp-server` tree scan](https://github.com/SigNoz/signoz-mcp-server) and [`SigNoz/signoz` tree scan](https://github.com/SigNoz/signoz) | Path scans for investigation, postmortem, evidence, bundle, artifact, schema, and format terms found dashboard schema/builder files, schema compatibility helpers, manifests, and planning docs, but no obvious investigation artifact schema. | This is not as strong as full authenticated code search, but it narrows the previous limitation: the likely public schema file names were not visible in the checked repositories. |
| [`signoz-mcp-server` `v0.4.1` release](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.4.1) | GitHub API check returned latest release `v0.4.1`, published 2026-05-21. Release body fixes Query Builder typed round-trip for PromQL and ClickHouse SQL. | The MCP server is active and query semantics are still being refined. The latest release did not publish an investigation schema. |
| [SigNoz `v0.125.1` release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1) | GitHub API check returned latest SigNoz release `v0.125.1`, published 2026-05-20. | The platform is active. This freshness raises watch priority but does not change the schema finding. |

Unauthenticated GitHub code search for exact phrases was not usable through the
public API during the earlier pass because it requires authentication. This pass
used repository tree/path scans instead. That is still weaker than full code
search and not proof that no schema exists anywhere. The claim is therefore
bounded to the checked official landing page, docs/use-case pages, MCP README,
agent-skills repository, tree/path scans, and release metadata.

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
- Watch SigNoz closely because it could turn the current evidence-pack workflow
  and open-format language into a real open schema quickly.

Prompt update: needed in this pass because the durable prompt should remember
that SigNoz now has a named postmortem evidence-pack use case, not only a
landing-page phrase.
