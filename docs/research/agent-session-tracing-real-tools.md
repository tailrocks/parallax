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
| [Codex CLI](https://developers.openai.com/codex/cli) | Codex CLI is local, open source, Rust-built, and can inspect repos, edit files, and run commands. This makes it a direct Parallax target for local coding-agent session tracing. |
| [Codex hooks](https://developers.openai.com/codex/hooks) | Codex hooks expose `session_id`, `cwd`, `hook_event_name`, `model`, `permission_mode`, `tool_name`, `tool_use_id`, and `tool_input` for events such as session start, tool use, permission requests, subagents, and stop. The docs warn that `transcript_path` is not a stable hook interface and that `PreToolUse`/`PostToolUse` interception is incomplete for some shell and non-shell tool paths. Parallax should use hooks for structured events, treat transcripts as raw refs only, and measure hook coverage against wrapper and repo-diff evidence. |
| [Codex MCP](https://developers.openai.com/codex/mcp) | Codex supports MCP servers in CLI and IDE clients, including local stdio and remote HTTP servers. Parallax can provide a read-only MCP context surface, but MCP configuration also introduces tokens, headers, and tool-call audit needs. |
| [OpenAI agent-improvement cookbook](https://developers.openai.com/cookbook/examples/agents_sdk/agent_improvement_loop) | OpenAI's own agent-improvement loop starts from traces, adds feedback, converts expectations into evals, and produces a Codex-ready handoff. That validates Parallax's "trace -> feedback -> eval -> better agent work" loop. |
| [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage) | Claude Code has the strongest first-party telemetry posture: opt-in OTel metrics/logs/events and beta traces. It records sessions, tool activity, API calls, costs, tokens, commits, PRs, active time, and MCP activity. Prompt text, tool details, tool content, and raw API bodies are disabled by default and require explicit flags. |
| [Amp manual](https://ampcode.com/manual) | Amp exposes threads, subagents, AGENTS.md loading, MCP, execute mode, non-interactive use, and streaming JSON for programmatic integration and real-time conversation monitoring. It does not appear to present a first-party OTel surface in the manual, so Parallax should ingest Amp through streaming JSON, CLI wrapping, and thread/outcome refs. |
| [OpenCode CLI](https://opencode.ai/docs/cli/) | OpenCode exposes session IDs, continuation/forking, `run --format json` raw JSON events, `session list`, `export` with a `--sanitize` flag, token/cost stats, a headless server, and ACP. This is a strong import and live-adapter target. |
| [OpenCode plugins](https://opencode.ai/docs/plugins/) | OpenCode plugins can hook command, file, message, permission, server, session, shell, tool, and TUI events. That makes OpenCode the easiest open tool to instrument deeply without parsing terminal output, but fixture runs still need class-by-class proof that enabled plugin events were observed and normalized. |
| [OpenCode MCP servers](https://opencode.ai/docs/mcp-servers/) | OpenCode supports local and remote MCP servers with commands, environment variables, headers, OAuth, enablement, and per-agent management. Parallax should treat MCP config as both context source and secret-bearing audit surface. |

## Adapter Strategy

Parallax should not wait for every agent to export the same trace format.
Instead, build adapters into one normalized session schema.

| Agent | Best initial adapter | Capture strength | Main gaps |
| --- | --- | --- | --- |
| Claude Code | Native OTel logs/events/traces into Parallax ingest. | Strongest first-party signal: sessions, tools, API requests, costs, tokens, commits, PRs, MCP, identity, and optional traces. | Traces are beta; raw prompt/tool content is intentionally off by default; subprocess telemetry requires explicit propagation/config. |
| Codex | Hook adapter plus Parallax CLI wrapper, repo diff/hash observation, and raw transcript refs. | Strong lifecycle/tool/permission signals, session IDs, model, cwd, subagents, and MCP tool inputs. | Transcript format is not stable; hook interception is incomplete for some tool paths; no first-party OTel export in the checked docs. |
| Amp | Streaming JSON adapter for execute mode plus thread refs and CLI wrapper. | Good programmatic stream for non-interactive sessions; threads and MCP provide useful refs. | Manual does not show native OTel; interactive session capture likely needs plugin/API or wrapper work; permissions are broad by default. |
| OpenCode | `run --format json`, `export --sanitize`, plugin hooks, and server/API adapter. | Strong open adapter path: raw JSON events, session export/list, plugins for session/tool/file/permission events. | Need fixture tests to prove JSON/export stability and sanitation quality across versions. |

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
- transcript use as `raw_ref_only`, never as the stable structured source.

### OpenCode Plugins Are Class-By-Class

OpenCode plugin support should not be inferred from one JSON stream or session
export. A fixture must list the plugin event classes it expects to exercise,
such as command, file, message, permission, session, shell, and tool events, and
then record which classes were observed and mapped. `export --sanitize` remains
a useful source feature, but it is not Parallax redaction proof and it does not
prove live plugin coverage.

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
not only trust the source tool's built-in masking.

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
| Redaction | Zero seeded canary leaks in JSON and Markdown projections. |
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
4. Either Amp streaming JSON or OpenCode JSON/export/plugin capture provides a
   second open/non-OTel adapter path.
5. The redaction suite has zero seeded canary leaks in agent-visible outputs.
6. Normalized Parallax sessions answer audit questions faster or more accurately
   than final-output-only and raw-transcript arms.
7. The adapter emits an honest lossiness report for every unsupported source
   event class.

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
   OpenTelemetry-shaped.
3. Implement Codex hook ingestion next, paired with a Parallax CLI wrapper and
   repo diff/hash capture, because Codex is already part of the Parallax
   operator workflow and exposes structured hook events.
4. Implement OpenCode export/plugin ingestion as the first open-tool adapter
   with deep session hooks.
5. Implement Amp streaming JSON ingestion for non-interactive tasks and treat
   interactive threads as raw refs until a stronger surface is proven.
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
  defines the `agent_session`, `agent_action`, and audit edge targets.
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
Parallax the cleanest native OTel path. Codex gives hooks. OpenCode gives export,
JSON, and plugins. Amp gives streaming JSON and thread refs. Together they are
enough to test whether Parallax session traces improve audit and fix-quality
loops, but not enough to claim perfect replay or universal raw transcript safety.
