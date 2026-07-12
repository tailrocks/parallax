# Plan 100: Restructure the UI by feature ownership

> **Executor instructions**: Preserve visual behavior, URL/search contracts,
> TanStack loader/cache semantics, and all existing tests while moving one
> vertical feature at a time. Run every package through Bun. Generated route and
> shadcn files are not manual ownership targets.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095; 099 soft
- **Category**: UI / architecture / performance
- **Planned at**: `eefa4617`, 2026-07-12
- **Status**: TODO

## Why

Several route files own queries, state, tables, charts, and presentation in one
place; the largest is about 1,500 lines. ESLint disables meaningful cycle
checks, GraphQL operations remain stringly/manual, live tails repeatedly sort
buffers, polling changes chart data identity, and route splitting/chunk weight
has not been proven.

## Target Layout

```text
ui/src/
  app/                 router, providers, shell, global boundaries
  features/
    dashboards/
    issues/
    logs/
    runs/
    services/
    traces/
  shared/
    api/
    components/
    hooks/
    lib/
  routes/              route/search/loader/composition only
```

Routes may depend on features/shared; features depend on shared through public
entries; shared never depends on features/routes; features do not import route
implementation.

## Scope

In scope:

- TypeScript-aware import/cycle enforcement.
- Feature-by-feature ownership moves and route size ratchets.
- Typed GraphQL operations/variables under Bun.
- Trace/log/run/services/dashboard/issue hotspot decomposition.
- Live-tail sorting, delta fetching, chart identity, and chunk graph evidence.
- Responsive visual smoke checks for moved surfaces.

Out of scope:

- Marketing/landing pages or visual redesign.
- Editing generated `routeTree.gen.ts` or shadcn-owned primitives.
- Node/npm/pnpm/yarn.
- Backend metric stub completion, owned by plan 105.

## Steps

### Step 1: Add import/cycle policy

Use ESLint/TypeScript-aware resolution under Bun, with fixture tests for aliases,
type-only imports, barrels, lazy imports, and generated exclusions. Fail every
forbidden direction and real cycle. Re-enable or replace disabled cycle rules
only after the current graph is measured and fixtures are trustworthy.

### Step 2: Establish typed GraphQL boundaries

Resolve the latest stable Bun-compatible GraphQL typing/code-generation path at
execution time using current docs. Generate or derive typed operation variables
and results from the canonical schema without introducing Node. Keep generated
files isolated, reproducible, and checked for drift. Retire manual string
escaping only after parity tests.

### Step 3: Move vertical features

Start with a bounded feature, then traces, logs, services, runs, issues, and
dashboards. Route files keep only route definitions, search parsing,
loader/preload wiring, error boundaries, and top-level composition. Queries,
tables, visualizations, state machines, and tests move to their feature.

Preserve the live cache/preload/SSE visibility behavior pinned by current route
tests and the 2026-07-11 closure evidence, plus all URL contracts.

### Step 4: Fix live data identity and work

- Replace repeated full-buffer sort with ordered insertion or one bounded sort
  per actual batch, proved by tests/measurement.
- Preserve chart series identity when data is unchanged.
- Add delta/cursor fetch only where server contracts support it; otherwise
  bound and memoize the existing window honestly.
- Verify list keys and live-tail identity across polling/SSE reconciliation.

### Step 5: Prove route/chunk boundaries

Inspect production build chunks. Ensure route/feature splitting is active and
heavy Recharts/Motion code does not inflate the shell without need. Motion is
currently theme-switcher-only; replace or lazy-load only if measured shell cost
justifies it and visual behavior remains identical.

### Step 6: Add measured size and visual gates

Ratchet route/component sizes after each split, excluding generated/shadcn
files. Use Playwright screenshots on representative desktop/mobile viewports
for moved routes and assert no overlap, clipping, or state loss. This is
structure/performance work, not a new visual system.

## Test Plan

- Import/cycle positive and negative fixtures.
- Generated GraphQL drift/type fixtures and operation parity.
- Existing route/search/loader/cache/SSE tests.
- Live-tail ordering/identity and chart memoization tests.
- Production chunk report comparison.
- Desktop/mobile visual smoke screenshots for every moved feature.
- Bun check/lint/typecheck/test/build.

## Done Criteria

- [ ] Import direction and cycles fail through TypeScript-aware tooling.
- [ ] Routes are thin orchestrators and feature ownership is explicit.
- [ ] GraphQL variables/results are typed and generated reproducibly under Bun.
- [ ] Named route hotspots shrink without behavior/visual changes.
- [ ] Live-tail/polling work is bounded and identity-stable.
- [ ] Chunk evidence proves route splitting and no accidental shell inflation.
- [ ] Generated/shadcn files are excluded from manual ratchets.
- [ ] All Bun and visual smoke gates pass.

## STOP Conditions

- A move changes route URLs/search/loader/cache semantics.
- GraphQL typing requires Node or a foreign package manager.
- Performance change drops/reorders live data.
- A chunk optimization changes theme/interaction behavior without approval.
- Visual checks show overlap, clipping, or responsive regression.

## Remove When

Delete this plan and row when feature ownership, typed GraphQL, live-data work,
chunking, and visual compatibility are enforced and green.
