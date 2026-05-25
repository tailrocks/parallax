# Agent and CLI OTel Semantic-Convention Mapping

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

Parallax already treats coding-agent sessions and CLI invocations as first-class
execution evidence. The weak point is not the idea; it is the boundary between
fast-moving OpenTelemetry semantic conventions and Parallax's durable evidence
schema.

This note defines the contract:

> OpenTelemetry GenAI, MCP, CLI, process, and CI/CD semantic conventions are
> ingestion vocabulary, not Parallax's storage contract. Parallax should map
> those spans into stable `agent_session`, `agent_action`, `cli_invocation`,
> `ci_run`, and audit-edge records, record the source convention version, and
> report lossiness whenever a vendor trace cannot prove an action.

This keeps Parallax interoperable without letting development-stage conventions
or vendor-specific trace shapes become the product schema.

## Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | GenAI, MCP, CLI, process, CI/CD, VCS, exception, and test areas are present in the current semantic convention catalog. | Parallax can reuse OTel names for ingestion and adapter tests, but should not require every source to emit every convention. |
| [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/) | GenAI agent conventions are development-stage; existing instrumentations should not change emitted convention versions by default and may use `OTEL_SEMCONV_STABILITY_OPT_IN=gen_ai_latest_experimental`. They define operations such as `create_agent`, `invoke_agent`, `invoke_workflow`, and `execute_tool`. | Every normalized record must store `semconv_version` and `stability_opt_in`. Missing those fields means the adapter result is useful but not proof of stable interoperability. |
| [OpenTelemetry GenAI client spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/) | Client spans include provider/model attributes, token usage, finish reasons, error type, and opt-in content fields such as input/output messages, system instructions, and tool definitions. | Token/cost/status fields are safe defaults. Prompts, outputs, system instructions, and tool definitions remain raw refs or disabled unless explicitly enabled and redacted. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP instrumentation defines client/server spans, method names, JSON-RPC request IDs, tool/prompt/resource attributes, session metrics, and transport values. MCP itself does not yet define a standard trace-context mechanism; OTel recommends `params._meta` while warning that this may change. | Parallax should propagate `traceparent` through MCP `_meta` when available, but must treat that path as provisional and test it per client/server pair. |
| [OpenTelemetry CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | CLI spans model short-lived command execution and require executable name, exit code, PID, and `error.type` on non-zero exits. Command args should not be collected by default without sanitization. | `cli_invocation` should always store structural command identity and exit/error status; raw argv is denied by default. |
| [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/) | Process args, command line, interactive flag, cgroup, parent PID, and working directory are opt-in. The spec says to prefer `process.command_args` and fall back to `process.command` plus `process.args_count` when args cannot be safely collected. | Parallax should store command family and arg count even when args are redacted, preserving audit value without secret exposure. |
| [OpenTelemetry CI/CD spans](https://opentelemetry.io/docs/specs/semconv/cicd/cicd-spans/) | CI/CD spans model pipeline runs and task runs with results such as success, failure, timeout, skipped, cancellation, and error. | Agent-run tests and validation commands should normalize to the same result vocabulary as CI jobs. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tool errors are split between JSON-RPC protocol errors and tool-execution errors; tool-execution errors should be actionable for model self-correction. Servers must validate inputs, enforce access control, rate-limit, sanitize outputs, and clients should log tool usage for audit. | Parallax MCP tool calls need both protocol status and tool-execution status, and denied/sensitive calls must be audit rows rather than only span errors. |

## Decision

Use OTel semantic conventions as **adapter input** and **wire observability**:

```text
vendor/agent/CLI/CI/MCP telemetry
  -> OTel semconv-aware adapter
  -> Parallax normalized action/session/invocation rows
  -> evidence graph nodes/edges
  -> CLI/API/MCP context bundle projections
```

Do not store only raw OTel spans for agent audit. Raw spans are too unstable and
too vendor-shaped. Store them as refs, then materialize stable Parallax rows.

## Mapping Table

| OTel / MCP source record | Parallax node or row | Required normalized fields | Lossiness flags |
| --- | --- | --- | --- |
| `gen_ai.operation.name=invoke_agent` | `agent_session` or `agent_turn` | provider, model, agent id/name/version when available, start/end, status, token/cost refs | `missing_agent_id`, `missing_model`, `missing_turn_id` |
| `gen_ai.operation.name=invoke_workflow` | `agent_action(kind=workflow)` | workflow name/id, parent session, status, duration, refs | `missing_workflow_id`, `workflow_shape_unknown` |
| `gen_ai.operation.name=chat/generate_content/text_completion` | `agent_action(kind=model_call)` | provider, requested model, response model, input/output token counts, finish reason, error type | `content_ref_only`, `missing_token_usage`, `provider_proxy_ambiguous` |
| `gen_ai.operation.name=execute_tool` | `agent_action(kind=tool_call)` | tool name, input hash/ref, output hash/ref, status, error type, parent model/turn | `missing_tool_result`, `raw_args_denied`, `double_count_candidate` |
| MCP client/server span | `agent_action(kind=mcp_tool_call)` and audit event | `mcp.method.name`, `gen_ai.tool.name?`, `jsonrpc.request.id?`, transport, protocol/tool status | `missing_jsonrpc_id`, `missing_traceparent`, `resource_uri_redacted` |
| MCP resource read | `agent_action(kind=context_load)` | resource URI/ref, scope, redaction policy, result hash/ref | `resource_uri_high_cardinality`, `raw_ref_denied` |
| CLI callee/caller span | `cli_invocation` or `agent_action(kind=shell_command)` | executable, command family, exit code, PID when available, duration, error type, args policy | `args_redacted`, `cwd_redacted`, `child_process_not_observed` |
| Process resource | `cli_invocation` metadata | executable name, command, arg count, working directory policy, parent PID when available | `command_line_denied`, `working_directory_denied` |
| CI/CD pipeline/task span | `ci_run`, `test_case`, or validation action | run/task id, task name, result, error type, URL/ref, commit/ref when available | `missing_commit`, `missing_job_url`, `local_ci_like_harness` |

## Stability Rules

Every semconv-derived record should carry:

```json
{
  "source_format": "otel",
  "semconv_version": "1.41.0",
  "semconv_area": "gen_ai|mcp|cli|process|cicd",
  "stability": "development|release_candidate|stable|mixed",
  "stability_opt_in": "gen_ai_latest_experimental|null",
  "adapter_name": "parallax-otel-agent-v0",
  "adapter_version": "0.1.0"
}
```

Rules:

- Development-stage conventions can create Parallax rows, but cannot be the only
  proof for a durable schema claim.
- If an instrumentation opts into latest experimental GenAI conventions, store
  that fact because field names and span shapes may change.
- Preserve raw OTel spans as refs for replay/backfill, but keep agent-visible
  bundles on normalized rows.
- If two OTel spans describe the same action, one row wins and the other becomes
  a supporting ref.

## Duplicate Suppression

Agent traces can double-count one action:

```text
model emits tool call
  -> GenAI execute_tool span
  -> MCP tools/call client span
  -> MCP tools/call server span
  -> CLI or HTTP request span inside the tool
```

Normalize that as one `agent_action(kind=tool_call)` with child refs unless the
inner span is a separate side effect worth exposing.

Deduplication key:

```text
session_id + turn_id? + tool_name + jsonrpc.request.id? + input_hash + start_time_bucket
```

When deduplication is uncertain, keep multiple rows but add
`possible_duplicate_of` so the audit graph does not overstate tool count.

## Content Capture Levels

OpenTelemetry marks important GenAI content fields as opt-in, including input
messages, output messages, system instructions, and tool definitions. Parallax
should mirror that caution.

| Level | Default | Agent-visible fields |
| --- | --- | --- |
| L0 structural | On | provider, model, operation, token counts, finish reason, tool name, status, hashes, refs, redaction report |
| L1 redacted excerpt | Project opt-in | bounded prompt/output/tool excerpts after redaction and canary checks |
| L2 raw ref | Sensitive opt-in | raw prompt/output/tool payload refs, never inline by default |

If L0 is not enough to answer an audit question, the bundle says which raw or
redacted ref is needed. It should not silently expand to full prompt/tool data.

## MCP Trace Propagation

Until MCP standardizes trace-context propagation:

- inject W3C `traceparent` and `tracestate` into MCP `params._meta` when the
  client/server supports it;
- extract `_meta.traceparent` as the remote parent for MCP server spans;
- link ambient context when the server already has one;
- never put secrets or authorization tokens into trace metadata;
- include `mcp_trace_context_provisional=true` in adapter metadata.

Gate:

| Check | Pass condition |
| --- | --- |
| Stdio tool call | A local stdio MCP tool call links client span, server span, and Parallax audit row by request id or fallback hash. |
| Streamable HTTP tool call | HTTP request span and MCP message span do not collapse into the same action incorrectly. |
| Missing `_meta` | Missing trace context produces `missing_traceparent`, not a fabricated parent. |
| Cross-client | At least Codex and Claude Code can call the Parallax MCP server and produce comparable audit rows. |

## Normalization Gates

Before Parallax claims OTel-native agent or CLI tracing:

| Gate | Target |
| --- | --- |
| `semconv_version_coverage` | 100 percent of semconv-derived rows include source version/area/stability. |
| `content_default_deny_rate` | 100 percent of opt-in GenAI content fields are absent from default agent-visible bundles. |
| `tool_call_mapping_rate` | >= 90 percent of surfaced tool calls map to `agent_action` rows with status and refs. |
| `mcp_trace_link_rate` | >= 80 percent of supported MCP tool calls link client/server/audit rows by request id or trace context. |
| `cli_exit_status_rate` | 100 percent of CLI invocations include exit code or signal/error class. |
| `args_policy_rate` | 100 percent of process/CLI rows state whether args are sanitized, denied, or raw-ref-only. |
| `duplicate_action_report_rate` | 100 percent of possible double-counted GenAI/MCP/tool spans are deduped or flagged. |
| `lossiness_report_rate` | 100 percent of adapters report unsupported/redacted/source-not-exposed fields. |

Failure wording:

- If GenAI spans are present but content is disabled, say "model call observed,
  content not captured."
- If MCP spans lack `_meta` trace context, say "MCP tool call observed, trace
  parent missing."
- If command args are denied, say "command identity and exit observed; raw args
  unavailable by policy."
- If vendor spans omit tool outputs, say "tool status observed; result only
  available as vendor raw ref or missing."

## Product Implication

Parallax should market this carefully:

- Good: "Parallax normalizes OTel GenAI/MCP/CLI traces into an execution audit
  graph."
- Good: "Parallax reports adapter lossiness and redaction before exposing agent
  evidence."
- Bad: "Parallax stores every prompt, tool argument, and output automatically."
- Bad: "OTel GenAI/MCP conventions are stable enough to be Parallax's schema."
- Bad: "MCP trace propagation is solved for every client."

The durable schema is the Parallax evidence graph. OTel makes ingestion and
interoperability possible; it does not replace the evidence contract.

## Relationship To Other Research

- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) defines
  why services, CI, CLI tools, and coding agents belong in one execution graph.
- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  defines the Codex, Claude Code, Amp, and OpenCode adapter gate.
- [Agent access surface: CLI, HTTP API, and MCP](agent-access-surface-cli-api-mcp.md)
  defines the read-only MCP context surface and projection-equivalence rule.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  normalized nodes and audit edges that semconv-derived rows feed.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
  owns the command args/env/stdout/stderr safety gate.
- [OpenTelemetry protocol and context layer](opentelemetry-protocol-and-context-layer.md)
  owns the general OTLP receiver and Collector/Rotel compatibility story.

## Bottom Line

OTel GenAI, MCP, CLI, process, and CI/CD conventions are a strong signal that
agent/CLI execution telemetry is becoming standardizable. They are not yet
stable enough to be Parallax's product schema. The safe path is to ingest them,
record their versions, normalize them into Parallax action/session/invocation
rows, report every loss, and keep raw prompts, args, tool outputs, and MCP
resources behind explicit redaction and raw-ref policy.
