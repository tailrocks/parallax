# Plan 133: Replace the UI TTL cache with feature-owned TanStack Query

> **Executor instructions**: Begin after structural plan 151 closes every
> feature move. Change cache ownership and invalidation only; plan 147 owns
> live-data algorithms and plan 148 owns bundle optimization. Migrate one feature
> at a time with named regressions and never retain the TTL cache beside TanStack
> Query as a completion state.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/package.json ui/bun.lock ui/vite.config.ts ui/test-matrix.json ui/tests/e2e ratchet.toml`
> Resolve baseline symbols through final feature/platform facades before editing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 101, 128, 129, 132, 144, 145, 151
- **Category**: UI / TanStack Query / cache correctness
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: BLOCKED — upstream UI and Playwright dependency plans are incomplete

## Why This Matters

At the baseline, `ui/src/lib/api.ts:8-90` owns a module-global 15-second result
cache and in-flight map keyed by interpolated query string. Mutations and
Refresh invalidate the Router but do not clear the cache, so immediate reload
can return stale data. The cache has no feature-owned keys, typed mutation
policy, request-level SSR ownership, or SSE reconciliation contract.

The final structure must have one cache with compiler-visible ownership. This
plan changes that behavior only after routes/features/tests are stable, keeping
cache regressions separate from source moves and performance work.

## Fixed Decisions

1. TanStack Query is the sole final server-state cache.
2. Each router/server request receives a fresh QueryClient; one browser router
   owns one stable QueryClient for its lifetime. Module singletons are forbidden.
3. Features own readonly key factories, shared `queryOptions`, mutations, and
   exact update/invalidation under `features/<feature>/queries/`.
4. Only Plan-152/153-decoded feature domain values enter cache; generic or
   unvalidated JSON cannot.
5. Loaders use `ensureQueryData`; components consume the same options through
   `useSuspenseQuery`/`useQuery` as appropriate.
6. Router preload stale time becomes `0` when Query owns loader freshness.
7. SSE reconciliation uses a feature-owned QueryClient path or remains an exact
   separately owned live boundary until plan 147; no duplicate result cache.
8. `graphqlCached`, its maps, and its cache-clear test hook are deleted after the
   final caller migrates.

## Target Shape

```text
features/<feature>/queries/
  keys.ts
  options.ts
  mutations.ts
  live.ts          # only when SSE/cache reconciliation is required
```

No empty directory or generic cross-feature query registry is allowed.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Dependencies | `cargo xtask dependencies --all` | one compatible TanStack set, no unused/duplicate cache package |
| Query policy | `cargo xtask policy --only ui.query-cache` | client/key/options/invalidation/sole-cache rules pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 |
| Unit/integration | `cd ui && bun run --bun test:ci` | all tests pass |
| Browser contracts | `cd ui && bun run test:browser` | cache/navigation/mutation rows pass |
| Full stack | `cd ui && bun run test:browser:full` | real-stack cache/mutation/SSE rows pass |
| Full aggregate | `cargo xtask ci --full` | exit 0 |

## Scope

In scope:

- Latest stable compatible TanStack Query/Router/Start set via Bun.
- QueryClient factory/router/provider composition.
- Feature key/options/mutation/live cache modules and exact tests.
- Old TTL/in-flight cache deletion and Query-specific architecture policy.
- Cache-owned stable matrix rows plus fixture-backed and real-stack assertions
  in the existing feature specs. Shared Playwright fixtures, projects, reporters,
  lifecycle code, and browser policy infrastructure remain read-only.

Out of scope:

- Feature/file/capability movement (plans 134-143, 149, and 150).
- Playwright infrastructure or unrelated coverage (plans 132 and 144-146); this
  plan still owns exact cache/request/invalidation scenario rows and assertions
  in their established specs.
- Live buffer/sort/render optimization (plan 147).
- Route chunk/minifier/dependency bundle optimization (plan 148).
- Backend/API contracts, visual redesign, a second cache, or unvalidated values.

## Git Workflow

- Stay on the single active branch; do not create a branch or PR.
- Land composition root, one pilot, remaining feature waves, and old-cache
  deletion as separate green changes.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Freeze observable cache behavior

Using plans 129, 144, and 145, record request counts, cache hits/misses,
preload/navigation, Refresh, mutations, visibility/reconnect, two-router
isolation, and current stale-after-mutation/Refresh behavior. Mark known stale
behavior as a failing target, not a permanent contract.

**Verify**: stable matrix IDs and machine-readable request/key oracles exist.

### Step 1: Add one QueryClient composition root

Install `@tanstack/react-query` through Bun in the same change that first imports
it. Build one factory with explicit defaults. Create a fresh client per router/
server request and one stable browser client. Wire provider/context without
migrating a feature.

Test two router/server instances, stable browser ownership, SPA shell hydrate/
dehydrate, error/cancel, and disposal. Do not quietly enable per-route SSR.

**Verify**: dependency, type/lint, isolation, build, and browser shell tests pass.

### Step 2: Migrate investigations as the pilot

Define feature-owned keys/options/mutations from the final Plan-152/153 decoded
adapters.
Prefetch with `ensureQueryData`, consume the same options in components, and
update/invalidate exact keys after create/update/delete. Remove all
investigations uses of the TTL cache without changing its feature structure.

**Verify**: request counts, preload/navigation/Refresh/mutation/two-client,
fixture browser, and full-stack Turso cases pass.

### Step 3: Migrate remaining features in bounded waves

Repeat the pilot for SQL/ecosystem where Query is appropriate, dashboards,
services, issues, runs, logs, traces, and overview. Each wave:

1. defines keys/options/mutations/live reconciliation;
2. switches loaders/components to the same options;
3. proves exact invalidation/update and decoded values;
4. removes old-cache calls for that feature; and
5. runs unit/browser/full-stack regressions before the next wave.

Set Router `defaultPreloadStaleTime` to `0` when the first loader-backed feature
is Query-owned. Keep SPA root-shell deployment semantics.

**Verify**: targeted and full command table passes after every wave.

### Step 4: Delete the old cache atomically

After the final caller migrates, delete `graphqlCached`, TTL/inflight maps,
cache-clear hooks, string-key behavior, and compatibility reexports. Keep one
decoded GraphQL transport. Use the Oxc graph to prove no hidden import/reexport/
dynamic caller remains.

Enforce QueryClient lifetime, feature keys/options, stable hook inputs, non-void
query functions, shared loader/component options, exact invalidation, and no
second cache with negative fixtures. Do not add ESLint or alpha plugins.

**Verify**: old symbols are unreachable, negative fixtures fail correctly, and
known stale regressions are green.

### Step 5: Close cache ownership

Update durable policy/docs with client lifetime, key naming, invalidation,
SSE/cache reconciliation, and feature placement. Remove temporary rows and keep
only exact measured exceptions with owner/expiry.

**Verify**: dependency/policy/type/test/browser/full-stack/full-CI commands pass
twice from clean state.

## Test Plan

- QueryClient request/browser lifetime, isolation, hydrate/dehydrate, cancel,
  error, and disposal tests.
- Feature key/options/query/mutation exact invalidation/update tests.
- Preload/navigation/Refresh/visibility/reconnect and stale-cache regressions.
- GraphQL/SSE runtime decoding before cache insertion.
- Oxc policy negatives for singleton client, unstable keys/hooks, void query,
  duplicate cache, broad invalidation, and old-cache reachability.
- Fixture-backed and real-stack browser mutation/persistence/SSE cases.

## Done Criteria

- [ ] TanStack Query is the only server-state cache and old cache symbols/maps
  are deleted with no hidden caller.
- [ ] QueryClient is fresh per server/router request and stable per browser
  router, with isolation proof.
- [ ] Every applicable feature owns decoded keys/options/mutations and exact
  invalidation/update through its facade.
- [ ] Router preload/cache/SPA behavior has one documented owner and stale
  regressions are fixed.
- [ ] No structure move, second schema/cache, live optimization, or bundle work
  is hidden here.
- [ ] All commands pass twice from clean state.

## STOP Conditions

Stop and report if:

- structural plan 151 or browser/full-stack plans 144/145 are incomplete;
- parity appears to require two caches;
- an unvalidated response would enter Query cache;
- per-request ownership cannot preserve current SPA behavior;
- a mutation cannot name affected keys without a product decision; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

New server-state features define keys/options/mutations/live reconciliation and
browser/full-stack evidence in the same change. Reviewers reject broad
invalidation, hidden raw JSON, module-singleton clients, and duplicate caches.

Delete this plan and README row only after sole-cache migration, old-cache
deletion, durable policy, and every required command are green.
