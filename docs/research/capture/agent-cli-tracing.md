# Agent and CLI Execution Tracing

<!-- markdownlint-disable MD013 -->

> Parallax should be an evidence engine for software execution — services, CI runs, CLI tools, and coding agents — not only an observability backend for services, and coding-agent sessions plus CLI invocations are first-class execution evidence. OpenTelemetry GenAI, MCP, CLI, process, and CI/CD semantic conventions (catalog `1.41.0`, still development-stage for GenAI/MCP/CLI) are ingestion vocabulary that must be mapped into stable Parallax `agent_session`, `agent_action`, `cli_invocation`, and `ci_run` rows with recorded `semconv_version`, lossiness reports, and state verification — never stored as the durable product schema. Agent-session tracing across real tools (Codex CLI `0.133.0`, Claude Code `2.1.150`, Amp `0.0.1779639467-g6d0650`, OpenCode `1.15.10`) is viable only as a lossy, redacted, normalized execution audit via per-tool adapters (native OTel, hooks, streaming/run JSON, export/plugin, server/API, ACP), not as complete reasoning capture, and the agent-session ledger status is **not measured** (initial level `not_measured`). CLI tracing is default-on only for structural capture; redacted output excerpts require a separate canary-plus-overhead proof gate, raw args/env/stdout/stderr/config are opt-in raw refs only, and the CLI trace safety ledger status is likewise **not measured**. The decided posture is conservative wording with default-deny redaction and explicit claim levels; the open gates are the dated four-arm measurement runs that move both ledgers above `not_measured` (at minimum two agents through non-brittle surfaces, zero seeded canary leaks across canonical JSON/Markdown/CLI/HTTP/MCP projections, state-verification rows before any "validated/changed/deployed/fixed" wording, and positive audit-value lift).

This note consolidates the following previously-separate research files, each preserved in full below:

- `agent-and-cli-execution-tracing.md`
- `agent-cli-otel-semconv-mapping.md`
- `agent-session-tracing-real-tools.md`
- `agent-session-tracing-ledger.md`
- `cli-trace-overhead-and-redaction.md`
- `cli-trace-safety-ledger.md`

## Agent and CLI Execution Tracing (thesis and trace models)

_Provenance: merged verbatim from `agent-and-cli-execution-tracing.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

This note extends the Parallax thesis beyond long-running services and CI runs.
It captures two additional first-class domains:

1. coding-agent execution traces;
2. CLI application execution traces.

Version freshness rule: this recommendation is based on current public docs and
source material checked on 2026-05-25. Every future benchmark or comparison must
use the latest reasonably available stable/public version of each candidate as
of the benchmark date, and must label older benchmark posts or architecture docs
as historical evidence.

### Thesis Update

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

### Why Agent Tracing Matters

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

### OpenTelemetry Fit

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
[Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
and measure real Codex, Claude Code, Amp, and OpenCode adapters through the
[agent session tracing ledger](agent-cli-tracing.md).

Follow-up source check on 2026-05-25: OTel CLI spans require process exit code
and define non-zero exit as an error, while command args are not default-safe
without sanitization. OTel CICD, test, and VCS registries provide useful result
and head/base vocabulary. They still do not prove external state. Parallax must
separate "the command ran and reported status" from "the system state changed or
was validated."

### Market Room

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

#### Where Parallax Still Has Space

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

### Agent Trace Model

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

### Agent Records

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

### CLI Trace Model

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
[CLI trace overhead and redaction](agent-cli-tracing.md):
structural capture is the default, redacted excerpts require canary and
overhead tests, and full raw args/env/output are opt-in refs only.
The [CLI trace safety ledger](agent-cli-tracing.md) defines the dated
result rows and claim levels required before Parallax can say structural CLI
tracing is default-ready or redacted excerpts are safe.
The OpenTelemetry-to-Parallax field mapping, semconv versioning, GenAI/MCP
deduplication, and lossiness gates are specified in
[Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md).
Real-tool adapter results and claim levels are specified in the
[agent session tracing ledger](agent-cli-tracing.md).

### Why CLI Tracing Is Strategically Useful

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

### Unified Execution Graph

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

### Product Implication

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
   [Agent session tracing across real tools](agent-cli-tracing.md).
5. Map OTel GenAI/MCP/CLI input through the
   [Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
   instead of storing raw OTel spans as the durable schema.
6. Use agent outcome feedback to improve evidence ranking and autonomy policy.

This makes the "agent context engine" claim more defensible: Parallax observes
not only failing software, but also the agent that acts on the failure.

### Risks

| Risk | Control |
| --- | --- |
| Capturing secrets in prompts/args/stdout | Hash and redacted excerpts by default; raw opt-in only. |
| Huge traces from agent loops | Session summaries plus span/event caps. |
| Vendor-specific agent formats | Normalize to Parallax schema plus OTel GenAI/MCP where possible; claim support per adapter, capture surface, tool version, and config. |
| Privacy of repo context | Same policy as production data: scoped access, audit, retention. |
| Overfitting to one agent | Store `agent_product`, `agent_version`, `adapter_name`, and `capture_surface`; avoid model-specific core schema. |
| False trust in agent traces | Trace what happened; do not assume reasoning text is truth. |

### Bottom Line

Agent and CLI tracing should be part of the core thesis, not a later side quest.

Services explain runtime failures. CI and CLI traces explain bounded execution
failures. Agent traces explain how autonomous software work used evidence and
what result it produced. Together they create the feedback loop that can make
Parallax defensible.

## Agent and CLI OTel Semantic-Convention Mapping

_Provenance: merged verbatim from `agent-cli-otel-semconv-mapping.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

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

### Current Primary-Source Checks

| Source | What it shows | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | GenAI, MCP, CLI, process, CI/CD, VCS, exception, and test areas are present in the current semantic convention catalog. | Parallax can reuse OTel names for ingestion and adapter tests, but should not require every source to emit every convention. |
| [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/) | GenAI agent conventions are development-stage; existing instrumentations should not change emitted convention versions by default and may use `OTEL_SEMCONV_STABILITY_OPT_IN=gen_ai_latest_experimental`. They define operations such as `create_agent`, `invoke_agent`, `invoke_workflow`, and `execute_tool`. | Every normalized record must store `semconv_version` and `stability_opt_in`. Missing those fields means the adapter result is useful but not proof of stable interoperability. |
| [OpenTelemetry GenAI client spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/) | Client spans include provider/model attributes, token usage, finish reasons, error type, and opt-in content fields such as input/output messages, system instructions, and tool definitions. | Token/cost/status fields are safe defaults. Prompts, outputs, system instructions, and tool definitions remain raw refs or disabled unless explicitly enabled and redacted. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP instrumentation defines client/server spans, method names, JSON-RPC request IDs, tool/prompt/resource attributes, session metrics, and transport values. The page still marks MCP conventions development-stage and recommends `params._meta` trace context while saying official MCP guidance should win when available. | Parallax should propagate `traceparent` through MCP `_meta` when available, but must record whether the source is OTel recommendation, stable MCP, or draft/SEP-414 semantics and test it per client/server pair. |
| [MCP draft changelog](https://modelcontextprotocol.io/specification/draft/changelog) and [SEP index](https://modelcontextprotocol.io/seps) | Latest stable remains `2025-11-25`, but the draft documents `_meta` trace-context keys and larger protocol drift such as stateless/sessionless transport, discovery, subscriptions, deterministic/cacheable lists, roots/sampling/logging deprecation, and task extension movement. | Store observed MCP spec/protocol version separately from OTel semconv version. Do not let session metrics or draft-only fields become Parallax storage keys until the stable spec and client behavior catch up. |
| [OpenTelemetry CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | CLI spans model short-lived command execution and require executable name, exit code, PID, and `error.type` on non-zero exits. Command args should not be collected by default without sanitization. | `cli_invocation` should always store structural command identity and exit/error status; raw argv is denied by default. |
| [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/) | Process args, command line, interactive flag, cgroup, parent PID, and working directory are opt-in. The spec says to prefer `process.command_args` and fall back to `process.command` plus `process.args_count` when args cannot be safely collected. | Parallax should store command family and arg count even when args are redacted, preserving audit value without secret exposure. |
| [OpenTelemetry CI/CD spans](https://opentelemetry.io/docs/specs/semconv/cicd/cicd-spans/) | CI/CD spans model pipeline runs and task runs with results such as success, failure, timeout, skipped, cancellation, and error. | Agent-run tests and validation commands should normalize to the same result vocabulary as CI jobs. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tool errors are split between JSON-RPC protocol errors and tool-execution errors; tool-execution errors should be actionable for model self-correction. Servers must validate inputs, enforce access control, rate-limit, sanitize outputs, and clients should log tool usage for audit. | Parallax MCP tool calls need both protocol status and tool-execution status, and denied/sensitive calls must be audit rows rather than only span errors. |

### Decision

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

### Mapping Table

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

### Stability Rules

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

### Duplicate Suppression

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

### Content Capture Levels

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

### MCP Trace Propagation

Until the latest stable MCP spec and tested clients converge on trace-context
propagation:

- inject W3C `traceparent` and `tracestate` into MCP `params._meta` when the
  client/server supports it;
- extract `_meta.traceparent` as the remote parent for MCP server spans;
- link ambient context when the server already has one;
- never put secrets or authorization tokens into trace metadata;
- include `mcp_trace_context_source=otel_recommendation|mcp_stable|mcp_draft`
  and keep `mcp_trace_context_provisional=true` unless the current stable MCP
  spec and tested client/server pair both support the same behavior.

Gate:

| Check | Pass condition |
| --- | --- |
| Stdio tool call | A local stdio MCP tool call links client span, server span, and Parallax audit row by request id or fallback hash. |
| Streamable HTTP tool call | HTTP request span and MCP message span do not collapse into the same action incorrectly. |
| Missing `_meta` | Missing trace context produces `missing_traceparent`, not a fabricated parent. |
| Cross-client | At least Codex and Claude Code can call the Parallax MCP server and produce comparable audit rows. |

### Normalization Gates

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

### Product Implication

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

### Relationship To Other Research

- [Agent and CLI execution tracing](agent-cli-tracing.md) defines
  why services, CI, CLI tools, and coding agents belong in one execution graph.
- [Agent session tracing across real tools](agent-cli-tracing.md)
  defines the Codex, Claude Code, Amp, and OpenCode adapter gate.
- [Agent session tracing ledger](agent-cli-tracing.md) defines the
  result rows and claim levels for per-tool adapter coverage, lossiness,
  redaction, overhead, and audit-value comparisons.
- [Agent access surface: CLI, HTTP API, and MCP](../decisions/agent-access-surface.md)
  defines the read-only MCP context surface and projection-equivalence rule.
- [Agent access surface safety ledger](../decisions/agent-access-surface.md)
  owns claim levels for MCP client fixtures, audit spans, redaction, and
  projection-equivalence results.
- [Evidence bundle and open schema](../architecture/evidence-bundle-schema.md) defines the
  normalized nodes and audit edges that semconv-derived rows feed.
- [CLI trace overhead and redaction](agent-cli-tracing.md)
  owns the command args/env/stdout/stderr safety gate.
- [CLI trace safety ledger](agent-cli-tracing.md) owns the result rows and
  claim levels for default-ready CLI capture, redacted excerpts, raw refs,
  child-process policy, and projection safety.
- [OpenTelemetry protocol and context layer](otlp.md)
  owns the general OTLP receiver and Collector/Rotel compatibility story.

### Bottom Line

OTel GenAI, MCP, CLI, process, and CI/CD conventions are a strong signal that
agent/CLI execution telemetry is becoming standardizable. They are not yet
stable enough to be Parallax's product schema. The safe path is to ingest them,
record their versions, normalize them into Parallax action/session/invocation
rows, report every loss, and keep raw prompts, args, tool outputs, and MCP
resources behind explicit redaction and raw-ref policy.

## Agent Session Tracing Across Real Tools

_Provenance: merged verbatim from `agent-session-tracing-real-tools.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

This note closes proof gate 10 from
[Strategic verdict and research coverage](../decisions/strategic-coverage.md):

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
[agent session tracing ledger](agent-cli-tracing.md) defines the
result rows and claim levels required before this becomes product wording.

### Current Primary-Source Checks

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

### Adapter Strategy

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
[Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
so development-stage semantic conventions feed stable Parallax rows with
explicit lossiness reports.

### Adapter Coverage Clarifications

#### Codex Hooks Are Guardrail Events

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

#### Claude Code Surfaces Are Split Claims

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

#### Amp Plugins Are A Primary Adapter Surface

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

#### Observed Output Is Not State Verification

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

#### OpenCode Plugins Are Class-By-Class

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

### Normalized Session Schema

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

### Redaction Defaults

Agent sessions are more sensitive than CLI traces because they join prompts,
repo context, tool inputs, file contents, shell output, and MCP responses.

Default Parallax capture:

- prompt length, prompt hash, and prompt ref, not prompt body;
- tool name, input field names, input hash, and redaction report, not raw args;
- file paths by policy, patch hashes, and bounded redacted diffs only when
  enabled;
- shell commands through the same policy as
  [CLI trace overhead and redaction](agent-cli-tracing.md);
- full transcript/export/session JSON as raw refs only, with short TTL and audit;
- model reasoning/thinking content excluded unless the source tool exposes it
  and the project explicitly allows it.

Agent-visible bundles must pass the
[redaction pipeline](redaction.md) after normalization,
not only trust the source tool's built-in masking. Synthetic and evaluation
fixture runs must also carry a passing source-field policy row before any
projection is claimable. The safe projection is the canonical bundle JSON with
`schema_ref`, post-redaction `canonical_hash`, `projection_manifest`, and
`access`; Markdown, CLI, HTTP, and MCP content are projections, and MCP delivery
must use `structuredContent` validated against the bundle `outputSchema`.

### Value Evaluation Gate

The proof gate is not "can we ingest events?" It is "does normalized session
tracing answer audit and improvement questions better than raw transcripts or
no trace?"
Results and claim status belong in the
[agent session tracing ledger](agent-cli-tracing.md), not in this
design note.

#### Dataset

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

#### Comparison Arms

| Arm | Input to evaluator |
| --- | --- |
| Final output only | Commit/diff/summary and test result, no session trace. |
| Native transcript/export | Tool-native session artifact where available. |
| Parallax normalized session | Common schema only, redacted by policy. |
| Parallax linked evidence | Normalized session plus linked runtime/CI/CLI evidence bundle. |

#### Metrics

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

### Pass/Fail Gate

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

### Build Sequence

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

### Product Decision

Ship wording should be conservative:

- "Parallax normalizes supported coding-agent session events."
- "Parallax links agent actions to CLI, CI, runtime, and outcome evidence."
- "Parallax captures prompts, tool content, and transcripts as raw refs only
  when explicitly enabled."
- "Parallax does not depend on hidden reasoning traces."
- "Adapter coverage is reported per agent product and version."

This keeps the thesis strong without pretending every agent exposes the same
observability interface.

### Relationship To Other Research

- [Agent and CLI execution tracing](agent-cli-tracing.md) defines
  why coding-agent sessions are first-class execution evidence.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
  defines how native OTel GenAI/MCP/CLI spans become stable Parallax rows without
  treating development-stage conventions as the storage schema.
- [Agent session tracing ledger](agent-cli-tracing.md) turns this
  adapter strategy into a tool/version matrix, coverage/lossiness rows,
  redaction results, audit-value comparisons, and claim levels.
- [Agent observability technical review](../reference/agent-observability-review.md)
  surveys the broader LLM/agent observability market and technical patterns.
- [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md)
  defines the `agent_session`, `agent_action`, source-field policy status,
  redaction report, and audit edge targets.
- [CLI trace overhead and redaction](agent-cli-tracing.md)
  supplies the shell-command policy used inside agent sessions.
- [CLI trace safety ledger](agent-cli-tracing.md) supplies the claimable
  shell-command safety rows for agent-session runs that include CLI execution.
- [Redaction pipeline and secret safety](redaction.md)
  is the veto gate before agent-session evidence becomes agent-visible.
- [Bundle-value Phase 0 runbook](../validation/a1-bundle-value/bundle-value-phase0-runbook.md) is the nearest
  existing experiment shape for measuring whether better context improves agent
  output.

### Bottom Line

Agent-session tracing is technically feasible, but the unit of truth is a
normalized, redacted, lossiness-reported execution audit. Claude Code gives
Parallax the cleanest native OTel path. Codex gives hooks. Amp gives plugin
events plus streaming JSON. OpenCode gives run JSON, export JSON, plugins,
server/API, and ACP. Together they are enough to test whether Parallax session
traces improve audit and fix-quality loops, but not enough to claim perfect
replay, default raw transcript exposure, or universal raw transcript safety.

## Agent Session Tracing Ledger

_Provenance: merged verbatim from `agent-session-tracing-ledger.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

[Agent session tracing across real tools](agent-cli-tracing.md)
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
[agent access surface safety ledger](../decisions/agent-access-surface.md): that
ledger controls safe CLI/API/MCP context retrieval; this one controls ingestion
and normalization of agent execution traces.

### Current Source Snapshot

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

### Claim Levels

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

### Result Artifacts

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

### Run Manifest

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

### Row Schemas

#### Tool Matrix Row

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

#### Adapter Event Result Row

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

#### Normalized Session Result Row

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

#### Coverage Result Row

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

#### State Verification Result Row

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

#### Lossiness Result Row

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

#### Redaction Result Row

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

#### Source Field Policy Result Row

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

#### Projection Result Row

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

#### Audit Value Result Row

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

#### Claim Ledger Row

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

### Counting Rules

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

### Initial Results Template

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

### Product Wording

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

### Refresh Triggers

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

### Relationship To Other Research

- [Agent session tracing across real tools](agent-cli-tracing.md)
  defines the adapter strategy this ledger measures.
- [Agent and CLI execution tracing](agent-cli-tracing.md) explains
  why agent sessions belong in the execution graph.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
  defines how native OTel GenAI/MCP/CLI spans map into stable Parallax rows.
- [CLI trace overhead and redaction](agent-cli-tracing.md)
  supplies the shell-command policy used inside agent sessions.
- [CLI trace safety ledger](agent-cli-tracing.md) supplies the
  shell-command result rows, claim levels, and expiry rules consumed by
  agent-session runs.
- [A6 redaction red-team ledger](redaction.md) controls
  whether agent-session evidence can become agent-visible.
- [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md)
  defines the target `agent_session`, `agent_action`, source-field policy
  status, redaction report, and audit edges.
- [Fixer outcome ledger](../decisions/fixer-boundary.md) consumes linked agent-session
  rows when measuring fixer runs, PRs, checks, review, and recurrence outcomes.

### Bottom Line

Agent-session tracing should be measured like an adapter compatibility contract.
The first credible claim is not "trace every agent." It is a dated matrix showing
that at least two real tools emit enough structured events for Parallax to
normalize sessions, report lossiness, verify or mark state claims, preserve
redaction/source-field/projection safety across canonical CLI/API/MCP
projections, and improve audit
reconstruction over final outputs alone.

## CLI Trace Overhead And Redaction

_Provenance: merged verbatim from `cli-trace-overhead-and-redaction.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

This note closes proof gate 9 from
[Strategic verdict and research coverage](../decisions/strategic-coverage.md):

> CLI trace capture overhead and secret redaction for args, env, config, stdout,
> and stderr.

The answer is not "capture everything and redact later." CLI invocations are a
high-value Parallax surface because they are bounded, reproducible, and common
inside CI and coding-agent work. They are also one of the easiest places to leak
tokens, database URLs, config files, user paths, prompts, and generated output.

Decision: **CLI tracing is default-on only for structural capture.** Redacted
output excerpts require a separate proof gate. Full raw args, environment,
stdout, stderr, and config content are opt-in raw refs with scoped access,
audit, and retention limits.

The companion [CLI trace safety ledger](agent-cli-tracing.md) defines the
result artifacts, row schemas, claim levels, and expiry rules required before
these capture modes can become product claims.

### Current Primary-Source Checks

| Source | What matters for Parallax |
| --- | --- |
| [OpenTelemetry CLI semantic conventions](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | OTel has a CLI span shape for short-lived command execution, including exit status semantics. Parallax should map to this vocabulary where possible instead of inventing an isolated command schema. |
| [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/) | Process attributes include command, command line, and command args, but the spec warns that command args can contain sensitive information and should not be collected by default unless sanitized. This directly supports Parallax's structural-default policy. |
| [Rust `tracing`](https://docs.rs/tracing/latest/tracing/) | `tracing` gives Parallax the Rust-native span/event layer, but `#[instrument]` can record function arguments. CLI entry points and config loaders must skip raw structs and emit only sanitized fields. |
| [OpenTelemetry Rust SDK](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/) | Short-lived CLIs must handle exporter flush/shutdown explicitly. Any OTLP export path has to be measured because the last flush can dominate wall time for small commands. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) and [Actions secrets](https://docs.github.com/en/actions/concepts/security/secrets) | CI logs cannot be treated as safe. Masking depends on values being registered before output, and transformed values are not guaranteed to be masked. Parallax must scan CLI and CI output as hostile text. |
| [GitHub secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub maintains provider and generic secret-pattern categories. Parallax should use maintained pattern corpora for fixtures and canaries rather than relying on a small handmade regex list. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | Logs should exclude or protect passwords, access tokens, connection strings, encryption keys, sensitive PII, payment data, and data users opted out of collecting. CLI traces are logs with stronger reproducibility, so the same rule applies. |
| [Sentry sensitive-data docs](https://docs.sentry.io/platforms/javascript/guides/nextjs/data-management/sensitive-data/) | Sentry recommends source-side scrubbing so sensitive data does not leave the local environment. Parallax should apply the same principle to CLI wrappers before ingest. |

### Capture Modes

Parallax should expose four explicit CLI capture modes:

| Mode | Default? | Captured data |
| --- | --- | --- |
| `off` | No | No Parallax CLI trace. Useful for sensitive commands or debugging Parallax itself. |
| `structural` | Yes | Command identity, subcommand, exit code, duration, repo/branch/commit, policy IDs, output byte counts, redaction status, and safe field names. No raw arg/env/output values. |
| `redacted_excerpt` | Conditional | Everything in `structural`, plus bounded stdout/stderr excerpts after streaming redaction and final output scanning. This can become default only after the canary and overhead gates pass. |
| `raw_ref` | Never | Full args/env/config/stdout/stderr stored behind object refs, short TTL, audit, and scoped human access. Agent-visible bundles never dereference these by default. |

The product default should be:

```text
developer CLI wrapper: structural
Parallax-owned commands: structural, then redacted_excerpt after gate
CI command capture: structural unless project policy enables excerpts
coding-agent shell commands: structural; redacted_excerpt only after gate
raw output: opt-in raw_ref, never default
```

### Field Policy

| Surface | Default policy | Allowed by exception |
| --- | --- | --- |
| Command name | Store low-cardinality executable or logical command. | N/A |
| Subcommand | Store parsed low-cardinality subcommand path. | N/A |
| Args | Store arg keys, presence, value class, redaction rule, and optional HMAC hash for joinable safe values. Do not store `process.command_line` by default. | Allowlisted values such as enum modes, boolean flags, and bounded numeric values. |
| Environment | Deny values by default. Store allowlisted variable names and selected safe values such as `CI`, `GITHUB_SHA`, or `RUST_BACKTRACE` when policy allows. | HMAC hash values needed for joins. Never store `*_TOKEN`, `*_KEY`, `*_SECRET`, `PASSWORD`, cookies, auth headers, or connection strings. |
| Config | Store config file path policy, content hash, schema/version, and parse status. | Raw config only as `raw_ref` after opt-in. |
| CWD and repo path | Store repo root identity and relative path policy. Hash user-specific path fragments unless a project policy allows paths. | Human-visible path detail in self-hosted private projects. |
| stdout/stderr | Store byte count, line count, truncation status, exit-relevant excerpts only after scanner pass, and raw ref policy. | Bounded redacted excerpts in `redacted_excerpt`; full stream only in `raw_ref`. |
| Child processes | Apply the same command/arg/env/output policy recursively. | None. Child-process raw values are not safer than parent values. |
| Panic/error chain | Store Rust error type, panic location, sanitized message, stack/span trace refs, and redaction status. | Raw panic payload only if scanned and policy allows. |

Implementation rule: parse with structured APIs first. For Parallax-owned Rust
CLIs, use `clap` metadata to separate arg names from values. For shell-wrapped
commands, use a shell parser to identify argv tokens before detector passes.
String-splitting the full command line is not enough.

### Schema Additions

The existing `cli_invocation` node should add safety and performance fields
before CLI tracing is enabled by default:

```json
{
  "node_type": "cli_invocation",
  "capture_mode": "structural",
  "args_policy": "names-and-classes",
  "env_policy": "deny-values-allowlisted-names",
  "config_policy": "path-and-hash",
  "stdout_policy": "count-only",
  "stderr_policy": "count-only",
  "redaction_report_ref": "redact_123",
  "raw_ref_count": 0,
  "canary_scan_status": "pass",
  "overhead_budget": "structural-v1",
  "overhead_observed": {
    "wall_time_delta_ms_p95": 3,
    "rss_delta_mb_p95": 8,
    "flush_duration_ms_p95": 24
  }
}
```

This makes overhead and safety visible evidence, not a hidden implementation
claim.

### Redaction Canary Matrix

The CLI fixture suite must seed canaries across every capture surface:

| Surface | Canary examples |
| --- | --- |
| Args | `--token`, `--password`, `--api-key=...`, `-p ...`, bearer token flags, database URLs, URLs with credentials, JWT-like values, SSH key paths, repo paths with usernames. |
| Environment | `GITHUB_TOKEN`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `OPENAI_API_KEY`, `DATABASE_URL`, `COOKIE`, `SESSION`, `PASSWORD`, cloud-provider credentials, `.env` expansions. |
| Config | `.env`, TOML, YAML, JSON, npm/yarn/pnpm configs, Cargo credentials, cloud CLI configs, Docker auth files, kubeconfigs. |
| stdout | Echoed secrets, debug dumps, stack traces with env/config, test snapshot output, JSON logs, multiline private keys, base64-encoded secrets, URL-encoded secrets. |
| stderr | Compiler/test errors that include command lines, auth failures with URLs, panic messages, backtraces, child-process command echoes. |
| Child processes | Secret args/env/output from `git`, package managers, database CLIs, cloud CLIs, test runners, deploy scripts. |
| Agent-driven shell | User prompt fragments, MCP output, Parallax bundle excerpts, local file content printed by an agent command. |

The required acceptance criterion is zero known seeded canary leaks in
agent-visible JSON and Markdown renderings. A leak in either rendering fails the
gate. A detector error fails closed by stripping the affected field or blocking
the bundle.

### Overhead Benchmark Gate

Measure overhead against a no-Parallax baseline. Each benchmark run should
record wall time, CPU time, RSS, output throughput, flush/shutdown latency,
dropped spans/events, WAL bytes, redaction duration, false-positive redactions,
and canary leaks.

#### Capture Matrix

| Variant | Purpose |
| --- | --- |
| Baseline command | Measures command without Parallax wrapper or SDK. |
| `structural`, no exporter | Measures parsing and local span creation only. |
| `structural`, local WAL/outbox | Measures default tiny-mode durability. |
| `structural`, OTLP batch export | Measures exporter and shutdown path. |
| `redacted_excerpt`, small output | Measures normal developer and CI command shape. |
| `redacted_excerpt`, high output | Measures ring-buffer scanning under stdout/stderr pressure. |
| `raw_ref`, high output | Measures opt-in raw artifact write overhead. |
| Panic/error path | Measures Rust panic hook, error chain, backtrace/spantrace capture. |
| Child-process fanout | Measures per-child span creation and policy recursion. |

#### Workload Matrix

| Workload | Why it matters |
| --- | --- |
| Tiny 20 ms command | Flush and startup overhead dominate. This decides whether default capture feels intrusive. |
| 100 ms command | Common local helper command. Structural overhead should stay nearly invisible. |
| 1 s command | Typical build/test/setup phase. Percentage overhead matters more than absolute milliseconds. |
| High-output command | Test runners and package managers can emit large streams. Excerpt mode must not buffer unbounded text. |
| Many child processes | Build systems and scripts spawn many subprocesses. Policy recursion must stay bounded. |
| Rust panic | Parallax needs error-chain value without making failure paths significantly slower. |
| CI runner command | CI shells often echo commands and env. The canary and throughput gates both matter. |

#### Initial Budgets

These budgets are hypotheses to validate, not product claims:

| Mode | Budget |
| --- | --- |
| `structural` | p95 overhead <= 5 ms for commands under 100 ms, or <= 5 percent for commands at or above 100 ms. |
| `structural` with local WAL | p95 overhead <= 8 ms for commands under 100 ms, or <= 7 percent for commands at or above 100 ms. |
| `redacted_excerpt` | p95 overhead <= 10 ms for normal commands, or <= 10 percent for commands at or above 100 ms. |
| High-output excerpts | Throughput penalty <= 10 percent while storing bounded excerpts only. |
| `raw_ref` | No default budget because it is opt-in, but the benchmark must report throughput, storage bytes, and flush time. |
| Shutdown/flush | Local/WAL flush p95 <= 100 ms. Explicit OTLP flush p95 <= 500 ms. |
| Memory | RSS delta p95 <= 16 MB in `structural`; <= 32 MB in `redacted_excerpt`. |
| Safety | Zero seeded canary leaks in agent-visible JSON/Markdown. |

If these budgets fail for common commands, CLI tracing stays opt-in until the
implementation changes. If only OTLP shutdown fails, keep local WAL as the
default and export asynchronously from the Parallax service.

### Failure Criteria

The gate fails if any of these happen:

1. A seeded canary appears in agent-visible JSON or Markdown.
2. `process.command_line` or equivalent full command text is captured by
   default.
3. Environment values are captured outside the explicit allowlist.
4. A redaction scanner error lets a field through instead of stripping or
   blocking it.
5. `redacted_excerpt` buffers unbounded stdout/stderr in memory.
6. Structural mode exceeds the overhead budget on common commands.
7. Exit-time export routinely causes user-visible lag or data loss.
8. Child-process capture bypasses parent redaction policy.

### Product Decision

CLI tracing is valuable enough to keep in the core thesis, but the product
wording must stay precise:

- "Parallax traces CLI invocations structurally by default."
- "Parallax can store redacted output excerpts after the project passes the
  redaction and overhead gates."
- "Parallax can retain raw command/output refs only when explicitly enabled."
- "Parallax does not claim full command capture is safe by default."

This protects the strongest wedge: reproducible local, CI, and agent-invoked
failures without making secret exposure part of the default value proposition.

### Relationship To Other Research

- [Agent and CLI execution tracing](agent-cli-tracing.md) defines
  the strategic reason and first-pass CLI trace model.
- [Agent session tracing ledger](agent-cli-tracing.md) consumes
  command/edit coverage, redaction, and overhead rows for shell commands inside
  coding-agent sessions.
- [CLI trace safety ledger](agent-cli-tracing.md) turns this proof gate
  into auditable workload runs, claim levels, projection checks, raw-ref policy
  rows, and expiry triggers.
- [Redaction pipeline and secret safety](redaction.md)
  defines the global default-deny redaction pipeline and red-team gate.
- [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md)
  must carry `redaction_report`, raw refs, and the CLI node fields used here.
- [Technical implementation concept](../architecture/implementation-concept.md)
  should treat this note as the CLI default-on safety gate.
- [Bundle-value Phase 0 runbook](../validation/a1-bundle-value/bundle-value-phase0-runbook.md) should not use
  raw CLI output in agent arms unless this gate has passed for that fixture set.

### Bottom Line

Default CLI trace capture should be structural, low-overhead, and
default-deny. Redacted excerpts are earned by tests. Raw command/output content
is never the default. This keeps CLI tracing useful for Parallax's evidence
graph without turning the CLI into the easiest path for secret leakage.

## CLI Trace Safety Ledger

_Provenance: merged verbatim from `cli-trace-safety-ledger.md` (2026-05-29 restructure)._

Research date: 2026-05-25

### Purpose

[CLI trace overhead and redaction](agent-cli-tracing.md)
defines the proof gate for default-on CLI tracing. This ledger defines the
result artifacts, row schemas, claim levels, and expiry rules required before
Parallax can say CLI tracing is safe, low-overhead, or default-ready.

Current status: **not measured**. The repository has a capture policy and
benchmark design, but no dated run artifacts. Until those results exist,
Parallax should describe CLI tracing as designed but not run-proven.

The central rule:

> No "default-on CLI tracing", "redacted CLI excerpts are safe", or
> "low-overhead CLI tracing" claim without dated workload runs covering
> structural capture, redacted excerpts, raw refs, stdout/stderr, args, env,
> config, child processes, panic/error paths, CI shells, redaction canaries,
> flush/shutdown cost, and agent-visible projections.

This ledger is deliberately narrower than the
[agent session tracing ledger](agent-cli-tracing.md): this one owns
CLI invocation safety. Agent-session runs consume these rows for shell commands.

### Current Source Snapshot

| Source | Current check | Parallax implication |
| --- | --- | --- |
| [OpenTelemetry semantic conventions 1.41.0](https://opentelemetry.io/docs/specs/semconv/) | CLI, process, CI/CD, exception, GenAI, and MCP areas are present in the current semantic-convention catalog. | Store `semconv_version` on every CLI-derived result and expire claims when relevant conventions change. |
| [OpenTelemetry CLI spans](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/) | CLI spans are development-stage and model short-lived command execution. Executable name, exit code, and PID are required; `error.type` is required on non-zero exit. Command args are recommended by the CLI convention but should not be default-collected without sanitization. | Structural CLI rows can align with OTel, but raw argv cannot become a default product claim. |
| [OpenTelemetry process resources](https://opentelemetry.io/docs/specs/semconv/resource/process/) | `process.command_args`, `process.command_line`, working directory, parent PID, and related fields are opt-in. The docs prefer `process.command_args` over command-line strings and suggest `process.command` plus `process.args_count` when privacy blocks args. | `process.command_line` default capture is a ledger failure. Arg count and command family preserve audit value when values are denied. |
| [`tracing` `#[instrument]`](https://docs.rs/tracing/latest/tracing/attr.instrument.html) | By default, `#[instrument]` records all function arguments as span fields; arguments can be skipped explicitly. | Parallax-owned CLIs must skip raw config/args structs and emit sanitized fields intentionally. |
| [`opentelemetry_sdk` 0.32.0](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/) and [`BatchSpanProcessor`](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html) | The Rust SDK exposes batch processor queues, batch sizes, force flush, shutdown, and environment limits. Shutdown exports buffered spans and can block in some runtime shapes. | Short-lived CLI benchmarks must measure flush and shutdown separately from command work. |
| [GitHub Actions log masking](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#masking-a-value-in-a-log) | `add-mask` works per value per job and must be registered before a value is output. | CI shell output cannot be assumed safe because masking depends on timing and exact value registration. |
| [GitHub supported secret scanning patterns](https://docs.github.com/en/code-security/reference/secret-security/supported-secret-scanning-patterns) | GitHub documents generic, AI-detected, and provider-specific pattern categories, including private keys, bearer/basic auth, database connection strings, passwords, and hundreds of provider tokens. | CLI canary fixtures should use maintained pattern families and not only handmade regexes. |
| [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html) | Logs should exclude or protect access tokens, passwords, database connection strings, encryption keys, sensitive PII, payment data, and other sensitive values; logging systems should be tested for failure and resource exhaustion. | CLI traces are high-fidelity logs and need the same default-deny treatment plus overhead/failure tests. |
| [Sentry filtering docs](https://docs.sentry.io/platforms/javascript/configuration/filtering/) | Sentry recommends client-level filtering and `beforeSend` as the last chance to edit or drop data before it is sent. | Parallax CLI wrappers should scrub before ingest, not rely only on server-side cleanup after capture. |

### Claim Levels

Use these levels in `claim-ledger.jsonl`:

| Level | Meaning | Allowed wording |
| --- | --- | --- |
| `not_measured` | No current CLI safety run exists. | "CLI tracing policy is designed but not run-proven." |
| `fixture_harness_ready` | Repeatable CLI, CI, child-process, panic, and canary workloads exist. | "CLI trace safety fixture harness prepared." |
| `structural_capture_safe` | Structural mode captures command identity, timing, exit status, policy refs, counts, and hashes without raw args/env/config/output. | "Structural CLI capture omits raw args, env, config, stdout, and stderr for the tested harness." |
| `structural_overhead_pass` | Structural mode meets wall-time, CPU, RSS, WAL, and flush budgets on the workload matrix. | "Structural CLI tracing is low-overhead for the tested workloads." |
| `redacted_excerpt_redaction_pass` | Redacted stdout/stderr excerpts leak zero seeded canaries in JSON and Markdown projections. | "Redacted CLI excerpts pass seeded redaction tests for the tested workload matrix." |
| `redacted_excerpt_overhead_pass` | Redacted excerpt mode meets normal and high-output overhead budgets. | "Redacted CLI excerpts meet overhead budgets for the tested workloads." |
| `child_process_policy_pass` | Child process spans inherit parent arg/env/output policy and report violations. | "Child-process CLI tracing inherits the tested parent policy." |
| `panic_error_capture_pass` | Rust panic/error paths capture useful structural evidence without leaking seeded canaries or exceeding failure-path overhead budgets. | "Rust CLI panic/error evidence is structurally captured for the tested workloads." |
| `ci_command_capture_pass` | CI shell workloads pass structural capture, masking-hostile canaries, and high-output checks. | "CI command tracing passes the tested structural and redaction fixtures." |
| `raw_ref_policy_pass` | Raw refs have TTL, scope, audit rows, dereference controls, and agent-visible denial checks. | "Raw CLI artifacts are retained only through scoped audited refs in the tested policy." |
| `projection_pass` | Agent-visible JSON and Markdown projections include redaction reports and never dereference raw refs by default. | "CLI trace projections pass redaction and raw-ref policy checks." |
| `cli_trace_default_ready` | Structural capture is safe, low-overhead, projected safely, and child/CI/panic surfaces are either passed or explicitly out of advertised scope. | "Parallax traces CLI invocations structurally by default for the tested workload matrix." |
| `redacted_excerpt_default_ready` | Redacted excerpt capture passes redaction, overhead, high-output, projection, child-process, and CI checks. | "Parallax can enable redacted CLI excerpts by default for the tested workload matrix." |
| `claim_expired` | Semconv, SDK, shell/platform, capture mode, redaction policy, projection, or workload coverage changed, or 90 days passed. | "CLI trace safety result expired; rerun required." |
| `claim_failed` | A required gate fails for the advertised level. | No claim for the failed mode/surface. |

Initial Parallax level: `not_measured`.

### Result Artifacts

Create these only for real CLI safety runs:

```text
docs/research/cli-trace-safety-results.md
docs/research/cli-trace-safety-runs/<run_id>/manifest.json
docs/research/cli-trace-safety-runs/<run_id>/workload-matrix.jsonl
docs/research/cli-trace-safety-runs/<run_id>/field-policy-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/redaction-canary-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/overhead-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/stdout-stderr-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/child-process-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/panic-error-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/projection-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/raw-ref-policy-results.jsonl
docs/research/cli-trace-safety-runs/<run_id>/claim-ledger.jsonl
docs/research/cli-trace-safety-runs/<run_id>/hashes.sha256
```

Do not commit raw args, raw env values, raw config, raw stdout/stderr, or real
secrets. Raw refs in research runs should point to synthetic fixtures unless
the operator explicitly approves a private retained artifact.

### Run Manifest

```json
{
  "run_id": "cli-trace-safety-YYYYMMDD-N",
  "research_date": "YYYY-MM-DD",
  "parallax_cli_commit": "<git-sha>",
  "schema_version": "cli-trace-safety-v0",
  "semconv_version": "1.41.0",
  "rust_tracing_version": "0.1.x",
  "opentelemetry_sdk_version": "0.32.0",
  "redaction_policy_version": "a6-default-deny-vN",
  "capture_modes": ["structural", "redacted_excerpt", "raw_ref"],
  "platforms": ["linux", "macos"],
  "shells": ["bash", "zsh", "pwsh"],
  "projection_formats": ["json", "markdown"],
  "workload_count": 0,
  "budgets": {
    "structural_p95_ms_under_100ms": 5,
    "structural_p95_percent_at_or_above_100ms": 5,
    "redacted_excerpt_p95_percent": 10,
    "high_output_throughput_penalty_percent": 10,
    "local_flush_p95_ms": 100
  },
  "notes": []
}
```

### Row Schemas

#### Workload Matrix Row

```json
{
  "workload_id": "tiny-20ms-001",
  "command_class": "tiny_20ms|100ms|1s|high_output|many_children|rust_panic|ci_runner|agent_shell",
  "capture_mode": "off|structural|redacted_excerpt|raw_ref",
  "platform": "linux|macos|windows",
  "shell": "bash|zsh|pwsh|cmd|none",
  "baseline_command_hash": "sha256:<hex>",
  "expected_canary_count": 0,
  "expected_child_count": 0,
  "expected_output_bytes": 0,
  "notes": []
}
```

#### Field Policy Result Row

```json
{
  "workload_id": "ci-runner-001",
  "capture_mode": "structural",
  "command_policy": "name-and-family",
  "arg_policy": "names-classes-and-count",
  "env_policy": "deny-values-allowlisted-names",
  "config_policy": "path-hash-and-schema",
  "cwd_policy": "repo-relative-or-hash",
  "stdout_policy": "count-only",
  "stderr_policy": "count-only",
  "child_policy": "inherit-parent-policy",
  "default_denied_field_count": 0,
  "policy_violation_count": 0,
  "pass": true
}
```

#### Redaction Canary Result Row

```json
{
  "workload_id": "agent-shell-canary-001",
  "surface": "args|env|config|stdout|stderr|child_process|agent_shell",
  "capture_mode": "redacted_excerpt",
  "seeded_canaries": 20,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "raw_ref_leaks": 0,
  "scanner_error_count": 0,
  "scanner_error_behavior": "fail_closed|strip_field|block_bundle",
  "pass": true
}
```

#### Overhead Result Row

```json
{
  "workload_id": "tiny-20ms-001",
  "capture_mode": "structural",
  "baseline_p95_ms": 20,
  "observed_p95_ms": 23,
  "delta_p95_ms": 3,
  "delta_p95_percent": 15,
  "cpu_delta_p95_ms": 1,
  "rss_delta_p95_mb": 4,
  "flush_p95_ms": 18,
  "shutdown_p95_ms": 20,
  "dropped_event_count": 0,
  "wal_bytes_p95": 512,
  "budget_pass": true
}
```

For commands under 100 ms, absolute overhead decides the structural claim. For
commands at or above 100 ms, percentage overhead decides it.

#### Stdout/Stderr Result Row

```json
{
  "workload_id": "high-output-001",
  "capture_mode": "redacted_excerpt",
  "stream": "stdout|stderr",
  "output_bytes": 10000000,
  "excerpt_bytes": 8192,
  "bounded_buffer": true,
  "truncated": true,
  "throughput_penalty_percent": 7,
  "seeded_canary_leaks": 0,
  "pass": true
}
```

#### Child Process Result Row

```json
{
  "workload_id": "many-children-001",
  "parent_command_family": "cargo",
  "child_command_family": "rustc",
  "child_count": 24,
  "policy_inherited": true,
  "arg_violation_count": 0,
  "env_violation_count": 0,
  "output_violation_count": 0,
  "unobserved_child_count": 0,
  "pass": true
}
```

#### Panic/Error Result Row

```json
{
  "workload_id": "rust-panic-001",
  "capture_mode": "structural",
  "panic_or_error": "panic|error_chain|nonzero_exit",
  "sanitized_message_captured": true,
  "backtrace_ref_policy": "none|structural|raw_ref",
  "spantrace_ref_policy": "none|structural|raw_ref",
  "symbolication_ref_policy": "none|structural|raw_ref",
  "seeded_canary_leaks": 0,
  "failure_path_overhead_p95_ms": 0,
  "pass": true
}
```

#### Projection Result Row

```json
{
  "workload_id": "ci-runner-001",
  "projection_format": "json|markdown",
  "capture_mode": "structural",
  "redaction_report_present": true,
  "raw_ref_count": 0,
  "raw_ref_dereferenced": false,
  "json_projection_leaks": 0,
  "markdown_projection_leaks": 0,
  "agent_visible_pass": true
}
```

#### Raw Ref Policy Result Row

```json
{
  "workload_id": "raw-ref-001",
  "raw_ref_kind": "args|env|config|stdout|stderr|panic_payload",
  "ttl_hours": 24,
  "scope": "human_debug_only|project_admin_only|break_glass",
  "audit_row_created": true,
  "agent_bundle_dereference_blocked": true,
  "access_denial_test_pass": true,
  "pass": true
}
```

#### Claim Ledger Row

```json
{
  "claim_level": "structural_overhead_pass",
  "status": "pass|fail|expired",
  "run_id": "cli-trace-safety-YYYYMMDD-N",
  "mode": "structural",
  "scope": "linux-zsh-local-dev",
  "passed_workload_count": 0,
  "failed_workload_count": 0,
  "blocking_failures": [],
  "allowed_wording": "Structural CLI tracing is low-overhead for the tested workloads.",
  "expires_on": "YYYY-MM-DD"
}
```

### Counting Rules

- `structural` can become default only if it captures no raw args, env values,
  config content, stdout, stderr, or command-line strings.
- `structural` must still capture enough audit value: executable or command
  family, subcommand where safe, arg count/classes, repo/ref policy, start/end,
  exit status, duration, redaction policy, and projection refs.
- `process.command_line` default capture fails the gate.
- `process.command_args` default capture fails unless every value is sanitized
  before ingest and the sanitizer is part of the run.
- Env values are denied by default. Allowlisted env names can be visible; values
  require explicit policy and canary coverage.
- Child processes inherit parent policy. If a wrapper observes the parent but
  misses child args/env/output, the claim must say child coverage is missing.
- Scanner errors fail closed by stripping the field, blocking the bundle, or
  marking the result failed. They cannot pass the original field through.
- High-output workloads must prove bounded buffers. Unbounded stdout/stderr
  buffering fails even when redaction passes.
- `redacted_excerpt` cannot become default until canary redaction and
  high-output overhead pass for both JSON and Markdown projections.
- `raw_ref` is never a default capture mode and is never dereferenced into
  agent-visible bundles by default.
- Agent shell commands use this CLI policy plus the
  [agent session tracing ledger](agent-cli-tracing.md) redaction,
  lossiness, and audit-value rows.

### Product Wording

Allowed after `not_measured`:

> CLI tracing policy is designed but not run-proven.

Allowed after `structural_overhead_pass`:

> Structural CLI tracing is low-overhead for the tested workloads.

Allowed after `cli_trace_default_ready`:

> Parallax traces CLI invocations structurally by default for the tested
> workload matrix.

Allowed after `redacted_excerpt_default_ready`:

> Parallax can enable redacted CLI excerpts by default for the tested workload
> matrix.

Avoid:

- "full command capture is safe";
- "records stdout/stderr by default";
- "safe for all CLIs";
- "raw command replay by default";
- "zero overhead";
- "CI logs are safe after GitHub masking";
- "redaction is AI-safe" without naming the run and projection formats.

### Refresh Triggers

Mark affected claims `claim_expired` when:

- OpenTelemetry CLI/process/CI semantic conventions change materially;
- Rust `tracing`, OpenTelemetry Rust SDK, exporter, or shutdown behavior
  changes;
- the Parallax CLI wrapper, shell parser, capture mode, field policy, or
  projection format changes;
- the redaction policy, detector runtime, or external scanner corpus changes;
- a supported shell, platform, CI provider, or agent shell wrapper changes;
- GitHub Actions masking, secret scanning patterns, or CI log behavior changes
  materially;
- a new workload class becomes part of product wording;
- 90 days pass since the last run during active development.

### Relationship To Other Research

- [CLI trace overhead and redaction](agent-cli-tracing.md)
  defines the capture modes, field policy, canary matrix, and initial budgets
  this ledger turns into result rows.
- [Agent and CLI execution tracing](agent-cli-tracing.md)
  defines why CLI invocations belong in the execution graph.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-tracing.md)
  defines how OTel CLI/process/CI spans feed stable Parallax rows.
- [Agent session tracing ledger](agent-cli-tracing.md) consumes this
  ledger's shell-command safety results for coding-agent sessions.
- [A6 redaction red-team ledger](redaction.md) remains the
  broader redaction veto; this ledger specializes it for CLI surfaces and
  overhead budgets.
- [Redaction detector toolchain](redaction.md) defines the
  runtime and offline scanners used by CLI canary rows.
- [Evidence bundle and open schema specification](../architecture/evidence-bundle-schema.md)
  must expose CLI redaction reports, raw-ref policy, overhead evidence, and
  missing-evidence warnings without leaking denied fields.
- [Technical implementation concept](../architecture/implementation-concept.md)
  should treat this ledger as the claim boundary for default-on CLI tracing.

### Bottom Line

CLI tracing remains a strong Parallax wedge because commands are bounded,
reproducible, and directly connected to CI and coding-agent work. But the
default can only be structural until real rows prove safety and overhead. The
ledger makes that trade explicit: capture enough to reconstruct execution,
prove that secrets stay out of agent-visible projections, and require opt-in
audited refs for everything raw.
