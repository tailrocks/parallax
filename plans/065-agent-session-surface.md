# Plan 065: Agent-session surface — `agentSession(runId)` resolver + gen_ai span renderer + tool timeline + token strip

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-core/src ui/src/routes/runs.\$runId.tsx ui/src/components/console`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: LOW (additive surface; renders empty without producer data)
- **Depends on**: advisor-plans/034 (playground execution-stack scenario —
  the ONLY producer of `invoke_agent`/`execute_tool` spans today; this plan
  is its paired consumer). advisor-plans/029 (story) is a sibling surface,
  not a dependency. Plan 058's `traceEvents` is available but not required.
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

Parallax's product thesis is execution observability — CLI/agent runs as
first-class objects — yet the UI renders agent telemetry as opaque attribute
bags: zero `gen_ai`/`mcp` awareness exists in `crates/` or `ui/src` (the only
"agent" hit is the issue page's copyable CLI handoff string). advisor-plans/
034 makes the playground emit an agent-session simulation (`invoke_agent`
root, `execute_tool`/shell children, `gen_ai.operation.name`, shared
`parallax.run.id`) and explicitly ships no consumer. This plan is that
consumer: a run-scoped agent view — the agent's tool timeline, per-step
durations/errors, and a token/cost strip when token attributes exist — the
research brief's agent lane, scoped to the stable-core semconv 034 uses.

## Current state

Verified at commit `ed5b10f`.

- No awareness anywhere: `rtk grep -rn "gen_ai\|invoke_agent\|execute_tool"
  crates/ ui/src` → no product-code hits. The issue-page "Agent handoff"
  card (`ui/src/routes/issues.$fingerprint.tsx` — copyable
  `parallax issue context <fingerprint>` string) is CLI handoff, not
  telemetry rendering.
- Run detail — `ui/src/routes/runs.$runId.tsx`: header (command, status,
  exit code), live SSE stream (`LiveStreamPanel`), traces list
  (`tracesByRun`), logs, metric strip, bundle preview. No agent section.
- Data access: `spans_by_run(run_id, limit)`
  (`crates/parallax-storage/src/greptime.rs:678-691` — trace-ids-from-logs
  subquery, LIMIT-capped at the resolver's `MAX_ROWS=500`,
  `lib.rs:1085`). Span attributes are JSON on the row.
- Producer contract (from advisor-plans/034's scope — verify against its
  landed commit): root span `invoke_agent`, children `execute_tool` /
  shell-command spans, attribute `gen_ai.operation.name`; token-usage
  attributes only if 034's sim emits them (STOP condition covers absence);
  everything shares `parallax.run.id`.
- Precedent for pure derivation: `parallax-core` modules (plan 051's
  `trace_analysis.rs`, plan 058's `span_events.rs`).
- UI conventions: domain sections built from spans (plan 059/060 pattern);
  `RelativeTime`, `HeatCell`, `StatCard` primitives available.

## Commands you will need

| Purpose | Command | Expected |
|---------|---------|----------|
| Rust | `rtk cargo build --workspace && rtk cargo clippy --workspace --all-targets && rtk cargo nextest run` | clean |
| UI (from `ui/`) | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `crates/parallax-core/src/agent_session.rs` (new — pure projection from
  `&[SpanRow]` to a session/step model)
- `crates/parallax-api/src/lib.rs` — `agentSession(runId)` resolver +
  objects
- `ui/src/components/console/agent-session.tsx` (new) +
  `ui/src/routes/runs.$runId.tsx` (mount, agent section between header and
  traces)
- Tests

**Out of scope** (do NOT touch):
- MCP semconv (`mcp.*`) — 034 defers emitting it; consuming it is a later
  additive step (named in Maintenance).
- Prompt/completion CONTENT rendering — content capture is opt-in/absent by
  producer design; structural steps only. Do not add content fields to the
  schema.
- Multi-agent grouping, sub-agent trees — 034 emits one agent; the model
  should not speculate.
- Issue-fingerprint clustering of agent failures — future.
- Story tab (029) — separate surface.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Pure projection — `agent_session.rs`

```rust
pub struct AgentStep {
    pub span_id: String,
    pub trace_id: String,
    pub kind: AgentStepKind,      // InvokeAgent | ExecuteTool | Shell | Other
    pub name: String,             // tool name / command / operation
    pub start_nanos: u128,
    pub duration_ns: u128,
    pub is_error: bool,
    pub gen_ai_operation: Option<String>,   // gen_ai.operation.name
    pub input_tokens: Option<i64>,          // gen_ai.usage.input_tokens
    pub output_tokens: Option<i64>,         // gen_ai.usage.output_tokens
}
pub struct AgentSession {
    pub root_span_id: Option<String>,       // the invoke_agent span
    pub steps: Vec<AgentStep>,              // time-ascending
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub error_count: usize,
}
pub fn project_agent_session(spans: &[SpanRow]) -> Option<AgentSession>
```

Classification: span name `invoke_agent` → root; `execute_tool` → tool step
(tool name from a `gen_ai.tool.name`-style attribute if present, else the
span name); spans whose attributes carry `process.command`-ish keys under
the invoke_agent subtree → Shell. Return `None` when no `invoke_agent` span
exists (the section then doesn't render). Token totals sum only present
values. **Before coding, read advisor-plans/034's landed producer code in
the playground repo for the exact span names/attribute keys and pin them as
consts at the top of the module with a comment naming the producer file.**

**Verify**: `rtk cargo nextest run -p parallax-core` — fixture tests:
session with 3 tool steps + 1 shell + 1 error; token summing with partial
presence; no-agent spans → `None`.

### Step 2: `agentSession(runId)` resolver

`lib.rs`: fetch `spans_by_run(&run_id, MAX_ROWS)` (same call/cap as
`tracesByRun`, `lib.rs:1083-1086`), run the projection, map to GraphQL
objects (`AgentSessionOut`/`AgentStepOut`, nanos as strings per the
`nanos_string` convention). Null when projection returns `None`.

**Verify**: `rtk cargo nextest run -p parallax-api` — in-memory test:
seeded agent spans under one run id → session with ordered steps; unrelated
run → null.

### Step 3: UI section

`agent-session.tsx`: mounted on `runs.$runId.tsx` when `agentSession` is
non-null:
- Strip: steps count, error count (rose when >0), total tokens in/out
  (hidden entirely when both totals are 0 — no fake zeros).
- Timeline list: one row per step — kind icon, name, `RelativeTime` start,
  duration with `HeatCell` against sibling durations, error badge; row
  links to `/traces/$traceId` (the step's trace).
- Loader: add `agentSession(runId: …) { … }` to the run-detail GraphQL
  document.

**Verify** (from `ui/`): `bun run typecheck && bun run lint && bun run test
&& bun run build` clean — component test renders steps + hides the token
strip at zero; run page without agent data renders no section.

### Step 4: Live pairing check

With advisor-plans/034's scenario run against a local `parallax serve`:
open the run → agent section shows the simulated tool steps. Record it (or
the blocked reason if 034 hasn't landed — in that case the fixture tests
carry the plan, and the README row for THIS plan notes "producer pending").

## Test plan

- Core fixtures (Step 1), API in-memory test (Step 2), component tests
  (Step 3) — model each on the nearest existing test named in plans
  058/059's test-plan sections (same harnesses).

## Done criteria

- [ ] Rust + UI gates all clean
- [ ] `agentSession(runId)` in the schema; null-safe; capped fetch reused
- [ ] Producer span-name/attribute consts pinned with a source comment
- [ ] Run page renders the section only for agent runs (tests)
- [ ] Live check recorded or explicitly "producer pending"
- [ ] `plans/README.md` status row updated

## STOP conditions

- advisor-plans/034 landed with different span names than
  `invoke_agent`/`execute_tool` — pin to what it ACTUALLY emits (read the
  playground commit); if its shape is fundamentally different (e.g. no root
  span), STOP and report a model mismatch.
- 034 emits no token attributes — expected; ship without the strip's data
  (it hides at zero) and note it. Do NOT fabricate token numbers.
- `spans_by_run`'s 500-span cap truncates a long agent session — surface
  honestly: if `spans.len() == MAX_ROWS`, set a `truncated: true` field on
  the session object (add it) rather than silently projecting a partial
  session.

## Maintenance notes

- MCP spans (`mcp.method.name` etc.) are the named next producer step after
  034 — `AgentStepKind` gains an `Mcp` variant then; keep the enum
  extensible.
- Multi-agent (034's deferred item) will need session grouping by agent
  identity — the `Option<AgentSession>` return becomes `Vec<AgentSession>`;
  keep the projection's entry point narrow so that change is contained.
- Reviewer: no prompt/completion content anywhere in schema or UI; token
  strip hides at zero; the 500-cap truncation flag must be honest.
