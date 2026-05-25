# Agent and CLI Execution Tracing

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note extends the Parallax thesis beyond long-running services and CI runs.
It captures two additional first-class domains:

1. coding-agent execution traces;
2. CLI application execution traces.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

## Thesis Update

Parallax should be an evidence engine for software execution, not only an
observability backend for services.

As teams move more work from direct human operation to coding agents and
operator agents, the system must make agent behavior inspectable. Agents will
read context, choose tools, run commands, edit files, call APIs, and sometimes
touch databases or deploy paths. Without a durable trace, the organization only
sees the final result and scattered terminal output, not the chain of actions
that produced it.

The target domains are:

| Domain | Execution shape | Why Parallax fits |
| --- | --- | --- |
| Services | Long-running runtime, request/job traces, production errors. | Sentry-compatible and OTLP-native runtime context. |
| CI runs | Bounded workflow/job/test execution. | Deterministic bundle, logs, artifacts, test history. |
| CLI apps | Bounded command invocation with args/env/stdout/stderr/exit. | Reproducible local failure context and Rust-first error capture. |
| Coding agents | Bounded or long-running agent session with prompts, tools, file edits, tests, PRs. | Agent trust, replay, audit, outcome feedback, and failure/fixer-outcome corpus. |

This is stronger than "observability for apps." It is:

> observability for autonomous software work.

## Why Agent Tracing Matters

If Parallax sends evidence to agents, Parallax must also trace what agents do
with that evidence.

Without agent traces, the system cannot answer:

- What context did the agent receive?
- Which hypothesis did it follow?
- Which files did it read?
- Which MCP/API tools did it call?
- Which shell commands did it run?
- What did it edit?
- Which tests did it run?
- Why did it open this PR?
- Did the PR fix the problem, get rejected, or cause a regression?

More importantly, it cannot answer audit questions after a bad outcome:

- Which agent, user, token, or workflow initiated the action?
- What prompt, issue, alert, ticket, or command triggered it?
- What approvals or policy checks were present?
- Which database, deploy, file, or external API operation was attempted?
- Was the action directly requested, inferred by the agent, or caused by a tool
  response?
- Which intermediate step first diverged from the expected path?

That audit trail matters because agent work can create many hidden side effects.
If a customer reports an error, a migration corrupts data, or a database table
is dropped, Parallax should help reconstruct how the event happened rather than
only show the final exception.

This creates the feedback loop that the earlier strategy docs identify as a
possible moat:

```text
failure evidence
  -> agent session trace
  -> tool calls / edits / tests
  -> PR or proposal
  -> accepted / rejected / modified / reverted
  -> better evidence ranking and agent policy
```

The result is "debug the debugger": Parallax can explain both the original
runtime failure and the agent behavior that tried to fix it.

The long-term product should support evidence-backed questions such as:

- What did the agent do before this incident?
- Which command or tool changed this table?
- Which files did the agent edit before the failing deploy?
- Which context did the agent rely on, and was it stale?
- Which guardrail or approval should have stopped this action?

That is stronger than logs scattered across terminals, CI, SaaS dashboards, and
agent transcripts. It is observability across the layers where autonomous work
happens.

## OpenTelemetry Fit

OpenTelemetry already has emerging vocabulary for this:

- Semantic Conventions `1.41.0` include Generative AI and CLI program areas.
- GenAI agent semantic conventions define agent invocation, workflow, and tool
  execution spans.
- GenAI client spans cover model inference, retrievals, tool calls, content
  capture, token usage, and streaming chunks.
- MCP semantic conventions define client/server spans, session metrics, resource
  URIs, JSON-RPC request IDs, tool names, prompt names, and trace propagation
  guidance through MCP metadata.
- CLI semantic conventions define short-lived CLI execution spans and mark
  non-zero `process.exit.code` as an error.
- Process resource conventions define command/process attributes and explicitly
  warn that full command args should not be collected by default without
  sanitization.

Sources:

- [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/)
- [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)
- [OpenTelemetry GenAI spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)
- [OpenTelemetry CLI semantic conventions](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/)
- [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/)

Current source check on 2026-05-25: the official semantic-convention catalog is
still `1.41.0`, and the GenAI agent, GenAI client, MCP, and CLI pages are still
development-stage. That makes OTel useful ingestion vocabulary, not proof that
real coding-agent tools emit a complete or stable trace. Parallax should map
OTel spans through the
[Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
and measure real Codex, Claude Code, Amp, and OpenCode adapters through the
[agent session tracing ledger](agent-session-tracing-ledger.md).

Follow-up source check on 2026-05-25: OTel CLI spans require process exit code
and define non-zero exit as an error, while command args are not default-safe
without sanitization. OTel CICD, test, and VCS registries provide useful result
and head/base vocabulary. They still do not prove external state. Parallax must
separate "the command ran and reported status" from "the system state changed or
was validated."

## Market Room

There is clearly room in the sense that many teams and vendors are already
looking for agent observability:

| Product / project | What it proves | Parallax implication |
| --- | --- | --- |
| LangSmith | LangChain docs say agent traces record every execution step, including tool calls, model interactions, and decision points. | Agent trace visibility is now expected by agent builders. |
| Langfuse | Uses tracing SDKs and OpenTelemetry packages; explicitly supports short-lived app flush/shutdown patterns. | OpenTelemetry-backed LLM tracing is already a self-hostable category. |
| Arize Phoenix | Open-source AI observability/evaluation; traces model calls, retrieval, tool use, and custom logic over OTLP/OpenTelemetry/OpenInference. | Trace plus eval plus replay is a validated AI-app workflow. |
| Braintrust | Captures LLM calls, tool calls, nested application logic, token usage, errors, scores, human feedback, and eval datasets. | Outcome/evaluation loop is already seen as core, not optional. |
| Datadog LLM Observability | Traces prompts, model responses, retrieval steps, and tool calls, and correlates AI behavior with backend services and infra. | Incumbents are joining agent traces to app/infra telemetry. |
| Helicone Sessions | Groups related LLM calls, vector DB queries, and tool calls to trace an entire agent flow. | Session-level grouping is table stakes for multi-call agents. |
| OpenLLMetry / Traceloop | Open-source LLM observability built on OpenTelemetry, exportable to existing observability stacks. | OTEL-native agent telemetry is becoming the interoperability path. |
| AgentTrace / AgentSight research | Recent papers explicitly frame agent observability as needed for reliable deployment, risk analysis, and bridging high-level intent with low-level actions. | Research community also sees a semantic gap in agent observability. |

Sources:

- [LangSmith observability docs](https://docs.langchain.com/oss/python/langchain/observability)
- [Langfuse observability SDK overview](https://langfuse.com/docs/observability/sdk/overview)
- [Arize Phoenix docs](https://arize.com/docs/phoenix)
- [Braintrust tracing docs](https://www.braintrust.dev/docs/instrument)
- [Datadog LLM Observability](https://www.datadoghq.com/product/ai/llm-observability/)
- [Helicone sessions docs](https://docs.helicone.ai/features/sessions)
- [OpenLLMetry GitHub repository](https://github.com/traceloop/openllmetry)
- [AgentTrace paper](https://arxiv.org/abs/2602.10133)
- [AgentSight paper](https://arxiv.org/abs/2508.02736)

### Where Parallax Still Has Space

The crowded part is "trace my LLM app." The open space is narrower:

1. **Coding-agent observability, not only LLM-app observability.** Most products
   target LangChain/RAG/agent applications. Parallax should target Codex,
   Claude Code, Amp, OpenCode, CI agents, shell commands, file edits, test runs,
   diffs, PRs, and outcomes.
2. **Runtime evidence plus agent trace.** Existing AI observability tools often
   know model/tool spans. Parallax also knows the production error, Sentry event,
   OTLP trace/log/metric context, deploy, CI run, and repo intent.
3. **CLI execution as first-class evidence.** CLI apps and agent-invoked commands
   are usually treated as shell output. Parallax can make each command a trace
   with exit code, error chain, redacted args/env, cwd/repo state, and spawned
   process spans.
4. **Outcome feedback.** Store whether a generated fix was accepted, edited,
   reverted, or linked to recurrence. That closes the loop from evidence to fix
   quality.
5. **Open self-hosted Rust-first system.** Langfuse/Phoenix/OpenLLMetry are
   open or OTEL-friendly, but they are not a Rust-first Sentry-compatible
   runtime-context engine for services, CLIs, CI, and coding agents.

So yes, people are already looking for this class of solution. That is good
validation. The Parallax opening is not generic agent tracing; it is the unified
execution graph connecting runtime failures, CLI commands, CI runs, agent
actions, patches, and outcomes.

## Agent Trace Model

Represent one observed agent run as one normalized Parallax session, backed by
the capture surface that produced it. When a source emits OTel spans, preserve
those spans as refs and map them into stable rows. When a source emits hooks,
plugin events, JSONL/stream JSON, exports, server/API events, ACP messages, or
wrapper observations, map those events into the same rows with explicit adapter
provenance and lossiness.

A query or UI may render the session as a trace, but the storage contract should
not require every tool to emit these exact span names:

```text
agent session trace
  -> context bundle loaded
  -> model call
  -> hypothesis created
  -> MCP/API tool call
  -> file read
  -> shell command
  -> file edit
  -> test command
  -> patch/PR proposal
  -> outcome
```

Normalized actions:

| Parallax row/action | Possible source signals | Notes |
| --- | --- | --- |
| `agent_session` | OTel session/event, Codex hook session events, Amp plugin lifecycle events, OpenCode run/export/session events, wrapper root. | Store tool binary/version/config, adapter name/version, capture surface, source schema snapshot, and claim level. |
| `agent_action(kind=context_load)` | Parallax bundle access, file/ADR/issue/trace/log load, MCP resource read. | Link every context item to source evidence and redaction status. |
| `agent_action(kind=model_call)` / `agent_turn` | OTel GenAI invoke/chat spans, stream JSON model objects, source tool metadata when exposed. | Store model/provider/token counts when available; prompt and output content are opt-in/redacted/raw-ref-only. |
| `agent_action(kind=tool_call)` | OTel `execute_tool`, MCP spans, Codex hooks, Amp/OpenCode plugin events, JSONL/stream JSON objects. | Tool name, input/output hashes or refs, status, error type, and deduplication refs. |
| `agent_action(kind=shell_command)` | CLI wrapper, tool hook/plugin events, run JSON, subprocess spans. | Store structural command identity, args policy, exit status, stdout/stderr refs, and traceparent when available. This proves reported execution, not durable state. |
| `agent_action(kind=file_read|file_edit)` | Hook/plugin file events, patches, repo diff/hash observation, export/import rows. | File path policy, patch hash/ref, added/deleted line counts, and gaps when the source cannot prove a read or edit. |
| `agent_action(kind=permission_decision)` | Approval hooks, permission plugin events, policy mode, dangerous CLI flags. | Record actor/source when available and separate denied actions from missing policy evidence. |
| `agent_action(kind=state_verification)` | Repo diff/hash, file stat/hash, test report, provider API readback, database readback, deployment status, runtime recurrence check, fixture expectation. | Required before projecting "validated," "changed," "deployed," or "fixed" claims. |
| `agent_action(kind=validation|outcome)` | Test/build/lint command, commit/PR/review refs, recurrence/revert/human decision rows. | Required before direct-fix claims and outcome-feedback claims. Test command success alone is a reported validation result, not production fix proof. |

## Agent Records

Store low-volume agent metadata in Turso and high-volume traces/logs/events in
GreptimeDB.

```text
agent_sessions(
  session_id,
  project_id,
  repo,
  branch,
  commit_sha,
  agent_product,
  agent_version,
  adapter_name,
  adapter_version,
  capture_surface,
  source_schema_snapshot,
  tool_config_ref,
  content_capture_level,
  model,
  trigger_kind,
  anchor_issue_id,
  context_bundle_id,
  redaction_report_ref,
  lossiness_report_ref,
  started_at,
  ended_at,
  outcome
)

agent_steps(
  step_id,
  session_id,
  parent_step_id,
  step_type,
  summary,
  started_at,
  ended_at,
  status,
  source_event_class,
  evidence_refs,
  redaction_level
)

agent_tool_calls(
  tool_call_id,
  session_id,
  step_id,
  tool_name,
  input_hash,
  output_hash,
  status,
  error_type,
  evidence_refs
)

agent_patches(
  patch_id,
  session_id,
  base_commit,
  head_commit,
  files_changed,
  diff_ref,
  validation_refs,
  outcome
)
```

Do not store full prompts, full model outputs, full shell output, or full diffs
by default. Store hashes, summaries, redacted excerpts, and artifact refs. Raw
capture must be opt-in and scoped.

Agent records are not product claims until fixture artifacts prove the target
capture surface. A passing Claude OTel fixture does not prove Claude
`stream-json`; a passing Codex hook fixture does not prove `codex exec --json`;
and OpenCode run JSON, export JSON, plugins, server/API, and ACP need separate
rows.

## CLI Trace Model

CLI apps are an excellent first-class target because they are bounded,
reproducible, and common in this project.

One CLI invocation should become one trace:

```text
parallax_cli_invocation trace
  -> parse args
  -> load config
  -> read repo state
  -> execute subcommand phases
  -> spawned process spans
  -> stdout/stderr events
  -> exit/panic/error event
```

For Rust CLIs, expose a tiny setup:

```rust
parallax::cli::init();
```

Capture:

| Field | Handling |
| --- | --- |
| command/subcommand | Always, low-cardinality names. |
| `argv` | Sanitized/opt-in; full args can contain secrets. |
| environment | Redacted allowlist only. |
| cwd/repo/commit | Capture when inside a repo. |
| config file paths | Capture path and hash, not secrets. |
| stdout/stderr | Bounded excerpts; full capture opt-in. |
| exit code/signal | Always. |
| panic/error chain | Always for Rust apps. |
| backtrace/span trace | Capture when enabled and redacted. |
| spawned child processes | Client spans with command name and exit code. |
| file/network effects | Later; only if observable and safe. |

Observation boundary:

- `exit_code=0` means the process reported successful completion.
- stdout/stderr excerpts are reported output, not policy or durable truth.
- a file/tool hook event means a source reported an operation, not necessarily
  that post-state matches the intended change;
- state-changing claims need readback: repo diff/hash, file stat/hash, test
  report, provider API readback, database readback, deployment status, runtime
  recurrence check, or fixture expectation.

Agent-visible wording should say "command reported success" unless a verifier
row supports stronger wording such as "patch validated" or "deploy succeeded."

This makes Parallax useful for local tools, deploy tools, migration tools,
developer CLIs, and agent-invoked commands.

The default-on safety and performance gate for this model is specified in
[CLI trace overhead and redaction](cli-trace-overhead-and-redaction.md):
structural capture is the default, redacted excerpts require canary and
overhead tests, and full raw args/env/output are opt-in refs only.
The [CLI trace safety ledger](cli-trace-safety-ledger.md) defines the dated
result rows and claim levels required before Parallax can say structural CLI
tracing is default-ready or redacted excerpts are safe.
The OpenTelemetry-to-Parallax field mapping, semconv versioning, GenAI/MCP
deduplication, and lossiness gates are specified in
[Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md).
Real-tool adapter results and claim levels are specified in the
[agent session tracing ledger](agent-session-tracing-ledger.md).

## Why CLI Tracing Is Strategically Useful

CLI failures are easier than production incidents:

- clear start/end;
- clear command input;
- local repo state often available;
- bounded stdout/stderr;
- exact exit code;
- easy rerun;
- agent can patch and retry quickly.

That makes CLI execution a strong bridge between CI bundles and production
runtime observability.

## Unified Execution Graph

The same graph can connect app, CI, CLI, and agent evidence:

```text
production error
  -> evidence bundle
  -> agent session
  -> CLI/test command
  -> patch
  -> CI run
  -> deploy
  -> recurrence fixed or not fixed
```

New edge types:

| Edge | Meaning |
| --- | --- |
| `agent_used_evidence` | Agent context included this event/span/log/test/doc. |
| `agent_called_tool` | Agent performed MCP/API/shell/file action. |
| `agent_changed_file` | Patch touched source file. |
| `agent_validated_with` | Test/build/CLI command reported a validation result for the patch. |
| `agent_verified_state` | Independent readback verified or contradicted a state claim. |
| `cli_spawned_process` | CLI invoked child command. |
| `cli_failed_with_error` | CLI produced error/exit/panic evidence. |
| `fix_attempt_for_issue` | Patch/session tried to fix issue. |
| `fix_outcome` | Accepted, rejected, reverted, no recurrence, recurrence. |

This turns Parallax into the audit trail for autonomous debugging.

## Product Implication

Update the product thesis:

> Parallax is an evidence engine for software execution: services, CI runs, CLI
> tools, and coding agents.

The product reason is simple: agents are becoming a primary way people operate
software systems. If the future workflow is "ask an agent to investigate,
change, deploy, or repair," then the future audit requirement is "show exactly
what the agent saw, decided, executed, changed, verified or failed to verify,
and produced."

Parallax should make those actions queryable, explainable, and tied back to
runtime evidence.

Near-term sequence:

1. Keep Sentry-compatible Rust service errors as the first production wedge.
2. Add CLI instrumentation for Rust tools because it is bounded and cheap.
3. Trace Parallax's own agent/MCP interactions from day one.
4. Add real-tool agent-session ingestion surface by surface, with separate
   native OTel, hook/plugin, JSONL/stream JSON, export/API/ACP, wrapper, and
   raw-ref claims controlled by the real-tool adapter gate in
   [Agent session tracing across real tools](agent-session-tracing-real-tools.md).
5. Map OTel GenAI/MCP/CLI input through the
   [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
   instead of storing raw OTel spans as the durable schema.
6. Use agent outcome feedback to improve evidence ranking and autonomy policy.

This makes the "agent context engine" claim more defensible: Parallax observes
not only failing software, but also the agent that acts on the failure.

## Risks

| Risk | Control |
| --- | --- |
| Capturing secrets in prompts/args/stdout | Hash and redacted excerpts by default; raw opt-in only. |
| Huge traces from agent loops | Session summaries plus span/event caps. |
| Vendor-specific agent formats | Normalize to Parallax schema plus OTel GenAI/MCP where possible; claim support per adapter, capture surface, tool version, and config. |
| Privacy of repo context | Same policy as production data: scoped access, audit, retention. |
| Overfitting to one agent | Store `agent_product`, `agent_version`, `adapter_name`, and `capture_surface`; avoid model-specific core schema. |
| False trust in agent traces | Trace what happened; do not assume reasoning text is truth. |

## Bottom Line

Agent and CLI tracing should be part of the core thesis, not a later side quest.

Services explain runtime failures. CI and CLI traces explain bounded execution
failures. Agent traces explain how autonomous software work used evidence and
what result it produced. Together they create the feedback loop that can make
Parallax defensible.
