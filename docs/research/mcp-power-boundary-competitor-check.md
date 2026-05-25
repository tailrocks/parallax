# MCP Power Boundary Competitor Check

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-test the claim that Parallax's first MCP surface should be a narrow read-only
bundle adapter, not a broad observability management MCP. The suspicious part of
the current research record is that "write-capable competitor MCP" could become
lazy positioning unless the repository records exactly what current primary
sources show.

## Verdict

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

## Source Matrix

| Source | Current power shape | Parallax interpretation |
| --- | --- | --- |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools are model-invoked, can expose input and output schemas, can return `structuredContent`, can notify tool-list changes, and the spec's security section requires validation, access control, rate limiting, sanitization, confirmation for sensitive operations, and audit logging. | Parallax should use MCP structured output for bundles, but protocol support does not by itself prove safety. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | Current guidance calls out token passthrough, local MCP server compromise, session hijack, SSRF, broad scopes, and local server execution risk. Scope minimization recommends small initial scopes, targeted elevation, down-scoping tolerance, and correlation-id logging. | A broad all-in-one observability MCP makes the first Parallax safety claim harder. Start with a low-risk read-only scope and no admin tools. |
| [Sentry MCP repository](https://github.com/getsentry/sentry-mcp), [Sentry MCP stdio testing guide](https://github.com/getsentry/sentry-mcp/blob/master/docs/testing-stdio.md), and [Sentry MCP releases](https://github.com/getsentry/sentry-mcp/releases) | Sentry says its MCP service targets human-in-the-loop coding agents rather than every Sentry API. The checked release page shows `0.35.0` on 2026-05-21. The README describes stdio for self-hosted Sentry as work in progress, lists setup scopes including `project:write`, `team:write`, and `event:write`, and says AI-powered search needs OpenAI or Anthropic provider configuration. The stdio testing guide also documents a read-only testing scope set: `org:read`, `project:read`, `team:read`, and `event:read`. | Sentry is not simply a broad admin MCP, and the old "no documented read-only path" objection is too strong. Parallax should still not claim MCP parity until it proves read-only tool availability, projection equivalence, redaction, and bundle output under those narrower scopes. |
| [Grafana Assistant MCP servers](https://grafana.com/docs/grafana-cloud/machine-learning/assistant/configure/mcp-servers/) and [Grafana Incident/Sift MCP guide](https://grafana.com/docs/grafana/latest/developer-resources/mcp/guides/use-grafana-incident-and-sift/) | Grafana Assistant supports only remote MCP servers, requires operators to trust connected servers and review tool calls, and frames MCP around issue/code lookup, ticket creation, Slack notifications, and similar actions. The Incident/Sift guide separates Viewer read operations from Editor write operations such as creating incidents, adding incident activity, and running Sift analyses that create investigations. | Grafana validates a human-approved action model. Parallax's first surface should be a context server, not an incident/ticket/action automation server. |
| [OpenObserve MCP docs](https://openobserve.ai/docs/integration/ai/mcp/) | OpenObserve documents a large MCP catalog with many create/update/delete/admin operations. Checked tool categories include alerts, authorization roles, dashboards, folders, functions, KV, organizations/settings, pipelines, search jobs, service accounts, sourcemaps, streams, and users, with destructive tools marked. It recommends a dedicated MCP user, secret handling, credential rotation, and client-side confirmation when possible. | OpenObserve is the clearest evidence that agentic observability MCP can become a management plane. Parallax must avoid copying that shape for the first context adapter. |
| [SigNoz MCP server docs](https://signoz.io/docs/ai/signoz-mcp-server/) and [SigNoz open investigation format check](signoz-open-investigation-format-check.md) | SigNoz exposes hosted and self-hosted MCP. The checked README/docs list metrics/logs/traces/query tools plus create/update/delete operations for alerts, dashboards, saved views, and notification channels, and a raw Query Builder tool. | SigNoz proves open self-hosted MCP is table stakes. Its tool shape is query plus management, not Parallax-style read-only bundle projection. |
| [Coroot MCP overview](https://docs.coroot.com/mcp/overview/) and [Coroot releases](https://github.com/coroot/coroot/releases) | Coroot's latest checked release page shows `1.20.2` as latest and includes the MCP server. The MCP docs use OAuth 2.0 and server-side RBAC; Community tools expose topology, alerts, incidents, traces, logs, metrics, raw telemetry, `select_project`, and `resolve_alerts`. Enterprise adds `investigate_anomaly`, which can persist RCA onto an incident when an incident key is supplied. | Coroot has a stronger auth story than many MCP examples, but it still mixes live production query, alert resolution, and persisted RCA. Treat it as a serious baseline, not as proof Parallax should add write tools early. |

GitHub REST release checks for several repositories were rate-limited during this
pass, so this note does not replace the existing release-version rows in the
competitor drift ledger. The claims above are about documented MCP tool power,
not about a new release-recency audit.

The same boundary now applies below the large observability-suite tier. The
[Lightweight error-tracker MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md)
found Rustrak and GoSnag MCP surfaces in small error trackers, including
project/issue/event/token/alert/ticket/user management and raw Sentry-envelope
event access. That makes MCP availability a table-stakes feature, not a moat,
while strengthening the case for a Parallax first server that is read-only,
redacted, schema-bound, and projection-equivalent.

## Product Boundary

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

## Falsification Criteria

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
