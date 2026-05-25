# Agent Session Tracing Across Real Tools

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note closes proof gate 10 from
[Strategic verdict and research coverage](strategic-verdict-and-research-coverage.md):

> Agent-session tracing value across real Codex, Claude Code, Amp, and OpenCode
> runs.

The answer is not one universal transcript parser. Current coding agents expose
different observability surfaces. Parallax should use a tool-adapter strategy:
native OpenTelemetry where the tool provides it, lifecycle hooks where the tool
provides hooks, streaming JSON where the tool exposes a machine stream, and
import/export adapters where session data is already available.

Decision: **agent-session tracing is viable, but only as a lossy normalized
execution audit, not as complete access to hidden reasoning or every raw token.**
The useful product is "what context, tools, files, commands, permissions,
patches, tests, and outcomes were visible", not "what the model secretly
thought." The companion
[agent session tracing ledger](agent-session-tracing-ledger.md) defines the
result rows and claim levels required before this becomes product wording.

## Current Primary-Source Checks

| Tool/source | What matters for Parallax |
| --- | --- |
| Local tool version probe | Re-probe in this workspace on 2026-05-25 found `/home/agent/.local/bin/codex` with `codex-cli 0.133.0`, `/home/agent/.local/bin/claude` with `2.1.150 (Claude Code)`, `/home/agent/.local/bin/amp` with `0.0.1779639467-g6d0650` released `2026-05-24T16:17:47.000Z`, and `/home/agent/.opencode/bin/opencode` with `1.15.10`. Amp's raw `--version` output includes a relative age suffix that changed across probes (`20h ago` to `21h ago` to `1d ago`), so the durable fields are the captured-at timestamp, raw output, normalized version, and release timestamp, not the relative age string. These are environment observations, not universal current versions. |
| [Codex CLI](https://developers.openai.com/codex/cli), [non-interactive mode](https://developers.openai.com/codex/noninteractive), and local `codex --help` / `codex exec --help` | Codex CLI is local, open source, Rust-built, and can inspect repos, edit files, and run commands. Current official docs say `codex exec --json` emits JSONL events such as `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`; item types include agent messages, reasoning, command executions, file changes, MCP tool calls, web searches, and plan updates. Local `0.133.0` help also shows `--ephemeral`, plugin management, `mcp-server`, and dangerous bypass flags for approvals/sandbox and hook trust. | Codex is a direct adapter target, but Parallax must separate interactive hooks, non-interactive JSONL event/item taxonomy, plugin-provided surfaces, Codex-as-MCP-server, and dangerous policy flags instead of treating "Codex support" as one claim. |
| [Codex hooks](https://developers.openai.com/codex/hooks) | Codex hooks expose `session_id`, `cwd`, `hook_event_name`, `model`, `permission_mode`, `tool_name`, `tool_use_id`, and `tool_input` for events such as session start, tool use, permission requests, subagents, and stop. The docs warn that `transcript_path` is not a stable hook interface and that `PreToolUse`/`PostToolUse` interception is incomplete for some shell and non-shell tool paths. They also document managed hooks, plugin-bundled hooks, and that only command handlers run today. | Parallax should use hooks for structured events, treat transcripts as raw refs only, measure hook coverage against wrapper/repo-diff evidence, and record hook source/trust mode because plugin or managed hooks change the trust boundary. |
| [Codex MCP](https://developers.openai.com/codex/mcp) | Codex supports MCP servers in CLI and IDE clients, including local stdio with environment variables and Streamable HTTP with bearer-token or OAuth auth. Current docs expose `enabled_tools`, `disabled_tools`, default/per-tool approval modes, OAuth callback overrides, static/env HTTP headers, and plugin-provided MCP servers. Parallax can provide a read-only MCP context surface, but Codex MCP configuration is also a secret-bearing and policy-bearing source. |
| [OpenAI agent-improvement cookbook](https://developers.openai.com/cookbook/examples/agents_sdk/agent_improvement_loop) | OpenAI's own agent-improvement loop starts from traces, adds feedback, converts expectations into evals, and produces a Codex-ready handoff. That validates Parallax's "trace -> feedback -> eval -> better agent work" loop. |
| [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage) | Claude Code has the strongest first-party telemetry posture: opt-in OTel metrics/logs/events and beta traces. It records sessions, tool activity, API calls, costs, tokens, commits, PRs, active time, plugin inventory, and MCP activity. Prompt text, tool details, tool content, and raw API bodies are disabled by default and require explicit flags. Generic `OTEL_*` exporter variables are not passed to Bash, hooks, MCP servers, or language servers, but active tracing injects `TRACEPARENT` into Bash/PowerShell. In `-p`/Agent SDK mode, Claude Code can also read inbound `TRACEPARENT`/`TRACESTATE`; interactive sessions ignore inbound trace context. |
| [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage), [programmatic usage](https://code.claude.com/docs/en/headless), and local `claude --help` | Current docs and local `2.1.150` help show `--output-format stream-json` for print mode, `--include-hook-events`, `--include-partial-messages`, stream JSON input, replayed user messages, session IDs, resume/fork/from-PR flags, `--no-session-persistence`, `--bare`, permission modes, `--allowedTools`/`--disallowedTools`/`--tools`, `--plugin-dir`/`--plugin-url`, `--mcp-config`, `--strict-mcp-config`, background agent defaults, and `claude mcp serve`. Local help also exposes remote-control naming, Chrome/IDE/Tmux/worktree context, startup file download specs, setting-source restriction, explicit settings JSON/file input, dynamic system-prompt section exclusion, fallback model, budget cap, JSON-schema output, brief mode, slash-command disabling, debug-file output, and `ultrareview`. The local `-p` help says workspace trust prompts are skipped in non-interactive mode and settings validation failures are silently ignored; `doctor` may spawn stdio MCP servers from `.mcp.json` for health checks. Bare mode skips auto-discovery of hooks, skills, plugins, MCP servers, auto memory, and `CLAUDE.md`; the docs say it is recommended for scripted calls and may become the default for `-p` later. | Claude Code has a second structured adapter surface besides OTel, but it is a print-mode stream claim. The run configuration can suppress, inject, or remotely control major context/control surfaces, so fixtures must store the effective flags and cannot infer interactive coverage or default-safe content capture from stream support alone. Non-interactive trust skips, silent settings-validation behavior, startup file downloads, and diagnostic MCP health checks are source-policy rows, not incidental CLI trivia. |
| [Claude Code hooks](https://code.claude.com/docs/en/hooks) | Hooks cover session start/end, setup/instructions, user prompts, tool use, permission requests/denials, subagents/tasks, stop/failure, compaction, config/cwd/file changes, worktrees, notifications, and MCP elicitation. Handlers can be command, HTTP, MCP tool, prompt, or agent. `PreToolUse` can allow, deny, ask, defer, and mutate tool input; `SessionStart`, `Setup`, `CwdChanged`, and `FileChanged` can persist environment variables through `CLAUDE_ENV_FILE`; `PostCompact` can expose a compaction summary. | Claude hooks are not only passive observation. Parallax must record handler type, source, decision, mutation, persisted environment, compaction-summary policy, and whether hooks were disabled by bare mode or settings before claiming hook coverage. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) | Claude Code supports local, project, user, plugin, and claude.ai MCP sources with precedence rules; stdio, SSE, and HTTP servers; OAuth metadata/scopes; dynamic `headersHelper` commands; MCP prompts/resources; elicitation; output limits through `MAX_MCP_OUTPUT_TOKENS` and `_meta["anthropic/maxResultSizeChars"]`; `claude mcp serve`; and managed MCP allow/deny controls. Project/local `headersHelper` commands run only after workspace trust. | MCP configuration is both context source and policy source. Agent-session rows need server scope, transport, auth/header source, OAuth scopes, output-limit behavior, elicitation hooks, duplicate/source precedence, and `claude mcp serve` state before MCP tool rows are comparable. |
| [Claude Code settings](https://code.claude.com/docs/en/settings), [permission modes](https://code.claude.com/docs/en/permission-modes), [plugins](https://code.claude.com/docs/en/plugins-reference), and [agent view](https://code.claude.com/docs/en/agent-view) | Settings precedence is managed, command line, local, project, then user; managed settings cannot be overridden, while array settings such as permissions merge across scopes. Permission modes range from read-only-ish `default`/`plan` through `acceptEdits`, `auto`, `dontAsk`, and `bypassPermissions`. Plugins can contribute skills, agents, hooks, MCP servers, LSP servers, monitors, executables added to Bash `PATH`, and limited plugin settings. `claude agents --json` exposes live background sessions for scripting, and dispatched agents can inherit settings/plugins/MCP defaults. | Claude capture must snapshot effective settings sources, permission mode, plugin component inventory, background-agent mode, and plugin load errors. Otherwise a "Claude Code session" claim hides materially different security and context behavior across machines. |
| [Amp manual](https://ampcode.com/manual) and local `amp --help` | Amp exposes threads, subagents, AGENTS.md loading, MCP, execute mode, non-interactive use, streaming JSON for programmatic integration, and TypeScript plugins. The manual documents plugin events for the thread/session lifecycle and tool calls/results, says plugins apply to both interactive `amp` sessions and `amp --execute` runs, and explicitly says there is no `session.end` event. Streaming JSON requires `--execute`; `--stream-json-thinking` extends the schema and is not Claude Code compatible; `--stream-json-input` supports multi-message stdin and a `steer` marker. Amp does not ask for tool approval by default; settings such as `amp.permissions`, `amp.guardedFiles.allowlist`, or `amp.dangerouslyAllowAll=false` activate an internal permissions plugin. Local help adds thread export, tool list/show/make/use commands, permission list/test/edit/add commands, MCP add/list/remove/OAuth/doctor/approve commands, skill loading, per-pattern tool enable/disable settings, Claude Code skill import control, commit coauthor/thread-trailer settings, IDE/JetBrains flags, and `--mcp-config`. It does not show a first-party OTel surface, so Parallax should treat Amp as a plugin-plus-streaming-JSON adapter target, not only a wrapper/import target. |
| [OpenCode CLI](https://opencode.ai/docs/cli/) and local `opencode --help` | OpenCode exposes session IDs, continuation/forking, `run --format json` raw JSON events, `run --attach` against a `serve` backend, `session list --format json`, `export` with a `--sanitize` flag, token/cost stats, a headless `serve` HTTP API with optional basic auth, and `acp` over nd-JSON. It also has `--thinking` and `--dangerously-skip-permissions` flags that must be recorded as sensitive capture/policy state. Local help adds `--pure` to disable external plugins plus `web`, `github`, `pr`, `plugin`, `db`, `models`, `upgrade`, and `uninstall` commands. This is a strong import, live-adapter, API, and protocol target, but fixture rows must distinguish plugin-enabled runs from `--pure` runs and treat GitHub/PR/session-database commands as side-effect surfaces. |
| [OpenCode plugins](https://opencode.ai/docs/plugins/) | OpenCode plugins can hook command, file, installation, LSP, message, permission, server, session, todo, shell, tool, and TUI events. `tool.execute.before` can mutate tool arguments, and `shell.env` can inject environment variables. That makes OpenCode a strong open tool to instrument deeply without parsing terminal output, but fixture runs must distinguish observational plugin events from mutating/control-plane plugin behavior and prove enabled event classes one by one. |
| [OpenCode MCP servers](https://opencode.ai/docs/mcp-servers/) | OpenCode supports local MCP servers with command/environment/timeout/enabled fields and remote MCP servers with URL, enabled, headers, OAuth config or OAuth disabled, timeout, remote defaults, global tool toggles, glob disables, and per-agent MCP enablement. Parallax should treat MCP config as both context source and secret-bearing/policy-bearing audit surface. |
| [OpenTelemetry CLI](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/), [CICD](https://opentelemetry.io/docs/specs/semconv/cicd/cicd-spans/), [test](https://opentelemetry.io/docs/specs/semconv/registry/attributes/test/), and [VCS](https://opentelemetry.io/docs/specs/semconv/registry/attributes/vcs/) semantic conventions `1.41.0` | CLI spans require process exit code and treat non-zero as an error; command args are explicitly not default-safe without sanitization. CICD task rows carry task-run result, test rows carry case/suite status, and VCS rows distinguish head/base refs and revisions. All checked pages are development-stage or registry vocabulary, not Parallax's durable schema. |

## Adapter Strategy

Parallax should not wait for every agent to export the same trace format.
Instead, build adapters into one normalized session schema.

| Agent | Best initial adapter | Capture strength | Main gaps |
| --- | --- | --- | --- |
| Claude Code | Native OTel logs/events/traces into Parallax ingest, hook-event fixtures, and `-p --output-format stream-json --include-hook-events` as a separate non-interactive adapter. | Strongest first-party signal: sessions, tools, API requests, costs, tokens, commits, PRs, MCP, identity, optional traces, plugin inventory, hook lifecycle/control events, and a structured print-mode stream for scripted fixture runs. | Traces are beta; raw prompt/tool content is intentionally off by default; stream JSON is print-mode/non-interactive coverage; hooks, MCP, plugins, settings, background agents, and bare/no-persistence modes materially change coverage and must be versioned/configured; subprocess telemetry needs precise handling because `TRACEPARENT` can propagate while generic OTEL exporter variables do not. |
| Codex | Hook adapter, `codex exec --json` non-interactive JSONL adapter, Parallax CLI wrapper, repo diff/hash observation, and raw transcript refs. | Strong lifecycle/tool/permission signals, session IDs, model, cwd, subagents, MCP tool inputs, and a scripted JSONL stream for fixture runs. | Transcript format is not stable; hook interception is incomplete for some tool paths; exec JSONL is non-interactive coverage; plugin/managed hooks and hook-trust bypass flags must be measured separately; no first-party OTel export in the checked docs. |
| Amp | Plugin-event adapter plus streaming JSON adapter for execute mode, thread refs, and CLI wrapper. | Stronger than previously assumed: plugin events cover session/agent lifecycle plus tool calls/results, while streaming JSON gives a programmatic non-interactive stream. | Manual does not show native OTel; manual explicitly says there is no `session.end`; plugin safety/version drift and event payload coverage need fixture proof; permissions are broad by default unless configured or enforced by plugins. |
| OpenCode | `run --format json`, `export --sanitize`, plugin hooks, `serve` HTTP API, and ACP adapter. | Strong open adapter path: raw JSON events, session export/list, plugins for session/tool/file/permission events, and nd-JSON protocol mode. | Need fixture tests to prove run JSON, export JSON, plugin hooks, ACP, permission flags, thinking capture, and sanitation quality separately across versions. |

The common product rule: **native surfaces are preferred, but Parallax never
depends on hidden model reasoning or unstable transcript formats for its core
audit claim.**
For tools that emit OpenTelemetry-shaped agent, MCP, or CLI spans, adapters must
follow the
[Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
so development-stage semantic conventions feed stable Parallax rows with
explicit lossiness reports.

## Adapter Coverage Clarifications

### Codex Hooks Are Guardrail Events

Codex hooks are a structured source, but they are not complete command, edit,
or side-effect coverage by themselves. `PreToolUse` and `PostToolUse` should be
counted as hook-event normalization unless the same fixture also records a
coverage denominator from the Parallax CLI wrapper, repo diff/hash observation,
or another independent command/file evidence source.

The Codex adapter must therefore report:

- expected hook classes for the fixture, including session, tool, permission,
  subagent, compaction, and stop events when those paths are exercised;
- observed hook classes and normalized rows;
- side effects seen by wrapper or repo observation but not by hooks;
- whether events came from user, project, managed, or plugin-bundled hooks;
- whether persisted hook trust was required, bypassed, or unavailable;
- whether dangerous approval/sandbox bypass flags were enabled for the run;
- `codex exec --json` event classes separately from interactive hook classes;
- `codex exec --json` event names and item types separately, including
  command-execution, file-change, MCP-tool-call, web-search, reasoning, plan,
  and error categories;
- Codex MCP client/server configuration: transport, token/header source,
  OAuth callback policy, enabled/disabled tool lists, default/per-tool approval
  modes, and plugin-provided server origin;
- transcript use as `raw_ref_only`, never as the stable structured source.

### Claude Code Surfaces Are Split Claims

Claude Code's OTel path remains the primary interactive capture surface, but
current CLI docs and local help expose several independent claim surfaces:
native OTel, hook events, print-mode stream JSON, MCP configuration/tool calls,
plugins, settings precedence, background agent sessions, and Claude-as-MCP-server
mode. Parallax should treat the print stream like Amp's streaming JSON: useful
for fixture tasks, scripted runs, and adapter validation, but not a substitute
for the OTel claim.

The Claude adapter must report:

- exact `claude --version`, CLI reference snapshot date, and flags used;
- whether the run used `--print`, `--output-format stream-json`,
  `--include-hook-events`, `--include-partial-messages`, `--input-format
  stream-json`, or `--replay-user-messages`;
- whether the run used `--bare`, `--no-session-persistence`, `--continue`,
  `--resume`, `--fork-session`, `--from-pr`, `--session-id`, `--agent`,
  `--agents`, `claude agents --json`, or `claude mcp serve`;
- effective setting sources and precedence: managed, command-line, local,
  project, user, plus merged permission arrays;
- permission/tool policy: `--permission-mode`,
  `--allow-dangerously-skip-permissions`, `--dangerously-skip-permissions`,
  `--allowedTools`, `--disallowedTools`, and `--tools`;
- context/control switches: remote-control name, Chrome/IDE context, Tmux,
  worktree, startup file specs, setting-source restrictions, explicit settings
  files or JSON, dynamic system-prompt section exclusion, fallback model, budget
  cap, JSON schema, brief mode, slash-command disabling, and debug-file output;
- plugin inputs and results: `--plugin-dir`, `--plugin-url`, loaded plugin
  inventory, plugin load errors, and plugin components such as hooks, MCP
  servers, LSP servers, monitors, agents, skills, executables, and supported
  plugin settings;
- MCP configuration: source scope, transport, duplicate/source precedence,
  `--mcp-config`, `--strict-mcp-config`, OAuth metadata/scopes, static headers,
  `headersHelper`, output-limit settings, elicitation events, and whether
  workspace trust was required;
- hook source, handler type, event class, decision, mutation, environment
  persistence, compaction-summary handling, and whether bare mode or settings
  disabled hook discovery;
- which hook lifecycle events appeared in the stream and which OTel events/spans
  were also present when telemetry was enabled;
- whether prompt bodies, partial message chunks, hook payloads, and tool
  details stayed structural, redacted, or raw-ref-only;
- whether non-interactive mode skipped workspace trust prompts, ignored invalid
  settings, downloaded startup file references, or ran diagnostic health checks
  that spawned workspace `.mcp.json` stdio servers;
- that stream support is a non-interactive adapter claim unless a separate
  interactive capture surface proves equivalent coverage.

### Amp Plugins Are A Primary Adapter Surface

Amp should no longer be treated as only a streaming-JSON/non-interactive target.
The current manual documents TypeScript plugins, project/system/global plugin
locations, and events that follow a thread session's lifecycle:
`session.start`, `agent.start`, `tool.call`, `tool.result`, and `agent.end`.
It also says plugin activation applies to interactive `amp` sessions and
`amp --execute` runs. That makes an Amp plugin adapter a plausible first-class
capture surface for interactive work.

The catch is that this is still not a measured Parallax claim. A fixture must
record:

- plugin location and activation mode: project, system, or global;
- whether the run is interactive, `--execute`, or `--execute --stream-json`;
- observed lifecycle events, especially the absence of a documented
  `session.end`;
- observed `tool.call` and `tool.result` payload fields for shell, file, MCP,
  and custom-tool cases;
- permission behavior, because the manual says Amp does not ask for approval
  before running tools unless `amp.permissions`,
  `amp.guardedFiles.allowlist`, `amp.dangerouslyAllowAll=false`, or a custom
  plugin changes that behavior;
- tool/skill/MCP policy inputs from local CLI state: thread export source,
  per-pattern `amp.tools.enable` and `amp.tools.disable`, Claude Code skill
  import setting, MCP OAuth or workspace approval state, `--mcp-config`, and
  commit coauthor/thread-trailer settings;
- plugin decisions for `tool.call`: `allow`, `reject-and-continue`, `modify`,
  or `synthesize`, and whether modified/synthesized tool results become
  raw-ref-only or agent-visible;
- streaming JSON rows when used, including whether `--stream-json-thinking` was
  disabled by policy, whether `--stream-json-input` was enabled, whether
  `steer` messages were present, and whether image/base64 payloads were
  captured only as raw refs.

Until those rows exist, the updated claim is: Amp has a plausible structured
plugin adapter path plus a non-interactive stream, not proven full session
tracing.

### Observed Output Is Not State Verification

Agent traces should distinguish three evidence classes:

1. **Execution observed.** A hook/plugin/stream/wrapper saw a tool call,
   command, file operation, or API request.
2. **Result reported.** The source reported exit code, status, stdout/stderr,
   tool result, test status, or error text.
3. **State verified.** An independent readback, repo diff/hash, test report,
   provider API read, deployment status, database query, or runtime signal
   confirms the state claim the agent wants to make.

`exit 0`, a green-looking stdout line, or a tool result object can support
"the command reported success." It does not by itself support "the file changed
as intended," "the migration was safe," "the deploy succeeded," or "production
is fixed." Those stronger claims need verification rows with deterministic
evidence refs.

The adapter must therefore report:

- the observed command/tool event and its source surface;
- the reported result fields and their redaction policy;
- the state claim, if any, that the agent or evaluator wants to infer;
- the verifier type: repo diff/hash, file stat/hash, test report, provider API
  readback, database readback, deployment status, runtime recurrence check, or
  `none`;
- whether the verifier supports, contradicts, or leaves the state claim
  unverified.

Unverified state claims should appear as missing evidence, not as adapter
success.

### OpenCode Plugins Are Class-By-Class

OpenCode support should be split into separate claim surfaces: run JSON, export
JSON, plugin hooks, HTTP server/API, and ACP. `run --format json` can prove a
non-interactive raw-event stream; `export --sanitize` can prove import/export
coverage for stored sessions; plugin hooks can prove live event interception;
ACP can prove protocol integration. None of those alone proves the others.

Plugin fixtures must list the event classes they expect to exercise, using the
documented names where possible: `command.executed`, `file.edited`,
`permission.asked`, `permission.replied`, `session.*`, `shell.env`,
`tool.execute.before`, and `tool.execute.after`. The fixture must then record
which classes were observed and mapped. It must also record whether plugin code
mutated tool arguments, injected shell environment, showed TUI prompts, or acted
only as an observer. `export --sanitize` remains a useful source feature, but
it is not Parallax redaction proof and it does not prove live plugin coverage.
`run --attach`, `serve` basic-auth configuration, MCP headers/OAuth/env vars,
global/per-agent tool toggles, `--thinking`, and
`--dangerously-skip-permissions` must be captured as policy-sensitive run
configuration, not normal defaults.
When `--pure` is used, the claim should record that external plugins were
suppressed; a pure run cannot prove plugin coverage unless the tested plugin
surface is explicitly re-enabled through a separate fixture.

## Normalized Session Schema

Use one schema with product-specific extension fields:

```text
agent_session
  session_id
  agent_product
  agent_version
  adapter_name
  adapter_version
  project_id
  repo
  branch
  start_time
  end_time
  status
  user_or_actor_ref
  permission_mode
  model_refs
  source_trace_ref
  redaction_report_ref
  source_field_policy_ref

agent_turn
  turn_id
  session_id
  prompt_ref_or_hash
  prompt_length
  context_refs
  model_ref
  started_at
  ended_at
  stop_reason

agent_action
  action_id
  turn_id
  kind
  tool_name
  input_policy
  input_hash
  output_policy
  output_hash
  cwd
  started_at
  ended_at
  status
  error_type
  evidence_refs
```

Required action kinds:

| Kind | Minimum fields |
| --- | --- |
| `context_load` | source type, source ref, path/query, redaction status |
| `model_call` | model ref, token/cost fields when available, status |
| `tool_call` | tool name, MCP/server ref when available, input/output policy, duration, status |
| `shell_command` | command identity, sanitized args, exit code, stdout/stderr refs, traceparent if available |
| `file_read` | path policy, hash/ref, range if available |
| `file_edit` | path policy, patch hash/ref, added/deleted line counts |
| `permission_decision` | requested tool/action, decision, source, actor if available |
| `subagent` | parent session/turn, subagent id/type, status |
| `compaction` | before/after token counts when available, summary ref/hash |
| `state_verification` | state claim, verifier kind, verifier ref, supported/contradicted/unverified status |
| `outcome` | PR/commit/patch/test/deploy refs, accepted/rejected/reverted/unknown |

## Redaction Defaults

Agent sessions are more sensitive than CLI traces because they join prompts,
repo context, tool inputs, file contents, shell output, and MCP responses.

Default Parallax capture:

- prompt length, prompt hash, and prompt ref, not prompt body;
- tool name, input field names, input hash, and redaction report, not raw args;
- file paths by policy, patch hashes, and bounded redacted diffs only when
  enabled;
- shell commands through the same policy as
  [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md);
- full transcript/export/session JSON as raw refs only, with short TTL and audit;
- model reasoning/thinking content excluded unless the source tool exposes it
  and the project explicitly allows it.

Agent-visible bundles must pass the
[redaction pipeline](redaction-pipeline-and-secret-safety.md) after normalization,
not only trust the source tool's built-in masking. Synthetic and evaluation
fixture runs must also carry a passing source-field policy row before any
projection is claimable. The safe projection is the canonical bundle JSON with
`schema_ref`, post-redaction `canonical_hash`, `projection_manifest`, and
`access`; Markdown, CLI, HTTP, and MCP content are projections, and MCP delivery
must use `structuredContent` validated against the bundle `outputSchema`.

## Value Evaluation Gate

The proof gate is not "can we ingest events?" It is "does normalized session
tracing answer audit and improvement questions better than raw transcripts or
no trace?"
Results and claim status belong in the
[agent session tracing ledger](agent-session-tracing-ledger.md), not in this
design note.

### Dataset

Run the same task set across Codex, Claude Code, Amp, and OpenCode:

| Task | Why it matters |
| --- | --- |
| Small deterministic bug fix | Tests whether actions, edits, validation, and outcome reconstruct cleanly. |
| Failing test investigation | Tests command, output, and file-read linkage. |
| Redaction canary task | Tests whether prompts, tool inputs, shell output, and diffs leak seeded secrets. |
| CLI failure repair | Tests linkage between agent session and CLI invocation evidence. |
| Documentation/research update | Tests long context, source refs, and final artifact traceability. |
| Permission-sensitive command | Tests approval/denial capture and side-effect audit. |

Initial size: at least five tasks per agent, one retry allowed per task, for 20
to 40 sessions. Store raw refs only in a controlled local fixture project.

### Comparison Arms

| Arm | Input to evaluator |
| --- | --- |
| Final output only | Commit/diff/summary and test result, no session trace. |
| Native transcript/export | Tool-native session artifact where available. |
| Parallax normalized session | Common schema only, redacted by policy. |
| Parallax linked evidence | Normalized session plus linked runtime/CI/CLI evidence bundle. |

### Metrics

| Metric | Target |
| --- | --- |
| Tool-call coverage | >= 90 percent of surfaced tool calls mapped to typed `agent_action` rows. |
| Command/edit coverage | 100 percent of surfaced shell commands and file edits captured when the source exposes them. |
| Audit answer accuracy | Evaluator can answer who/what/when/which tool/which file/which command/which outcome for >= 80 percent of sessions from normalized data alone. |
| Evidence citation completeness | Agent-produced diagnosis or PR proposal cites deterministic session/evidence refs for each material claim. |
| State verification coverage | 100 percent of agent-visible state-change or validation claims have verifier rows, or are explicitly labeled unverified. |
| Redaction | Zero seeded canary leaks in canonical JSON, Markdown, CLI/HTTP output, and MCP `structuredContent`. |
| Projection safety | Raw transcript/export/tool payload refs are present only as refs, are not dereferenced in agent-visible projections, and every projection matches the canonical bundle hash. |
| Source-field policy | Synthetic/evaluation fixtures pass source-field policy checks before redaction or projection claims pass. |
| Adapter lossiness report | Every unmapped source event is counted with reason: unsupported, redacted, raw-ref-only, parse failure, or source-not-exposed. |
| Overhead | Capture must not make the agent workflow noticeably slower; measure wall time delta and adapter CPU/RSS for each tool. |
| Outcome linkage | Patch/test/commit/PR outcome can be linked back to the session in >= 80 percent of successful runs. |

## Pass/Fail Gate

Pass the agent-session gate only if:

1. At least two agents pass through supported, non-brittle capture surfaces
   without parsing unstable transcripts as the only source.
2. Claude Code OTel ingestion maps sessions, tools, API calls, and identity
   into the normalized schema.
3. Codex hooks plus wrapper or repo-diff observation map session lifecycle, tool
   calls, permission requests, subagent starts/stops, command/edit evidence, and
   uncovered tool paths, with transcript files stored only as raw refs.
4. Either Amp plugin/streaming capture or OpenCode JSON/export/plugin capture
   provides a second open/non-OTel adapter path.
5. The redaction, source-field, and projection suites have zero seeded canary
   leaks, zero source-field violations, zero default raw-ref dereferences,
   matching canonical hashes across CLI/API/MCP projections, and valid MCP
   `structuredContent` when MCP is tested.
6. Normalized Parallax sessions answer audit questions faster or more accurately
   than final-output-only and raw-transcript arms.
7. The adapter emits an honest lossiness report for every unsupported source
   event class.
8. State-change and validation claims are backed by verification rows; command
   exit code and stdout/stderr alone can only support reported-result wording.

Fail or narrow if:

- useful reconstruction requires storing full prompts, full tool outputs, or
  full transcripts by default;
- tool-specific formats change too often to maintain adapters;
- normalized traces do not improve audit or fix-quality evaluation over raw
  transcripts;
- redaction strips so much data that the trace no longer answers audit
  questions;
- only one proprietary tool can be captured well.

## Build Sequence

1. Build a neutral `agent_session` importer and lossiness report.
2. Implement Claude Code OTel ingest first because it is native and
   OpenTelemetry-shaped; add the Claude stream-json adapter as a fixture and
   non-interactive validation surface, not as a replacement for OTel.
3. Implement Codex hook ingestion next, paired with a Parallax CLI wrapper and
   repo diff/hash capture, plus a separate `codex exec --json` fixture adapter,
   because Codex is already part of the Parallax operator workflow and exposes
   structured hook events.
4. Implement Amp plugin-event ingestion plus streaming JSON ingestion, because
   Amp plugins now appear to cover interactive and execute-mode lifecycle/tool
   events.
5. Implement OpenCode export/plugin ingestion as the second open-tool adapter
   with deep session hooks.
6. Run the value evaluation gate across all four tools before claiming
   "agent-session tracing" as a general capability.

## Product Decision

Ship wording should be conservative:

- "Parallax normalizes supported coding-agent session events."
- "Parallax links agent actions to CLI, CI, runtime, and outcome evidence."
- "Parallax captures prompts, tool content, and transcripts as raw refs only
  when explicitly enabled."
- "Parallax does not depend on hidden reasoning traces."
- "Adapter coverage is reported per agent product and version."

This keeps the thesis strong without pretending every agent exposes the same
observability interface.

## Relationship To Other Research

- [Agent and CLI execution tracing](agent-and-cli-execution-tracing.md) defines
  why coding-agent sessions are first-class execution evidence.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
  defines how native OTel GenAI/MCP/CLI spans become stable Parallax rows without
  treating development-stage conventions as the storage schema.
- [Agent session tracing ledger](agent-session-tracing-ledger.md) turns this
  adapter strategy into a tool/version matrix, coverage/lossiness rows,
  redaction results, audit-value comparisons, and claim levels.
- [Agent observability technical review](agent-observability-technical-review.md)
  surveys the broader LLM/agent observability market and technical patterns.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
  defines the `agent_session`, `agent_action`, source-field policy status,
  redaction report, and audit edge targets.
- [CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md)
  supplies the shell-command policy used inside agent sessions.
- [CLI trace safety ledger](cli-trace-safety-ledger.md) supplies the claimable
  shell-command safety rows for agent-session runs that include CLI execution.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  is the veto gate before agent-session evidence becomes agent-visible.
- [Bundle-value Phase 0 runbook](bundle-value-phase0-runbook.md) is the nearest
  existing experiment shape for measuring whether better context improves agent
  output.

## Bottom Line

Agent-session tracing is technically feasible, but the unit of truth is a
normalized, redacted, lossiness-reported execution audit. Claude Code gives
Parallax the cleanest native OTel path. Codex gives hooks. Amp gives plugin
events plus streaming JSON. OpenCode gives run JSON, export JSON, plugins,
server/API, and ACP. Together they are enough to test whether Parallax session
traces improve audit and fix-quality loops, but not enough to claim perfect
replay, default raw transcript exposure, or universal raw transcript safety.
