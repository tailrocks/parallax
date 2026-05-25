# Agent Access Surface Safety Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This ledger turns the CLI/HTTP/MCP access-surface decision into auditable claim
levels. It consumes the design in
[Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
and the normalization contract in
[Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md).

Current status: **not measured**. Parallax has an access-surface design, but no
implementation, projection-equivalence run, MCP client fixture, redaction run,
or audit-log result. Until those exist, Parallax should describe CLI/API/MCP as
a planned design direction, not as a proven safe agent surface.

The central rule:

> No "agent-native MCP" claim until CLI, HTTP, and MCP return the same
> canonical bundle hash for the same authorized request, and the MCP adapter
> proves read-only scope, redaction, source-field policy preservation,
> output-budget, audit, and negative-tool catalog behavior.

## Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [MCP server overview](https://modelcontextprotocol.io/specification/2025-11-25/server/index) | MCP servers expose prompts, resources, and tools, with tools as model-controlled operations and resources as application-controlled context. | Parallax should expose evidence bundles as resources and narrow tools, not generic automation power. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools are model-controlled, should keep a human in the loop, use JSON Schema input, optional output schemas, structured content, annotations, error results, optional task-support metadata, and security requirements around validation, access control, rate limiting, sanitization, confirmation, and audit logging. | Every Parallax MCP tool needs a closed schema, bounded output, audit row, and explicit denial of task-augmented execution unless a later fixture proves it safe. |
| [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization) | Authorization is optional for MCP overall; HTTP-based transports that support it should follow the spec, while stdio should retrieve credentials from the environment instead of using the HTTP authorization flow. Remote MCP uses OAuth-style authorization with protected-resource metadata, resource indicators in authorization and token requests, audience validation, HTTPS, redirects, PKCE, secure token handling, and explicit token-passthrough prohibitions. | Remote Parallax MCP cannot be a bearer-token side door into evidence; protected-resource metadata, resource indicators, audience, PKCE, and no-token-passthrough behavior need their own rows. Local stdio trust and credential-source behavior must be measured separately. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | Official guidance emphasizes least privilege, precise scope challenges, resource indicators, token audience validation, correlation IDs, and avoiding broad scopes. | The first MCP server must start read-only and deny wildcard/admin scopes. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP client/server spans, JSON-RPC request IDs, transport values, tool/resource/prompt attributes, session metrics, `elicitation/create`, `sampling/createMessage`, `notifications/tools/list_changed`, and provisional `_meta` trace propagation are defined with development-stage status. | MCP calls and server-initiated capability attempts must be observable and normalized into Parallax audit/action rows without treating development-stage semconv names as stable storage fields. |
| [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp) | OpenAI documents MCP as a docs integration surface for Codex and other agent clients. | Cross-client MCP is a distribution requirement, not a unique moat. |
| [Codex MCP](https://developers.openai.com/codex/mcp), [Codex MCP server guide](https://developers.openai.com/codex/guides/agents-sdk), [Codex config reference](https://developers.openai.com/codex/config-reference), and local `codex 0.133.0` help | Codex supports MCP in the CLI and IDE extension with shared `config.toml` configuration. Current docs cover user and trusted-project config paths, stdio and Streamable HTTP, fixed `env`, whitelisted `env_vars` with local/remote source, remote stdio placement, bearer-token env vars, static/env HTTP headers, startup/tool timeouts, enabled/required servers, enabled/disabled tools, default and per-tool approval modes, OAuth resource/scopes/callback URL/port/credential store, and plugin-provided MCP servers. Codex can also run as a stdio MCP server exposing `codex` and `codex-reply` tools with approval-policy, sandbox, config, cwd, model, and profile controls. Local help confirms `codex mcp add` and `codex mcp-server --strict-config` flags. | Codex client fixtures must record config path/trust, local versus remote env/header sources, OAuth resource/scope/callback behavior, tool approval policy, plugin origin, required-server startup behavior, and Codex-as-MCP-server topology. A Codex MCP success does not by itself prove a safe read-only Parallax context surface. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) and local `claude mcp --help` on `2.1.150` | Claude Code supports local, project, user, plugin, claude.ai connector, and managed MCP sources with source precedence. Current docs define project `.mcp.json` approval, environment expansion in command/args/env/url/headers, OAuth callback/client credentials/metadata override/scope pinning, dynamic `headersHelper` commands gated by workspace trust, output warning and limit behavior, per-tool `_meta["anthropic/maxResultSizeChars"]`, and `claude mcp serve`. Local help confirms stdio/SSE/HTTP, headers, env vars, scope, client credentials, callback port, and warns that `mcp get`/`list` skip the workspace trust dialog and spawn stdio servers for health checks. | Claude Code client fixtures must record configuration source, precedence, auth/header source, output-budget behavior, workspace trust, health-check side effects, and whether Claude-as-MCP-server is in play. Cross-client MCP safety is not proven by a generic "Claude supports MCP" row. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | NSA's May 2026 guidance treats MCP as widely adopted but security-maturing, with risks around dynamic tool invocation, implicit trust, context sharing, serialization, token/session handling, overbroad tools, and unauthorized servers. | The safe path is a narrow read-only adapter over canonical bundles, not a broad production-control toolset. |
| [Agentic observability competitor drift ledger](agentic-observability-competitor-drift-ledger.md) | MCP is already present in Sentry-adjacent, Grafana, SigNoz, Coroot, Rustrak, and GoSnag-like surfaces. | Parallax must prove safer evidence semantics, not merely MCP availability. |
| [MCP power boundary competitor check](mcp-power-boundary-competitor-check.md) | Current primary docs for Sentry, Grafana, OpenObserve, SigNoz, and Coroot show MCP surfaces ranging from coding-agent read/query to broad management, alert resolution, incident creation, ticket/Slack actions, or persisted RCA. | "Read-only" must exclude production/project mutation tools, not only generic shell and SQL tools. |
| [Lightweight error-tracker MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md) and [GoSnag Sentry AI MCP recheck](gosnag-sentry-ai-mcp-recheck.md) | Current primary docs/source for Rustrak and GoSnag show MCP surfaces in small Sentry-compatible trackers, including project/issue/event/token/alert/ticket/user management and raw Sentry-envelope or recent-event access. GoSnag's checked MCP is a Bearer-token management API wrapper over projects/issues/alerts/tags/tickets/users. | MCP availability is now table stakes even for lightweight competitors. Parallax's claim must be read-only, redacted, canonical bundle projection plus audit/outcome semantics, not "has MCP." |

Updated implication from the A1/A6 source-field pass: projection equivalence now
means preserving the full canonical bundle safety contract, not just producing a
similar user-visible answer. MCP tools that return bundles must expose
`structuredContent` conforming to the evidence-bundle output schema, including
`redaction_report.source_field_policy`. Text-only JSON or Markdown can be a
compatibility projection, but it cannot be the evidence that unlocks an
agent-native claim.

## Claim Levels

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No access-surface fixture run exists. | "CLI, HTTP, and MCP access surfaces are planned." |
| `cli_http_bundle_parity` | CLI JSON and HTTP API return the same canonical bundle hash for the same authorized request. | "CLI and API expose the same evidence bundle contract." |
| `mcp_local_smoke` | A local MCP server starts, lists only approved Parallax tools/resources, and returns a redacted sample bundle. | "Experimental local MCP context adapter." |
| `mcp_projection_equivalent` | CLI, HTTP, and MCP return the same canonical bundle hash and equivalent redaction/source-field report for the same principal/project/anchor/window/schema. | "MCP projects the same canonical evidence bundle as CLI/API." |
| `mcp_read_only_safe` | Scope, negative-tool, raw-ref, redaction, source-field, prompt-injection, output-budget, and audit fixtures pass. | "Read-only MCP context adapter for the tested policy set." |
| `mcp_cross_client_safe` | At least Codex and Claude Code can call the same local/remote MCP fixture with equivalent hashes, scopes, denials, and audit rows. | "Cross-client MCP context adapter for tested clients." |
| `agent_native_context_surface` | MCP safety, projection equivalence, OTel MCP audit spans, bundle schema conformance, source-field checks, and redaction ledgers are green and fresh. | "Agent-native evidence context surface for the tested clients and policies." |
| `claim_expired` | MCP spec/security guidance/client behavior/Parallax schema/redaction/auth/audit code changed or freshness elapsed. | "Access-surface result expired; rerun required." |
| `claim_failed` | Any required fixture fails for the advertised level. | No claim for the affected surface/client/policy. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Access-surface runs should be durable and diffable:

```text
docs/research/agent-access-surface-results.md
docs/research/agent-access-surface-runs/<run_id>/manifest.json
docs/research/agent-access-surface-runs/<run_id>/canonical-bundles/<case_id>.json
docs/research/agent-access-surface-runs/<run_id>/cli-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/http-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/mcp-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/capability-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/client-config-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/scope-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/auth-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/stdio-trust-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/redaction-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/source-field-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/output-budget-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/negative-tool-catalog.json
docs/research/agent-access-surface-runs/<run_id>/audit-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/otel-mcp-spans.jsonl
docs/research/agent-access-surface-runs/<run_id>/claim-ledger.jsonl
docs/research/agent-access-surface-runs/<run_id>/hashes.sha256
```

Do not create run directories for hypothetical data. Add them only when a real
fixture run exists.

## Run Manifest

Each `manifest.json` should include:

```json
{
  "run_id": "agent-access-surface-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_commit": "<git-sha>",
  "bundle_schema_version": "parallax-bundle-vN",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "auth_policy_version": "agent-access-auth-vN",
  "audit_schema_version": "parallax-audit-vN",
  "source_snapshot": {
    "mcp_spec": "2025-11-25",
    "mcp_spec_latest_label_checked": "2025-11-25 (latest on official site)",
    "otel_semconv": "1.41.0",
    "otel_mcp_semconv_status": "development",
    "otel_mcp_example_protocol_version": "2025-06-18",
    "codex_client": "<version-or-snapshot>",
    "claude_code_client": "<version-or-snapshot>",
    "lightweight_mcp_watch": "rustrak_gosnag_management_raw_event_mcp_checked_2026-05-25"
  },
  "surfaces": ["cli", "http", "mcp"],
  "clients": ["codex", "claude-code"],
  "transport_modes": ["stdio", "streamable-http"],
  "mcp_features_allowed": ["tools", "resources"],
  "mcp_features_denied": ["sampling", "elicitation", "task-augmented execution for context tools"],
  "client_config_matrix": ["codex:stdio", "codex:streamable-http", "claude-code:stdio", "claude-code:http"],
  "notes": []
}
```

The manifest must separate protocol/spec versions, client versions, Parallax
bundle/redaction/auth/audit versions, and transport modes. A pass in one
combination does not carry over to another.

## Row Schemas

### Projection Result Row

```json
{
  "case_id": "issue_context_basic",
  "surface": "cli|http|mcp",
  "client": "human|codex|claude-code|null",
  "principal": "agent-readonly",
  "project_id": "demo",
  "anchor_type": "issue|trace|agent_session|cli_invocation",
  "anchor_id": "issue_123",
  "window": "2026-05-25T00:00:00Z/PT15M",
  "schema_version": "parallax-bundle-vN",
  "canonical_bundle_hash": "sha256:<hex>",
  "redaction_report_hash": "sha256:<hex>",
  "source_field_policy_hash": "sha256:<hex>|null",
  "status": "pass|fail",
  "differences": []
}
```

### MCP Tool Catalog Row

```json
{
  "run_id": "agent-access-surface-YYYYMMDD-N",
  "client": "codex",
  "tool_names": [
    "parallax_issue_context",
    "parallax_trace_context",
    "parallax_hypothesis_check",
    "parallax_agent_session_show",
    "parallax_cli_invocation_show"
  ],
  "resource_prefixes": ["parallax://bundles/", "parallax://evidence/"],
  "forbidden_tools_present": [],
  "forbidden_tool_classes_present": [],
  "all_tools_have_input_schema": true,
  "bundle_tools_have_output_schema": true,
  "structured_content_schema_valid": true,
  "task_support_for_context_tools": "forbidden",
  "annotations_treated_as_untrusted": true,
  "tools_list_changed_notifications_audited": true,
  "all_context_tools_read_only": true
}
```

### MCP Client Configuration Row

```json
{
  "run_id": "agent-access-surface-YYYYMMDD-N",
  "client": "claude-code",
  "client_version": "2.1.150",
  "client_binary_path": "/home/agent/.local/bin/claude",
  "transport": "stdio|sse|http",
  "config_path": "~/.codex/config.toml|.codex/config.toml|~/.claude.json|.mcp.json|cli_inline|unknown",
  "config_source": "local|project|user|plugin|claude_ai|managed|cli_inline",
  "source_precedence_observed": ["local", "project", "user", "plugin", "claude_ai"],
  "project_scope_approval_required": true,
  "trusted_project_required": true,
  "project_scope_approval_reset_tested": true,
  "health_check_spawns_stdio": true,
  "env_var_expansion_fields": ["command", "args", "env", "url", "headers"],
  "static_header_sources": [],
  "headers_helper_present": false,
  "headers_helper_workspace_trusted": false,
  "oauth_metadata_override_present": false,
  "oauth_scopes_pinned": false,
  "client_output_warning_threshold_tokens": 10000,
  "client_output_limit_tokens": 25000,
  "tool_meta_max_result_size_chars": null,
  "tool_result_persisted_to_disk": false,
  "claude_mcp_serve_exposed": false,
  "codex_config": {
    "project_config_trusted": false,
    "env_vars_sources": ["local", "remote"],
    "experimental_environment": "local|remote|null",
    "startup_timeout_sec": 10,
    "tool_timeout_sec": 60,
    "required_server": false,
    "enabled_tools": [],
    "disabled_tools": [],
    "default_tools_approval_mode": "auto|prompt|approve|null",
    "per_tool_approval_modes": {},
    "oauth_resource": null,
    "oauth_callback_port": null,
    "oauth_callback_url": null,
    "oauth_credentials_store": "auto|file|keyring|null",
    "plugin_provided_server": false,
    "codex_mcp_server_exposed": false,
    "mcp_server_strict_config": false
  },
  "notes": []
}
```

This row is mandatory for client-specific fixture claims. It is especially
important because Codex and Claude Code make different trust, config, and
output-budget decisions even when they call the same Parallax MCP server.

### MCP Capability Result Row

```json
{
  "case_id": "sampling_denied",
  "client": "codex",
  "server_feature": "sampling|elicitation|task_support|tools_list_changed",
  "requested_or_observed": true,
  "allowed": false,
  "status": "pass|fail",
  "audit_row_emitted": true,
  "otel_mcp_span_present": true,
  "notes": "First Parallax MCP server allows tools/resources only."
}
```

### Scope Result Row

```json
{
  "case_id": "raw_ref_denied",
  "tool": "parallax_raw_ref_read",
  "principal": "agent-readonly",
  "requested_scope": "evidence:read_sensitive",
  "granted": false,
  "status_code": "permission_denied",
  "audit_row_emitted": true,
  "correlation_id": "corr_123"
}
```

### Authorization Result Row

```json
{
  "case_id": "remote_mcp_resource_indicator",
  "transport": "streamable-http",
  "principal": "agent-readonly",
  "protected_resource_metadata_present": true,
  "resource_parameter_in_authorization_request": true,
  "resource_parameter_in_token_request": true,
  "token_audience_validated": true,
  "token_audience_bound_to_mcp_server": true,
  "pkce_s256_required": true,
  "token_passthrough_denied": true,
  "https_required": true,
  "localhost_redirect_policy_checked": true,
  "precise_scope_challenge": true,
  "downscoped_token_accepted": true,
  "status": "pass|fail",
  "audit_row_emitted": true
}
```

### Local Stdio Trust Result Row

```json
{
  "case_id": "local_stdio_trust",
  "transport": "stdio",
  "client": "codex|claude-code",
  "client_config_source": "local|project|user|plugin|managed|cli_inline",
  "trusted_project_required": true,
  "explicit_install_trust_required": true,
  "repo_checkout_auto_enable_denied": true,
  "project_scope_approval_required": true,
  "health_check_spawns_stdio": false,
  "approved_credential_sources": ["environment", "local_config"],
  "codex_env_vars_sources": ["local"],
  "codex_remote_stdio_used": false,
  "env_var_expansion_values_logged": false,
  "dynamic_header_helper_executed": false,
  "ambient_token_forwarding_denied": true,
  "credential_values_logged": false,
  "status": "pass|fail",
  "audit_row_emitted": true
}
```

### Redaction Result Row

```json
{
  "case_id": "seeded_cli_output_secret",
  "surface": "mcp",
  "seeded_canaries": 12,
  "leaked_canaries": 0,
  "raw_refs_leaked": 0,
  "redaction_report_present": true,
  "source_field_policy_status": "pass|not_applicable",
  "source_field_policy_violations": 0,
  "redaction_policy_version": "a6-default-deny-vN"
}
```

### Source Field Result Row

```json
{
  "case_id": "eval_bundle_source_policy",
  "surface": "mcp",
  "client": "codex",
  "bundle_id": "bundle_123",
  "source_field_policy_status": "pass",
  "source_field_policy_hash": "sha256:<hex>",
  "source_field_policy_violations": 0,
  "denied_zones_checked": ["runner_private", "grader_private", "triage_private"],
  "structured_content_schema_valid": true,
  "text_projection_matches_canonical_hash": true,
  "status": "pass"
}
```

### Output Budget Result Row

```json
{
  "case_id": "oversized_logs",
  "surface": "mcp",
  "client": "claude-code",
  "inline_bytes": 24576,
  "max_inline_bytes": 32768,
  "client_warning_threshold_tokens": 10000,
  "client_limit_tokens": 25000,
  "tool_meta_max_result_size_chars": null,
  "resource_refs": 4,
  "truncated": true,
  "client_persisted_result_to_disk": false,
  "client_file_ref_agent_visible": false,
  "canonical_hash_preserved": true
}
```

### Audit Result Row

```json
{
  "case_id": "mcp_issue_context",
  "surface": "mcp",
  "tool": "parallax_issue_context",
  "principal": "agent-readonly",
  "scopes": ["evidence:read"],
  "bundle_id": "bundle_123",
  "status": "success",
  "audit_row_present": true,
  "otel_mcp_span_present": true,
  "jsonrpc_request_id_present": true
}
```

### Claim Ledger Row

```json
{
  "run_id": "agent-access-surface-YYYYMMDD-N",
  "claim_level": "mcp_read_only_safe",
  "claim_status": "pass|fail|expired",
  "version_matrix": {
    "mcp_spec": "2025-11-25",
    "otel_semconv": "1.41.0",
    "bundle_schema": "parallax-bundle-vN",
    "redaction_policy": "a6-default-deny-vN",
    "source_field_policy": "phase0-source-field-policy-vN"
  },
  "product_wording": "Read-only MCP context adapter for the tested policy set.",
  "required_caveats": ["no generic shell tools", "no generic SQL tools"],
  "expires_at": "YYYY-MM-DD"
}
```

## Counting Rules

- No "agent-native MCP" claim without projection equivalence across CLI, HTTP,
  and MCP for the same authorized request.
- No MCP differentiation claim based on protocol presence alone. Competitors at
  both observability-suite and lightweight-error-tracker levels now expose MCP;
  Parallax claims must compare evidence-bundle shape, redaction/source-field
  proof, auditability, output bounds, and outcome loop.
- No schema-safe MCP claim unless bundle-returning tools have output schemas and
  valid `structuredContent`; Markdown/text alone is a projection.
- No read-only context claim if sampling, elicitation, task-augmented execution,
  or unreviewed `tools/list_changed` behavior can expand what the server asks of
  the client or model during a context request.
- No "safe MCP" claim unless negative tools are absent: no generic shell, SQL,
  deploy, rollback, delete, or broad production-control tools.
- No read-only context claim if the first context server includes alert,
  dashboard, saved-view, stream, sourcemap, role, user, organization, pipeline,
  notification-channel, incident, ticket, or search-job create/update/delete
  tools.
- No read-only context claim if the first context server can resolve or suppress
  alerts, send notifications, create incidents, update tickets, or persist RCA
  onto a production incident. Append-only Parallax audit/outcome rows are a
  separate allowed write only after their own fixtures pass.
- No eval/corpus-derived bundle claim unless `source_field_policy_status` is
  `pass`, the policy hash is present, and violations are zero across CLI, HTTP,
  and MCP.
- No raw-reference read claim unless `evidence:read_sensitive` denial and
  approval paths are tested and audited.
- No cross-client claim unless at least Codex and Claude Code pass the same
  fixture suite and publish client configuration rows. The rows must identify
  transport, config source, source precedence, auth/header source, output-limit
  behavior, and any client-specific server/tool hiding.
- No remote MCP claim unless protected-resource metadata, resource indicators
  in authorization and token requests, token audience validation, PKCE S256,
  HTTPS, localhost redirect policy, precise scope challenges, down-scoping
  tolerance, and token-passthrough denial are tested separately from local
  stdio mode.
- No local stdio MCP claim unless explicit install/trust, approved credential
  sources, no repository auto-enable, no ambient-token forwarding, and
  credential log redaction are proven for the tested client.
- Codex client claims must record whether configuration came from
  `~/.codex/config.toml`, trusted-project `.codex/config.toml`, CLI override,
  or plugin-provided MCP server configuration. Project-scoped config cannot
  stand in for trusted install behavior unless the trust state is recorded.
- Codex stdio fixture rows must distinguish fixed `env` values from whitelisted
  `env_vars`, and local versus remote environment sources. Remote stdio
  placement, remote env sources, and `experimental_environment = "remote"` are
  separate transport/topology claims.
- Codex HTTP and OAuth fixture rows must record `bearer_token_env_var`, static
  `http_headers`, `env_http_headers`, `oauth_resource`, configured scopes,
  server-advertised scopes, callback port or URL, and credentials-store mode.
  A successful OAuth login does not prove least privilege if scopes or resource
  indicators were not captured.
- Codex tool-policy claims must record `enabled_tools`, `disabled_tools` after
  allow-list filtering, default tool approval mode, per-tool approval modes,
  startup/tool timeouts, and whether `required = true` caused fail-closed
  startup or resume behavior.
- Codex plugin-provided MCP servers must record plugin origin and the user
  config that controls enablement and tool policy under the plugin server key.
- Codex `mcp-server` exposes `codex` and `codex-reply` tools that can run Codex
  sessions with approval-policy, sandbox, config, cwd, model, and profile
  controls. Treat it as a separate automation topology, not as proof that
  Parallax's read-only context MCP server is safe.
- Claude Code client claims must record local/project/user/plugin/claude.ai
  source precedence, project `.mcp.json` approval, reset behavior for project
  choices, and whether `mcp get`/`mcp list` or other health checks start stdio
  servers. Do not run health-check probes against untrusted project configs.
- Claude Code dynamic header helpers are local code execution. A client fixture
  must record workspace trust, command identity, secret source, and redaction
  before treating `headersHelper` as safe.
- Claude Code OAuth fixture rows must record callback port/client credentials,
  metadata override, pinned scopes, and any offline-access behavior rather than
  assuming a generic OAuth success proves least privilege.
- Claude Code `claude mcp serve` exposes Claude Code tools to another MCP
  client. It is not a substitute for the Parallax MCP context server; if present
  in a fixture, record it as a separate client/server topology and keep it out
  of read-only Parallax server claims.
- No prompt-injection safety claim unless malicious telemetry, issue text, PR
  text, logs, and transcripts fail to change tool policy, scopes, windows,
  redaction, or output limits.
- No output-budget claim if MCP can inline unbounded logs, traces, transcripts,
  terminal output, or source-map data. Client-side output warnings, default
  limits, `_meta["anthropic/maxResultSizeChars"]`, and persisted-file
  substitutions do not replace Parallax's own bounded bundle contract.
- No audit claim unless allowed and denied calls produce audit rows and OTel MCP
  spans with correlation ids.
- Markdown output is a projection. Claim levels are based on canonical JSON
  bundle hashes, not visual similarity.

## Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- MCP specification, authorization guidance, or security guidance changes;
- MCP task-augmented execution, sampling, elicitation, or dynamic tool-list
  behavior changes;
- OpenTelemetry semantic conventions or MCP semantic conventions change;
- Codex, Claude Code, Cursor, VS Code/Copilot, or other claimed clients change
  MCP configuration, output limits, auth, or resource behavior;
- a lightweight Sentry-compatible or OTLP-native competitor ships a read-only,
  redacted, schema-bound MCP evidence bundle with projection-equivalence hashes;
- Parallax bundle schema, redaction policy, auth policy, audit schema, or tool
  catalog changes;
- source-field policy or eval/corpus source-field schema changes;
- new agent-visible evidence surfaces are added, especially database, raw logs,
  frontend replay, terminal output, or agent transcript content;
- a dependency CVE or security advisory affects the MCP server, auth stack, or
  serialization path;
- 60 days pass since the last run.

## Product Wording

Allowed after `cli_http_bundle_parity`:

> CLI and HTTP API expose the same canonical evidence bundle contract for the
> tested cases.

Allowed after `mcp_projection_equivalent`:

> MCP projects the same canonical evidence bundle as CLI/API for the tested
> clients and policies, including redaction and source-field policy status.

Allowed after `mcp_read_only_safe`:

> Read-only MCP context adapter for the tested policy set, with redaction,
> source-field checks, output limits, denied raw refs, and audit logging.

Avoid:

- "MCP-native" as a differentiator;
- "safe for agents" without fixture results;
- "read-only" if any tool can mutate production systems or project state beyond
  outcome records;
- "works with all MCP clients";
- "secure MCP" without naming tested auth mode, client, policy, and spec
  revision.

## Relationship To Other Research

- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  makes the access-surface decision this ledger turns into claim levels.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  canonical bundle whose hash must match across surfaces.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
  owns MCP span normalization and action/audit deduplication.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  and [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) have veto
  power before any agent-visible claim, including source-field preservation.
- [Production database evidence access gate](production-database-evidence-access.md)
  keeps generic SQL tools out of the context server.
- [Production database evidence ledger](production-database-evidence-ledger.md)
  defines the stricter claim contract before any direct database evidence can
  appear through CLI, HTTP, or MCP.
- [Agentic observability competitor drift ledger](agentic-observability-competitor-drift-ledger.md)
  explains why MCP availability alone is no longer differentiating.
- [MCP power boundary competitor check](mcp-power-boundary-competitor-check.md)
  records the current competitor tool-power matrix behind the negative-tool and
  management-tool rules.

## Bottom Line

Parallax should ship CLI and HTTP first, then MCP as a thin read-only projection
of the same canonical bundle. The market already has MCP. The product claim
must be that Parallax's MCP surface is equivalent, bounded, redacted, audited,
source-field-safe, and narrow enough for coding agents to use safely.
