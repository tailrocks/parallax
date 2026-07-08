# Plan 061: Trace waterfall view modes — errors-only collapse, service lanes, minimap, clock-skew warning

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- ui/src/lib/trace-tree.ts ui/src/components/console/trace-waterfall.tsx ui/src/routes/traces.\$traceId.tsx`
> On mismatch with the excerpts below, STOP. (Plans 040/051 touch the same
> files — see "Coordination" in Current state.)

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (waterfall ordering/layout math is shared by every trace render)
- **Depends on**: best after plans 040 (waterfall windowing/memoization) and
  051 (critical-path toggle + `highlightIds` prop) land — their diffs touch
  the same files; rebase order matters more than logic. No hard API deps.
  Pairs with playground plan 063 (A19 stress trace + skew scenario feed it).
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

The trace view is tree-waterfall-only. On the traces plan 063 will generate
(hundreds of spans, deep nesting, multi-service), three questions the brief
requires become slow or impossible: "show me only the failing path" (today: a
jump-to-first-error banner, no filter), "group by service so I can see hops"
(today: one depth-indented lane), and "am I being lied to by clocks?" (today:
`positionPct` silently clamps, so a child starting 'before' its parent renders
plausibly instead of warning). This plan adds three view modes and an honest
skew banner while preserving the existing default exactly.

## Current state

Verified at commit `ed5b10f`.

- `ui/src/lib/trace-tree.ts` (119 lines) — the math: `buildTraceTree`
  (parent/child + orphan handling), `orderSpans` (depth-first),
  `computeWindow` (min start / max end), `positionPct` clamps to [0,100]
  (`:76-108` region), `compareByStart` (`:33-38`). **No skew detection.**
- `ui/src/components/console/trace-waterfall.tsx` — one render path;
  builds the tree once (`:31`), j/k keyboard nav; plan 040 adds windowing
  >300 rows; plan 051 adds a `highlightIds` prop + critical-path toggle in
  the route.
- `ui/src/routes/traces.$traceId.tsx` — waterfall card `:232-243`; failed
  spans computed at `:159-161`; error banner `:213-228` (jump-to-first
  only).
- Mode precedent in the codebase: `ViewToggle` segmented control exists
  unused (`ui/src/components/console/view-toggle.tsx:8` — dead code, no
  importers). Either adopt it here for the mode switch or (if its API
  doesn't fit) delete it in the same commit — do not leave it dead.
- Coordination: 051 introduces a "Critical path" toggle on this same
  card — the mode switch this plan adds must absorb that toggle's placement
  (modes: Tree | Errors | Lanes; critical-path stays an orthogonal overlay
  checkbox, not a fourth mode).

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck/lint/test/build | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `ui/src/lib/trace-tree.ts` — pure additions: `errorPathSpanIds(spans)`,
  `groupByService(ordered)`, `detectSkew(spans)`
- `ui/src/components/console/trace-waterfall.tsx` — mode prop, lane
  headers, minimap strip
- `ui/src/routes/traces.$traceId.tsx` — mode state in URL search
  (`view=tree|errors|lanes`), skew banner, wire `ViewToggle` or replacement
- `ui/src/components/console/view-toggle.tsx` — adopt or delete
- Tests

**Out of scope** (do NOT touch):
- DAG/linked-trace rendering — advisor-plans/028.
- Color-by-attribute — deferred (needs a palette-mapping design; note it).
- Critical-path computation — plan 051 owns it; this plan only preserves its
  overlay.
- Story tab — advisor-plans/029.
- Any GraphQL/API change. Skew detection is client-side over loaded spans.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Pure helpers in `trace-tree.ts`

1. `errorPathSpanIds(spans): Set<string>` — every ERROR span plus all its
   ancestors to the root (walk `parentSpanId`; orphans include themselves
   only). This is "errors-only with ancestors preserved" (brief rule).
2. `groupByService(ordered: OrderedSpan[]): Array<{ service: string;
   spans: OrderedSpan[] }>` — stable partition of the existing depth-first
   order into contiguous service runs (a new group whenever the service
   changes walking the ordered list) — lanes keep temporal/causal reading.
3. `detectSkew(spans): SkewReport` — for each parent/child pair compute
   `childStart - parentStart` and `parentEnd - childEnd`; report pairs where
   the child starts >50ms before its parent or ends >50ms after
   (cross-service pairs only — same-service pairs share a clock). Return
   `{ suspectPairs: Array<{parentId, childId, driftMs}>, maxDriftMs }`.
   Threshold as an exported const with a doc comment.

**Verify**: `bun run test` — unit tests: ancestor chain kept for a
grandchild error; lane partition stability; skew fixture (child −200ms) →
one suspect pair, same-service drift ignored.

### Step 2: Waterfall modes

`trace-waterfall.tsx` gains `mode: "tree" | "errors" | "lanes"` (default
`"tree"`, behavior byte-identical to today):
- `errors`: filter the ordered rows to `errorPathSpanIds`; empty set →
  render the tree with a muted "No errored spans" line above (mode still
  selectable).
- `lanes`: render `groupByService` groups — a sticky mini-header per lane
  (service name + span count + lane duration) above its rows; indentation
  within a lane unchanged.
- Minimap: a fixed-height (~24px) strip above the rows, all modes: every
  span as a 1-2px bar positioned by `positionPct` against the whole-trace
  window, error spans in the destructive color; click seeks the scroll
  container to that span's row (`scrollIntoView`). For >2000 spans render
  every ⌈n/2000⌉-th span (cap the DOM); with plan 040's windowing, minimap
  click must force the window to include the target row (read 040's
  windowing API when rebasing).

**Verify**: `bun run test` — mode render tests on a 3-service fixture with
2 errors: errors mode shows error+ancestors only; lanes mode shows 3+ lane
headers (contiguity may split a service into two lanes — assert on the
fixture's real partition); minimap bar count.

### Step 3: Route wiring + skew banner

1. `traces.$traceId.tsx`: `view` in the search schema
   (`tree|errors|lanes`, default tree — follow the existing search-param
   pattern in the route, e.g. how `selectedId`/range params parse). Mode
   switch UI on the Waterfall card header: adopt `ViewToggle` if its
   cards/table API generalizes to labeled segments; else replace with a
   small `ToggleGroup` (shadcn `toggle-group` exists in `ui/`) and delete
   `view-toggle.tsx` + its dead export.
2. Skew banner: `detectSkew(spans)` in a `useMemo`; when `suspectPairs`
   non-empty render an amber banner under the error banner
   (`:213-228` pattern): "Clock skew suspected: N parent/child pairs
   disagree by up to X ms across services — span order may be misleading."
   Non-dismissible, informational.
3. The error banner's "Open first" stays; in errors mode it's redundant but
   harmless — leave it.

**Verify**: `bun run typecheck && bun run lint && bun run test && bun run
build` all clean. Manual: `?view=errors` deep-link renders errors mode
(record check).

## Test plan

- `trace-tree.test.ts` additions (Step 1 cases) — extend the existing test
  file for this lib (grep `trace-tree` in `ui/src` tests; model on it).
- Waterfall mode + minimap tests (Step 2).
- Route-level: search-param default + banner presence (fixture with skew).

## Done criteria

- [ ] `bun run typecheck && bun run lint && bun run test && bun run build` all exit 0
- [ ] Default render byte-identical (existing waterfall tests still green,
      no snapshot churn in tree mode)
- [ ] `?view=errors|lanes` deep-links work; mode persists in URL
- [ ] Skew banner appears on the skew fixture, absent otherwise (tests)
- [ ] `view-toggle.tsx` is either imported by the route or deleted —
      `rtk grep -rn "ViewToggle" ui/src` shows usage or nothing
- [ ] `plans/README.md` status row updated

## STOP conditions

- Plans 040/051 landed with waterfall changes that contradict the excerpts
  (windowing API, highlight prop) — re-read their diffs and rebase this
  plan's steps; on structural conflict (e.g. 040 virtualized in a way that
  breaks lane headers), STOP and report options.
- Lane grouping breaks j/k keyboard navigation order — fix by navigating the
  flat ordered list regardless of visual grouping; if that's not achievable
  in the component's current structure, STOP.
- The minimap causes measurable jank on the A19 stress trace (>16ms frame on
  scroll) — ship without click-seek (render-only) and note it.

## Maintenance notes

- Plan 063's A19 + skew scenarios are the demo feed; after both land, the
  TOUR (plan 054) should add a "modes" beat.
- Color-by-attribute and a DAG mode remain deferred (named here so the
  README's deferred list stays honest).
- Reviewer: `detectSkew` threshold (50ms) and cross-service-only rule are
  judgment calls — check they're documented in the code comment; the banner
  must never block interaction.
