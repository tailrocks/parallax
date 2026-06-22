# TMA1 Deep Research

Research date: 2026-06-22

TMA1 (`github.com/tma1-ai/tma1`) is the **closest architectural cousin to Parallax** found in the
entire competitor sweep — a single Go binary that embeds GreptimeDB, ingests OTLP, and serves a
read-only MCP "bundle" to coding agents. This deep-dive tears the repo apart to answer the decisive
question: **is TMA1's "context bundle" the same thing as Parallax's evidence bundle?** Short answer:
**no** — same shape, fundamentally different artifact. The detail below both confirms TMA1 as the
reference competitor and *widens* Parallax's differentiation.

Grounded in primary sources (the repo's README, `docs/`, and Go source read via the GitHub contents
API) checked June 2026. Source paths cited inline.

## Sources

- [github.com/tma1-ai/tma1](https://github.com/tma1-ai/tma1) — repo, README, releases, file tree
- `tma1/docs/architecture.md`, `docs/mcp-tools.md`, `docs/anomalies.md`
- `tma1/server/internal/perception/bundle.go` — the `Bundle` struct + rendering
- `tma1/server/internal/perception/anomaly.go` — the 6 anomaly rules
- `tma1/server/internal/mcp/tools.go` — the 7 MCP tool definitions
- `tma1/server/internal/derive/derive.go` — ingest-time field derivation
- `tma1/server/internal/greptimedb/process.go` — child-process embedding
- `tma1/server/internal/install/install.go` — GreptimeDB download (minRequiredVersion v1.0.2)
- [github.com/tma1-ai](https://github.com/tma1-ai) (org: tma1, devtap, openfuse), [tma1.ai](https://tma1.ai/)
- [github.com/tma1-ai/openfuse](https://github.com/tma1-ai/openfuse) — Langfuse v3.184.1 fork

## What TMA1 Is

> "Local-first observability your agent reads back. TMA1 records every LLM call on your machine, then
> routes what it sees into the agent's next turn via hooks and MCP." Tagline: *"A monolith for your
> agent's loop. Silent until it talks back."* (Named after TMA-1, the *2001* monolith.)

- **Category:** **local AI-coding-agent observability + loop-feedback** — NOT production-services
  observability. The whole product is framed around the local agent loop (Claude Code, Codex, Copilot
  CLI, OpenClaw). Any OTel GenAI-SDK app can also send telemetry, but that is secondary.
- **License:** Apache-2.0 (same as Parallax).
- **Languages:** Go ~51% (server), JavaScript ~35% (vanilla-JS embedded dashboard, uPlot charts), Astro (site).
- **Maturity:** ~98 stars, 10 forks. 30 releases; latest **`v0.2.0-alpha7` (2026-06-08)**; **alpha/pre-1.0**;
  active commits through 2026-06-22.
- **Org:** `tma1-ai` ("Building things for Agents"); also ships `devtap` (Go, MIT, ~13★ — dev/build output
  → AI tools via MCP, predecessor to the build sensor) and `openfuse` (below).
- **Commercial product:** **none** — no pricing/cloud/SaaS/auth/multi-tenant. Explicitly a local-only tool.

## Architecture

Single binary **`tma1-server`** (Go, `CGO_ENABLED=0`). Subcommands: `mcp-serve`, `install`, `uninstall`,
`build`. Dashboard embedded via `embed.FS`, served from the same process.

### GreptimeDB embedding — child process, downloaded on first run

- **Run as a child process, not linked in.** `greptimedb.Start()` does `exec.Command(binPath, args...)`,
  pipes stdout/stderr, parents it to `tma1-server`, polls `http://localhost:14000/health` (≤30s).
- **Binary downloaded on first start** (not vendored): `install.EnsureGreptimeDB()` pulls the release
  tarball from `GreptimeTeam/greptimedb/releases`, verifies SHA-256, extracts to `~/.tma1/bin/greptime`.
  Version via `TMA1_GREPTIMEDB_VERSION` (default `latest`); **`minRequiredVersion = v1.0.2` enforced**.
- **GreptimeDB standalone** mode; data in `~/.tma1/data/`. Uses GreptimeDB **natively**: native OTLP tables,
  the **Flow engine** for continuous 1-minute aggregations (`tma1_cost_1m`, etc.), SQL via HTTP (14000),
  MySQL proto (14002), gRPC (14001); trace ingest injects `x-greptime-pipeline-name: greptime_trace_v1`;
  SQL uses GreptimeDB-isms (`json_get_string`, `matches_term()` FULLTEXT, `uddsketch` percentiles,
  SKIPPING/INVERTED/FULLTEXT indexes).

### No separate metadata store — the key architectural contrast

**TMA1 has NO Turso/SQLite metadata DB.** Everything — telemetry + `tma1_*` operational tables + the
anomaly emit log + a home-grown `tma1_schema_version` migration ledger — lives in the **one GreptimeDB
instance**. Local config is just `~/.tma1/settings.json`. Parallax, by contrast, pairs GreptimeDB with
**Turso** for projects/issues/users — a deliberate split TMA1 does not make.

### Other

- **Backpressure:** no external broker; internal `writeq.Sem` bounded write semaphore (default 64 in-flight)
  with drop + recovered-panic counters guarding GreptimeDB. (Parallax plans an **Apache Iggy** durable stream
  — TMA1 has only an in-process semaphore.)
- **Dashboard:** vanilla JS (no framework), uPlot charts, 8 views, embedded in the binary.
- **Install:** `curl -fsSL https://tma1.ai/install.sh | bash` (or PowerShell on Windows); optional
  `TMA1_ADAPTER=claude-code`. No brew, no `go install` advertised.

## Backend & Data Flow

See [backend-and-data-flow.md](backend-and-data-flow.md) for the cross-tool side-by-side. TMA1 summary —
**GreptimeDB is the only store** (no metadata DB, no broker):

```
 Coding agents (Claude Code / Codex / Copilot CLI / OpenClaw):
    OTel GenAI ──OTLP/HTTP :14318/v1/{traces,logs,metrics}──► tma1-server (OTLP reverse-proxy)
                                                                 │ inject x-greptime-pipeline-name
    Claude Code hooks ──POST /api/hooks (27 hook types)─────►   │ writeq.Sem (64 in-flight)
    JSONL transcripts (~/.claude, ~/.codex, ~/.copilot) ────►   ▼
    tma1-server build -- <cmd>  (build/dev sensor) ────────► GreptimeDB :14000 (standalone, ~/.tma1/data)
                                                                 │  native OTLP tables + tma1_* tables
                                                                 │  Flow engine 1-min rollups (cost, etc.)
    Dashboard (vanilla JS, embed.FS) ◄──SQL/HTTP──────────────┘
    MCP (tma1-server mcp-serve, stdio per session) ◄──SQL──────┘  connects to parent DB, never starts its own
```

- **Write:** mostly **raw OTLP stored as-is**, plus a thin `derive` step that lifts four helper columns
  (`tool_file_path`, `tool_command_prefix`, `tool_success`, `tool_error_summary`) from tool-call payloads
  via **regex** (inputs are often truncated). `PostToolUseFailure` → `success=false` + truncated error text.
  **No error fingerprinting, no grouping/dedup, no issue model.**
- **Read:** dashboard + MCP query GreptimeDB by SQL; anomalies via side-effect-free `DetectPreview`.
- **Multi-channel ingest:** beyond OTLP it ingests Claude Code hooks (`/api/hooks`, all 27 types →
  `tma1_hook_events`) and JSONL session transcripts for Claude/Codex/Copilot/OpenClaw. Agent ingest is
  **not purely OTLP**.
- **Throughput/footprint:** sparse — "a few hundred MB/month" disk (default `TMA1_DATA_TTL=60d`); **no RAM/CPU
  numbers**. `TMA1_QUERY_CONCURRENCY=4` exists "because GreptimeDB OOMs on 30d queries" → the embedded DB
  dominates resource use.
- **Designed for:** laptop-local agent-loop observability with instant agent-readable context. **Not for:**
  production-services scale, multi-tenant, or auth'd deployment (none exist).

## OTLP / Ingest

- **Endpoint:** `http://localhost:14318/v1/otlp` (+ `/v1/{traces,metrics,logs}`). `tma1-server` is itself an
  **OTLP reverse-proxy** on 14318 forwarding to GreptimeDB's OTLP on 14000. **HTTP only** at the proxy.
- **Signals:** traces, logs, metrics. **No OTel Collector required** (listed as a deliberate absence).
- **Sentry:** **none** — no envelope ingest anywhere.
- **Error derivation vs Parallax:** **much weaker.** The `derive` package extracts per-tool-call failure
  columns for *anomaly rules* — it is **not** production error-event derivation. No stack-signature
  fingerprinting, no error grouping/dedup, no issue lifecycle. Parallax's `error_event` derivation
  (exception span-events / span error status / ERROR-FATAL logs → fingerprint) has no TMA1 analogue.

## MCP / Agent Surface

**`tma1-server mcp-serve`** — JSON-RPC 2.0 over **stdio**, spawned per agent session; connects to the parent
server's GreptimeDB (does not start its own). Registered into `~/.claude.json` and `~/.codex/config.toml`.
**Strictly read-only** — pull tools use side-effect-free `DetectPreview`; there are **no write/mutate tools**.

| Tool | What it does |
|---|---|
| `get_context_bundle` | Aggregate entry point: project + session state (tool history, tokens, focus, recent files) + active anomalies + build status + recent external changes + project structure. "Same payload the UserPromptSubmit hook injects." Derives project from `os.Getwd()`. |
| `get_session_state` | One session's tool-history aggregates, tokens, focus, recent files; `verbose` adds a chronological actions array. |
| `get_anomalies` | Active anomalies for a session (side-effect-free preview). Per anomaly: `kind`, `severity`, `channel`, `evidence`, `suggestion`, `related_files`, `first_emitted_at`. |
| `get_build_status` | Last build/dev output from `tma1-server build -- <cmd>`: exit code, error count (30 min), last error, stale flag. |
| `get_external_changes` | Files changed outside the agent loop + git commits/branch moves, attributed `human`/`agent` (30-min window). |
| `get_project_state` | Indexed repo structure: language, build system, test framework, key files, top-level dirs (24h TTL). |
| `get_peer_sessions` | Recent sessions from *other* coding agents on the same project (cross-agent collaboration), self-excluding via `TMA1_MCP_CALLER`. |

### The "context bundle" — same name, different artifact

The bundle is the Go struct `perception.Bundle` (`bundle.go`):

```go
type Bundle struct {
    Project      string
    GeneratedAt  time.Time
    Session      *SessionState
    Anomalies    []Anomaly
    Build        *BuildStatus
    External     *ExternalChanges
    ProjectState *ProjectState
}
```

Rendered as indented JSON (MCP) or a compact `<tma1-context>` markdown digest bounded to ~500 tokens/2KB
(hook injection). Decisive properties:

- **NOT versioned.** No schema version, no content-addressing, no immutable artifact. It's a **live query
  response regenerated on every call** (`GeneratedAt: time.Now()`). The `Phase 0.x` comments are code-
  evolution phases, not a wire-schema version.
- **NOT redacted.** **Zero redaction layer.** It surfaces raw file paths, raw command prefixes, raw build
  error text (truncated for *size*, not for secrets). Redaction is not a concept in the codebase.
- **NOT portable evidence.** It is a perception snapshot of the *current local agent session* (to re-orient
  an agent after compaction) — not a redacted, signed, shareable package about a production incident.

So TMA1's "context bundle" = **ephemeral live session-context JSON for loop continuity**. Parallax's evidence
bundle = **redacted, versioned, portable evidence about production errors**. Same-sounding, different thing.

### Anomaly detection — heuristics, no ML

Six SQL+Go rules (`anomaly.go`): `stale_file_view` (edited a file whose last Read predates a human change),
`build_broken_after_my_edit`, `repeated_failed_build` (same cmd prefix failed 3+ in 30 min), `test_stuck`,
`human_modified_during_session`, `context_pressure` (input tokens ≥100k). A **10-min suppression window**,
per-rule **resolution checks** (re-Read resolves stale-view; a passing build resolves broken-build), and
three validation gates (precision ≥70%, ≤5 emits/kind/day, ≥30% action-follow-rate). Routed to channels:
`user_prompt_submit` (next-turn `<tma1-context>` prepend), `stop_block` (Stop hook blocks on HIGH only),
`post_tool_use` (reserved).

## Feature Inventory

8 dashboard views: per-agent (Claude Code / Codex / Copilot CLI / OpenClaw / OTel GenAI) with
Overview/Tools/Cost/Anomalies/Traces; **Sessions** (list + detail, Insights+Timeline, file heatmap, agent
hierarchy, waterfall, canvas animation, **Replay** + **Live SSE**); **Prompts** (heuristic scoring + optional
LLM-as-judge, verb grouping); **Anomalies** (cross-session). Plus full-text search across sessions/
conversations/tool calls; per-tool p50/p95 latency, call counts, success rates; **cost breakdown** (per-model
cost, burn-rate, cache-hit ratios — first-class); lightweight security monitoring (flags shell commands, URL
fetches, injected prompts, webhook errors). **No alerting/paging** — anomalies are in-loop nudges, not alerts.

Heavily **LLM-token/cost + agent-session focused**, not general production telemetry.

## OpenFuse (sibling)

A **fork of Langfuse (v3.184.1) that swaps the analytics store ClickHouse → GreptimeDB**: "the Langfuse
product, public APIs, and SDKs stay the same; GreptimeDB becomes the source of truth for traces, observations,
scores, and analytics." TS ~98.8%; **`v1.0.0-alpha.2` (2026-06-22)**; MIT core (`ee/` keeps Langfuse EE →
GitHub shows NOASSERTION). Storage layering: **Postgres** keeps app/config; **GreptimeDB** = append-only event
store (`raw_events` + projection tables + indexed EAV side-tables to emulate ClickHouse query patterns);
**Redis** runs BullMQ; object storage optional (local FS default). Read path **parity-checked byte-for-byte**
vs upstream (`docs/greptimedb-migration/parity/PARITY-REPORT.md`). Distributed as `tma1ai/openfuse-{web,worker,
standalone}`. **Same org, shared GreptimeDB thesis, separate product** — concrete proof GreptimeDB drops in
where ClickHouse was.

## Strengths and Gaps vs Parallax

| Capability | TMA1 | Parallax |
|---|---|---|
| Single binary, embedded GreptimeDB | ✅ (Go, child process) | ✅ (Rust) |
| OTLP-native ingest | ✅ traces/logs/metrics (HTTP proxy) | ✅ |
| Read-only MCP serving a "bundle" | ✅ (7 tools) | ✅ |
| Embedded dashboard | ✅ (vanilla JS) | ✅ |
| **Separate metadata store (Turso)** | ❌ GreptimeDB-only | ✅ Turso |
| **Durable stream / backpressure** | 🟡 in-process semaphore only | ✅ Apache Iggy planned |
| **Sentry envelope ingest** | ❌ | ✅ (future adapter) |
| **Derived `error_event` + fingerprinting (production)** | ❌ per-tool-call failure columns only; no grouping/issues | ✅ |
| **Redaction as a gate** | ❌ zero redaction; raw paths/commands/errors | ✅ (A6) |
| **Portable VERSIONED bundle schema** | ❌ live unversioned JSON snapshot | ✅ |
| **Fix-outcome loop (accepted/rejected/reverted)** | 🟡 anomaly *resolution* + follow-rate gate; no explicit state machine | ✅ |
| **CI / deploy capture** | ❌ (build ✅ local; CI/deploy ❌) | ✅ |
| Agent-session capture | ✅ rich (hooks + JSONL, 4 agents) | ✅ |
| Cross-agent peer sessions | ✅ `get_peer_sessions` | n/a |
| Production-services scope (auth/multi-tenant) | ❌ local-only | ✅ |

## Threat Assessment for Parallax

**Highest architectural overlap of any tool; low product-collision today — but the most important watch target.**

TMA1 proves the Parallax *shape* is buildable and already shipping: single binary + embedded GreptimeDB +
OTLP + read-only MCP "bundle" for coding agents. But it is a **narrower, dev-machine tool**. Tearing the repo
apart shows the spine of the Parallax product is absent:

1. **No separate metadata store** — GreptimeDB-only; no projects/issues/users model.
2. **No Sentry path.**
3. **No production error-event derivation / fingerprinting / issue model** — only per-tool-call failure columns.
4. **No redaction** — bundles ship raw paths, commands, and error text.
5. **No versioned/portable evidence artifact** — the "context bundle" is a live, regenerated JSON snapshot.
6. **No explicit fix-outcome ledger** — anomaly *resolution* is the closest analogue, not accepted/rejected/reverted.
7. **No CI/deploy capture; no auth/multi-tenant** — local-only.

The earlier ranked-analysis line "near-mirror" holds on *architecture* but should be read with this nuance:
**the bundle artifacts are different in kind**, which actually widens Parallax's differentiation rather than
narrowing it.

### Watch Triggers (high priority)

Re-evaluate immediately if TMA1:

- Adds a **separate metadata store** + projects/issues model (moves toward a platform).
- Adds **production error-event derivation + fingerprinting** (an issue model).
- Ships a **versioned, redacted, portable bundle artifact** (vs the live JSON snapshot).
- Adds an **explicit accepted/rejected/reverted outcome ledger**.
- Adds **Sentry ingest** or **CI/deploy capture**, or grows beyond the local-only/dev-machine scope.

Any two of these together = direct collision; track `tma1-ai/tma1` releases closely (alpha, fast-moving).

## Summary Verdict

TMA1 is the single most important competitor to watch because it independently arrived at Parallax's exact
architecture — Go single binary, embedded GreptimeDB (downloaded, run as a child process, native OTLP +
Flow engine), an OTLP reverse-proxy on `:14318`, a vanilla-JS embedded dashboard, and a **strictly read-only
7-tool MCP** that hands coding agents a `get_context_bundle`. It even has a sibling (OpenFuse) proving
GreptimeDB drops in where ClickHouse was. The GreptimeDB bet is validated; the shape is no longer unique.

But the resemblance is **architectural, not product**. TMA1 is local-only AI-coding-agent observability whose
"context bundle" is an ephemeral, unversioned, **unredacted** session snapshot for loop continuity — not a
redacted, versioned, portable evidence package about production errors. It has no metadata store, no Sentry
path, no fingerprinted error-issue model, no redaction gate, no real outcome ledger, and no CI/deploy capture.
Parallax's defensible delta is exactly that missing spine: **production-error derivation + Sentry-compat +
redaction-as-a-gate + versioned portable bundles + a fix-outcome loop + CI/deploy/agent capture**, on a Rust
binary with a Turso metadata split and an Iggy durable stream. Borrow TMA1's child-process GreptimeDB
embedding and its read-only MCP discipline; differentiate on everything downstream of ingest.
