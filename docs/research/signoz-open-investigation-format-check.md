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
become an "open investigation format" for a team.

The checked primary sources did **not** expose a source-linked, versioned,
portable investigation schema or artifact with provenance, redaction, raw-ref,
query-manifest, missing-evidence, or outcome semantics. Treat the phrase as a
competitive pressure signal, not as evidence that SigNoz has closed Parallax's
open evidence-bundle moat.

## Sources Checked

| Source | Current evidence | Interpretation |
| --- | --- | --- |
| [SigNoz agent-native observability](https://signoz.io/agent-native-observability/) | Positions SigNoz inside coding agents and says the same investigation workflow can become an "open investigation format" a team standardizes on. | This is the only checked source where the phrase appears. It validates market direction, but it is product/workflow language unless paired with a schema or artifact. |
| [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/) | Docs page dated 2026-05-13 covers Claude Desktop, Claude Code, OpenAI Codex, Cursor, GitHub Copilot, Gemini CLI, Windsurf, Zed, hosted MCP, and self-hosted MCP. | Agent access is real and current. The docs describe connection and tools, not a canonical investigation artifact. |
| [`SigNoz/signoz-mcp-server` README](https://github.com/SigNoz/signoz-mcp-server) | README describes natural-language access to metrics, traces, logs, alerts, dashboards, and services; lists tools such as `signoz_query_metrics`, `signoz_search_logs`, `signoz_aggregate_traces`, `signoz_get_trace_details`, `signoz_execute_builder_query`, and create/update/delete tools for alerts, dashboards, saved views, and notification channels. | The MCP surface is primarily query plus management. It is not shaped like Parallax's intended read-only bundle projection. |
| [`signoz-mcp-server` `v0.4.1` release](https://github.com/SigNoz/signoz-mcp-server/releases/tag/v0.4.1) | GitHub API check returned latest release `v0.4.1`, published 2026-05-21. Release body fixes Query Builder typed round-trip for PromQL and ClickHouse SQL. | The MCP server is active and query semantics are still being refined. The latest release did not publish an investigation schema. |
| [SigNoz `v0.125.1` release](https://github.com/SigNoz/signoz/releases/tag/v0.125.1) | GitHub API check returned latest SigNoz release `v0.125.1`, published 2026-05-20. | The platform is active. This freshness raises watch priority but does not change the schema finding. |

Unauthenticated GitHub code search for exact phrases was not usable through the
public API during this pass because it requires authentication. That is a search
limitation, not proof that no schema exists anywhere. The claim is therefore
bounded to the checked official landing page, docs page, MCP README, and release
metadata.

## What Would Count As Closing The Gap

A future SigNoz source should be treated as A3-relevant if it publishes any of
the following:

- a JSON/YAML/Protobuf schema for a canonical investigation object;
- positive and negative fixtures or conformance tests for that object;
- an exported investigation artifact that can be validated without a live SigNoz
  tenant;
- an MCP structured output schema whose canonical payload is an investigation
  artifact rather than free-form text or ad hoc query results;
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
- Watch SigNoz closely because it could turn the current workflow language into
  a real open schema quickly.

Prompt update: not needed in this pass. The active deep-research prompt already
requires watching SigNoz's open-investigation-format claim and rechecking whether
it becomes a published schema or portable artifact.
