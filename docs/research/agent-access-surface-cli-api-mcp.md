# Agent Access Surface: CLI, HTTP API, And MCP

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The deep-research prompt asks whether Parallax should expose evidence only
through a CLI, or whether it also needs a dedicated MCP server. This note gives
the focused answer:

> Build the canonical context contract once, expose it through HTTP first, ship
> a CLI from the start, and add a read-only MCP adapter before claiming
> agent-native distribution. CLI-only is enough for Phase 0/1 validation, but it
> is not enough for the product direction.

The contract is not "three different products." It is one evidence-bundle JSON
object, one redaction policy, one authorization model, and three transports:
HTTP API, CLI, and MCP.

Results and product-claim status should be published through the
[Agent access surface safety ledger](agent-access-surface-safety-ledger.md),
not inferred from this design alone.

## Current Primary Sources

| Source | What matters for Parallax |
| --- | --- |
| [MCP server overview](https://modelcontextprotocol.io/specification/2025-11-25/server/index) | MCP exposes prompts, resources, and tools; tools are model-controlled while resources are application-controlled. This maps cleanly to Parallax's split between bounded context resources and explicit investigation tools. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools have JSON Schema input and optional output schemas, support structured content, can advertise task-support metadata, and carry security requirements for input validation, access control, rate limits, output sanitization, user confirmation for sensitive operations, and audit logging. Bundle-returning Parallax tools should use `structuredContent` with an output schema, not text-only JSON, and should mark task support as forbidden for the first context adapter. |
| [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization) | Authorization is optional overall, but HTTP-based transports that support it should follow the spec. The current spec requires protected-resource metadata, resource indicators in authorization and token requests, token audience validation by MCP servers, secure token storage, HTTPS, localhost/HTTPS redirect checks, PKCE for public clients, and no token passthrough. Stdio transports should not use this OAuth flow and should retrieve credentials from the environment. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | The official guidance favors least-privilege scopes, targeted elevation, precise scope challenges, correlation IDs, and avoiding wildcard or omnibus scopes. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP calls should be observable as MCP-specific spans and metrics; MCP does not yet define its own standard trace-context propagation, so instrumentation needs explicit propagation in message metadata. |
| [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp) | Codex, VS Code/Copilot Agent mode, Cursor, and Claude Code can consume MCP servers; OpenAI's own docs server uses MCP as the cross-client integration surface. |
| [Codex MCP](https://developers.openai.com/codex/mcp), [Codex MCP server guide](https://developers.openai.com/codex/guides/agents-sdk), [Codex config reference](https://developers.openai.com/codex/config-reference), and local `codex 0.133.0` help | Codex supports MCP in the CLI and IDE extension. Current docs put MCP config in `~/.codex/config.toml` or project-scoped `.codex/config.toml` for trusted projects; support stdio and Streamable HTTP; distinguish fixed `env`, whitelisted `env_vars` with local/remote source, remote stdio placement, bearer-token env vars, static and env HTTP headers, startup/tool timeouts, enabled/required servers, enabled/disabled tools, default and per-tool approval modes, OAuth resource/scopes/callback URL/port/credential store, and plugin-provided MCP servers. Codex can also run as a stdio MCP server exposing `codex` and `codex-reply` tools with approval-policy, sandbox, config, cwd, model, and profile controls. Local help confirms `codex mcp add` supports stdio `--env`, HTTP `--url`, `--bearer-token-env-var`, and `codex mcp-server --strict-config`. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) and local `claude mcp --help` on `2.1.150` | Claude Code supports local, project, user, plugin, claude.ai connector, and managed MCP sources. Current docs define source precedence, project `.mcp.json` approval, environment expansion in command/args/env/url/headers, OAuth callback/client credentials/metadata override/scope pinning, dynamic `headersHelper` commands gated by workspace trust, output warnings and limits, per-tool `_meta["anthropic/maxResultSizeChars"]`, and `claude mcp serve`. Local help confirms `add` supports stdio/SSE/HTTP, headers, env vars, scope, client credentials, callback port, and warns that `mcp get`/`list` skip the workspace trust dialog and spawn stdio servers for health checks. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | As of May 2026, NSA describes MCP as widely adopted but security-maturing, with risks around dynamic tool invocation, implicit trust, context sharing, serialization, token/session handling, overbroad tools, and unauthorized servers. |

Version note: the official MCP pages checked for this pass show
`2025-11-25` as the latest specification revision. Do not cite or implement a
future-dated spec revision until the official site publishes it as current. The
OpenTelemetry semantic-convention page checked in the same pass still shows
`1.41.0`, with MCP conventions marked development-stage.

## Decision

Parallax should use this hierarchy:

| Layer | Role | Required when |
| --- | --- | --- |
| Canonical bundle builder | Builds the evidence bundle, redaction report, missing-evidence report, and evidence refs. | First. No transport should bypass it. |
| HTTP API | Stable service-to-service and UI integration surface. | Tiny tier. The CLI and MCP server call this or the same internal library. |
| CLI | Shell-native human, CI, and coding-agent surface. | Day one. Required for Phase 0/1 and for agents that already work through terminal commands. |
| MCP server | Agent-native discovery and typed tool/resource surface. | Before broad agent pilots, schema adoption claims, or "agent-native" product language. |

The practical answer is **CLI first, API underneath, MCP required before the
agent-facing product claim**.

## Why CLI-Only Is Not Enough

The CLI is necessary but not sufficient.

Strengths:

- Works immediately in local dev, CI, SSH, and coding-agent shells.
- Easy to script, record, replay, and compare in the Phase 0 bundle eval.
- Avoids early MCP protocol churn while the tiny tier is still proving bundle
  value.
- Gives humans and agents the same visible command surface.

Limits:

- Agents do not reliably discover shell commands, arguments, and output schemas
  without hand-written instructions.
- Shell output encourages Markdown/text parsing unless the agent is forced to
  request JSON.
- Secrets can leak through argv, environment variables, cwd, config paths,
  stdout, and stderr.
- A shell command cannot express scoped tool permissions as cleanly as a typed
  agent tool catalog.
- CLI output can exceed context limits unless Parallax enforces transport-level
  budgets and raw refs.

CLI-only is acceptable for validating A1 bundle value because the experimental
question is whether the bundle beats raw telemetry. It is not acceptable for the
long-term agent-access surface because it keeps Parallax in the "bag of shell
commands" category.

## Why MCP Is Required

MCP earns its place when Parallax wants agents to consume context without custom
per-agent glue.

What MCP adds:

- standard tool discovery through `tools/list`;
- JSON Schema input contracts for each context tool;
- optional output schemas and structured tool results;
- resource links for raw or expanded evidence that should not be dumped inline;
- cross-client integration with Codex, Claude Code, Cursor, and VS Code/Copilot
  Agent mode;
- a natural place to express least-privilege read scopes;
- MCP-specific observability through OpenTelemetry semantic conventions.

The first MCP server should be a read-only context adapter, not an automation
control plane. It should make Parallax evidence easy for agents to request, but
it should not give agents generic system power. The first server should expose
tools and resources only; sampling, elicitation, and task-augmented execution are
future surfaces that need separate safety fixtures.

## Why MCP Must Not Be First

MCP is still too security-sensitive to be the first and only interface.

The NSA guidance is the important correction to the "MCP everywhere" instinct:
MCP's adoption is real, but safe deployment depends heavily on implementation
discipline. Dynamic tool invocation, implicit trust between components, large
shared context, permissive serialization, bearer-token handling, and local
server exposure are exactly the risks Parallax is trying to audit.

Therefore:

- The HTTP API remains the canonical integration surface.
- The CLI remains the deterministic test and fallback surface.
- MCP is an adapter over the same redacted bundle contract.
- No MCP tool may produce evidence unavailable through the canonical API.
- No MCP tool may skip bundle size limits, redaction, authorization, or audit
  logging.

This keeps MCP from becoming an unreviewed side door into production evidence.

## First Surface Contract

All three surfaces return the same canonical object:

```text
anchor -> bundle builder -> canonical JSON bundle
                      |-> CLI JSON/Markdown projection
                      |-> HTTP response
                      |-> MCP structuredContent + resource links
```

Required invariant:

> For the same principal, project, anchor, time window, redaction policy, and
> schema version, CLI, HTTP, and MCP must produce the same canonical JSON hash,
> including equivalent `redaction_report.source_field_policy` status.

Markdown is a projection. Raw logs, raw envelopes, source maps, terminal output,
and agent transcripts are refs unless a caller has explicit read-sensitive
permission.

## First CLI Commands

The CLI remains the first usable interface:

| Command | Output |
| --- | --- |
| `parallax issue context <issue_id> --format json` | Canonical bundle JSON. |
| `parallax issue context <issue_id> --format markdown` | Deterministic human/agent projection. |
| `parallax trace context <trace_id> --format json` | Trace-anchored bundle subset. |
| `parallax agent session show <session_id> --format json` | Sanitized agent timeline and evidence refs. |
| `parallax cli invocation show <invocation_id> --format json` | Sanitized command, output refs, side effects, and redaction report. |

CLI output should default to redacted JSON or bounded Markdown. Full raw output
requires an explicit flag plus a principal that has read-sensitive permission.

## First HTTP Endpoints

HTTP is the stable API and the cleanest implementation seam:

```text
GET /api/projects/:project/issues/:issue_id/context
GET /api/projects/:project/traces/:trace_id/context
GET /api/projects/:project/agent-sessions/:session_id
GET /api/projects/:project/cli-invocations/:invocation_id
POST /api/projects/:project/hypotheses/check
```

The HTTP API owns auth decisions, redaction decisions, query-window limits, and
canonical response hashes. CLI and MCP should share this code path or call it
directly.

## First MCP Tools And Resources

MCP should expose a small read-only set:

| MCP item | Type | Scope | Returns |
| --- | --- | --- | --- |
| `parallax_issue_context` | Tool | `evidence:read` | Canonical bundle JSON in `structuredContent`, bounded Markdown in text, raw refs as resources. |
| `parallax_trace_context` | Tool | `evidence:read` | Trace-anchored spans/logs/errors/metrics bundle subset. |
| `parallax_hypothesis_check` | Tool | `evidence:read` | Deterministic checks for one proposed cause; no code mutation. |
| `parallax_agent_session_show` | Tool | `agent_session:read` | Sanitized session timeline, tool calls, file/test refs, outcome refs. |
| `parallax_cli_invocation_show` | Tool | `cli_invocation:read` | Sanitized command record, child process refs, stdout/stderr refs, side-effect refs. |
| `parallax_raw_ref_read` | Tool | `evidence:read_sensitive` | Narrow, audited raw-ref fetch; disabled by default for third-party models. |
| `parallax://bundles/{bundle_id}` | Resource | `evidence:read` | Canonical stored bundle. |
| `parallax://evidence/{ref_id}` | Resource | varies | Redacted evidence object or denied raw object. |

Rejected in the context server:

- `run_shell`
- `run_sql`
- `kubectl`
- `ssh`
- `deploy`
- `rollback`
- `delete_data`
- generic database query tools

Those are separate automation-control problems. Parallax's first MCP server is
for context retrieval and deterministic checks, not production mutation.

## Security And Observability Rules

| Rule | Requirement |
| --- | --- |
| Scope model | Use small read scopes first. No wildcard scope, no `admin`, no bundled future privileges. Emit precise scope challenges, accept down-scoped tokens, and audit elevation attempts. |
| Remote auth | Use HTTPS, protected-resource metadata, resource indicators in authorization and token requests, audience validation, secure token storage, short-lived tokens, PKCE where applicable, and token-passthrough denial. |
| Stdio/local auth | Treat local stdio MCP as local code execution. The OAuth authorization spec does not apply to stdio; require explicit install/trust, retrieve credentials only from approved local configuration or environment, never log them, and never auto-enable from a repo. |
| Tool schemas | Every tool has a closed JSON Schema input. Bundle-returning tools have an output schema for structured results. |
| Server-initiated capabilities | Disable sampling, elicitation, and task-augmented execution in the first context server. Audit any `tools/list_changed` notification or dynamic catalog change. |
| Output limits | Return bounded summaries plus resource refs; do not inline unbounded logs, traces, terminal output, or transcripts. |
| Redaction and source-field policy | Run the same redaction pipeline as CLI/API, and include `redaction_report.source_field_policy` in agent-visible responses. |
| Prompt injection | Treat telemetry, issues, PRs, logs, and transcripts as untrusted data; never let tool output redefine policy. |
| Audit | Emit an audit event and OpenTelemetry MCP span for every tool call, denied call, elevation request, and raw-ref access. |
| Trace context | Propagate request trace context through MCP metadata where supported, and link MCP spans back to bundle/evidence refs. |
| Errors | Return structured, self-correctable errors for invalid windows, missing scopes, missing evidence, and oversized requests. |

MCP spans should be normalized through
[Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
so `tools/call` client/server spans, JSON-RPC request IDs, and provisional
`params._meta` trace context produce one audited `agent_action` rather than
double-counted tool activity.

Bundle-returning MCP tools should provide an `outputSchema` that references the
canonical evidence-bundle schema. The text content can mirror bounded Markdown
or serialized JSON for compatibility, but the claimable result is the
`structuredContent` object and its canonical hash. A text-only MCP response does
not satisfy the Parallax schema/adoption or source-field policy gates.

## Implementation Order

1. **Phase 0:** no product surface needed. Hand-built bundles can be passed as
   files to agents for A1 eval.
2. **Phase 1 tiny tier:** build the canonical bundle builder, HTTP context API,
   and CLI. This proves the evidence contract and simplicity claim.
3. **Phase 2:** add projection-equivalence tests across CLI and HTTP, plus the
   redaction and schema conformance gates.
4. **Phase 3:** add the read-only MCP adapter before broad agent pilots or public
   agent-native positioning.
5. **Later:** consider write/proposal MCP tools only for draft PR creation,
   never direct production mutation.

## Gate For Shipping MCP

MCP should not ship until these tests pass:

| Gate | Pass condition |
| --- | --- |
| Projection equivalence | CLI, HTTP, and MCP return the same canonical JSON hash for the same request. |
| Client fixture | The same local server is callable from at least Codex and Claude Code using official MCP configuration paths, with each client's config source, server precedence, auth/header source, output-budget behavior, and local trust prompts recorded. |
| Scope fixture | Calls without `evidence:read` fail closed; raw refs require `evidence:read_sensitive`. |
| Remote auth fixture | Streamable HTTP proves protected-resource metadata, resource indicators in authorization and token requests, MCP-server audience validation, PKCE S256 for public clients, HTTPS policy, and token-passthrough denial. |
| Local stdio fixture | Stdio server startup requires explicit local trust, reads only approved credential sources, redacts credentials from logs/audit rows, and cannot be auto-enabled by a repository checkout. |
| Redaction fixture | Seeded secrets in logs, CLI output, agent transcripts, and frontend breadcrumbs do not appear in MCP output. |
| Source-field fixture | Eval/corpus-derived bundles preserve `source_field_policy.status = pass`, policy hash, and zero violations across CLI, HTTP, and MCP. |
| Output budget | Oversized bundles return summary + refs, not unbounded text, and remain within both Parallax's own budget and the tested client's MCP output behavior. |
| Audit fixture | Every MCP call emits an audit row and OpenTelemetry span with caller, tool, scopes, bundle id, status, and redaction policy. |
| Negative tool catalog | Generic shell, SQL, deploy, rollback, and delete tools are absent. |
| Capability fixture | Sampling, elicitation, task-augmented execution, and unreviewed tool-list changes are denied or audited for the read-only context server. |

If these fail, keep CLI/API available and do not claim MCP safety.

## Relationship To Other Research

- [Technical implementation concept](technical-implementation-concept.md) owns
  the end-to-end architecture this decision plugs into.
- [Evidence bundle and open schema](evidence-bundle-and-schema.md) defines the
  canonical JSON object that every surface must emit.
- [Causal reconstruction and agent safety](causal-reconstruction-and-agent-safety.md)
  defines the safety posture and rejects generic production-control tools.
- [Redaction pipeline and secret safety](redaction-pipeline-and-secret-safety.md)
  remains the gate for every agent-visible surface.
- [Production database evidence access gate](production-database-evidence-access.md)
  is the reason generic SQL tools stay out of the context server.
- [Agent and CLI OTel semantic-convention mapping](agent-cli-otel-semconv-mapping.md)
  defines how MCP spans and tool calls become stable Parallax audit rows.
- [Agent access surface safety ledger](agent-access-surface-safety-ledger.md)
  turns projection-equivalence, client, scope, redaction, output-budget,
  negative-tool, and audit fixtures into claim levels.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps MCP out of the tiny tier until the bundle and safety contracts are
  strong enough.

## Bottom Line

The CLI is the first path because it is simple, scriptable, and natural for
coding agents today. MCP is still required because Parallax's strategic surface
is agent-native context, and agents increasingly expect typed, discoverable
tools and resources. The safe design is not CLI versus MCP. It is canonical API
plus CLI first, then a narrow read-only MCP adapter with the same bundle,
redaction, authorization, and audit path.
