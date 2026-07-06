# Plan 031: Derive a service dependency graph from span pairing and add an Ecosystem page

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-storage/src/adapter.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs ui/src/components/nav.ts ui/package.json`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED
- **Depends on**: advisor-plans/024 (depth/complexity guard) recommended before the
  graph resolver ships, since graph queries fan out
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The brief names the Ecosystem service graph as the single change that most
changes how the product feels, and its own suggested execution order puts it
second (right after inventory). It answers "who connects to whom, who replied,
how long, where is the problem?" — nodes = services, edges = calls with RED
metrics, edge drilldown = traces/logs for exactly that relationship. The audit
confirmed the raw material exists (spans carry `service`, `parent_span_id`,
`kind`) but there is **no** edge query, no materialization, and the UI has no
graph library. This plan builds the smallest honest version: a **trace-path**
edge derivation (client-span→server-span pairing within traces) exposed as one
resolver, rendered as a labelled graph, with the graph mode explicitly stated.

## Current state

- Span rows carry `service`, `parent_span_id`, `kind`
  (`SPAN_KIND_CLIENT`/`SPAN_KIND_SERVER`) — `greptime.rs:340-353`,
  `crates/parallax-storage/src/model.rs`.
- `spans_by_trace` (`adapter.rs:145`) returns all spans of one trace with
  parent/child + kind — enough to pair a server span to its parent client span
  **within a trace**.
- `traces_search` returns only one representative root span per trace
  (`adapter.rs:31-39`) — **not** enough for edges; a graph needs full span
  sets. So a live graph over an arbitrary window must fetch many traces' spans
  (N+1) — this plan bounds that with a window + trace cap and states the graph
  mode, rather than pretending to a full aggregate.
- `serviceList` (`lib.rs:933`) gives per-node stats (spans, errors, p95).
- No graph library in `ui/package.json` (only `recharts@3.8.0`); no DAG
  rendering anywhere. Nav config: `ui/src/components/nav.ts`.
- UI conventions: `ui/AGENTS.md` — shadcn on Base UI, `@tabler/icons-react`,
  one data path, add shadcn components via CLI only.
- Repo conventions: zero clippy warnings; cargo-nextest; Bun-only.

## Commands you will need

(Rust fmt/clippy/nextest at repo root; UI typecheck/lint/test/build from
`ui/`, as in prior plans.)

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs` (edge result type + method)
- `crates/parallax-storage/src/greptime.rs` and `memory.rs` (impl)
- `crates/parallax-api/src/lib.rs` (`serviceMap` resolver)
- `ui/src/routes/ecosystem.tsx` (new route)
- `ui/src/components/nav.ts` (nav entry)
- `ui/src/components/console/ecosystem-graph.tsx` (new component)
- possibly `ui/package.json` (one graph layout dep — see Step 4)
- test files

**Out of scope**:
- Topology **modes** beyond trace-path (one-hop/transitive/endpoint) — ship
  trace-path only, labelled; defer the others.
- The execution graph (CLI/daemon/container/agent) — needs playground
  telemetry (plan 034).
- A materialized `service_edges_minute` table — live derivation only; note the
  materialization as the scale follow-up.
- Edge drilldown to filtered traces beyond a simple "traces for this edge"
  link — keep drilldown minimal.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Edge result type + storage method (trace-path derivation)

In `adapter.rs`:

```rust
pub struct ServiceEdge {
    pub source: String,       // caller service
    pub target: String,       // callee service
    pub call_count: u64,
    pub error_count: u64,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

async fn service_map(
    &self,
    range: RangeInclusive<u128>,
    max_traces: usize,
) -> anyhow::Result<Vec<ServiceEdge>>;
```

Implement in `greptime.rs`: select spans in the window (bounded to
`max_traces` distinct traces to cap cost), then pair within each trace — for
each server span, find its parent span's service; emit an edge
`(parent.service → server.service)` when they differ, accumulating count,
errors (`span_status_code = 'STATUS_CODE_ERROR'`), and duration percentiles.
Prefer doing the pairing in SQL (a self-join on `parent_span_id = span_id`
within `trace_id`) so the aggregation stays in the engine; if the percentile
math is awkward in one query, fetch the paired durations and compute p50/p95
in Rust over the bounded set. **Bound the scan window and trace count** — do
not scan all history (the storage audit flagged `traces_search`'s unbounded
aggregate; do not repeat it here).

Implement in `memory.rs` by the same pairing over in-memory spans.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: `serviceMap` resolver

In `lib.rs`, add `ServiceEdge` + `ServiceNode` GraphQL objects and a
`serviceMap(fromNanos, toNanos, maxTraces?)` resolver returning nodes
(reuse `serviceList` stats for node RED) + edges. Enforce a `maxTraces` cap.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Storage + resolver tests

- Memory-store test: seed two traces `A(client)→B(server)` and
  `B(client)→C(server)`; assert `service_map` returns edges A→B and B→C with
  correct counts, and that an error span bumps `error_count`.
- Determinism + window bound: a span outside the window is excluded.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Graph rendering component (choose the lightest option)

The UI has no graph library. Two acceptable paths — pick the one that fits
`ui/AGENTS.md` and Bun:

- **(a) Dependency-free SVG layout:** compute a simple layered/left-right
  layout in TS (nodes by service, edges as SVG paths) inside
  `ecosystem-graph.tsx`. No new dependency; fine for the dozen-node local
  scale. **Preferred** — it avoids a framework commitment (design principle 9:
  no stack expansion by default).
- **(b) A single small layout lib** (e.g. `@dagrejs/dagre` for layout only,
  render with your own SVG). Only if (a)'s layout is inadequate. If you add a
  dep, use `bun add` (never npm/pnpm), and justify it in the commit.

Render nodes colored by health (reuse heat conventions), edges labelled with
rate/error/p95, click a node → service detail (`/services/$service`), click an
edge → a drawer listing recent traces for that edge (link to traces filtered
by the two services if the traces query supports it, else the source service).
**Label the active graph mode** ("trace-path") visibly, per the brief's rule
that each mode makes a different causality claim.

**Verify (from `ui/`)**: `rtk bun run typecheck`/`lint`/`build` → exit 0.

### Step 5: Route + nav

Add `ui/src/routes/ecosystem.tsx` (loader calls `serviceMap` over the URL
time range via the existing range convention; `validateSearch` with the range
schema). Add an "Ecosystem" entry to `primaryNav` in `ui/src/components/
nav.ts` beside Services/Traces. The route tree regenerates
(`routeTree.gen.ts` stays committed).

**Verify (from `ui/`)**: `rtk bun run build` → exit 0.

### Step 6: UI test

A vitest render test of `EcosystemGraph` with a fixture of nodes+edges:
assert nodes and edge labels render, the graph-mode label shows "trace-path",
and clicking a node/edge produces the expected link. Model on
`waterfall.test.tsx`.

**Verify (from `ui/`)**: `rtk bun run test` → all pass.

## Test plan

- Rust: `service_map` edge derivation, error counting, window bounding,
  determinism (Step 3).
- UI: graph render + interactivity + mode label (Step 6).
- Pattern: memory-store server tests; `waterfall.test.tsx`.

## Done criteria

- [ ] Rust: `fmt` no diff, `clippy -D warnings` exit 0, `nextest` exit 0 with
      new tests
- [ ] UI: `typecheck`/`lint`/`build`/`test` all exit 0 (from `ui/`)
- [ ] `serviceMap(...)` resolver returns nodes + edges; `maxTraces` cap
      enforced (grep + code inspection)
- [ ] Ecosystem route reachable from nav; graph shows nodes, labelled edges,
      and a visible "trace-path" mode label (asserted by UI test)
- [ ] Edge derivation is window-bounded (no all-history scan) — code
      inspection
- [ ] If a dep was added, it went through `bun add` and `bun.lock` is the only
      lockfile changed (no `package-lock.json`/`pnpm-lock.yaml`)
- [ ] No out-of-scope files modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- The in-trace self-join to pair client/server spans is not expressible
  against GreptimeDB SQL and the Rust-side pairing over a bounded trace set is
  too large to stay under a reasonable cost — STOP and propose the
  `service_edges_minute` materialization as a prerequisite plan.
- Option (a) SVG layout produces unreadable graphs for the playground's
  topology — report before adding a heavy graph framework; a small layout-only
  lib is the ceiling.
- Rendering requires binding non-loopback or a second data path — it must not;
  everything goes through `/graphql`.

## Maintenance notes

- **Deferred (named):** the structural scale fix is a `service_edges_minute`
  (or `topology_edges_minute`) materialization so the graph is not derived by
  scanning traces per request; the `issue_buckets` minute-bucket upsert
  (`metadata.rs`) and the `run_metric_points` bootstrap/insert
  (`greptime.rs:81-112`) are the templates. Track in README.
- **Deferred:** the other three topology modes (one-hop/transitive/endpoint)
  and the execution graph — each is additive on this edge model.
- Reviewer: confirm the graph-mode label is present (the brief insists each
  mode is labelled because it makes a different causality claim), and that the
  edge derivation is window-bounded.
