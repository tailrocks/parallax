# Agent Access Surface — CLI, HTTP API, and MCP

> Parallax will build one canonical evidence-bundle contract (one redaction policy, one authorization model) and expose it through three transports: a stable HTTP API as the canonical service-to-service surface, a CLI shipped day one for human/CI/coding-agent use, and a read-only MCP adapter. The decision is **CLI first, HTTP API underneath, MCP required before any agent-native product claim** — MCP must not be the first or only interface because it remains security-sensitive (per NSA guidance), and no MCP tool may produce evidence unavailable through the canonical API or skip bundle size limits, redaction, authorization, or audit logging. The MCP adapter is gated: it ships only after projection-equivalence (CLI, HTTP, and MCP return the same canonical JSON hash) plus the read-only safety, client-fixture, scope, redaction, source-field, output-budget, audit, and negative-tool-catalog gates pass. Current status is **not measured**: there is a design but no implementation, projection-equivalence run, MCP client fixture, redaction run, or audit-log result, so Parallax must describe CLI/API/MCP as a planned design direction (claim level `not_measured`), not a proven safe agent surface. The safety ledger turns these fixtures into auditable claim levels with allowed product wording, run artifacts, row schemas, and expiry triggers.

This decision record consolidates the following previously-separate research files, each preserved in full below:

- `agent-access-surface-cli-api-mcp.md`
- `agent-access-surface-safety-ledger.md`

## Access Surface Decision — CLI, HTTP API, and MCP

_Provenance: merged verbatim from `agent-access-surface-cli-api-mcp.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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
The competitor pressure behind the read-only boundary is tracked in
[MCP power boundary competitor check](mcp-power-boundary-competitor-check.md).
The lightweight end of the same pressure is tracked in
[Lightweight error-tracker MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md).

### Current Primary Sources

| Source | What matters for Parallax |
| --- | --- |
| [MCP server overview](https://modelcontextprotocol.io/specification/2025-11-25/server/index) | MCP exposes prompts, resources, and tools; tools are model-controlled while resources are application-controlled. This maps cleanly to Parallax's split between bounded context resources and explicit investigation tools. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools have JSON Schema input and optional output schemas, support structured content, can advertise task-support metadata, and carry security requirements for input validation, access control, rate limits, output sanitization, user confirmation for sensitive operations, and audit logging. Bundle-returning Parallax tools should use `structuredContent` with an output schema, not text-only JSON, and should mark task support as forbidden for the first context adapter. |
| [MCP resources specification](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) | Resources are application-driven context with list/read APIs, optional subscriptions/list-changed behavior, annotations, and resource contents that can be text or binary. The spec says clients decide how to incorporate resource context and that sensitive resources need access controls. Parallax resource links are therefore not inert citations; `resources/read` must enforce the same redaction, scope, output-budget, and audit rules as tools. |
| [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization) | Authorization is optional overall, but HTTP-based transports that support it should follow the spec. The current spec requires protected-resource metadata, resource indicators in authorization and token requests, token audience validation by MCP servers, secure token storage, HTTPS, localhost/HTTPS redirect checks, PKCE for public clients, and no token passthrough. Stdio transports should not use this OAuth flow and should retrieve credentials from the environment. |
| [MCP draft changelog](https://modelcontextprotocol.io/specification/draft/changelog) and [SEP index](https://modelcontextprotocol.io/seps) | The official site still labels `2025-11-25` as latest, but the draft and final/accepted SEPs point to material protocol drift: stateless/sessionless Streamable HTTP, no `Mcp-Session-Id`, `server/discover`, `subscriptions/listen`, deterministic/cacheable list results, standard MCP request headers, `_meta` trace context, roots/sampling/logging deprecation, and a tasks extension outside core. Do not implement draft behavior as a stable requirement yet, but do make the shipping gate version-aware. Parallax MCP must avoid protocol-level session dependence and record which stable or draft semantics each fixture exercised. |
| [MCP 2026 roadmap](https://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/) | The March 2026 roadmap says the current spec release came out in November 2025 and no newer version has been cut. It sets 2026 priorities around transport scalability, agent communication, governance maturation, and enterprise readiness; "on the horizon" items include triggers/event-driven updates, streamed/reference-based result types, deeper security/authorization, and extensions. This confirms `2025-11-25` remains current while making protocol churn strategically relevant. Parallax should treat roadmap items as refresh triggers and fixture-design pressure, not as stable implementation requirements. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | The official guidance favors least-privilege scopes, targeted elevation, precise scope challenges, correlation IDs, and avoiding wildcard or omnibus scopes. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP calls should be observable as MCP-specific spans and metrics. The current OTel page still marks MCP conventions development-stage and recommends `_meta` trace context while saying official MCP guidance should take precedence when available. |
| [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp) | Codex, VS Code/Copilot Agent mode, Cursor, and Claude Code can consume MCP servers; OpenAI's own docs server uses MCP as the cross-client integration surface. |
| [Codex MCP](https://developers.openai.com/codex/mcp), [Codex MCP server guide](https://developers.openai.com/codex/guides/agents-sdk), [Codex config reference](https://developers.openai.com/codex/config-reference), and local `codex 0.133.0` help | Codex supports MCP in the CLI and IDE extension. Current docs put MCP config in `~/.codex/config.toml` or project-scoped `.codex/config.toml` for trusted projects; support stdio and Streamable HTTP; distinguish fixed `env`, whitelisted `env_vars` with local/remote source, remote stdio placement, bearer-token env vars, static and env HTTP headers, startup/tool timeouts, enabled/required servers, enabled/disabled tools, default and per-tool approval modes, OAuth resource/scopes/callback URL/port/credential store, granular `mcp_elicitations` approval, plugin-provided MCP servers, and memory controls such as `features.memories`, `memories.generate_memories`, `memories.use_memories`, and `memories.disable_on_external_context`. Codex can also run as a stdio MCP server exposing `codex` and `codex-reply` tools with approval-policy, sandbox, config, cwd, model, and profile controls. Local help confirms `codex mcp add` supports stdio `--env`, HTTP `--url`, `--bearer-token-env-var`, and `codex mcp-server --strict-config`. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) and local `claude mcp --help` on `2.1.150` | Claude Code supports local, project, user, plugin, claude.ai connector, and managed MCP sources. Current docs define source precedence, project `.mcp.json` approval, environment expansion in command/args/env/url/headers, OAuth callback/client credentials/metadata override/scope pinning, dynamic `headersHelper` commands gated by workspace trust, output warnings and limits, per-tool `_meta["anthropic/maxResultSizeChars"]`, resource `@` mentions that auto-fetch resources as attachments, tool-search deferral enabled by default, and `claude mcp serve`. Local help confirms `add` supports stdio/SSE/HTTP, headers, env vars, scope, client credentials, callback port, and warns that `mcp get`/`list` skip the workspace trust dialog and spawn stdio servers for health checks. |
| [NSA MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf?ver=bmgiSbNQLP6Z_GiWtRt6bg%3D%3D) | As of May 2026, NSA describes MCP as widely adopted but security-maturing, with risks around dynamic tool invocation, implicit trust, context sharing, serialization, token/session handling, overbroad tools, and unauthorized servers. |

Version note: the official MCP pages checked for this pass show
`2025-11-25` as the latest specification revision, and the March 2026 MCP
roadmap says no newer spec version has been cut since the November 2025 release.
Do not cite or implement a future-dated spec revision until the official site
publishes it as current. However, the draft changelog, SEP index, and roadmap
are now important watch inputs because the next revision could change
transport/session assumptions, discovery, result-reference behavior, enterprise
auth/gateway expectations, and `_meta` trace context. The OpenTelemetry
semantic-convention page checked in the same pass still shows `1.41.0`, with
MCP conventions marked development-stage; its examples still use
`mcp.protocol.version = "2025-06-18"`, so Parallax should record observed MCP
handshake/spec versions separately from the semconv example version.

### 2026-05-25 Access-Boundary Recheck

Current checks kept the CLI-first, MCP-later decision, but narrowed the product
claim:

- The official MCP site labels `2025-11-25` as latest. The tools spec still
  supports `structuredContent`, optional `outputSchema`, `tools/list_changed`,
  and tool-level `taskSupport` with `forbidden` as the default value.
- The 2026 roadmap confirms no newer stable spec release, but it also confirms
  that production MCP pressure is moving toward transport scalability, agent
  communication, governance delegation, enterprise audit/SSO/gateway/config
  portability, and possible reference-based result types. These are fixture
  refresh triggers, not Phase 1 feature scope.
- The official draft is not yet the stable spec, but it is strategically
  relevant: it removes protocol-level sessions, adds explicit discovery,
  changes change-notification shape, standardizes cache/list hints, deprecates
  roots/sampling/logging, moves tasks into an extension, and documents `_meta`
  trace propagation. A Parallax MCP adapter should be written and tested so a
  future spec bump changes transport glue, not the evidence-bundle contract.
- The authorization spec still makes remote MCP an auth/security project, not a
  simple bearer-token tunnel: protected-resource metadata, resource indicators,
  audience validation, PKCE S256, HTTPS/localhost redirect rules, and token
  passthrough denial remain fixture requirements.
- Local `codex-cli 0.133.0` and `Claude Code 2.1.150` still expose MCP client
  and MCP-server modes, but their configuration/trust surfaces differ enough
  that a cross-client claim needs explicit rows rather than a generic "MCP
  works" assertion.
- Claude Code's current docs say Tool Search is enabled by default and can defer
  MCP tools instead of loading every schema into the model context upfront.
  Cross-client fixtures therefore need a discovery row: Parallax tool names,
  descriptions, server instructions, and resource links must be findable through
  client-specific deferred discovery, not only present in a raw `tools/list`
  response.
- MCP resource links need their own safety row, not just a tool-output row. The
  current MCP resource spec lets clients decide how resources enter model
  context, and Claude Code documents `@` resource mentions that auto-fetch
  resources as attachments. Parallax must treat `resources/read` as an
  agent-visible projection path with scope, redaction, output-budget, and audit
  checks.
- Codex's current config surface adds two client-specific leak paths to record:
  `approval_policy.granular.mcp_elicitations` controls whether MCP elicitation
  prompts can surface, and `memories.disable_on_external_context` can keep
  threads that use MCP, web, or tool-search context out of memory generation.
  High-sensitivity Parallax fixtures need to record both rather than assuming
  tool output disappears after the turn.
- The lightweight competitor pass found Rustrak and GoSnag MCP surfaces in small
  error trackers. That falsifies any weak claim that "Parallax has MCP" is a
  moat. It does not falsify the read-only boundary because those checked
  surfaces are management/write/raw-event shaped rather than canonical,
  redacted, hash-equivalent evidence-bundle projections.

Implication: keep MCP out of the tiny-tier critical path, but do not defer it so
long that Parallax sounds non-agent-native. The claimable gap is the evidence
contract, not protocol support.

### Decision

Parallax should use this hierarchy:

| Layer | Role | Required when |
| --- | --- | --- |
| Canonical bundle builder | Builds the evidence bundle, redaction report, missing-evidence report, and evidence refs. | First. No transport should bypass it. |
| HTTP API | Stable service-to-service and UI integration surface. | Tiny tier. The CLI and MCP server call this or the same internal library. |
| CLI | Shell-native human, CI, and coding-agent surface. | Day one. Required for Phase 0/1 and for agents that already work through terminal commands. |
| MCP server | Agent-native discovery and typed tool/resource surface. | Before broad agent pilots, schema adoption claims, or "agent-native" product language. |

The practical answer is **CLI first, API underneath, MCP required before the
agent-facing product claim**.

### Why CLI-Only Is Not Enough

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

### Why MCP Is Required

MCP earns its place when Parallax wants agents to consume context without custom
per-agent glue.

What MCP adds:

- standard tool discovery through `tools/list`;
- client-specific deferred discovery and tool search when supported;
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

### Why MCP Must Not Be First

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

### First Surface Contract

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

### First CLI Commands

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

### First HTTP Endpoints

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

### First MCP Tools And Resources

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
- alert/dashboard/view/role/user/organization/pipeline/notification-channel
  create/update/delete tools
- incident/ticket creation or updates
- alert resolution, suppression, or notification-sending tools
- persisted RCA writes

Those are separate automation-control problems. Parallax's first MCP server is
for context retrieval and deterministic checks, not production mutation.

### Security And Observability Rules

| Rule | Requirement |
| --- | --- |
| Scope model | Use small read scopes first. No wildcard scope, no `admin`, no bundled future privileges. Emit precise scope challenges, accept down-scoped tokens, and audit elevation attempts. |
| Remote auth | Use HTTPS, protected-resource metadata, resource indicators in authorization and token requests, audience validation, secure token storage, short-lived tokens, PKCE where applicable, and token-passthrough denial. |
| Stdio/local auth | Treat local stdio MCP as local code execution. The OAuth authorization spec does not apply to stdio; require explicit install/trust, retrieve credentials only from approved local configuration or environment, never log them, and never auto-enable from a repo. |
| Tool schemas | Every tool has a closed JSON Schema input. Bundle-returning tools have an output schema for structured results. |
| Tool discovery | Tool names, descriptions, server instructions, resource names, and negative-tool catalog checks must pass both direct `tools/list` inspection and client-specific deferred discovery such as Claude Code Tool Search. |
| Resources | `resources/list`, `resources/read`, resource templates, and client-side resource attachment paths enforce the same scope, redaction, output-budget, source-field, and audit controls as tools. Raw refs are denied without `evidence:read_sensitive` even when a client auto-fetches or fuzzy-searches resources. |
| Protocol drift | Record stable spec version, observed protocol version, and any draft/SEP feature under test. Do not depend on protocol-level sessions or `Mcp-Session-Id`; use explicit, auditable state handles when state is unavoidable. |
| Server-initiated capabilities | Disable roots, sampling, elicitation, multi-round-trip (MRTR) input requests, and task-augmented execution in the first context server. Audit stable `tools/list_changed` behavior and draft-style `subscriptions/listen`/catalog changes separately. |
| Client retention | For clients that can store memories, persisted outputs, attachments, or local files from MCP context, record the setting and prove high-sensitivity Parallax evidence is excluded or redacted before persistence. |
| Output limits | Return bounded summaries plus resource refs; do not inline unbounded logs, traces, terminal output, or transcripts. |
| Redaction and source-field policy | Run the same redaction pipeline as CLI/API, and include `redaction_report.source_field_policy` in agent-visible responses. |
| Prompt injection | Treat telemetry, issues, PRs, logs, and transcripts as untrusted data; never let tool output redefine policy. |
| Audit | Emit an audit event and OpenTelemetry MCP span for every tool call, denied call, elevation request, and raw-ref access. |
| Trace context | Propagate W3C trace context through MCP `_meta` where supported, record whether it comes from OTel recommendation or draft/SEP-414 semantics, and link MCP spans back to bundle/evidence refs. |
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

### Implementation Order

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

### Gate For Shipping MCP

MCP should not ship until these tests pass:

| Gate | Pass condition |
| --- | --- |
| Projection equivalence | CLI, HTTP, and MCP return the same canonical JSON hash for the same request. |
| Client fixture | The same local server is callable from at least Codex and Claude Code using official MCP configuration paths, with each client's config source, server precedence, auth/header source, output-budget behavior, and local trust prompts recorded. |
| Tool-discovery fixture | Direct `tools/list`, resource listing, client tool search, deferred-tool loading, server instructions, and negative-tool absence are all recorded for each claimed client. |
| Resource-read fixture | `resources/list`, `resources/read`, resource templates, Claude Code `@` resource attachment, and raw-ref denial preserve redaction, scope, source-field policy, output bounds, and audit rows. |
| Scope fixture | Calls without `evidence:read` fail closed; raw refs require `evidence:read_sensitive`. |
| Remote auth fixture | Streamable HTTP proves protected-resource metadata, resource indicators in authorization and token requests, MCP-server audience validation, PKCE S256 for public clients, HTTPS policy, and token-passthrough denial. |
| Local stdio fixture | Stdio server startup requires explicit local trust, reads only approved credential sources, redacts credentials from logs/audit rows, and cannot be auto-enabled by a repository checkout. |
| Redaction fixture | Seeded secrets in logs, CLI output, agent transcripts, and frontend breadcrumbs do not appear in MCP output. |
| Source-field fixture | Eval/corpus-derived bundles preserve `source_field_policy.status = pass`, policy hash, and zero violations across CLI, HTTP, and MCP. |
| Output budget | Oversized bundles return summary + refs, not unbounded text, and remain within both Parallax's own budget and the tested client's MCP output behavior. |
| Audit fixture | Every MCP call emits an audit row and OpenTelemetry span with caller, tool, scopes, bundle id, status, and redaction policy. |
| Negative tool catalog | Generic shell, SQL, deploy, rollback, and delete tools are absent. |
| Management-tool catalog | Alert, dashboard, role, user, pipeline, notification, incident, ticket, saved-view, stream, sourcemap, and search-job create/update/delete tools are absent from the context server. |
| Protocol-drift fixture | Fixture records latest-stable spec, observed protocol version, no session-id dependence, deterministic/cacheable list behavior where supported, and explicit handling for draft-only features. |
| Capability fixture | Roots, sampling, elicitation, multi-round-trip (MRTR) input requests, task-augmented execution, and unreviewed catalog changes are denied or audited for the read-only context server. |
| Client-retention fixture | Codex memory settings, Claude persisted-file behavior, resource attachments, and any client-side output persistence are recorded; sensitive evidence is excluded from memory or persisted only as redacted bounded artifacts. |

If these fail, keep CLI/API available and do not claim MCP safety.

### Relationship To Other Research

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
- [MCP power boundary competitor check](mcp-power-boundary-competitor-check.md)
  records why competitor query/management MCP catalogs should not pull the first
  Parallax context adapter into production-control scope.
- [Lightweight error-tracker MCP boundary check](lightweight-error-tracker-mcp-boundary-check.md)
  records why Rustrak/GoSnag-style MCP availability at the lightweight end makes
  MCP table stakes rather than a moat.
- [Build roadmap and validation sequence](build-roadmap-and-validation-sequence.md)
  keeps MCP out of the tiny tier until the bundle and safety contracts are
  strong enough.

### Bottom Line

The CLI is the first path because it is simple, scriptable, and natural for
coding agents today. MCP is still required because Parallax's strategic surface
is agent-native context, and agents increasingly expect typed, discoverable
tools and resources. The safe design is not CLI versus MCP. It is canonical API
plus CLI first, then a narrow read-only MCP adapter with the same bundle,
redaction, authorization, and audit path.

## Safety Ledger — Claim Levels and Run Contract

_Provenance: merged verbatim from `agent-access-surface-safety-ledger.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

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

### Current Source Snapshot

| Source | Current check | Why it matters |
| --- | --- | --- |
| [MCP server overview](https://modelcontextprotocol.io/specification/2025-11-25/server/index) | MCP servers expose prompts, resources, and tools, with tools as model-controlled operations and resources as application-controlled context. | Parallax should expose evidence bundles as resources and narrow tools, not generic automation power. |
| [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tools are model-controlled, should keep a human in the loop, use JSON Schema input, optional output schemas, structured content, annotations, error results, optional task-support metadata, and security requirements around validation, access control, rate limiting, sanitization, confirmation, and audit logging. | Every Parallax MCP tool needs a closed schema, bounded output, audit row, and explicit denial of task-augmented execution unless a later fixture proves it safe. |
| [MCP resources specification](https://modelcontextprotocol.io/specification/2025-11-25/server/resources) | Resources expose context through `resources/list`, `resources/read`, templates, optional change notifications/subscriptions, annotations, and text/binary contents. The spec leaves incorporation of resources into model context to host applications and explicitly calls for access controls on sensitive resources. | Parallax cannot treat resource links as harmless raw refs. `resources/read`, resource templates, and client attachment paths need the same redaction, scope, output-budget, source-field, and audit rows as tools. |
| [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization) | Authorization is optional for MCP overall; HTTP-based transports that support it should follow the spec, while stdio should retrieve credentials from the environment instead of using the HTTP authorization flow. Remote MCP uses OAuth-style authorization with protected-resource metadata, resource indicators in authorization and token requests, audience validation, HTTPS, redirects, PKCE, secure token handling, and explicit token-passthrough prohibitions. | Remote Parallax MCP cannot be a bearer-token side door into evidence; protected-resource metadata, resource indicators, audience, PKCE, and no-token-passthrough behavior need their own rows. Local stdio trust and credential-source behavior must be measured separately. |
| [MCP draft changelog](https://modelcontextprotocol.io/specification/draft/changelog) and [SEP index](https://modelcontextprotocol.io/seps) | Latest stable remains `2025-11-25`, but the draft and final/accepted SEPs show likely next-revision changes: sessionless/stateless transport, `server/discover`, `subscriptions/listen`, deterministic/cacheable lists, standard MCP request headers, `_meta` trace context, roots/sampling/logging deprecation, and tasks as an extension. | Access-surface fixtures must record stable-vs-draft semantics, avoid `Mcp-Session-Id` dependence, and deny or separately gate task, multi-round-trip (MRTR), and server-initiated features. |
| [MCP 2026 roadmap](https://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/) | Current stable remains the November 2025 release, but 2026 priorities include transport scalability, agent communication, governance maturation, enterprise audit/SSO/gateway/config portability, triggers/event-driven updates, streamed/reference-based results, deeper security and authorization, and extensions. | Claim freshness must track roadmap and SEP movement. Parallax should not hard-code current session/result assumptions into the safety matrix, and should add rows when reference-based results, DPoP, workload identity, or enterprise gateway guidance becomes stable. |
| [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices) | Official guidance emphasizes least privilege, precise scope challenges, resource indicators, token audience validation, correlation IDs, and avoiding broad scopes. | The first MCP server must start read-only and deny wildcard/admin scopes. |
| [OpenTelemetry MCP semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/mcp/) | MCP client/server spans, JSON-RPC request IDs, transport values, tool/resource/prompt attributes, session metrics, `elicitation/create`, `sampling/createMessage`, `notifications/tools/list_changed`, and `_meta` trace propagation are defined with development-stage status. The OTel page says to prioritize official MCP guidance if it lands. | MCP calls and server-initiated capability attempts must be observable and normalized into Parallax audit/action rows without treating development-stage semconv names as stable storage fields. |
| [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp) | OpenAI documents MCP as a docs integration surface for Codex and other agent clients. | Cross-client MCP is a distribution requirement, not a unique moat. |
| [Codex MCP](https://developers.openai.com/codex/mcp), [Codex MCP server guide](https://developers.openai.com/codex/guides/agents-sdk), [Codex config reference](https://developers.openai.com/codex/config-reference), and local `codex 0.133.0` help | Codex supports MCP in the CLI and IDE extension with shared `config.toml` configuration. Current docs cover user and trusted-project config paths, stdio and Streamable HTTP, fixed `env`, whitelisted `env_vars` with local/remote source, remote stdio placement, bearer-token env vars, static/env HTTP headers, startup/tool timeouts, enabled/required servers, enabled/disabled tools, default and per-tool approval modes, OAuth resource/scopes/callback URL/port/credential store, granular `approval_policy.granular.mcp_elicitations`, plugin-provided MCP servers, and memory controls including `features.memories`, `memories.generate_memories`, `memories.use_memories`, and `memories.disable_on_external_context`. Codex can also run as a stdio MCP server exposing `codex` and `codex-reply` tools with approval-policy, sandbox, config, cwd, model, and profile controls. Local help confirms `codex mcp add` and `codex mcp-server --strict-config` flags. | Codex client fixtures must record config path/trust, local versus remote env/header sources, OAuth resource/scope/callback behavior, tool approval policy, MCP elicitation behavior, external-context memory behavior, plugin origin, required-server startup behavior, and Codex-as-MCP-server topology. A Codex MCP success does not by itself prove a safe read-only Parallax context surface. |
| [Claude Code MCP docs](https://code.claude.com/docs/en/mcp) and local `claude mcp --help` on `2.1.150` | Claude Code supports local, project, user, plugin, claude.ai connector, and managed MCP sources with source precedence. Current docs define project `.mcp.json` approval, environment expansion in command/args/env/url/headers, OAuth callback/client credentials/metadata override/scope pinning, dynamic `headersHelper` commands gated by workspace trust, output warning and limit behavior, per-tool `_meta["anthropic/maxResultSizeChars"]`, resource `@` mentions that auto-fetch resources as attachments, Tool Search enabled by default with deferred tool loading, and `claude mcp serve`. Local help confirms stdio/SSE/HTTP, headers, env vars, scope, client credentials, callback port, and warns that `mcp get`/`list` skip the workspace trust dialog and spawn stdio servers for health checks. | Claude Code client fixtures must record configuration source, precedence, auth/header source, output-budget behavior, resource auto-fetch behavior, workspace trust, health-check side effects, deferred tool discovery/tool-search behavior, and whether Claude-as-MCP-server is in play. Cross-client MCP safety is not proven by a generic "Claude supports MCP" row or by raw `tools/list` availability alone. |
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

### Claim Levels

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

### Result Artifacts

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
docs/research/agent-access-surface-runs/<run_id>/resource-read-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/scope-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/auth-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/stdio-trust-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/redaction-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/source-field-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/output-budget-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/client-retention-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/negative-tool-catalog.json
docs/research/agent-access-surface-runs/<run_id>/audit-results.jsonl
docs/research/agent-access-surface-runs/<run_id>/otel-mcp-spans.jsonl
docs/research/agent-access-surface-runs/<run_id>/claim-ledger.jsonl
docs/research/agent-access-surface-runs/<run_id>/hashes.sha256
```

Do not create run directories for hypothetical data. Add them only when a real
fixture run exists.

### Run Manifest

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
    "mcp_roadmap_watch": "2026 roadmap checked; transport_scalability, agent_communication, enterprise_audit_sso_gateway_config, reference_based_results, security_authorization_extensions",
    "mcp_draft_watch": "draft_changelog_checked; sessionless/stateless, server/discover, subscriptions/listen, cacheable lists, trace_context_meta, tasks_extension",
    "otel_semconv": "1.41.0",
    "otel_mcp_semconv_status": "development",
    "otel_mcp_example_protocol_version": "2025-06-18",
    "codex_client": "<version-or-snapshot>",
    "codex_mcp_elicitations": "approval_policy.granular.mcp_elicitations checked",
    "codex_memory_external_context": "memories.disable_on_external_context checked",
    "claude_code_client": "<version-or-snapshot>",
    "claude_code_tool_search": "default_enabled_checked_2026-05-25",
    "claude_code_resource_auto_fetch": "resource @ mention attachment behavior checked",
    "lightweight_mcp_watch": "rustrak_gosnag_management_raw_event_mcp_checked_2026-05-25"
  },
  "surfaces": ["cli", "http", "mcp"],
  "clients": ["codex", "claude-code"],
  "transport_modes": ["stdio", "streamable-http"],
  "mcp_features_allowed": ["tools", "resources"],
  "mcp_features_denied": ["roots", "sampling", "elicitation", "mrtr_input_requests", "task-augmented execution for context tools", "mcp_tasks_extension"],
  "client_config_matrix": ["codex:stdio", "codex:streamable-http", "claude-code:stdio", "claude-code:http"],
  "notes": []
}
```

The manifest must separate protocol/spec versions, client versions, Parallax
bundle/redaction/auth/audit versions, and transport modes. A pass in one
combination does not carry over to another.

### Row Schemas

#### Projection Result Row

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

#### MCP Tool Catalog Row

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
  "server_instructions_present": true,
  "tool_search_discovery_passed": true,
  "deferred_tool_catalog_observed": false,
  "all_context_tools_read_only": true
}
```

#### MCP Client Configuration Row

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
  "tool_search_enabled": true,
  "tool_search_mode": "default|auto|disabled|unknown",
  "deferred_tool_catalog_observed": true,
  "tool_search_or_wait_tool_observed": "ToolSearch|WaitForMcpServers|none|unknown",
  "tool_result_persisted_to_disk": false,
  "resource_auto_fetch_observed": false,
  "resource_attachment_policy_checked": false,
  "external_context_memory_generation_disabled": null,
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
    "approval_policy_granular_mcp_elicitations": false,
    "features_memories": false,
    "memories_generate_memories": false,
    "memories_use_memories": false,
    "memories_disable_on_external_context": true,
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

#### Resource Read Result Row

```json
{
  "case_id": "bundle_resource_read",
  "client": "claude-code",
  "transport": "stdio|http",
  "resource_uri": "parallax://bundles/bundle_123",
  "resource_kind": "bundle|redacted_evidence|raw_ref|template",
  "principal": "agent-readonly",
  "required_scope": "evidence:read",
  "scope_granted": true,
  "resources_listed": true,
  "resources_read_called": true,
  "client_auto_fetch_path": "@-mention|autocomplete|direct_resources_read|none",
  "redaction_report_present": true,
  "source_field_policy_status": "pass|not_applicable",
  "canonical_hash_preserved": true,
  "inline_bytes": 24576,
  "resource_refs_returned": 0,
  "raw_ref_denied_without_sensitive_scope": true,
  "audit_row_emitted": true,
  "status": "pass"
}
```

Resource rows are mandatory because client behavior can turn a Parallax URI into
model-visible context without a `tools/call` response. For Claude Code, include
both direct `resources/read` behavior and `@` resource attachment behavior.
For Codex, record whether resource reads are exposed by the tested MCP client
path and whether external-context memory generation is disabled for the thread.

#### MCP Capability Result Row

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

#### Scope Result Row

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

#### Authorization Result Row

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

#### Local Stdio Trust Result Row

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

#### Redaction Result Row

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

#### Source Field Result Row

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

#### Output Budget Result Row

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

#### Client Retention Result Row

```json
{
  "case_id": "codex_memory_external_context_disabled",
  "client": "codex",
  "client_version": "0.133.0",
  "surface": "mcp",
  "evidence_sensitivity": "public|redacted|sensitive",
  "tool_result_persisted_to_disk": false,
  "resource_attachment_persisted": false,
  "features_memories": false,
  "memories_generate_memories": false,
  "memories_use_memories": false,
  "external_context_memory_generation_disabled": true,
  "persisted_artifact_redacted": true,
  "raw_ref_material_persisted": false,
  "status": "pass"
}
```

This row does not replace Parallax server-side redaction. It records whether the
tested client can retain MCP-derived evidence after the turn and whether
high-sensitivity context is excluded, redacted, or persisted only through scoped
local artifacts.

#### Audit Result Row

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

#### Claim Ledger Row

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

### Counting Rules

- No "agent-native MCP" claim without projection equivalence across CLI, HTTP,
  and MCP for the same authorized request.
- No MCP differentiation claim based on protocol presence alone. Competitors at
  both observability-suite and lightweight-error-tracker levels now expose MCP;
  Parallax claims must compare evidence-bundle shape, redaction/source-field
  proof, auditability, output bounds, and outcome loop.
- No schema-safe MCP claim unless bundle-returning tools have output schemas and
  valid `structuredContent`; Markdown/text alone is a projection.
- No resource-link safety claim unless `resources/list`, `resources/read`,
  resource templates, and client-specific resource attachment paths preserve
  scope, redaction, source-field policy, output bounds, canonical hashes, and
  audit rows. Claude Code `@` resource mentions count as an agent-visible path.
- No read-only context claim if sampling, elicitation, task-augmented execution,
  or unreviewed `tools/list_changed` behavior can expand what the server asks of
  the client or model during a context request.
- No read-only context claim if the adapter depends on protocol-level sessions
  or `Mcp-Session-Id`. Any cross-request state must use explicit server-minted
  handles that appear in audit rows and projection-equivalence fixtures.
- No draft-protocol claim unless `server/discover`, standard MCP request
  headers, deterministic/cacheable list results, `subscriptions/listen`, and
  `_meta` trace-context behavior are either tested or explicitly out of scope.
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
- No cross-client discovery claim unless each claimed client proves both raw
  catalog visibility and its normal deferred-discovery path. For Claude Code,
  record Tool Search mode, whether tools were deferred, whether server
  instructions and tool descriptions were sufficient to find the Parallax
  context tools, and whether forbidden tools stayed absent from the discoverable
  catalog.
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
- Codex client claims must record `approval_policy.granular.mcp_elicitations`.
  The first Parallax context server should not request elicitation; if a client
  can surface MCP elicitation prompts anyway, the fixture must prove they are
  denied or audited for read-only context workflows.
- Codex memory claims must record `features.memories`,
  `memories.generate_memories`, `memories.use_memories`, and
  `memories.disable_on_external_context`. High-sensitivity Parallax evidence
  should either run with memories disabled or with external-context memory
  generation disabled; otherwise the run cannot support a safe client-retention
  claim.
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
- Claude Code resource claims must record `@` resource autocomplete/fetch
  behavior, attachment size, whether resource contents are redacted before model
  visibility, and whether raw refs remain denied without
  `evidence:read_sensitive`.
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

### Refresh Triggers

Rerun the matrix and mark affected claims `claim_expired` when any of these
change:

- MCP specification, roadmap, SEP priorities, authorization guidance, or
  security guidance changes;
- MCP task-augmented execution, roots, sampling, elicitation,
  multi-round-trip input requests, list/subscription, session/state, or dynamic
  tool-catalog behavior changes;
- MCP reference-based result types, enterprise gateway/SSO/audit guidance,
  DPoP, workload identity, or metadata-discovery behavior become stable enough
  to affect fixture expectations;
- OpenTelemetry semantic conventions or MCP semantic conventions change;
- Codex, Claude Code, Cursor, VS Code/Copilot, or other claimed clients change
  MCP configuration, output limits, auth, resource behavior, tool-search
  behavior, or deferred-catalog behavior;
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

### Product Wording

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

### Relationship To Other Research

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

### Bottom Line

Parallax should ship CLI and HTTP first, then MCP as a thin read-only projection
of the same canonical bundle. The market already has MCP. The product claim
must be that Parallax's MCP surface is equivalent, bounded, redacted, audited,
source-field-safe, and narrow enough for coding agents to use safely.
