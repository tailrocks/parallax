# Plan 160: Audit every Parallax UI surface against the corner-case corpus, fix every display defect, enforce generic-attributes-only

> **Executor instructions**: Follow this plan step by step. This is a
> verify-then-fix plan: no fix lands without a browser-observed defect record
> and a regression test, and no defect record closes without a browser-observed
> re-verification. Read `ui/AGENTS.md` first. Honor STOP conditions; update
> this plan's status row in `plans/README.md` when done.
>
> **Drift check (run first)**: plans 156, 157, 158, and 161 must be
> implemented on `main` (both repositories; direct-to-main delivery model)
> before this plan starts (`git log --oneline -20` on both). The corpus ids
> referenced below come from the playground's `docs/corner-case-matrix.md`;
> if that file is absent, STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED (touches the trace-rendering hot paths users already rely on)
- **Depends on**: plans/156-unified-cli-observability-contract.md,
  plans/157-cli-invocation-observability-ui.md,
  plans/161-playground-corner-case-matrix.md
- **Category**: bug / correctness / UI
- **Planned at**: commit `39f172c` (+ same-day restructuring commits), 2026-07-17
- **Operator directive (2026-07-17)**: parts of the current UI render
  incorrectly — the operator specifically remembers defects in **how spans
  render inside a trace** (waterfall internals). Every surface must be
  verified against real corpus data with a browser, defects fixed, usability
  checked (visibility, accuracy, no display glitches). Parallax implements
  business functionality over generic attributes only.

## Why this matters

An observability product that mis-renders a span tree destroys trust in every
other number it shows. The trace waterfall, span inspector, and their pure
models (`trace-tree.ts` and friends) predate the corner-case corpus and have
never been exercised against deep/wide/multi-root/orphan/skewed traces
systematically. This plan is the systematic pass: corpus in, browser walk per
surface, defect ledger, fix + regression test per defect, browser
re-verification — plus a sweep proving no application-specific attribute
logic survives anywhere in the product.

## Current state

(verified at `39f172c`; plan 157 adds the invocations surfaces on the same
branch before this plan runs)

- **Trace rendering (priority-1 audit target)**:
  `ui/src/components/console/trace-waterfall.tsx` — TanStack-Virtual above
  300 rows, view modes tree/errors/lanes, minimap sampling ≤ 2000 bars,
  keyboard nav, "Whole trace" synthetic row, bars colored by span kind.
  Pure models: `ui/src/lib/trace-tree.ts` (`buildTraceTree`, `computeWindow`,
  `errorPathSpanIds`, `groupByService`, `detectSkew`). Inspector inside
  `ui/src/routes/traces.$traceId.tsx` (`TraceInspector` region ~:1244-1447):
  attributes/resource/events/links, span-correlated logs
  (`logs.filter(l => l.spanId === span.spanId)` ~:1307), stacktrace,
  critical path, compare, evidence gaps, skew banner, GraphQL ops
  (`lib/graphql-trace.ts`), RPC streams (`lib/rpc-trace.ts`).
- **Other audited surfaces**: logs (`routes/logs.tsx` + `components/
  logs-table.tsx`: live tail, histogram, saved views, document sheet),
  metrics (`components/metric-strip.tsx`, `components/runtime-snapshot.tsx`,
  dashboards routes, `routes/services.$service.tsx` RED/exemplars), issues
  (`routes/issues.index.tsx` virtualized list, `routes/issues.$fingerprint.tsx`
  stacktrace folding via `lib/stacktrace.ts`, trend buckets, breadcrumbs),
  ecosystem (`components/console/ecosystem-graph.tsx` hand-rolled SVG),
  overview (`routes/index.tsx` charts), plus every plan-157 invocations
  surface (list, hub tabs, journey).
- **Usability bar**: the six-item browser checklist defined in plan 157's
  "Browser verification protocol" section (data correctness, links, states,
  layout at 1440px/375px, live behavior, clean console).
- **Generic-attributes-only invariant** (operator, 2026-07-17; contract
  decision in plan 156): business logic only over generic keys; any
  application-specific/vendor attribute appears only inside generic
  attribute-list views (inspector attribute tables, field explorer) as opaque
  data. Known suspects to sweep: remaining `parallax.*`/`jackin`-flavored
  constants in `ui/src/shared/semconv.ts` after 156's regeneration, any
  hardcoded attribute-name special cases in components
  (`grep -rn "parallax\.\|jackin" ui/src`).
- **Evidence home**:
  `docs/research/validation/2026-07-unified-cli-observability/` (created by
  plans 157/159).

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Backend + corpus | `parallax serve` (real invocation per `crates/parallax-cli`) + playground stack + `scenarios/run.sh --all-corner-cases` | corpus loaded; ids printed |
| One scenario | `scenarios/run.sh t-wide` | exit 0 |
| UI unit tests | `cd ui && bun run --bun test:ci` | all pass |
| Focused | `cd ui && bun run --bun test:ci -- src/lib/__tests__/-trace-tree.test.ts` | pass |
| Types/lint/format | `cd ui && bun run typecheck && bun run lint && bun run check` | exit 0 |
| Build | `cd ui && bun run build` | exit 0 |

## Scope

**In scope (fix-on-evidence, ui/ plus narrowly-matched backend):**
- Every file named in Current state, their pure models and tests.
- `crates/parallax-api`/`crates/parallax-greptime` ONLY when a browser-proven
  defect's root cause is a wrong query/mapping (each such fix is its own
  commit labeled with the defect id and stays inside the existing
  module/test conventions).
- The defect ledger:
  `docs/research/validation/2026-07-unified-cli-observability/ui-defect-ledger.md`.
- The generic-attributes conformance sweep across `ui/src` and
  `crates/parallax-api`.

**Out of scope:**
- New product features (157 owns the invocations surface; do not extend it
  here), visual redesign, theme changes, shadcn primitive edits.
- Live-stream performance/merge algorithms (plan 147) beyond fixing
  observable display bugs.
- Playground changes — a missing corpus shape goes back to plan 161 as a new
  scenario id, not an inline hack.

## Git workflow

- Work directly on `main` (operator delivery model: no branches, no PRs).
  One commit per defect fix (`fix(ui): D-007 orphan spans vanish from
  waterfall`), ledger updated in the same commit, pushed when green.

## Steps

### Step 1: Build the audit grid and run the walk

Create the ledger with one section per surface × relevant corpus ids (from
`docs/corner-case-matrix.md`): traces × `t-*`/`p-*`, logs × `l-*`, metrics ×
`m-*`, issues × `e-*`, ecosystem × `eco-full`, invocations/journey × `j-*`,
overview × sweep. Load the corpus, then walk every cell with the browser
tooling applying the six-item checklist. Record each failure as
`D-NNN: surface, corpus id, expected (from the matrix), observed (screenshot
path), severity (broken|wrong|degraded|cosmetic)`. Record passes too (the
grid must be complete — an unvisited cell is not a pass).

**Verify**: ledger contains every grid cell with pass/fail + screenshot;
zero cells "not checked".

### Step 2: Fix trace-rendering defects first

For each `D-NNN` on the waterfall/tree/inspector (operator's priority):
reproduce in a unit test against the pure model (`trace-tree.ts` fixtures
mirroring the corpus shape — deep/wide/multi-root/orphan/skew/zero-duration),
fix, then browser re-verify the same corpus id and attach the after
screenshot. Expected defect classes to check explicitly: lost multi-roots,
orphans dropped from the tree, negative/NaN bar widths under skew,
zero-duration spans invisible or overlapping, minimap sampling hiding the
error path, virtualization scroll desync, inspector event ordering,
span-log correlation misses when span ids repeat across traces.

**Verify**: per defect — new unit test red→green; browser re-check recorded;
`bun run --bun test:ci` green.

### Step 3: Fix remaining surfaces

Same discipline for logs (ordering with identical timestamps, huge bodies,
ANSI, histogram vs live pause), metrics (counter reset, gaps, exemplar
links), issues (multi-language stacktrace folding, trend bucket edges),
ecosystem (node/edge rendering with all three kinds, self-loops if present),
overview, invocations/journey cells.

**Verify**: ledger shows every non-cosmetic defect fixed + re-verified;
cosmetic ones either fixed or explicitly deferred with rationale.

### Step 4: Generic-attributes conformance sweep

`grep -rn "parallax\.\|jackin" ui/src crates/parallax-api/src` — every hit is
either (a) deleted dead code, (b) a generic display of arbitrary attributes,
or (c) a defect to fix. No component/resolver may branch on an
application-specific attribute name. Confirm the attribute
inspector/field-explorer render unknown vendor attributes (corpus emits none
after plan 158 — add a one-off manual OTLP post with a `custom.vendor.attr`
to prove display-as-opaque works; document the command in the ledger).

**Verify**: grep audit clean per the rule above; opaque-attr screenshot in
the ledger.

### Step 5: Closure

Re-run the FULL grid walk once (fixes can regress neighboring cells).
Summarize in the ledger: defects found/fixed/deferred, per-surface verdict.

**Verify**: second full walk all-pass (or explicitly deferred-cosmetic);
`cd ui && bun run typecheck && bun run lint && bun run check && bun run
--bun test:ci && bun run build` all exit 0.

## Test plan

- One regression unit test per fixed defect, colocated with the owning
  module's tests, named with the defect id in the test description.
- `trace-tree.ts` gains corpus-shaped fixtures (deep/wide/multiroot/orphan/
  skew/zero) usable by future plans.
- The ledger itself is the audit artifact — complete grid, before/after
  screenshots, commands.

## Done criteria

- [ ] Ledger grid complete: every surface × corpus id cell has pass/fail +
  screenshot; every non-cosmetic defect fixed with a red→green regression
  test and re-verification capture.
- [ ] Operator's span-rendering area explicitly closed: all `t-*` cells pass.
- [ ] Step-4 conformance sweep clean; opaque vendor-attr display proven.
- [ ] Full UI gate set green (`typecheck`/`lint`/`check`/`test:ci`/`build`).
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report back (do not improvise) if:
- A defect's root cause is in GreptimeDB itself (native table/pipeline
  behavior) — that is a fix-forward/upstream-consult situation per repo
  policy, not a UI patch; document the live evidence.
- A fix requires changing the plan-156 contract (new key, changed priority) —
  route to 156 on the same branch, do not fork the contract here.
- The corpus cannot reproduce a defect the operator remembers — record what
  was tested and the negative result; do not claim the memory wrong, and add
  the missing scenario to plan 161 instead of hand-waving.
- A waterfall fix regresses the `console_frame`-style perf budget (if the UI
  has a perf test lane at execution time) or visibly janks 500-span scrolling.

## Maintenance notes

- The grid walk (step 1/5) is the template for future release verification;
  keep the ledger format stable.
- Every future rendering bug: corpus scenario first (plan 161 discipline),
  then the fix with the scenario-keyed regression test.
- Reviewer focus: fixes must not special-case corpus ids or attribute names —
  generic code paths only.
