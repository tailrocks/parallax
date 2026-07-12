# Plan 100: Restructure the UI by feature ownership

> **Executor instructions**: Preserve visual behavior, URL/search contracts,
> TanStack loader/cache semantics, and all existing tests while moving one
> vertical feature at a time. Run every package through Bun. Generated route and
> shadcn files are not manual ownership targets. Plans 128/129 establish strict
> boundaries and characterization first. TanStack Query is the sole target
> server-state cache; do not retain the current TTL cache beside it.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 101, 128, 129; 099 soft
- **Category**: UI / architecture / performance
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

Several route files own queries, caching, state, tables, charts, and
presentation in one place; the largest is 1,500 lines. The current generic
GraphQL cache keeps data for 15 seconds while mutations and Refresh call only
router invalidation, so a reload can immediately receive stale cached data.
Route modules export implementation components/functions for tests, which can
prevent TanStack automatic code splitting. Server/client imports, loader search
dependencies, dynamic GraphQL aliases, SSE ownership, and chunk weight are not
enforced.

## Target Layout

```text
ui/src/
  app/                 bootstrap, router creation, QueryClient, providers
  layout/              shell, navigation, application boundaries
  features/
    <feature>/
      api/              documents, runtime schemas, transport calls
      model/            domain state and pure transforms
      queries/          queryOptions, keys, mutations
      components/       feature presentation
      hooks/            feature orchestration
      __tests__/        feature-owned tests
      index.ts          reviewed public entry
  shared/
    api/
    components/
    hooks/
    lib/
  routes/              route/search/loader/composition only
  test/                deterministic harness from plan 129
```

Routes may depend on the reviewed layout entry, feature public entries, and
shared; only the root route imports layout. Layout may depend on reviewed
feature entries and shared. Features depend on shared and may depend on another
feature only through a declared public entry. Shared never depends on layout,
features, or routes. `app` composes router/QueryClient/providers and nothing
imports `app`. Generated route-tree composition is the only declared reverse
edge. Features never import route implementations.

## Scope

In scope:

- TypeScript-aware import/cycle and TanStack server/client enforcement.
- Feature-by-feature ownership moves and route size ratchets.
- Placement of plan 128's typed/validated GraphQL and SSE contracts.
- One TanStack Query cache with feature-owned query options/mutations.
- Trace/log/run/services/dashboard/issue hotspot decomposition.
- Live-tail sorting, delta fetching, chart identity, and chunk graph evidence.
- Responsive visual smoke checks for moved surfaces.

Out of scope:

- Marketing/landing pages or visual redesign.
- Editing generated `routeTree.gen.ts` or shadcn-owned primitives by hand.
- Node/npm/pnpm/yarn.
- Backend metric stub completion, owned by plan 105.

## Steps

### Step 1: Add import/cycle policy

Extend plan 095's single Rust-native Oxc parser/resolver graph; do not create an
ESLint or second import graph. Fixture aliases, type-only imports, package
exports, barrels, lazy imports, generated exclusions, and root-route-to-layout
acyclicity. Fail every forbidden direction and real cycle. Oxlint
`import/no-cycle` may be a fast supplemental rule only with `ignoreTypes:false`,
`allowUnsafeDynamicCyclicDependency:false`, unlimited depth, and parity against
the authoritative fixture corpus.

Define the complete matrix for app/routes/layout/features/shared, cross-feature
public entries, generated route composition, `.server`/`.client` modules, and
dynamic imports. Current source has no measured internal cycle, so do not create
a legacy cycle allowlist. Verify server-only modules, environment secrets,
filesystem/process APIs, and server functions cannot enter production client
chunks. Feature `index.ts` facades use reviewed explicit exports; handwritten
`export *` barrels fail.

Every TanStack Start server function validates its input at the boundary with
plan 128's schema, returns a decoded contract, and is callable only from the
intended client surface. Fixture SSR and client navigation separately because
Start modules are isomorphic unless protected.

### Step 2: Place typed data boundaries

Mechanically relocate plan 128's authoritative SDL output, named GraphQL
documents, generated variables/results/runtime schemas, dynamic-widget
contract, search schemas, and typed SSE frames. Static operations live under
feature `api/`; common envelope/transport belongs in `shared/api`. Generated
files remain reproducible under Bun and drift-checked. No route owns a raw query
string, JSON cast, event decoder, value interpolation, or manual escaping. This
plan may not create a second schema/codegen/decoder pipeline.

Update `ui/AGENTS.md` in the implementation change so browser data policy
truthfully permits canonical same-origin GraphQL queries/mutations plus the two
typed same-origin SSE feeds. No other endpoint is implied.

### Step 3: Make TanStack Query the only cache

Install the latest mutually compatible stable `@tanstack/react-query` through
Bun if plan 101 confirms the dependency suite. Create one fresh QueryClient for
each router/server request and keep one stable browser client for that router
lifetime; a module singleton is forbidden. Features own stable keys and
`queryOptions`; route loaders call `ensureQueryData`, loader-backed components
may call `useSuspenseQuery`, and optional/on-demand work uses `useQuery` or
`fetchQuery`. Mutations invalidate or update exact keys. Set Router
`defaultPreloadStaleTime` to `0` so Query owns freshness and reconcile SSE
updates through QueryClient.

Encode the useful TanStack Query lint invariants without retaining ESLint or an
alpha Oxlint JS plugin: the Oxc-backed xtask AST/facade policy proves stable
QueryClient lifetime, feature-owned exhaustive query-key/options factories, no
unstable dependency passed into hooks, no void query function, and preferred
shared query options. Each invariant has a negative TS/TSX fixture.

Extend plan 129's shared test harness with an isolated QueryClient builder in
that same first-use slice; no test may share cache state across cases.

Delete `graphqlCached` and its cache after parity tests. Remove an unused Router/
Query bridge or configure it end to end; partial integration and dual caches are
forbidden. Test mutation, Refresh, navigation preload, SSR hydration, reconnect,
and visibility timing against current behavior, including the stale-cache
regression. Preserve SPA deployment semantics: only root-shell prerender is
expected. Test two router/server instances for cache isolation and browser
client stability; do not quietly enable per-route SSR to make hydration tests
pass.

### Step 4: Move vertical features

Start with a bounded feature, then traces, logs, services, runs, issues, and
dashboards. Route files keep only route definitions, search parsing,
loader/preload wiring, error boundaries, and top-level composition. Search uses
typed `validateSearch`, and `loaderDeps` includes every value affecting data.
Queries, tables, visualizations, state machines, and tests move to their
feature.

Before each wave, its persisted plan 129 matrix rows must be green. Add only the
Query/key/invalidation/move-specific cases required by this wave, then persist
the updated row before moving source. A missing characterization case blocks
the wave; it cannot be deferred to this plan's final checklist.

Route files export only the file-route `Route` contract. Move testable
components/loaders/transforms to features instead of exporting route properties
that defeat automatic splitting. Deep consumers use `getRouteApi` rather than
importing route definitions. Loaders return `void` or minimal identifiers when
Query owns data, avoiding oversized inferred route types.

Preserve the live cache/preload/SSE visibility behavior pinned by current route
tests and the 2026-07-11 closure evidence, plus all URL contracts.

### Step 5: Fix live data identity and work

- Replace repeated full-buffer sort with ordered insertion or one bounded sort
  per actual batch, proved by tests/measurement.
- Preserve chart series identity when data is unchanged.
- Add delta/cursor fetch only where server contracts support it; otherwise
  bound and memoize the existing window honestly.
- Verify list keys and live-tail identity across polling/SSE reconciliation.

### Step 6: Prove route/chunk boundaries

Inspect production build chunks. Ensure route/feature splitting is active and
heavy Recharts/Motion code does not inflate the shell without need. Motion is
currently theme-switcher-only; replace or lazy-load only if measured shell cost
justifies it and visual behavior remains identical.

Enable/verify automatic route code splitting using the current TanStack plugin
contract. Assert each lazy property is actually split and that exported route
implementation cannot pull it back into the shell. Add a client-bundle manifest
check for server-only code and secret-bearing modules.

Record the resolved Vite 8/Rolldown production path and prove its upstream-owned
Oxc minifier is the only minification pass. Do not add direct `oxc-transform`,
`oxc-minify`, `unplugin-oxc`, or Vite+ ownership. Compare source maps, runtime
behavior, and chunk identity across two clean builds; direct Oxc build adoption
requires a later TanStack-supported migration plan.

Capture initial and post-wave entry/route chunk identities and gzip/Brotli
sizes. Lazy route properties remain separate. Each row has a shrink-only or
operator-approved exact delta budget; a report with no numeric/structural oracle
does not pass. The shell budget includes accidental duplicate framework/runtime
code, and client reachability to server modules is always zero.

### Step 7: Add measured size and visual gates

Ratchet route files to the 150-line target, handwritten TS/TSX modules to 300,
and functions/components/hooks to 60 after each split. Existing larger items are
exact shrink-only rows. Moving an intact oversized component does not pass.
Generated/shadcn files are excluded only from manual size ownership.

Use plan 129's Bun-invoked deterministic browser harness and viewport/data/
clock/theme/motion contract for moved routes. Assert no overlap, clipping,
hydration/console error, server bundle leak, or state loss. This is structure/
performance work, not a new visual system.

## Test Plan

- Oxc-backed import/cycle/root-layout positive and negative fixtures plus
  supplemental Oxlint parity.
- Existing generated GraphQL drift/type/operation parity after mechanical move;
  no second pipeline.
- Existing route/search/loader/cache/SSE/SSR tests plus stale-after-mutation and
  stale-after-Refresh regressions.
- Query key/options/AST invariants, mutation invalidation, two-router isolation,
  stable browser ownership, SPA root-shell hydration, navigation, and sole-cache
  tests.
- Live-tail ordering/identity and chart memoization tests.
- Two-clean-build Vite/Rolldown/Oxc-minifier evidence plus numeric compressed
  chunk identity/budget and source-map comparison.
- Desktop/mobile visual smoke screenshots for every moved feature.
- Bun check/lint/typecheck/test/build.

## Done Criteria

- [ ] Import direction and cycles fail through TypeScript-aware tooling.
- [ ] Server/client violations and server-only client bundle content fail.
- [ ] Server functions validate inputs and pass separate SSR/client execution
  fixtures.
- [ ] Routes are thin orchestrators and feature ownership is explicit.
- [ ] Feature public entries contain only reviewed explicit exports and cannot
  grow silently.
- [ ] GraphQL variables/results are typed and generated reproducibly under Bun.
- [ ] Every runtime response/SSE frame is decoded by plan 128's owned schema.
- [ ] TanStack Query is the only server-state cache and invalidation is exact.
- [ ] QueryClient ownership is fresh per router/server request, stable in the
  browser, and enforced with Query-specific AST negative fixtures without ESLint.
- [ ] Route files export no implementation that defeats automatic splitting.
- [ ] Named route hotspots shrink without behavior/visual changes.
- [ ] Live-tail/polling work is bounded and identity-stable.
- [ ] Chunk evidence proves route splitting, one framework-owned Oxc minifier,
  numeric compressed budgets, source-map parity, and no accidental shell
  inflation.
- [ ] Generated/shadcn files are excluded from manual ratchets.
- [ ] All Bun and visual smoke gates pass.

## STOP Conditions

- A move changes route URLs/search/loader/cache semantics.
- GraphQL typing requires Node or a foreign package manager.
- TanStack Query parity cannot preserve loader/SSR/SSE semantics; STOP rather
  than ship two caches.
- A server-only module or secret-bearing dependency appears in a client chunk.
- Performance change drops/reorders live data.
- A chunk optimization changes theme/interaction behavior without approval.
- Visual checks show overlap, clipping, or responsive regression.

## Remove When

Delete this plan and row when feature ownership, typed GraphQL, live-data work,
chunking, and visual compatibility are enforced and green.
