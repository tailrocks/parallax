# Plan 163: Rebuild the trace timeline around a viewport reducer — drag-zoom, pan, color-by, self-time, flamegraph

> **Executor instructions**: Follow this plan step by step. Read `ui/AGENTS.md`
> first (it carries the browser-verification checklist — apply it after every
> step against playground trace scenarios). STOP conditions are binding.
> Update this plan's status row in `plans/README.md` when done.
>
> **Drift check (run first)**:
> `git diff --stat <wave2-base>..HEAD -- ui/src/components/console/trace-waterfall.tsx ui/src/lib/trace-tree.ts 'ui/src/routes/traces.$traceId.tsx'`
> `<wave2-base>` = the `main` commit closing Wave 1 (plan 159's evidence
> commit). Plan 160's defect fixes to these
> files are the expected baseline; any OTHER drift → STOP.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: MED-HIGH (rewrites the product's most-used analytical view)
- **Depends on**: plans 160 (waterfall defects fixed first), 162 (tokens/
  color lib)
- **Category**: direction / UI / traces
- **Planned at**: `2288011`, 2026-07-17

## Why this matters

The trace view is where incidents get solved, and it is the area the
operator flagged as historically buggy (plan 160 fixes correctness). What it
still lacks after those fixes is *navigation*: today the only ways to move
through a large trace are scrolling and the minimap. The reference product
(Maple) shows the target interaction grammar: every viewport change is an
action against a pure reducer, gestures map to those actions, the minimap is
a second controller over the same state, and a flamegraph gives an
aggregated alternative view. This makes 500-span traces (`t-wide`) and
12-deep traces (`t-deep`) navigable in seconds and makes the whole thing
unit-testable.

## Reference (self-contained)

Interaction grammar to implement (Maple `packages/ui/src/components/traces/`
— `use-timeline-interactions.ts`, `trace-timeline-types.ts`,
`timeline-reducer.test.ts` — clone `https://github.com/MapleTechLabs/maple`
for detail; the contract below is complete):

- **Viewport state** `{startMs, endMs}` relative to the trace window, all
  mutations via a reducer with actions: `ZOOM` (factor, anchorMs), `PAN`
  (deltaMs), `ZOOM_TO_SPAN`, `ZOOM_TO_RANGE`, `ZOOM_TO_FIT`, plus
  `SET_SEARCH`, `TOGGLE_COLLAPSE`. Constants: minimum visible window 0.1ms;
  traces longer than 10s open zoomed to the first 10s.
- **Gestures**: drag across the timeline = marquee → `ZOOM_TO_RANGE`; a
  pointer travel < 4px stays a span click; Shift+drag (or middle button) =
  pan; Ctrl/⌘+wheel = cursor-anchored zoom (factor 1.15); Shift+wheel or
  horizontal wheel = pan; plain wheel = native vertical scroll; double-click
  row = `ZOOM_TO_SPAN`; `+`/`-` keys zoom; `0` = fit. Native `wheel`
  listener with `passive:false`. Pointer-down captures the viewport so pans
  dispatch relative deltas (no stale-closure drift).
- **Bar rendering rules**: skip bars fully outside the viewport; clamp bar
  rects to [-50%, 150%] when deeply zoomed (a full-trace span must not
  create a gigapixel element); `width: max(2px, N%)` minimum hit target;
  in-bar span-name label only when bar ≥ ~56px, duration label ≥ ~140px.
- **Minimap**: depth-stacked overview; draggable viewport rectangle with
  edge-resize zones vs interior-pan vs click-to-recenter; outside-viewport
  regions dimmed.
- **Color-by**: strategy = service (default, plan-162 deterministic colors)
  | span kind | status | any span/resource attribute; encoded in the URL
  search param.
- **Self-time**: per-span self time = duration minus union of child
  intervals (merge overlapping children before subtracting); shown in the
  tooltip and span detail.
- **Flamegraph tab**: icicle layout with greedy lane packing per depth
  (siblings that don't overlap in time share a lane); click = select,
  Shift+click = focus subtree (re-layout relative to focused span); labels
  gated on percent width; own minimap optional.

## Current state

(verified at `2288011`; plan 160 will have amended these files)

- `ui/src/components/console/trace-waterfall.tsx` — TanStack-Virtual above
  300 rows; view modes tree/errors/lanes; minimap (sampled ≤2000 bars);
  keyboard j/k; "Whole trace" synthetic row; **no drag-zoom, no pan, no
  wheel zoom, no zoom-to-span, no color-by, no self-time**; minimap is
  display-plus-click, not a draggable viewport controller.
- `ui/src/lib/trace-tree.ts` — `buildTraceTree`, `computeWindow`,
  `errorPathSpanIds`, `groupByService`, `detectSkew` (+ plan-160 corpus
  fixtures for deep/wide/multiroot/orphan/skew/zero shapes).
- `ui/src/routes/traces.$traceId.tsx` — hosts the waterfall + TraceInspector
  (attributes/events/links/logs/stacktrace), critical path, compare, story,
  GraphQL/RPC panels. Search params via zod.
- Conventions: `ui/AGENTS.md` (incl. plan-162 additions: color axes,
  checklist); components ≤60-line functions where feasible; tests colocated
  `__tests__/`.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Focused tests | `cd ui && bun run --bun test:ci -- src/lib/__tests__/-timeline-reducer.test.ts` | pass |
| Full gates | `cd ui && bun run typecheck && bun run lint && bun run check && bun run --bun test:ci && bun run build` | exit 0 |
| Corpus traces | playground `scenarios/run.sh t-deep t-wide t-multiroot t-orphan t-skew t-links t-events` | trace ids printed |

## Scope

**In scope:**
- New `ui/src/lib/timeline-viewport.ts` (reducer + constants + px↔ms math,
  pure, exhaustively tested) and `ui/src/hooks/use-timeline-interactions.ts`
  (gesture binding).
- `trace-waterfall.tsx` rework onto the viewport model; minimap upgraded to
  a controller (drag/resize/recenter); color-by strategy (new
  `ui/src/lib/color-by.ts`, URL-encoded); label-width gating; self-time in
  `trace-tree.ts` (+ tooltip/inspector display).
- New `ui/src/components/console/trace-flamegraph.tsx` + lane-packing pure
  function in `trace-tree.ts`; new "Flame" tab beside tree/errors/lanes in
  `traces.$traceId.tsx`.
- Keyboard additions registered in the shortcuts surface that exists at
  execution time (command palette help or plan-164's registry if landed).

**Out of scope:** TraceInspector content, critical path/compare features,
trace LIST page, backend queries (span data shape unchanged), live
streaming behavior.

## Git workflow

- Work directly on `main` — no branches, no pull requests (operator
  delivery model, 2026-07-17; see plans/README.md Execution Preflight).
- Commit OFTEN: one small green slice per commit (a step, a component, a
  fixed defect), Conventional Commits, DCO `-s`, exactly one agent trailer.
- **Push to `main` immediately after every commit** — never batch pushes,
  never hold local-only work; never push a slice whose targeted checks are
  red. The parallax ruleset's "Bypassed rule violations" push notice is
  expected.

## Steps

### Step 1: Viewport reducer (pure)

`timeline-viewport.ts`: state, actions, constants (MIN_VISIBLE_MS=0.1,
DRAG_THRESHOLD_PX=4, DEFAULT_MAX_WINDOW_MS=10_000, ZOOM_FACTOR=1.15),
px↔ms conversion given container width + sidebar offset. Test every action:
zoom clamping at min window and trace bounds, anchor invariance (the ms
under the cursor stays fixed through zoom), pan clamping, fit, zoom-to-span
padding.

**Verify**: reducer test file passes; 100% branch coverage on the reducer
(if coverage tooling absent, enumerate each action × boundary in tests).

### Step 2: Gestures

`use-timeline-interactions.ts`: pointer/wheel/keyboard handlers dispatching
reducer actions per the grammar; marquee overlay rendering; `passive:false`
wheel; pointer capture. jsdom tests for: tap-vs-drag threshold, marquee →
ZOOM_TO_RANGE payload, shift-drag pan deltas, ctrl-wheel anchor math.

**Verify**: hook tests pass; manual browser check on `t-wide`: drag-zoom
into a 20ms window, pan across, wheel-zoom out — no scroll hijacking when
plain-wheeling the row list.

### Step 3: Waterfall on the viewport + minimap controller + color-by + self-time

Rework bar positioning to viewport-relative percentages with the clamp and
skip rules; minimap becomes the second controller; color-by picker (service
default; attribute mode lists keys present in the loaded trace); self-time
computed in `trace-tree.ts` (pure fn + tests with overlapping children).

**Verify**: all existing waterfall tests (incl. plan-160 regression suite)
still pass with the new engine — this is the compatibility gate; new tests
for clamp/skip/label gating; browser: `t-deep`, `t-orphan`, `t-skew` render
correctly zoomed and panned (screenshots).

### Step 4: Flamegraph

Lane packing (pure, tested: non-overlapping siblings share a lane;
overlapping ones stack), icicle render, focus-subtree, Flame tab.

**Verify**: lane-packing tests; browser: `t-wide` flamegraph shows hot
subtree at a glance; Shift+click focuses; screenshot.

### Step 5: Closure

Full gate set; browser walk of all `t-*`/`p-*` scenarios per the
`ui/AGENTS.md` checklist; screenshots to
`docs/research/validation/2026-07-wave2/163/`.

## Playground verification

Uses plan-161 scenarios exclusively: `t-deep`, `t-wide` (navigation +
virtualization under zoom), `t-multiroot`, `t-orphan` (must stay visible
under all viewport states), `t-skew` (no negative bars while zoomed),
`t-zero` (min 2px hit targets), `t-links`, `t-events`, `p-grpc-stream`.
No new scenarios required.

## Done criteria

- [ ] Reducer + gesture + lane-packing + self-time pure tests pass; plan-160
  regression suite passes unchanged.
- [ ] Full UI gates green.
- [ ] Browser evidence: drag-zoom/pan/wheel/zoom-to-span on `t-wide`;
  flamegraph focus; color-by attribute mode; self-time in tooltip —
  screenshots + clean console.
- [ ] URL round-trip: a zoomed viewport + color-by choice survives reload.
- [ ] `plans/README.md` status row updated.

## STOP conditions

- The existing virtualizer cannot host viewport-relative bars without
  per-row re-render storms (frame drops on `t-wide` while zooming) — report
  measurements before attempting a canvas rewrite; that is a separate
  decision.
- Plan-160 regression tests conflict with the new engine's rendering
  semantics — reconcile by fixing the engine, never by weakening a
  regression test.
- Search-param schema changes would break existing trace permalinks beyond
  adding new optional params.

## Maintenance notes

- The reducer is the single source of viewport truth — future features
  (span diffing, log-overlay lanes) dispatch actions, never mutate scroll or
  zoom state directly.
- Reviewer focus: anchor invariance math, clamp rules, and that keyboard/
  mouse paths produce identical action payloads.
