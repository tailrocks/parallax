# Plan 166: Service map v2 — ELK layout, focus & declutter, external dependency nodes from generic span attributes

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> (browser-verification checklist applies after every step, against playground
> topology scenarios). STOP conditions binding. Update this plan's status row
> in `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- ui/src/routes/ecosystem.tsx ui/src/components/console/ecosystem-graph.tsx crates/parallax-api crates/parallax-greptime`
> `<wave2-base>` = the `main` commit closing Wave 1. Plan 157 added node kinds
> (cli/browser/service) — that is the expected baseline.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (layout rewrite of an existing page; new derived queries)
- **Depends on**: plans 156/157 (node kinds), 162 (service colors)
- **Category**: direction / UI + API / topology
- **Planned at**: `2288011`, 2026-07-17

### Landed (preliminary, helper agents) — peer verify and wire

**Do not retire yet.** `ui/src/lib/ecosystem-topology.ts` is the sole pure
focus/declutter model: caller+callee hop neighborhoods, dim/hide projection,
max-edge-relative traffic filtering with hidden counts, external dependency
identity helpers, and safe behavior for stale focus permalinks. Commit
`322d6fd` briefly introduced an overlapping focus module during concurrent
work; the follow-up consolidation deleted it and ported its stale-permalink
case into this broader engine so divergent threshold/focus semantics cannot
reach graph wiring.

The peer executor owns review/deepening, focus/declutter and URL wiring, ELK verification,
external-node backend work, full gates, and browser evidence. Confirm the
threshold semantics against final preset labels before wiring; do not treat
this helper slice as closure evidence.

ELK foundation also landed preliminarily: `elkjs@0.11.1` via Bun
(`f4aeea0`) and `ui/src/lib/ecosystem-layout.ts` plus its Vite worker
(`0ad35fd`). It provides stable topology keys, a bounded 32-entry promise
cache, deterministic sorted `layered`/RIGHT ELK input, browser-worker error
fallback, and a deterministic Kahn-layer fallback for Bun/Vitest/SSR. Six
focused tests, targeted lint, and format pass. Context7 was attempted first
but its monthly quota was exhausted; elkjs's shipped declarations/README were
used. The peer must still run full typecheck/build after its
concurrent metrics route regenerates route types, confirm the worker chunk,
measure fallback layout below the plan's 100ms STOP threshold, and capture
browser evidence.

`dcdc020` wires that layout into `ecosystem-graph.tsx`: deterministic fallback
renders immediately, the worker result replaces it asynchronously, stale
promises cannot overwrite newer topology, and the existing links/edge labels/
9-node no-overlap behavior remain intact. The combined graph/layout/topology
suite passes 25 tests and targeted lint passes. Full typecheck was blocked by
the peer's concurrent unfinished Plan-163 trace variables, not this slice.
Fallback timing on the operator's arm64 host (2026-07-17): 1,000 layouts of a
50-node/100-edge synthetic topology completed in 40.27ms total (0.0403ms
mean), far below the plan's 100ms sync-fallback STOP threshold. Peer still
verifies the real corpus and browser worker path.

`e83a414` adds URL-persisted focus service, 1/2-hop, dim/hide, and relative
traffic controls. Projection uses the full topology for hop membership before
traffic filtering, preventing a low-rate link from silently shrinking the
focus neighborhood. Range changes preserve focus state; stale/invalid params
canonicalize safely; dimmed nodes/edges and combined hidden counts render in
the graph. Four focused files pass 28 tests, targeted lint, full UI typecheck,
and the production build. Vite emitted the real ELK worker chunk
(`ecosystem-layout.worker-*.js`, 1.43 MB), closing the worker-bundling check.
Peer still owns live corpus/browser proof and may deepen the control layout.

## Why this matters

The ecosystem page answers "who calls whom", but only for instrumented
services, only with a BFS-layer layout that tangles at ~15 nodes, and with
no way to focus on one service's neighborhood. Real systems hang off
*uninstrumented* dependencies — databases, queues, third-party HTTP APIs —
that are visible in CLIENT/PRODUCER span attributes (`db.system.name`,
`messaging.system`, `server.address`) but rendered nowhere today. The
reference product shows the full pattern: proper layered layout, focus
mode with hop radius, low-traffic decluttering, and external-resource nodes
inferred purely from generic attributes — exactly Parallax's
generic-attributes-only doctrine.

## Reference (self-contained)

From Maple (`apps/web/src/components/service-map/`; clone
`https://github.com/MapleTechLabs/maple` for detail — the contract below is
complete):
- **Layout**: ELK "layered" algorithm (direction RIGHT) via `elkjs`, run in
  a web worker (main-thread fallback for jsdom/tests); deterministic —
  memoize on a topology key. Parallax keeps its hand-rolled SVG renderer;
  only the coordinate assignment changes.
- **Focus mode**: pick a service → show 1-hop or 2-hop neighborhood; other
  nodes either dimmed or hidden (user choice); URL-persisted
  (`focus`, `hops`, `focusMode` params).
- **Low-traffic filter**: hide edges below N% of the max edge call-rate
  (presets All / >0.1% / >1% / >5%), with a "N hidden" chip.
- **External nodes** (derived from CLIENT/PRODUCER spans that have no
  matching SERVER/CONSUMER child in another instrumented service):
  - database node when `db.system.name` (fallback legacy `db.system`)
    present; name = `db.namespace` → `db.name` → `server.address`;
  - queue/broker node when `messaging.system` present; name =
    `messaging.destination.name`;
  - external HTTP node otherwise when `server.address` present (group by
    host).
  Edge stats (calls, error rate, p95) aggregate from those client spans.
- **Edge labels**: call count + error-rate chip (severity-colored by
  thresholds); edge width ~ log of call count.
- Optional polish (only after core lands): animated traffic dots on edges
  drawn on ONE shared canvas with a global particle budget allocated
  proportionally to call rate; `prefers-reduced-motion` disables.

## Current state

(verified at `2288011`; plan 157 adds `kind` to nodes)

- `ui/src/routes/ecosystem.tsx` — loads `serviceMap { nodes {name
  lastSeenNanos spanCount errorCount p95Ms} edges {source target callCount
  errorCount p50Ms p95Ms} }` (`maxTraces: 100`), range params.
- `ui/src/components/console/ecosystem-graph.tsx` — hand-rolled SVG: BFS
  depth layering (`layoutNodes`), bezier edges, `log2(callCount)` widths,
  nodes → `/services/$service`, edge labels → `/traces?service=…`; no
  focus, no traffic filter, no external nodes.
- Backend `service_map` resolver (`crates/parallax-api/src/lib.rs:135`)
  derives service→service edges from trace parent/child span relationships
  in `crates/parallax-greptime` (trace-path walk, bounded by `maxTraces`).
- Semconv constants for `db.*`/`messaging.*`/`server.address` exist or are
  standard names addable via `telemetry/semconv/contract.yaml`
  (+ `cargo xtask semconv generate`).
- Playground topology: browser → checkout → pricing/inventory/
  recommendation; storefront → catalog/payment; fulfillment ⇄ Kafka
  (redpanda) → payment/notifications; inventory + catalog use Postgres
  (sqlx / JDBC auto-instrumentation ⇒ `db.system` client spans); CLI →
  checkout. So databases + the Kafka broker are real, currently-invisible
  external dependencies.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Storage/API tests | `cargo nextest run --locked -p parallax-greptime -p parallax-api` | pass |
| Live engine | `cargo nextest run --locked -p parallax-server -E 'binary(/greptime/)'` | pass |
| UI gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Corpus | playground `scenarios/run.sh eco-full p-kafka-lag` + steady demo load | topology populated |

## Scope

**In scope:**
- `crates/parallax-greptime` — external-node derivation query: CLIENT/
  PRODUCER spans grouped by (service, dependency-identity) with call/error/
  p95 aggregates, dependency-identity resolved by the attribute ladder
  above; exclusion of pairs already covered by instrumented service edges.
- `crates/parallax-api` — `serviceMap` gains `nodes[].kind` values
  `database|queue|external` (extending 157's `cli|browser|service`) and
  `nodes[].system` (e.g. `postgresql`, `kafka`); edges to external nodes.
- `ui/` — `bun add elkjs` (workspace-lockfile update); layout module
  `ui/src/lib/ecosystem-layout.ts` wrapping ELK in a worker with sync
  fallback (deterministic; memoized on topology hash); `ecosystem-graph.tsx`
  consumes computed coordinates, adds focus mode + hop selector +
  dim/hide toggle + low-traffic filter + hidden-count chip + node-kind
  glyphs (database/queue/globe icons) + severity-colored error-rate edge
  chips; `ecosystem.tsx` URL params (`focus`, `hops`, `focusMode`,
  `minTraffic`).
- Optional step 5 (skippable without failing the plan): canvas particle
  overlay with global budget.

**Out of scope:** ReactFlow or any graph-framework adoption (hand-rolled
SVG stays); vendor-specific inference (no Hyperdrive-style product logic —
generic attributes only, per the binding invariant); node position
persistence; namespace grouping (defer until a corpus scenario needs it).

## Git workflow

- Work directly on `main` in BOTH repositories — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: External-node derivation (backend)

Storage query + resolver + tests. Rules: a CLIENT span whose
trace-child SERVER span belongs to an instrumented service produces NO
external node (it's an internal edge); `db.system.name`/`db.system` →
database; `messaging.system` → queue (PRODUCER side; CONSUMER side links
queue → consuming service); else `server.address` → external HTTP host.
Live-engine test over seeded corpus: Postgres node behind inventory AND
catalog; redpanda queue between fulfillment producer/consumer; no external
node for checkout→pricing (instrumented pair).

**Verify**: cargo lanes pass, incl. the negative case.

### Step 2: ELK layout module

`elkjs` layered layout in a worker (fallback sync path for Vitest); input =
nodes+edges, output = coordinates; memoized on a stable topology key;
deterministic across runs (test: two runs, equal output).

**Verify**: layout unit tests pass under Vitest (fallback path); bundle
builds with the worker chunk (`bun run build`).

### Step 3: Graph UI — kinds, focus, declutter

Render kind glyphs + system label; focus mode (1/2 hops, dim vs hide),
low-traffic presets + hidden-count chip; error-rate chips on edges;
database/queue nodes link to `/traces?…` filtered by the dependency
attribute (via plan-164 `attributeFilters` when available, else service
filter); URL round-trip.

**Verify**: component tests (focus subgraph computation pure fn; traffic
filter thresholds; URL round-trip); UI gates green.

### Step 4: Browser closure

Walk per checklist against `eco-full` + `p-kafka-lag` + demo load:
databases and the Kafka broker visible with correct fan-in; focus checkout
1-hop then 2-hop; hide-mode; >1% traffic filter hides the low-rate CLI
edge with the chip counting it. Screenshots to
`docs/research/validation/2026-07-wave2/166/`.

### Step 5 (optional): Traffic particles

One shared canvas, global budget (~400) allocated by call-rate
(largest-remainder), viewport culling, reduced-motion off-switch. Skip
freely if step-4 evidence is complete and time-boxed effort is exceeded —
record the skip in the status row.

## Playground verification

Existing: `eco-full`, `p-kafka-lag`, demo k6 load. New scenario (linked
playground main, plan-161 discipline): `eco-external` — checkout calls a
third-party HTTP API (httpbin container or an unroutable-but-attributed
target) producing `server.address` client spans WITHOUT any instrumented
server side, so the external-HTTP node class is assertable.

## Done criteria

- [ ] Backend derivation tests pass incl. the instrumented-pair negative.
- [ ] Deterministic-layout test passes; UI gates green.
- [ ] Browser evidence: database + queue + external-HTTP nodes, focus
  modes, traffic filter with hidden-count, error-rate chips.
- [ ] `eco-external` scenario + matrix row landed on the playground's main.
- [ ] No vendor-specific attribute logic anywhere
  (`grep -rn "hyperdrive\|planetscale" ui/src crates/` → 0).
- [ ] `plans/README.md` status row updated.

## STOP conditions

- ELK worker bundling fights the Vite/Bun build — use the sync fallback
  permanently ONLY if layout of the corpus topology stays <100ms; report
  either way.
- External-node derivation produces unbounded cardinality on
  `server.address` (per-IP explosion) — cap + bucket to a "N more hosts"
  node and report, do not ship an unreadable graph.
- The `maxTraces:100` sampling makes edge stats misleading for the corpus —
  report with numbers; widening the sample is a backend decision, not a
  silent UI change.

## Maintenance notes

- Node-kind vocabulary (`cli|browser|service|database|queue|external`) is
  part of the GraphQL contract — extend via contract change, not ad hoc
  strings.
- Reviewer focus: the internal-pair exclusion rule (no duplicate external
  node for instrumented callees), determinism of layout, URL round-trip.
