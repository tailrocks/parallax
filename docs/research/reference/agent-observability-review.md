# Agent Observability Technical Review

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This note reviews agent-observability tools from a technical perspective and
extracts reference patterns for Parallax.

The prior note established why agent and CLI execution tracing matters. This
note asks how existing systems actually do it: instrumentation, trace model,
storage shape, evaluation loop, redaction, self-hosting, and technical gaps.

Version freshness rule: comparisons in this note use public docs checked on
2026-05-25. Future comparisons must use the latest reasonably available
stable/public version of each candidate as of the comparison date.

## Short Verdict

Agent observability has converged on one technical pattern:

```text
SDK / proxy / auto-instrumentation
  -> OpenTelemetry-like spans
  -> trace/session tree
  -> LLM / tool / retrieval / function spans
  -> UI for timeline, tree, graph, conversation
  -> evals, scores, feedback, datasets
```

This validates the category. It also leaves a clear Parallax gap:

```text
coding-agent session
  -> files read/written
  -> shell/CLI commands
  -> DB/API/deploy actions
  -> tests and CI
  -> patch/PR
  -> runtime issue fixed or worsened
  -> audit and question-answering over evidence
```

Most tools can show that an agent called a tool. Fewer can prove what happened
outside the LLM app boundary. Parallax should specialize in that boundary:
runtime evidence plus coding-agent and CLI side effects.

## Reference Tool Matrix

| Tool | Technical model | What to reuse | Parallax gap |
| --- | --- | --- | --- |
| LangSmith | LangChain/LangGraph-native tracing plus OpenTelemetry ingest for custom apps. Runs become hierarchical traces with model, tool, and decision steps. | Easy default instrumentation, strong framework-native trace tree, OTEL compatibility. | Not Rust/self-host-first; does not make repo file edits, shell commands, DB effects, and production error evidence one audit graph by default. |
| Langfuse | OpenTelemetry-based SDKs, traces/observations/sessions/scores, async ingest through S3/Redis/worker, ClickHouse for traces, Postgres for transactional state. | OTEL SDK foundation, async ingestion decoupling, score object for delayed eval/human feedback, masking hook before send. | Self-hosting is now a multi-service stack; focuses on LLM app traces more than coding-agent OS actions and Sentry/OTLP runtime evidence. |
| Arize Phoenix / OpenInference | Open-source collector/UI with OpenInference instrumentors exporting over OTLP HTTP. | OpenInference conventions and plugin ecosystem for LLM/tool/retrieval spans. | Great for AI app debugging/evals, but not a complete audit trail for coding-agent file, shell, deploy, and DB actions. |
| Braintrust | Trace viewer across logs, experiments, and human review. Span types include eval/task/LLM/tool-like application spans; traces tie directly to scores and datasets. | Treat traces as eval artifacts, not only debug artifacts. Promote failed traces into datasets. | Eval-centric more than runtime-observability-centric; does not own Sentry-compatible runtime events or CLI process traces. |
| Datadog LLM Observability | LLM spans inside broader Datadog APM/infra telemetry; SDK/API instrumentation; eval workflow included. | Incumbent lesson: agent traces are stronger when correlated with app and infra telemetry. | Closed commercial platform; not open Rust-first or productized around coding-agent audit. |
| Helicone Sessions | Proxy/request-level grouping with session headers and path hierarchy. Can group LLM calls, vector DB queries, tool calls, and arbitrary logged requests. | Very low-friction session model; path strings are useful for representing parent/child agent steps. | Header/proxy model cannot observe local shell/file/DB side effects unless the app explicitly logs them. |
| OpenLLMetry / Traceloop | OpenTelemetry extensions and SDK/instrumentors for LLM providers, vector DBs, and frameworks. | Reuse or interoperate with its OTEL instrumentors where possible. | Instrumentation layer, not Parallax's full evidence/audit product. |
| OpenLIT | OpenTelemetry-native SDKs, auto-instrumentation, MCP/GPU/vector DB support, and Controller using eBPF plus automatic SDK injection. | Zero-code and eBPF-assisted capture are useful for later broad coverage. | Broad AI engineering platform; Parallax should stay narrower around runtime failure, CLI command, agent action, and fix outcome graphs. |
| AgentOps | Agent-specific trace lifecycle, decorators for agents/tools/operations, OpenTelemetry status mapping, self-host docs. | Explicit `agent`, `tool`, and `operation` annotations are useful for a Parallax agent SDK. | Python/agent-app oriented; not a runtime/CLI/coding-agent audit store by itself. |
| Comet Opik | Open-source/local option with Python/TypeScript SDKs, OpenTelemetry, REST API, agent graph, tool call review, and offline evals. | Agent graph UI and local self-host path are relevant references. | Still mainly traces LLM/agent applications, not complete coding-agent execution over repos, commands, CI, and runtime incidents. |
| Langtrace | Open-source SDK and dashboard with OpenTelemetry-based traces exportable to other observability stacks. | Vendor-neutral trace export is the right interoperability posture. | Similar to OpenLLMetry: useful instrumentation reference, not the unified audit/evidence graph. |
| HoneyHive | Agent observability plus evaluation-driven development: traces, agent graphs, trajectories, threads, timeline, alerts, datasets, experiments. | Trajectory and thread views are strong UI references for long agent sessions. | Product scope is AI agent quality/evals, not Rust-first self-hosted runtime and CLI evidence. |

## Common Technical Pattern

Existing tools converge on these building blocks:

| Building block | Typical implementation | Parallax implication |
| --- | --- | --- |
| Trace root | One request, session, thread, eval case, or agent run. | Use one trace root per coding-agent task or CLI invocation. |
| Span hierarchy | Agent/workflow spans parent model/tool/retrieval/function spans. | Keep OTEL-compatible spans, but add Parallax-specific `agent.*`, `cli.*`, and `repo.*` events where semconv is missing. |
| Auto instrumentation | SDK wrappers, framework callbacks, decorators, proxies, or eBPF/controller injection. | Start with explicit Rust/Python/TS SDKs; add proxy/eBPF later only where it improves coverage. |
| Context propagation | OTEL context, session IDs, request headers, framework run IDs. | Propagate `trace_id`, `agent_session_id`, `cli_invocation_id`, `issue_id`, and `context_bundle_id`. |
| Data capture | Inputs, outputs, token usage, latency, model, provider, tool name, metadata, errors. | Capture hashes and redacted excerpts by default; never full prompts/args/stdout by default. |
| Eval link | Scores, feedback, datasets, experiments, annotation queues. | Store `fix_outcome`, `human_review`, `recurrence`, and `agent_quality_score` as first-class records. |
| Storage split | OLAP store for traces/spans; relational store for product metadata. | Keep GreptimeDB for high-volume evidence and Turso for sessions, issues, policies, outcomes. |
| UI | Timeline, tree, graph, conversation/thread, dashboards. | First UI should answer audit questions, not only render pretty spans. |

## Technical Lessons For Parallax

### 1. Use OpenTelemetry, But Do Not Wait For Perfect SemConv

OpenTelemetry already has development-stage GenAI agent spans, MCP conventions,
and CLI conventions. That is enough to choose OTLP-compatible trace transport and
standard span shapes.

But coding-agent execution needs fields that are not fully covered:

- files read and written;
- patch hashes and diff refs;
- shell command argv/env/cwd/exit;
- repo/branch/commit state;
- test/build command outcomes;
- user approval and policy decisions;
- DB/API/deploy action refs;
- final fix outcome.

Recommendation: emit normal OTEL spans plus a Parallax namespace:

```text
agent.product
agent.version
agent.session.id
agent.context.bundle_id
agent.action.kind
agent.action.approval_id
repo.path
repo.branch
repo.commit
cli.argv.redacted
cli.exit.code
parallax.evidence.refs
parallax.outcome.kind
```

### 2. Make Agent Audit Different From LLM Debugging

LLM debugging asks:

```text
Why did this model/tool step produce this answer?
```

Agent audit asks:

```text
What happened in the system, who or what caused it, and which action should
have been blocked, reviewed, or changed?
```

That means Parallax must store action provenance, not only span timing:

| Audit field | Why it matters |
| --- | --- |
| initiating user/workflow/token | Accountability and access review. |
| prompt/ticket/alert/issue anchor | Explains why the agent started. |
| context bundle refs | Shows what the agent knew. |
| tool/command/API/DB action | Shows what changed the world. |
| approval/policy decision | Shows whether controls worked. |
| file/patch/test refs | Shows software change path. |
| production/CI recurrence | Shows whether outcome improved. |

### 3. Treat CLI As The Concrete Side-Effect Layer

Most coding agents touch the world through CLI commands:

```text
git
cargo
npm
psql
kubectl
terraform
gh
custom deploy tools
```

So CLI tracing should be a first prototype primitive, not a later feature.

Minimum span model:

```text
cli.invocation
  -> cli.parse_args
  -> cli.load_config
  -> cli.subcommand.<name>
  -> process.spawn.<child>
  -> test/build/deploy span
  -> stdout/stderr events
  -> exit/error/panic event
```

Default capture policy:

| Data | Default |
| --- | --- |
| command name | capture |
| subcommand | capture |
| cwd/repo/branch/commit | capture |
| args | redact or hash unless allowlisted |
| env | deny by default; allowlist names only |
| stdout/stderr | bounded redacted excerpts |
| full output | artifact ref only with opt-in retention |
| exit code/signal | capture |
| Rust panic/error chain | capture |

### 4. Copy Langfuse's Async Ingestion Lesson, Not Its Initial Complexity

Langfuse's current self-host architecture validates a split:

```text
API receives trace batches
  -> object storage reference
  -> queue
  -> worker
  -> OLAP trace store
  -> relational product metadata
```

Parallax should copy that separation for durable mode, but not require it for
the tiny mode. The first build should be:

```text
parallax-server
  -> local WAL/outbox
  -> GreptimeDB
  -> Turso
```

Durable mode can add:

```text
object storage + Apache Iggy + workers
```

### 5. Add Scores, But Make Them Software Outcomes

Existing tools use scores for correctness, relevance, safety, and human review.
Parallax should use the same pattern, but grounded in engineering outcomes:

| Score/outcome | Source |
| --- | --- |
| `fix_accepted` | merged PR, accepted patch, human review. |
| `fix_modified` | human changed generated patch. |
| `fix_reverted` | revert commit, rollback, rejected PR. |
| `issue_recurs` | same fingerprint after release. |
| `tests_added` | patch diff and test command. |
| `evidence_cited` | agent output references evidence refs. |
| `unsafe_action_attempted` | policy/audit event. |

This is the feedback loop that makes traces improve future agent behavior.

### 6. Build Question-Answering Over Evidence, Not Free-Form Transcript Search

The user-facing promise should be operational questions:

```text
What did the agent do before this incident?
Which command changed this table?
Which deploy included this agent patch?
Which file edits happened between error first-seen and fix PR?
Which context item was stale?
Which policy check allowed this command?
```

Those answers require typed graph edges:

| Edge | Meaning |
| --- | --- |
| `agent_started_from` | Prompt, issue, alert, ticket, CI failure. |
| `agent_loaded_context` | Bundle, source file, trace, log, doc, issue. |
| `agent_executed_command` | CLI invocation span. |
| `command_touched_resource` | DB, file, network endpoint, deploy target. |
| `agent_changed_file` | Patch to repo file. |
| `patch_validated_by` | Test/build/lint command. |
| `patch_deployed_in` | Release/deploy marker. |
| `issue_recurred_after` | Regression after attempted fix. |
| `action_allowed_by_policy` | Approval/guardrail decision. |

## Reference Architecture

Prototype:

```text
Parallax SDK / wrapper
  -> OTLP spans and Parallax events
  -> parallax-server
       - DSN/auth
       - redaction
       - local WAL/outbox
       - trace/event normalizer
       - evidence graph builder
  -> GreptimeDB evidence tables
  -> Turso metadata/outcome tables
  -> API/MCP audit query tools
```

Durable:

```text
SDKs / CLI wrappers / agent plugins / OTEL collectors
  -> parallax-ingest
  -> object storage raw refs
  -> Apache Iggy stream
  -> workers
       - normalize
       - redact
       - correlate
       - build graph
       - compute outcomes
  -> GreptimeDB
  -> Turso or Postgres scale-out fallback
  -> API/MCP/UI
```

Optional later:

```text
eBPF / process boundary tracer
  -> process spawn, network, file, DB client hints
  -> correlate with agent/CLI trace IDs where possible
```

## MVP Technical Acceptance Criteria

The technical review suggests a stronger MVP than generic trace display:

1. A Rust CLI can call `parallax::cli::init()` and produce one trace with
   sanitized args, cwd, repo, commit, exit, stdout/stderr excerpts, and
   panic/error chain.
2. A coding-agent wrapper can record one session with context bundle, model/tool
   spans, files read, commands run, files edited, tests run, patch ref, and
   outcome. The real-tool adapter gate is specified in
   [Agent session tracing across real tools](agent-session-tracing-real-tools.md).
3. A Sentry-compatible runtime issue can link to the agent session that tried to
   fix it.
4. A query can answer: "what did the agent do before this failure or deploy?"
5. A query can answer: "which command/tool touched this resource?"
6. Redaction report is stored with every bundle/session/CLI invocation.
7. Full prompts, full command output, full args, and full diffs are opt-in only.
8. Data exports as OTLP-compatible traces plus Parallax evidence metadata.

## Bottom Line

Similar tools prove agent observability is real and technically converging on
OpenTelemetry-shaped traces plus evals. Parallax should use them as references,
but not copy the generic LLM-app observability product.

The defensible technical position is:

> Parallax observes autonomous software work end to end: runtime failure,
> evidence bundle, agent action, CLI side effect, patch, validation, deploy, and
> outcome.

That is the technical idea worth testing.

## Sources

- [LangSmith OpenTelemetry tracing](https://docs.langchain.com/langsmith/trace-with-opentelemetry)
- [LangSmith observability docs](https://docs.langchain.com/oss/python/langchain/observability)
- [Langfuse self-hosting](https://langfuse.com/self-hosting)
- [Langfuse platform architecture](https://langfuse.com/handbook/product-engineering/architecture)
- [Langfuse masking](https://langfuse.com/docs/observability/features/masking)
- [Langfuse scores](https://langfuse.com/docs/evaluation/scores/overview)
- [Arize Phoenix tracing architecture](https://arize.com/docs/phoenix/tracing/concepts-tracing/how-tracing-works)
- [OpenInference](https://arize-ai.github.io/openinference/)
- [Braintrust trace viewer](https://www.braintrust.dev/docs/observe/examine-traces)
- [Datadog LLM Observability](https://www.datadoghq.com/product/ai/llm-observability/)
- [Helicone sessions](https://docs.helicone.ai/features/sessions)
- [OpenLLMetry GitHub repository](https://github.com/traceloop/openllmetry)
- [OpenLIT docs](https://docs.openlit.io/)
- [AgentOps traces](https://docs.agentops.ai/v2/concepts/traces)
- [AgentOps tracking agents](https://docs.agentops.ai/v2/usage/tracking-agents)
- [Comet Opik tracing](https://www.comet.com/docs/opik/tracing/log_traces)
- [Langtrace docs](https://docs.langtrace.ai/introduction)
- [HoneyHive docs](https://docs.honeyhive.ai/)
- [OpenTelemetry GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)
- [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/)
- [OpenTelemetry CLI semantic conventions](https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/)
- [OpenTelemetry process resource conventions](https://opentelemetry.io/docs/specs/semconv/resource/process/)
