# Agent Session Tracing Ledger

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

[Agent session tracing across real tools](agent-session-tracing-real-tools.md)
defines the adapter strategy for Codex, Claude Code, Amp, and OpenCode. This
ledger defines the result artifacts and claim levels required before Parallax
can say it supports agent-session tracing across real coding agents.

Current status: **not measured**. The repository has an adapter design and value
gate, but no run artifacts. Until those results exist, Parallax should describe
agent-session tracing as a planned proof gate, not a proven capability.

The central rule:

> No "Parallax traces coding-agent sessions" claim without a dated tool/version
> matrix, adapter coverage rows, normalized session snapshots, lossiness reports,
> redaction results, source-field policy status, projection raw-ref denial,
> overhead rows, and an audit-value comparison.

This ledger is separate from the
[agent access surface safety ledger](agent-access-surface-safety-ledger.md): that
ledger controls safe CLI/API/MCP context retrieval; this one controls ingestion
and normalization of agent execution traces.

## Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| Local tool version probe | `--version` checks in this workspace on 2026-05-25 returned Codex CLI `0.133.0`, Claude Code `2.1.150`, Amp `0.0.1779639467-g6d0650` released 2026-05-24, and OpenCode `1.15.10`. | Real runs must store the exact tool binary path, version output, and docs snapshot date; agent surfaces change too quickly for product-level claims without version evidence. |
| [Codex hooks](https://developers.openai.com/codex/hooks) | Hooks expose structured JSON with `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`, `turn_id`, and `permission_mode` for session, tool, prompt, permission, subagent, compaction, and stop events. The docs warn that transcript format is not a stable hook interface and that tool interception is incomplete for some shell and non-shell paths. | Codex capture can be structured, but transcripts must stay raw refs and hook gaps must be measured against wrapper, repo diff/hash, or other independent evidence. |
| [Codex CLI](https://developers.openai.com/codex/cli) | Codex CLI is a local command-line agent surface and supports repo work, file edits, command execution, and automation workflows. | Codex is a first adapter target because it runs where Parallax can observe local repo, shell, and file evidence. |
| [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage) | Claude Code exports opt-in OpenTelemetry metrics, logs/events, and optional beta traces; prompt text, tool details, tool content, and raw API bodies are disabled by default and require explicit flags. It does not pass generic `OTEL_*` exporter variables to subprocesses, but when tracing is active Bash/PowerShell inherit `TRACEPARENT`. | Claude Code is the strongest native OTel target, but content capture must remain opt-in/redacted and subprocess coverage must distinguish trace-context inheritance from full telemetry-exporter inheritance. |
| [Amp manual](https://ampcode.com/manual) | Amp supports streaming JSON output in `--execute` mode for programmatic integration and real-time conversation monitoring; optional thinking blocks extend the schema. The same manual documents TypeScript plugins, project/system/global plugin locations, lifecycle events such as `session.start`, `agent.start`, `tool.call`, `tool.result`, and `agent.end`, and plugin activation for both interactive sessions and `amp --execute` runs. | Amp should be measured through both plugin-event fixtures and non-interactive stream fixtures. Thinking blocks are sensitive opt-in, not default capture. Plugin events are a stronger interactive capture surface than the prior wrapper/thread-ref assumption, but still need payload and version proof. |
| [OpenCode CLI](https://opencode.ai/docs/cli/) | OpenCode supports `run --format json` raw JSON events, session continuation/forking, `session list --format json`, export JSON with `--sanitize`, headless `serve`, ACP over nd-JSON, stats, and permission/thinking flags. | OpenCode is a strong JSON/export/plugin/API/protocol adapter target; `--sanitize` is helpful but does not replace Parallax redaction, and `--thinking` / `--dangerously-skip-permissions` must be recorded as sensitive run configuration. |
| [OpenCode plugins](https://opencode.ai/docs/plugins/) | Plugins expose documented event names including `command.executed`, `file.edited`, `permission.asked`, `permission.replied`, `session.*`, `shell.env`, `tool.execute.before`, and `tool.execute.after`. | OpenCode can provide deep structured events without terminal parsing, but support must be proven per enabled event class and must not be inferred from run JSON or export JSON alone. |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | Current semconv catalog includes GenAI, MCP, CLI, process, CI/CD, VCS, exception, and test areas. | Adapters should record source semantic-convention versions instead of hard-coding unstable span shapes into Parallax storage. |
| [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/), [MCP](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/), and [CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | GenAI and MCP conventions are useful ingestion vocabulary; CLI conventions define short-lived command spans and exit/error semantics. | OTel-native sources feed normalized Parallax rows, but raw spans are not the durable product schema. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current run exists. | "Agent-session tracing design exists; results pending." |
| `fixture_harness_ready` | Repeatable tasks and fixture repos exist for the target agents. | "Agent-session tracing fixture harness prepared." |
| `claude_otel_ingest_supported` | Claude Code OTel metrics/logs/events/traces ingest and normalize for a dated configuration. | "Claude Code OTel session events normalize for the tested version/config." |
| `codex_hooks_supported` | Codex hook events normalize for a dated CLI/config without relying on transcript parsing as the source of truth. | "Codex hook events normalize for the tested version/config." |
| `opencode_run_json_supported` | OpenCode `run --format json` events normalize for a dated version/config. | "OpenCode run JSON events normalize for the tested version/config." |
| `opencode_export_supported` | OpenCode session export/list JSON normalizes for a dated version/config. | "OpenCode session export normalizes for the tested version/config." |
| `opencode_plugin_supported` | OpenCode plugin events normalize for a dated version/config. | "OpenCode plugin events normalize for the tested version/config." |
| `opencode_acp_supported` | OpenCode ACP nd-JSON session protocol normalizes for a dated version/config. | "OpenCode ACP events normalize for the tested version/config." |
| `amp_stream_json_supported` | Amp `--execute --stream-json` events normalize for a dated version/config. | "Amp streaming JSON sessions normalize for the tested version/config." |
| `amp_plugin_supported` | Amp plugin lifecycle/tool events normalize for a dated version/config. | "Amp plugin session events normalize for the tested version/config." |
| `normalized_session_schema_pass` | A tested adapter set maps sessions, turns, actions, commands, edits, permissions, and outcomes into stable Parallax rows. | "Normalized agent-session schema passes for the tested adapter set." |
| `lossiness_reported` | Every unmapped, redacted, source-not-exposed, raw-ref-only, or parse-failed event class is counted. | "Adapter lossiness is reported for the tested agents." |
| `redaction_safe` | Agent-visible JSON and Markdown projections leak zero seeded canaries. | "Agent-session projections pass seeded redaction tests." |
| `projection_safe` | Redaction report, source-field policy status, missing-evidence flags, and raw-ref dereference denial pass for JSON and Markdown. | "Agent-session projections are safe for the tested adapter set." |
| `audit_value_positive` | Normalized Parallax sessions answer audit questions better than final-output-only and at least as usefully as raw transcript/export arms. | "Normalized sessions improve audit reconstruction in the tested task set." |
| `multi_agent_trace_supported` | At least two agents, including one native OTel path and one non-OTel structured path, pass schema, lossiness, redaction, source-field/projection, and audit-value gates. | "Parallax normalizes coding-agent session traces for the tested agent matrix." |
| `claim_expired` | Agent docs/version/config, OTel semconv, adapter, schema, redaction/source-field/projection policy, or 90-day timer changed. | "Agent-session tracing result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the affected tool/version/path. |

Initial Parallax level: `not_measured`.

## Result Artifacts

Create these only for real adapter runs:

```text
docs/research/agent-session-tracing-results.md
docs/research/agent-session-tracing-runs/<run_id>/manifest.json
docs/research/agent-session-tracing-runs/<run_id>/tool-matrix.jsonl
docs/research/agent-session-tracing-runs/<run_id>/source-snapshot.jsonl
docs/research/agent-session-tracing-runs/<run_id>/raw-ref-manifest.jsonl
docs/research/agent-session-tracing-runs/<run_id>/adapter-event-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/normalized-session-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/coverage-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/lossiness-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/redaction-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/source-field-policy-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/projection-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/overhead-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/audit-value-results.jsonl
docs/research/agent-session-tracing-runs/<run_id>/claim-ledger.jsonl
docs/research/agent-session-tracing-runs/<run_id>/hashes.sha256
```

Raw transcripts, prompts, tool payloads, shell output, file contents, and model
outputs are raw refs only. Do not commit them unless the operator explicitly
approves a redacted synthetic fixture.

## Run Manifest

```json
{
  "run_id": "agent-session-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_adapter_commit": "<git-sha>",
  "normalized_schema_version": "agent-session-v0",
  "redaction_policy_version": "a6-default-deny-vN",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "semconv_version": "1.41.0",
  "raw_ref_policy": "transcripts_exports_prompts_tool_payloads_not_agent_visible_by_default",
  "tool_version_probe": {
    "codex": "codex --version",
    "claude_code": "claude --version",
    "amp": "amp --version",
    "opencode": "opencode --version"
  },
  "fixture_repo_commit": "<git-sha>",
  "task_count": 0,
  "agents": ["codex", "claude_code", "amp", "opencode"],
  "comparison_arms": ["final_output_only", "native_transcript_or_export", "parallax_normalized", "parallax_linked_evidence"],
  "notes": []
}
```

## Row Schemas

### Tool Matrix Row

```json
{
  "tool": "codex|claude_code|amp|opencode",
  "tool_version": "unknown",
  "tool_binary_path": "/path/to/tool",
  "tool_version_probe_output": "raw version output",
  "docs_checked_at": "YYYY-MM-DD",
  "adapter_name": "parallax-codex-hooks",
  "adapter_version": "0.1.0",
  "capture_surface": "hooks|otel|run_json|stream_json|json_export|plugin|server_api|acp|wrapper|raw_ref",
  "config": {
    "content_capture": "structural|redacted_excerpt|raw_ref",
    "thinking_capture": "disabled|raw_ref|redacted_excerpt",
    "subprocess_trace_propagation": "none|traceparent_env|otel_env|wrapper",
    "source_field_policy_required": true,
    "content_bearing_flags_enabled": [],
    "secret_bearing_config_refs": [],
    "expected_event_classes": ["SessionStart", "PreToolUse", "PostToolUse"],
    "coverage_denominator_source": "native_events|wrapper_observation|repo_diff|manual_fixture"
  },
  "claim_target": "codex_hooks_supported"
}
```

### Adapter Event Result Row

```json
{
  "event_id": "agt_evt_001",
  "tool": "codex",
  "fixture_task_id": "task_bugfix_001",
  "source_event_type": "SessionStart|PreToolUse|tool.execution|message.updated|session.start|agent.start|tool.call|tool.result|agent.end|command.executed|file.edited|permission.asked|permission.replied|session.created|session.idle|shell.env|tool.execute.before|tool.execute.after|stream_json_object|ndjson_object|unknown",
  "source_event_class": "hook|plugin|otel|run_json|json_export|stream_json|server_api|acp|wrapper",
  "source_event_schema": "docs-checked-YYYY-MM-DD",
  "source_event_hash": "sha256:<hex>",
  "accepted": true,
  "maps_to_action_kind": "tool_call|shell_command|file_edit|permission_decision|null",
  "coverage_gap": false,
  "normalized_row_refs": ["agent_session:sess_001"],
  "raw_ref_only": false,
  "parse_error": null,
  "notes": []
}
```

### Normalized Session Result Row

```json
{
  "session_id": "sess_001",
  "tool": "claude_code",
  "fixture_task_id": "task_bugfix_001",
  "session_start_captured": true,
  "session_end_captured": true,
  "turn_count": 4,
  "action_count": 18,
  "tool_call_count": 9,
  "shell_command_count": 3,
  "file_edit_count": 2,
  "permission_decision_count": 1,
  "outcome_linked": true,
  "content_capture_level": "structural",
  "raw_ref_count": 0,
  "redaction_report_ref": "redaction-results.jsonl#task_bugfix_001",
  "source_field_policy_ref": "source-field-policy-results.jsonl#task_bugfix_001"
}
```

### Coverage Result Row

```json
{
  "fixture_task_id": "task_bugfix_001",
  "tool": "opencode",
  "expected_event_classes": ["command", "file", "permission", "session", "shell", "tool"],
  "observed_event_classes": ["session", "tool", "shell"],
  "uncovered_side_effects": [],
  "coverage_denominator_source": "plugin_events+wrapper_observation",
  "surface_tool_calls": 10,
  "mapped_tool_calls": 9,
  "surface_shell_commands": 3,
  "mapped_shell_commands": 3,
  "surface_file_edits": 2,
  "mapped_file_edits": 2,
  "tool_call_mapping_rate": 0.9,
  "command_edit_coverage_pass": true,
  "outcome_linked": true
}
```

### Lossiness Result Row

```json
{
  "tool": "amp",
  "fixture_task_id": "task_research_001",
  "event_class": "thinking_block|tool_output|permission_decision|subagent|raw_transcript|uncovered_hook_tool_path|plugin_event_disabled|source_schema_changed",
  "lossiness_reason": "source_not_exposed|redacted|raw_ref_only|unsupported|parse_failed|unstable_format",
  "count": 0,
  "user_visible_warning": "Thinking blocks disabled by policy.",
  "claim_impact": "none|narrows_claim|fails_claim"
}
```

### Redaction Result Row

```json
{
  "fixture_task_id": "task_canary_001",
  "tool": "codex",
  "seeded_canaries": 20,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "raw_ref_leaks": 0,
  "redaction_policy_version": "a6-default-deny-vN",
  "redaction_report_hash": "sha256:<hex>",
  "agent_visible_pass": true
}
```

### Source Field Policy Result Row

```json
{
  "fixture_task_id": "task_canary_001",
  "tool": "codex",
  "source_kind": "synthetic_fixture|evaluation_fixture|direct_local_session",
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_version": "phase0-source-field-policy-vN",
  "source_field_policy_hash": "sha256:<hex>",
  "denied_zone_count": 0,
  "violation_count": 0,
  "not_applicable_reason": "direct local session without mixed eval/corpus source rows"
}
```

### Projection Result Row

```json
{
  "fixture_task_id": "task_canary_001",
  "tool": "codex",
  "projection_format": "json|markdown",
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "missing_evidence_present": true,
  "raw_transcript_ref_count": 1,
  "raw_export_ref_count": 0,
  "raw_tool_payload_ref_count": 0,
  "raw_ref_dereferenced": false,
  "thinking_content_visible": false,
  "prompt_body_visible": false,
  "tool_payload_visible": false,
  "seeded_canary_leaks": 0,
  "agent_visible_pass": true
}
```

### Audit Value Result Row

```json
{
  "fixture_task_id": "task_bugfix_001",
  "tool": "claude_code",
  "arm": "final_output_only|native_transcript_or_export|parallax_normalized|parallax_linked_evidence",
  "questions_answered": 0,
  "questions_total": 0,
  "accuracy": 0.0,
  "time_to_answer_seconds": 0,
  "evidence_refs_required": 0,
  "material_errors": 0,
  "notes": []
}
```

### Claim Ledger Row

```json
{
  "run_id": "agent-session-YYYYMMDD-N",
  "claim_level": "not_measured",
  "claim_status": "pass|fail|expired",
  "tool_matrix": ["codex@unknown", "claude_code@unknown"],
  "product_wording": "Agent-session tracing design exists; results pending.",
  "required_caveats": ["not complete transcript replay", "no hidden reasoning capture"],
  "expires_at": "YYYY-MM-DD"
}
```

## Counting Rules

- Count claims per agent product, version, adapter, capture surface, and config.
  Do not generalize from one tool to all agents.
- Claims above `fixture_harness_ready` require a resolved tool version and
  binary/package source. `tool_version: unknown` can only support exploratory
  design notes.
- A transcript/export can support audit, but cannot be the only source for a
  structured tracing claim if the tool documents it as unstable.
- Claude Code content-bearing telemetry gates must remain disabled for default
  runs unless the redaction suite explicitly tests them.
- Claude Code subprocess propagation must be counted precisely: active tracing
  can inject `TRACEPARENT` into Bash/PowerShell, but generic `OTEL_*` exporter
  variables are not inherited by spawned subprocesses by default.
- Codex `transcript_path` is a raw ref. Hook events are the claimable structured
  source, but hook support proves structured hook normalization rather than
  complete shell/file side-effect coverage unless wrapper, repo-diff, or
  equivalent independent evidence rows also pass.
- Amp streaming JSON claims apply to `--execute --stream-json`.
- Amp plugin claims apply per plugin location, activation mode, event class, and
  run mode. A plugin fixture must cover interactive and/or `--execute` separately
  and must not infer `session.end` support from the documented lifecycle, because
  the manual calls out `session.start` but does not document a `session.end`
  event.
- OpenCode `--sanitize` is a source feature, not Parallax redaction proof.
  Parallax redaction must still pass on normalized projections.
- OpenCode plugin support requires coverage rows for enabled event classes.
  JSON/export rows alone do not prove live side-effect coverage.
- OpenCode run JSON, export JSON, plugin hooks, server/API, and ACP are separate
  claim surfaces. Do not collapse them into one support claim.
- OpenCode `--thinking` and `--dangerously-skip-permissions` must be recorded as
  run configuration and excluded from default-safe product claims unless the
  redaction and policy rows explicitly cover them.
- Coverage denominators must come from native events, wrapper observation, repo
  diff/hash evidence, or manual fixture expectations. Do not calculate coverage
  only from events the adapter happened to see.
- `multi_agent_trace_supported` requires at least one native OTel path and at
  least one non-OTel structured path.
- No claim may depend on hidden chain-of-thought or private model reasoning.
- Agent-visible JSON and Markdown must leak zero seeded canaries and must not
  dereference raw transcript/export/tool payload refs by default.
- Synthetic or evaluation fixture runs require `source_field_policy_status:
  pass` before redaction or projection claims can pass. Direct local sessions may
  use `not_applicable` only when no mixed eval/corpus source rows are present.
- If a tool changes event schema or docs materially, mark only the affected
  adapter claim expired.

## Initial Results Template

When measurement begins, create `docs/research/agent-session-tracing-results.md`:

```markdown
# Agent Session Tracing Results

Research window:
Last updated:
Current claim level: not_measured

## Gate Snapshot

| Metric | Current | Threshold for multi_agent_trace_supported | Status |
| --- | ---: | ---: | --- |
| Agents with passing structured adapters | 0 | >=2 | Pending |
| Native OTel adapter pass | 0 | >=1 | Pending |
| Non-OTel structured adapter pass | 0 | >=1 | Pending |
| Tool-call mapping rate | 0% | >=90% | Pending |
| Command/edit coverage | 0% | 100% where surfaced | Pending |
| Lossiness report coverage | 0% | 100% | Pending |
| Agent-visible canary leaks | 0 | 0 | Pending |
| Source-field policy violations | 0 | 0 | Pending |
| Raw refs dereferenced by projections | 0 | 0 | Pending |
| Audit-value lift over final output only | 0 | Positive | Pending |

## Tool Matrix

## Coverage And Lossiness

## Redaction

## Audit Value

## Current Allowed Wording

## Decision
```

## Product Wording

Allowed after `not_measured`:

> Agent-session tracing is designed but not yet run-proven.

Allowed after a single adapter level:

> Parallax normalizes [tool] session events for the tested version/configuration.

Allowed after `multi_agent_trace_supported`:

> Parallax normalizes coding-agent session traces for the tested agent matrix.

Avoid:

- "universal agent tracing";
- "complete replay";
- "records every prompt/tool output by default";
- "captures model reasoning";
- "supports Codex/Claude/Amp/OpenCode" without a version/config matrix;
- "safe transcript ingestion" before redaction, source-field, and projection
  rows pass.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- Codex, Claude Code, Amp, or OpenCode docs/version/config surfaces change;
- OpenTelemetry GenAI/MCP/CLI semantic conventions change materially;
- Parallax normalized session schema changes;
- adapter parser logic changes;
- redaction policy, source-field policy, or projection schema changes;
- a source tool adds or removes hooks, OTel, streaming JSON, export, plugin, or
  permission events;
- 90 days pass since the last run during active development.

## Relationship To Other Research

- [Agent session tracing across real tools](agent-session-tracing-real-tools.md)
  defines the adapter strategy this ledger measures.
- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) explains
  why agent sessions belong in the execution graph.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
  defines how native OTel GenAI/MCP/CLI spans map into stable Parallax rows.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
  supplies the shell-command policy used inside agent sessions.
- [CLI trace safety ledger](cli-trace-safety-ledger.md) supplies the
  shell-command result rows, claim levels, and expiry rules consumed by
  agent-session runs.
- [A6 redaction red-team ledger](a6-redaction-red-team-ledger.md) controls
  whether agent-session evidence can become agent-visible.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  defines the target `agent_session`, `agent_action`, source-field policy
  status, redaction report, and audit edges.
- [Fixer outcome ledger](fixer-outcome-ledger.md) consumes linked agent-session
  rows when measuring fixer runs, PRs, checks, review, and recurrence outcomes.

## Bottom Line

Agent-session tracing should be measured like an adapter compatibility contract.
The first credible claim is not "trace every agent." It is a dated matrix showing
that at least two real tools emit enough structured events for Parallax to
normalize sessions, report lossiness, preserve redaction/source-field/projection
safety, and improve audit reconstruction over final outputs alone.
