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
> state-verification rows, redaction results, source-field policy status,
> projection raw-ref denial, canonical bundle hashes, projection manifests, MCP
> structured-output validation, overhead rows, and an audit-value comparison.

This ledger is separate from the
[agent access surface safety ledger](agent-access-surface-safety-ledger.md): that
ledger controls safe CLI/API/MCP context retrieval; this one controls ingestion
and normalization of agent execution traces.

## Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| Local tool version probe | `command -v` plus `--version` checks re-run in this workspace on 2026-05-25 found `/home/agent/.local/bin/codex` with Codex CLI `0.133.0`, `/home/agent/.local/bin/claude` with Claude Code `2.1.150`, `/home/agent/.local/bin/amp` with version `0.0.1779639467-g6d0650` released `2026-05-24T16:17:47.000Z`, and `/home/agent/.opencode/bin/opencode` with OpenCode `1.15.10`. Amp's raw output includes a relative age suffix that changed across probes (`20h ago` to `21h ago` to `1d ago`). | Real runs must store the exact tool binary path, raw version output, probe captured-at timestamp, normalized version/release fields, and docs snapshot date. Relative age strings are per-probe raw text, not durable freshness evidence. |
| [Codex hooks](https://developers.openai.com/codex/hooks) | Hooks expose structured JSON with `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`, `turn_id`, and `permission_mode` for session, tool, prompt, permission, subagent, compaction, and stop events. The docs warn that transcript format is not a stable hook interface and that tool interception is incomplete for some shell and non-shell paths. They also document managed hooks, plugin-bundled hooks, and command-only handler support. | Codex capture can be structured, but transcripts must stay raw refs; hook gaps must be measured against wrapper, repo diff/hash, or other independent evidence; and every hook claim must record hook source and trust mode. |
| [Codex CLI](https://developers.openai.com/codex/cli), [non-interactive mode](https://developers.openai.com/codex/noninteractive), and local `codex --help` / `codex exec --help` | Codex CLI is a local command-line agent surface and supports repo work, file edits, command execution, and automation workflows. Current official docs say `codex exec --json` emits JSONL events such as `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`; item types include agent messages, reasoning, command executions, file changes, MCP tool calls, web searches, and plan updates. Local `0.133.0` help shows `--ephemeral`, plugin management, `mcp-server`, and dangerous approval/sandbox and hook-trust bypass flags. | Codex needs separate claim rows for interactive hooks, non-interactive JSONL event/item taxonomy, plugin-provided surfaces, Codex-as-MCP-server, and policy-sensitive dangerous flags. |
| [Codex MCP](https://developers.openai.com/codex/mcp) | Codex supports local stdio MCP servers, Streamable HTTP servers, bearer-token env vars, OAuth login/logout for supported HTTP servers, static or environment HTTP headers, `enabled_tools`, `disabled_tools`, default and per-tool approval modes, OAuth callback overrides, plugin-provided MCP servers, and `codex mcp-server` as a stdio server for other clients. | Codex MCP config is both secret-bearing and policy-bearing. Agent-session fixture rows must record transport, token/header source, OAuth mode, enabled/disabled tool lists, approval mode, and plugin/server origin before MCP tool-call rows are treated as safe or comparable. |
| [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage) | Claude Code exports opt-in OpenTelemetry metrics, logs/events, and optional beta traces; prompt text, tool details, tool content, and raw API bodies are disabled by default and require explicit flags. It records session/tool/API/token/cost/MCP/plugin activity, does not pass generic `OTEL_*` exporter variables to Bash, hooks, MCP servers, or language servers, but active tracing injects `TRACEPARENT` into Bash/PowerShell. In `-p`/Agent SDK mode it can accept inbound `TRACEPARENT`/`TRACESTATE`; interactive sessions ignore inbound trace context. | Claude Code is the strongest native OTel target, but content capture must remain opt-in/redacted, plugin and MCP inventory must be recorded, and subprocess coverage must distinguish trace-context inheritance from full telemetry-exporter inheritance. |
| [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage), [programmatic usage](https://code.claude.com/docs/en/headless), and local `claude --help` | Current docs and local `2.1.150` help show `--output-format stream-json` in print mode, `--include-hook-events`, `--include-partial-messages`, stream JSON input, replayed user messages, session IDs, resume/fork/from-PR flags, `--no-session-persistence`, `--bare`, permission modes, tool allow/deny/restrict flags, plugin dirs/URLs, MCP config, strict MCP config, background-agent defaults, and `claude mcp serve`. Local help also shows remote-control naming, Chrome/IDE/Tmux/worktree context, startup file specs, setting-source restriction, explicit settings JSON/file input, dynamic system-prompt section exclusion, fallback model, budget cap, JSON-schema output, brief mode, slash-command disabling, debug-file output, and `ultrareview`. Local non-interactive help says workspace trust prompts are skipped and settings validation failures are silently ignored; `doctor` may spawn stdio MCP servers from `.mcp.json` for health checks. Bare mode skips auto-discovery of hooks, skills, plugins, MCP servers, auto memory, and `CLAUDE.md`; the docs say it is recommended for scripted calls and may become the default for `-p` later. | Claude stream JSON is a separate non-interactive structured adapter claim. It can support fixture automation and hook-event validation, but it must not be counted as interactive OTel coverage or as default-safe prompt/tool-content capture. Bare/no-persistence/scripted modes must be stored because they can suppress evidence and change replay/resume behavior. Remote control, context-injection flags, startup file downloads, trust skips, silent settings-validation behavior, and diagnostic MCP health checks must be recorded as source-policy state. |
| [Claude Code hooks](https://code.claude.com/docs/en/hooks) | Hooks cover session start/end, setup/instructions, user prompts, tool use, permission requests/denials, subagents/tasks, stop/failure, compaction, config/cwd/file changes, worktrees, notifications, and MCP elicitation. Handlers can be command, HTTP, MCP tool, prompt, or agent. `PreToolUse` can allow, deny, ask, defer, and mutate tool input; some hooks can persist environment through `CLAUDE_ENV_FILE`; `PostCompact` can expose a compaction summary. | Hook rows must prove event-class coverage and record handler type, source, decision, mutation, persisted environment, compaction-summary policy, and whether hooks were disabled by bare mode or settings. Hooks are a control-plane surface, not just observation. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) | Claude Code supports local, project, user, plugin, and claude.ai MCP sources with precedence rules; stdio, SSE, and HTTP servers; OAuth metadata/scopes; dynamic `headersHelper` commands; MCP prompts/resources; elicitation; output limits through `MAX_MCP_OUTPUT_TOKENS` and `_meta["anthropic/maxResultSizeChars"]`; `claude mcp serve`; and managed MCP allow/deny controls. Project/local `headersHelper` commands run only after workspace trust. | MCP tool-call rows need server scope, transport, auth/header source, OAuth scopes, output-limit behavior, elicitation hooks, duplicate/source precedence, workspace trust, and Claude-as-MCP-server state before cross-agent MCP comparisons are claimable. |
| [Claude Code settings](https://code.claude.com/docs/en/settings), [permission modes](https://code.claude.com/docs/en/permission-modes), [plugins](https://code.claude.com/docs/en/plugins-reference), and [agent view](https://code.claude.com/docs/en/agent-view) | Settings precedence is managed, command line, local, project, then user; managed settings cannot be overridden, while array settings such as permissions merge across scopes. Permission modes include `default`, `acceptEdits`, `plan`, `auto`, `dontAsk`, and `bypassPermissions`. Plugins can contribute skills, agents, hooks, MCP servers, LSP servers, monitors, Bash executables, and limited plugin settings. `claude agents --json` exposes live background sessions for scripting, and dispatched agents can inherit settings/plugins/MCP defaults. | Claude fixtures must snapshot effective settings sources, permission mode, plugin component inventory, background-agent mode, and plugin load errors, or the same "Claude Code" claim will hide materially different context and security behavior. |
| [Amp manual](https://ampcode.com/manual) and local `amp --help` | Amp supports streaming JSON output in `--execute` mode for programmatic integration and real-time conversation monitoring; optional thinking blocks extend the schema and are not Claude Code compatible. `--stream-json-input` supports multi-message stdin and a `steer` marker. The same manual documents TypeScript plugins, project/system/global plugin locations, lifecycle events such as `session.start`, `agent.start`, `tool.call`, `tool.result`, and `agent.end`, plugin activation for both interactive sessions and `amp --execute` runs, and explicitly notes there is no `session.end` event. Amp does not ask before running tools by default; `amp.permissions`, `amp.guardedFiles.allowlist`, or `amp.dangerouslyAllowAll=false` activate an internal permissions plugin. Local help adds thread export, tool list/show/make/use commands, permission list/test/edit/add commands, MCP add/list/remove/OAuth/doctor/approve commands, skill loading, per-pattern tool enable/disable settings, Claude Code skill import control, commit coauthor/thread-trailer settings, IDE/JetBrains flags, and `--mcp-config`. | Amp should be measured through both plugin-event fixtures and non-interactive stream fixtures. Thinking blocks, stdin messages, `steer` messages, and image/base64 payloads are sensitive input modes, not default-safe capture. Plugin events are a stronger interactive capture surface than the prior wrapper/thread-ref assumption, but still need payload, permissions, and version proof. Tool enable/disable patterns, skill import, thread export, MCP OAuth/workspace approval, and commit-trailer settings are policy or provenance fields, not incidental CLI options. |
| [OpenCode CLI](https://opencode.ai/docs/cli/) and local `opencode --help` | OpenCode supports `run --format json` raw JSON events, `run --attach` against a `serve` backend, session continuation/forking, `session list --format json`, export JSON with `--sanitize`, headless `serve` API with optional basic auth, ACP over nd-JSON, stats, and permission/thinking flags. Local help also exposes `--pure` to disable external plugins plus `web`, `github`, `pr`, `plugin`, `db`, `models`, `upgrade`, and `uninstall` commands. | OpenCode is a strong JSON/export/plugin/API/protocol adapter target; `--sanitize` is helpful but does not replace Parallax redaction, and `--thinking`, `--dangerously-skip-permissions`, server attach URL, basic-auth credential source, CORS, host/port, mDNS settings, plugin suppression, GitHub/PR commands, and session-database commands must be recorded as sensitive run configuration. |
| [OpenCode plugins](https://opencode.ai/docs/plugins/) | Plugins expose event families for command, file, installation, LSP, message, permission, server, session, todo, shell, tool, and TUI behavior. `tool.execute.before` can mutate tool arguments; `shell.env` can inject environment variables. | OpenCode can provide deep structured events without terminal parsing, but support must be proven per enabled event class, and fixtures must separate observer-only plugins from plugins that mutate tool calls, environment, or TUI behavior. |
| [OpenCode MCP servers](https://opencode.ai/docs/mcp-servers/) | OpenCode supports local MCP servers with command/environment/timeout/enabled fields and remote MCP servers with URL, enabled, headers, OAuth config or OAuth disabled, timeout, remote defaults, global tool toggles, glob disables, and per-agent MCP enablement. | MCP settings are secret-bearing and policy-bearing. OpenCode tool-call rows need transport, header/env/OAuth source, enabled/global/per-agent tool state, and timeout provenance before cross-agent MCP comparisons are claimable. |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | Current semconv catalog includes GenAI, MCP, CLI, process, CI/CD, VCS, exception, and test areas. | Adapters should record source semantic-convention versions instead of hard-coding unstable span shapes into Parallax storage. |
| [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/), [MCP](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/), and [CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | GenAI and MCP conventions are useful ingestion vocabulary; CLI conventions define short-lived command spans and exit/error semantics. | OTel-native sources feed normalized Parallax rows, but raw spans are not the durable product schema. |
| [OpenTelemetry CICD spans](https://opentelemetry.io/docs/specs/semconv/cicd/cicd-spans/), [test attributes](https://opentelemetry.io/docs/specs/semconv/registry/attributes/test/), and [VCS attributes](https://opentelemetry.io/docs/specs/semconv/registry/attributes/vcs/) | CICD task runs expose result states, test attributes expose case/suite statuses, and VCS attributes distinguish head/base refs and revisions. These are development-stage/registry vocabularies, not proof that an agent command changed external state. | Agent-session rows need a separate state-verification artifact before wording can move from "command reported success" to "patch validated," "file changed," "deploy succeeded," or "issue fixed." |
| [MCP tools specification `2025-11-25`](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) and [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) | MCP tool results can include JSON `structuredContent` validated by `outputSchema`; JCS defines deterministic JSON for repeatable hashes. | Agent-session projection rows must bind redaction/source-field checks to canonical JSON and prove CLI/HTTP/MCP projections preserve the same safety fields. |

## Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current run exists. | "Agent-session tracing design exists; results pending." |
| `fixture_harness_ready` | Repeatable tasks and fixture repos exist for the target agents. | "Agent-session tracing fixture harness prepared." |
| `claude_otel_ingest_supported` | Claude Code OTel metrics/logs/events/traces ingest and normalize for a dated configuration. | "Claude Code OTel session events normalize for the tested version/config." |
| `claude_hooks_supported` | Claude Code hook lifecycle/control events normalize for a dated configuration without hiding handler source, decisions, mutation, or environment persistence. | "Claude Code hook events normalize for the tested version/config." |
| `claude_stream_json_supported` | Claude Code print-mode `stream-json` events, with hook lifecycle events when enabled, normalize for a dated version/config. | "Claude Code stream JSON sessions normalize for the tested version/config." |
| `codex_hooks_supported` | Codex hook events normalize for a dated CLI/config without relying on transcript parsing as the source of truth. | "Codex hook events normalize for the tested version/config." |
| `codex_exec_json_supported` | Codex non-interactive `exec --json` JSONL events normalize for a dated CLI/config. | "Codex exec JSONL sessions normalize for the tested version/config." |
| `opencode_run_json_supported` | OpenCode `run --format json` events normalize for a dated version/config. | "OpenCode run JSON events normalize for the tested version/config." |
| `opencode_export_supported` | OpenCode session export/list JSON normalizes for a dated version/config. | "OpenCode session export normalizes for the tested version/config." |
| `opencode_plugin_supported` | OpenCode plugin events normalize for a dated version/config. | "OpenCode plugin events normalize for the tested version/config." |
| `opencode_acp_supported` | OpenCode ACP nd-JSON session protocol normalizes for a dated version/config. | "OpenCode ACP events normalize for the tested version/config." |
| `amp_stream_json_supported` | Amp `--execute --stream-json` events normalize for a dated version/config. | "Amp streaming JSON sessions normalize for the tested version/config." |
| `amp_plugin_supported` | Amp plugin lifecycle/tool events normalize for a dated version/config. | "Amp plugin session events normalize for the tested version/config." |
| `normalized_session_schema_pass` | A tested adapter set maps sessions, turns, actions, commands, edits, permissions, and outcomes into stable Parallax rows. | "Normalized agent-session schema passes for the tested adapter set." |
| `lossiness_reported` | Every unmapped, redacted, source-not-exposed, raw-ref-only, or parse-failed event class is counted. | "Adapter lossiness is reported for the tested agents." |
| `state_verification_reported` | Every state-change, validation, deploy, database, file, or outcome claim either has a verifier row or is explicitly marked unverified. | "State verification is reported for the tested agent-session claims." |
| `redaction_safe` | Agent-visible canonical JSON, Markdown, CLI/HTTP output, and MCP `structuredContent` leak zero seeded canaries. | "Agent-session projections pass seeded redaction tests." |
| `projection_safe` | Redaction report, source-field policy status, missing-evidence flags, raw-ref dereference denial, schema/hash metadata, projection manifest, and MCP structured-output validation pass for the tested projection set. | "Agent-session projections are safe for the tested adapter set." |
| `audit_value_positive` | Normalized Parallax sessions answer audit questions better than final-output-only and at least as usefully as raw transcript/export arms. | "Normalized sessions improve audit reconstruction in the tested task set." |
| `multi_agent_trace_supported` | At least two agents, including one native OTel path and one non-OTel structured path, pass schema, lossiness, state-verification, redaction, source-field/projection, and audit-value gates. | "Parallax normalizes coding-agent session traces for the tested agent matrix." |
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
docs/research/agent-session-tracing-runs/<run_id>/state-verification-results.jsonl
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
  "bundle_schema_ref": {
    "uri": "https://parallax.dev/schemas/evidence-bundle/v0.json",
    "hash": "sha256:...",
    "canonicalization": "jcs-rfc8785"
  },
  "projection_surfaces_required": ["bundle_json", "bundle_markdown", "cli_output", "http_api", "mcp_structuredContent"],
  "mcp_output_schema_required": true,
  "semconv_version": "1.41.0",
  "raw_ref_policy": "transcripts_exports_prompts_tool_payloads_not_agent_visible_by_default",
  "tool_version_probe_captured_at": "YYYY-MM-DDTHH:MM:SSZ",
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
  "tool_version_probe_captured_at": "YYYY-MM-DDTHH:MM:SSZ",
  "tool_release_timestamp": "YYYY-MM-DDTHH:MM:SSZ|null",
  "version_relative_age_present": false,
  "version_normalization_notes": [],
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
    "dangerous_flags_enabled": [],
    "hook_source": "none|user|project|managed|plugin|mixed",
    "hook_trust_mode": "persisted|bypassed|not_applicable|unknown",
    "plugin_hooks_enabled": false,
    "context_injection_surfaces": [],
    "settings_validation_mode": "strict|silent_ignore|unknown",
    "workspace_trust_dialog_skipped": false,
    "cloud_or_remote_control_mode": "none|remote_control|cloud_review|unknown",
    "amp_config": {
      "mode": "smart|deep|rush|large|unknown",
      "stream_json": false,
      "stream_json_input": false,
      "stream_json_thinking": false,
      "steer_messages_present": false,
      "permission_settings_present": false,
      "guarded_files_allowlist_present": false,
      "dangerously_allow_all": false,
      "thread_export_used": false,
      "tool_enable_patterns": [],
      "tool_disable_patterns": [],
      "mcp_oauth_or_workspace_approval": false,
      "claude_code_skill_import_enabled": false,
      "git_trailer_settings": [],
      "plugin_decision_actions_observed": []
    },
    "mcp_config": {
      "servers_configured": [],
      "transport_modes": [],
      "token_or_header_sources": [],
      "enabled_tools": [],
      "disabled_tools": [],
      "approval_modes": [],
      "plugin_server_origins": []
    },
    "claude_config": {
      "mode": "interactive|print|background_agent|agent_view|mcp_server|unknown",
      "output_format": "text|json|stream-json|null",
      "input_format": "text|stream-json|null",
      "bare_mode": false,
      "no_session_persistence": false,
      "session_resume_mode": "new|continue|resume|fork|from_pr|unknown",
      "settings_sources": [],
      "setting_sources_restricted": [],
      "explicit_settings_input": false,
      "managed_settings_present": false,
      "settings_array_merge_observed": false,
      "permission_mode": "default|acceptEdits|plan|auto|dontAsk|bypassPermissions|unknown",
      "allow_dangerously_skip_permissions": false,
      "dangerously_skip_permissions": false,
      "allowed_tools": [],
      "disallowed_tools": [],
      "tools_restricted": false,
      "plugin_sources": [],
      "plugin_components_enabled": [],
      "plugin_load_errors": [],
      "hook_handler_types": [],
      "hook_mutates_tool_input": false,
      "hook_persists_environment": false,
      "mcp_scope_sources": [],
      "mcp_strict_config": false,
      "mcp_headers_helper_present": false,
      "mcp_oauth_scopes_pinned": false,
      "mcp_output_limit_tokens": null,
      "claude_mcp_serve": false,
      "traceparent_inbound": false,
      "traceparent_outbound": false,
      "remote_control": false,
      "chrome_enabled": false,
      "ide_enabled": false,
      "tmux_enabled": false,
      "worktree_enabled": false,
      "file_startup_refs": [],
      "dynamic_system_prompt_sections_excluded": [],
      "fallback_model": null,
      "max_budget_usd": null,
      "json_schema_supplied": false,
      "brief_mode": false,
      "slash_commands_disabled": false,
      "debug_file": null,
      "workspace_trust_dialog_skipped": false,
      "settings_validation_mode": "strict|silent_ignore|unknown",
      "doctor_spawned_workspace_mcp": false,
      "ultrareview_used": false
    },
    "opencode_config": {
      "run_format": "default|json",
      "attach_url": null,
      "serve_enabled": false,
      "web_mode": false,
      "server_auth_mode": "none|basic",
      "server_credential_sources": [],
      "cors_origins": [],
      "mdns_enabled": false,
      "mcp_tool_enablement_scope": "global|per_agent|mixed|unknown",
      "plugin_event_classes_enabled": [],
      "pure_mode": false,
      "plugin_installed_for_run": false,
      "github_or_pr_command_used": false,
      "db_command_used": false,
      "models_command_used": false,
      "plugin_mutates_tool_args": false,
      "plugin_injects_shell_env": false,
      "export_sanitize": false,
      "thinking_visible": false,
      "dangerously_skip_permissions": false
    },
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
  "source_event_type": "SessionStart|SessionEnd|PreToolUse|PostToolUse|PermissionRequest|PermissionDenied|Notification|UserPromptSubmit|Stop|SubagentStop|PreCompact|PostCompact|Elicitation|ElicitationResult|CwdChanged|FileChanged|WorktreeCreate|WorktreeRemove|system_init|system_plugin_install|system_api_retry|tool.execution|message.updated|session.start|agent.start|tool.call|tool.result|agent.end|command.executed|file.edited|permission.asked|permission.replied|session.created|session.idle|shell.env|tool.execute.before|tool.execute.after|thread.started|turn.started|turn.completed|turn.failed|item.started|item.updated|item.completed|error|stream_json_object|ndjson_object|unknown",
  "source_item_type": "agent_message|reasoning|command_execution|file_change|mcp_tool_call|web_search|plan_update|null",
  "source_event_class": "hook|plugin|otel|run_json|json_export|stream_json|jsonl|server_api|acp|wrapper",
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
  "state_verification_count": 3,
  "unverified_state_claim_count": 0,
  "outcome_linked": true,
  "content_capture_level": "structural",
  "raw_ref_count": 0,
  "redaction_report_ref": "redaction-results.jsonl#task_bugfix_001",
  "source_field_policy_ref": "source-field-policy-results.jsonl#task_bugfix_001",
  "schema_ref_hash": "sha256:...",
  "canonical_session_bundle_hash": "sha256:...",
  "projection_manifest_hash": "sha256:...",
  "projection_equivalence_pass": true
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

### State Verification Result Row

```json
{
  "fixture_task_id": "task_bugfix_001",
  "tool": "codex",
  "action_id": "agent_action:shell_003",
  "observed_event_ref": "adapter-event-results.jsonl#agt_evt_003",
  "reported_result": {
    "exit_code": 0,
    "status": "success",
    "stdout_ref": "raw-ref-manifest.jsonl#stdout_003",
    "stderr_ref": null
  },
  "state_claim": "patch_validation_passed|file_changed|database_changed|deploy_succeeded|issue_fixed|none",
  "verifier_kind": "repo_diff|file_hash|test_report|provider_api_readback|database_readback|deployment_status|runtime_recurrence_check|manual_fixture_expectation|none",
  "verifier_ref": "coverage-results.jsonl#repo_diff_003",
  "verification_status": "supported|contradicted|unverified|not_applicable",
  "agent_visible_wording": "command_reported_success|state_verified|state_unverified",
  "missing_evidence": []
}
```

### Lossiness Result Row

```json
{
  "tool": "amp",
  "fixture_task_id": "task_research_001",
  "event_class": "thinking_block|tool_output|permission_decision|subagent|raw_transcript|uncovered_hook_tool_path|bare_mode_suppressed_surface|no_session_persistence|plugin_load_error|hook_mutated_tool_input|hook_persisted_environment|mcp_output_truncated|settings_source_unknown|plugin_event_disabled|source_schema_changed|state_verifier_missing|unverified_state_claim",
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
  "canonical_json_leaks": 0,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "cli_output_leaks": 0,
  "http_api_leaks": 0,
  "mcp_structured_content_leaks": 0,
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
  "projection": "bundle_json|bundle_markdown|cli_output|http_api|mcp_structuredContent",
  "schema_ref_hash": "sha256:...",
  "canonical_bundle_hash": "sha256:...",
  "projection_hash": "sha256:...",
  "projection_manifest_hash": "sha256:...",
  "projection_derives_from_canonical": true,
  "redaction_report_present": true,
  "source_field_policy_status": "pass|fail|not_applicable",
  "source_field_policy_hash": "sha256:<hex>|null",
  "source_field_policy_violations": 0,
  "mcp_output_schema_valid": null,
  "mcp_structured_content_hash": null,
  "safety_fields_only_in_meta": false,
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
- Version probes must preserve raw output, captured-at timestamp, normalized
  version, release timestamp when present, and normalization notes separately.
  Relative strings emitted by CLIs, such as "20h ago", cannot be used as
  durable freshness evidence after the run date.
- A transcript/export can support audit, but cannot be the only source for a
  structured tracing claim if the tool documents it as unstable.
- Claude Code content-bearing telemetry gates must remain disabled for default
  runs unless the redaction suite explicitly tests them.
- Claude Code subprocess propagation must be counted precisely: active tracing
  can inject `TRACEPARENT` into Bash/PowerShell, but generic `OTEL_*` exporter
  variables are not inherited by Bash, hooks, MCP servers, or language servers
  by default. Inbound `TRACEPARENT`/`TRACESTATE` applies to `-p`/Agent SDK
  mode only, not ordinary interactive sessions.
- Claude Code stream JSON claims apply to `claude -p --output-format
  stream-json` and must record whether `--include-hook-events`,
  `--include-partial-messages`, stream input, replayed user messages, or
  content-bearing flags were enabled. This claim does not prove interactive OTel
  coverage and cannot imply prompt/tool-content safety without redaction rows.
- Claude Code `--bare` claims must record which normally discovered surfaces
  were suppressed: hooks, skills, plugins, MCP servers, auto memory, and
  `CLAUDE.md`. A bare scripted run cannot prove ordinary interactive context or
  hook/MCP coverage unless those surfaces are explicitly re-added by flags.
- Claude Code `--no-session-persistence` claims must record that sessions cannot
  be resumed from disk and that transcript/resume-based evidence is unavailable
  or raw-ref-only for that run.
- Claude Code context/control claims must record remote-control mode,
  Chrome/IDE/Tmux/worktree context, startup file references, setting-source
  restrictions, explicit settings JSON or file input, excluded dynamic
  system-prompt sections, fallback model, budget cap, JSON schema, brief mode,
  disabled slash commands, debug-file output, and cloud review modes such as
  `ultrareview`.
- Claude Code non-interactive and diagnostic claims must record whether
  workspace trust prompts were skipped, invalid settings were silently ignored,
  startup file references were downloaded, or `doctor` spawned stdio MCP servers
  from workspace `.mcp.json`. These side effects can change source trust and
  must not be hidden under a generic "print mode" row.
- Claude Code hook claims apply per event class and handler type. Rows must
  record whether the handler was command, HTTP, MCP tool, prompt, or agent;
  whether it allowed, denied, asked, deferred, or mutated a tool call; whether
  it persisted environment through `CLAUDE_ENV_FILE`; and whether compaction
  summaries or elicitation payloads were raw-ref-only.
- Claude Code settings/plugin claims must record effective setting sources and
  precedence: managed, command line, local, project, user, plus array-merge
  behavior for permissions. Plugin rows must list enabled component classes
  such as skills, agents, hooks, MCP, LSP, monitors, Bash `PATH` executables,
  and supported plugin settings, plus plugin load errors.
- Claude Code MCP session claims must record server scope and precedence, local
  versus project versus user versus plugin versus claude.ai source, transport,
  static headers or `headersHelper` source, OAuth metadata/scopes, output-limit
  settings, workspace-trust requirement, managed MCP allow/deny controls, and
  whether the run exposed `claude mcp serve`.
- Claude Code background-agent claims must record whether the session came from
  `claude agents --json`, `--bg`, agent view, or a dispatched agent, and which
  model, effort, permission, settings, plugin, and MCP defaults were applied.
- Codex `transcript_path` is a raw ref. Hook events are the claimable structured
  source, but hook support proves structured hook normalization rather than
  complete shell/file side-effect coverage unless wrapper, repo-diff, or
  equivalent independent evidence rows also pass.
- Codex hook claims must record whether hooks came from user, project, managed,
  plugin-bundled, or mixed sources, and whether persisted hook trust was
  required or bypassed. `--dangerously-bypass-hook-trust` and approval/sandbox
  bypass flags are policy-sensitive run configuration, not normal defaults.
- Codex `exec --json` claims are separate from interactive hook claims. JSONL
  fixture support must preserve event names and item types, and cannot prove
  interactive coverage unless a separate coverage row links equivalent side
  effects.
- Codex MCP-related session claims must record configured servers, transport
  modes, token/header sources, OAuth mode, enabled/disabled tool lists,
  default/per-tool approval modes, plugin-provided server origins, and whether
  the run used `codex mcp-server` as a stdio server.
- Amp streaming JSON claims apply to `--execute --stream-json`; they must
  record whether `--stream-json-input`, `--stream-json-thinking`, queued
  `steer` messages, image/base64 payloads, or stdin closure behavior were part
  of the fixture.
- Amp plugin claims apply per plugin location, activation mode, event class, and
  run mode. A plugin fixture must cover interactive and/or `--execute` separately
  and must not infer `session.end` support from the documented lifecycle, because
  the manual explicitly says there is no `session.end` event.
- Amp permission claims must record whether default no-approval behavior,
  `amp.permissions`, `amp.guardedFiles.allowlist`,
  `amp.dangerouslyAllowAll=false`, or custom plugin decisions produced
  `allow`, `reject-and-continue`, `modify`, or `synthesize` outcomes.
- Amp CLI-policy claims must record thread export use, tool enable/disable
  patterns, permission command state, MCP config/OAuth/workspace approval,
  Claude Code skill import setting, and git commit coauthor/thread-trailer
  settings. These fields affect provenance and agent-visible context even when
  the streaming or plugin event schema itself is unchanged.
- OpenCode `--sanitize` is a source feature, not Parallax redaction proof.
  Parallax redaction must still pass on normalized projections.
- OpenCode plugin support requires coverage rows for enabled event classes.
  JSON/export rows alone do not prove live side-effect coverage. Plugin rows
  must record whether the plugin only observed events or mutated tool
  arguments, injected shell environment, or affected TUI/server behavior.
- OpenCode run JSON, export JSON, plugin hooks, server/API, and ACP are separate
  claim surfaces. Do not collapse them into one support claim.
- OpenCode `--pure` suppresses external plugins. Pure-mode fixture rows cannot
  prove plugin-event coverage; plugin-enabled and plugin-suppressed runs are
  separate capture configurations.
- OpenCode `web`, `github`, `pr`, `plugin`, `db`, and `models` commands must be
  recorded when used because they can change server/API exposure, repository
  side effects, plugin state, session database state, or model selection.
- OpenCode server/API and `run --attach` claims must record attach URL,
  host/port, CORS, mDNS, basic-auth mode, and credential source. Local run JSON
  does not prove attached-server behavior.
- OpenCode MCP claims must record local/remote transport, command/env/header
  sources, OAuth enabled/disabled state, timeout, remote defaults, global tool
  toggles, glob disables, and per-agent enablement.
- OpenCode `--thinking` and `--dangerously-skip-permissions` must be recorded as
  run configuration and excluded from default-safe product claims unless the
  redaction and policy rows explicitly cover them.
- Coverage denominators must come from native events, wrapper observation, repo
  diff/hash evidence, or manual fixture expectations. Do not calculate coverage
  only from events the adapter happened to see.
- Command/tool observation, exit code, stdout/stderr, and tool result objects
  prove reported execution only. They do not prove durable file, repo, database,
  deploy, runtime, or issue state.
- A `state_verification` row is required before an agent-visible projection may
  say a patch was validated, a file changed as intended, a migration succeeded,
  a deploy completed, or an issue was fixed. Without a verifier row, use
  "command reported success" or mark the state claim unverified.
- Test/build/lint command rows may support "the test runner reported success"
  from exit code and test output, but "the fix works" still requires linked
  issue/runtime/recurrence or fixture expectation evidence.
- State verifiers must identify their method: repo diff/hash, file stat/hash,
  test report, provider API readback, database readback, deployment status,
  runtime recurrence check, or manual fixture expectation. Verifier refs are
  subject to the same redaction, source-field, raw-ref, and projection gates as
  other agent-session evidence.
- `multi_agent_trace_supported` requires at least one native OTel path, at
  least one non-OTel structured path, and passing state-verification rows for
  projected state claims.
- No claim may depend on hidden chain-of-thought or private model reasoning.
- Agent-visible canonical JSON, Markdown, CLI, HTTP, and MCP outputs must leak
  zero seeded canaries and must not dereference raw transcript/export/tool
  payload refs by default.
- Projection-safe agent-session rows require `schema_ref`, post-redaction
  `canonical_hash`, `projection_manifest`, `access`, `redaction_report`,
  `source_field_policy`, lossiness, and missing-evidence fields in canonical
  JSON. A text-only transcript/export summary is not enough.
- CLI, HTTP, and MCP projections must carry the same canonical bundle hash for
  the same session/anchor. Projection hash mismatch, missing projection rows, or
  unscanned output paths fail `projection_safe`.
- MCP-delivered agent-session evidence counts only when `structuredContent`
  validates against the evidence-bundle `outputSchema`; JSON pasted into a text
  block is a projection, not schema-safe structured evidence.
- Safety fields for agent-session evidence cannot live only in MCP `_meta`, tool
  descriptions, annotations, or model-prompt wrapper metadata.
- Agent-session rows that fail `projection_safe` can debug adapters, but cannot
  feed A1 bundle-value, A4 reconstruction, or fixer-outcome claims.
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
| State verification coverage | 0% | 100% for projected state claims | Pending |
| Unverified state claims in product wording | 0 | 0 | Pending |
| Lossiness report coverage | 0% | 100% | Pending |
| Agent-visible canary leaks | 0 | 0 | Pending |
| Source-field policy violations | 0 | 0 | Pending |
| Raw refs dereferenced by projections | 0 | 0 | Pending |
| Projection hash mismatches | 0 | 0 | Pending |
| MCP structured output validation failures | 0 | 0 | Pending |
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
- "validated the patch" from exit code/stdout alone;
- "changed production state" without readback evidence;
- "safe transcript ingestion" before redaction, source-field, and projection
  rows pass.

## Refresh Triggers

Mark affected claims `claim_expired` when:

- Codex, Claude Code, Amp, or OpenCode docs/version/config surfaces change;
- OpenTelemetry GenAI/MCP/CLI semantic conventions change materially;
- Parallax normalized session schema changes;
- adapter parser logic changes;
- redaction policy, source-field policy, bundle schema, canonicalization method,
  projection renderer, MCP output schema, or access-surface projection behavior
  changes;
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
normalize sessions, report lossiness, verify or mark state claims, preserve
redaction/source-field/projection safety across canonical CLI/API/MCP
projections, and improve audit
reconstruction over final outputs alone.
