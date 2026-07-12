# Plan 143: Move app, layout, and shell behind explicit boundaries

> **Executor instructions**: Run this final product-ownership migration after
> Plans 134-142, 149, and 150 plus the shared browser foundations are complete.
> Move router/root composition, shell, navigation, command palette, health status,
> theme, and global route fallbacks into `app`, `layout`, `app-status`, and
> `quick-navigation`. Preserve every URL, root document/head/hydration behavior,
> SPA transition, preload/scroll setting, shell interaction, request, cancellation,
> retained state, error, theme, accessibility, and visual contract. Consume Plan
> 137's dashboard-navigation facade exactly and remove its two handoff exceptions.
> Materialize Plan 145's separate `@shell` reservation in
> `full-stack/shell.spec.ts`; do not duplicate `@storage` or Plan 150's `@overview`.
> Do not move overview, Plan-149 capabilities, completed feature internals, or
> generic residual files. Plan 151 owns the verification-only final sweep.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/__root.tsx ui/src/router.tsx ui/src/components/parallax-shell.tsx ui/src/components/nav.ts ui/src/components/nav-icon.tsx ui/src/components/route-fallbacks.tsx ui/src/components/theme-switcher.tsx ui/src/components/console/command-palette.tsx ui/src/lib/quick-jump.ts ui/src/components/__tests__/shell.test.tsx ui/src/components/console/__tests__/command-palette.test.tsx ui/src/lib/__tests__/quick-jump.test.ts ui/src/app ui/src/layout ui/src/features/app-status ui/src/features/quick-navigation ui/test-matrix.json ui/tests/e2e/datasets/shell.ts ui/tests/e2e/screens/shell-screen.ts ui/tests/e2e/contracts/shell.spec.ts ui/tests/e2e/full-stack/shell.spec.ts ui/tests/e2e/accessibility/shell-accessibility.spec.ts ui/tests/e2e/mobile/shell-mobile.spec.ts ui/tests/e2e/visual/shell.visual.spec.ts ui/tests/e2e/visual/goldens ratchet.toml`
> Resolve every moved prerequisite path through Plan 100's live ledger and the
> final feature facades. Stop if a named responsibility lacks one owner, Plan
> 137's handoff is absent/different, or observable root/shell behavior has drifted.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 134, 135, 136, 137, 138, 139, 140, 141, 142, 145, 146, 149, 150, 152, 153
- **Category**: TypeScript / app / layout / shell migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The root document and router composition are split across file routes, a root
router module, and generic components. The shell owns navigation, direct raw
health and dashboard requests, command-palette data access, global shortcuts,
theme, and route fallbacks. These responsibilities need explicit owners, but the
move must wait until every product feature facade exists so layout never imports
route implementations or feature internals.

Plans 149 and 150 remove the route-less capability and overview work from this
plan. This plan now has one coherent outcome: app composes the router; layout
composes the product shell; app-status and quick-navigation own their decoded
data contracts; the root route is thin; and shell-specific browser evidence is
complete. Mechanical repository-wide closure remains Plan 151 work.

## Fixed Decisions

1. `app` owns router/provider/composition creation. No lower layer imports app.
   If TanStack Start requires `ui/src/router.tsx`, it remains a thin framework
   adapter delegating to `app` and exports only the required registration contract.
2. `routes/__root.tsx` exports only `Route` and retains file-route head/document/
   boundary wiring. It imports only the reviewed layout root plus required
   route/framework assets.
3. `layout` owns root layout, shell, primary/workspace navigation, active state,
   navigation icons, command-palette UI/keyboard lifecycle, theme switcher, and
   global pending/error/not-found boundaries. It may import only reviewed feature
   facades, domain, and shared; never a route, feature internal, platform, or app.
4. `features/app-status` owns the health document/schema/decoded adapter, readonly
   status projection, typed expected error, and explicit facade. Layout never
   calls `fetch`, GraphQL, or platform transport directly.
5. `features/quick-navigation` owns service names, recent trace/run projections,
   ID classification, matching, request composition, mapping, typed errors, and
   explicit facade. Layout owns only dialog/keyboard/focus/rendering lifecycle.
6. Plan 137's facade is the sole dashboard navigation data owner. Layout imports
   exactly `loadDashboardNavigation({ signal })` and
   `DashboardNavigationItem`. The operation remains raw and uncached, selects only
   dashboard `id`/`name`, starts only when the pathname begins with `/dashboards`,
   uses the shell `AbortSignal`, ignores `AbortError`, retains prior items when
   leaving the dashboard area or refresh fails, and preserves inline error clear/
   display behavior. Delete Plan 137's shell-query and unused-export exceptions
   after the switch; no replacement exception or wrapper is allowed.
7. Preserve the root HTML/head/styles/scripts/hydration suppression, default and
   system theme, shell/outlet placement, global fallbacks, router preload/scroll
   behavior, direct refresh, client navigation, active links, collapsible sidebar,
   shortcuts, palette open/close/reset, search/order/limits/navigation, health
   labels, and every accessible/visual state.
8. PageHeader, time-range, story, and runtime metrics are final Plan-149 owners.
   Overview is a final Plan-150 owner. This plan only consumes their facades where
   shell composition needs them; it does not move or edit their implementations.
9. Completed Plans 134-142 own all product feature code. This plan may consume
   their explicit facades for navigation/status/search projections but cannot
   deep-import, move, repair, or rewrite their feature internals.
10. Shell source tests live under `app/tests/**`, `layout/tests/**`,
    `features/app-status/tests/**`, `features/quick-navigation/tests/**`, or
    `routes/tests/**`. Browser code remains in the single Plans 132/144-146 stack.
11. `full-stack/shell.spec.ts` converts Plan 145's exact `@shell` reservation to
    one implemented row. It reuses Plan-145 public seeds and tests shell-specific
    composition; it does not repeat `@storage` discovery or `@overview` behavior.
12. Plan 133 owns Query/cache changes, Plan 147 owns live performance, Plan 148
    owns chunks/bundles, and Plan 151 owns final ledger/docs/zero-debt verification.

## Target Ownership

```text
ui/src/
  app/
    create-router.tsx
    router-contract.ts
    tests/create-router.test.tsx
  layout/
    root-layout.tsx
    app-shell.tsx
    navigation.ts
    nav-icon.tsx
    command-palette.tsx
    theme-switcher.tsx
    route-boundaries.tsx
    index.ts
    tests/
      root-layout.test.tsx
      application-shell.test.tsx
      command-palette-composition.test.tsx
      route-boundaries.test.tsx
  features/
    app-status/
      api/
        app-status.graphql
        app-status.generated.ts
        load-app-status.ts
      model/
        app-status.ts
        app-status-error.ts
      tests/
        api/app-status-api.test.ts
        model/app-status.test.ts
      index.ts
    quick-navigation/
      api/
        quick-navigation.graphql
        quick-navigation.generated.ts
        load-quick-navigation.ts
        quick-navigation-mapper.ts
      model/
        quick-jump.ts
        navigation-candidate.ts
        quick-navigation-error.ts
      tests/
        api/quick-navigation-api.test.ts
        model/quick-jump.test.ts
        model/navigation-candidate.test.ts
      index.ts
  routes/
    __root.tsx
    tests/root-route.test.tsx
  router.tsx                    # only when required by TanStack Start
ui/tests/e2e/
  datasets/shell.ts
  screens/shell-screen.ts
  contracts/shell.spec.ts
  full-stack/shell.spec.ts
  accessibility/shell-accessibility.spec.ts
  mobile/shell-mobile.spec.ts
  visual/shell.visual.spec.ts
  visual/goldens/
```

Use Plan 152's exact `.graphql`/`.generated.ts` template and handoff rows. Do not
create placeholder directories or generic `types`, `helpers`, `utils`, or
`common` modules. `routeTree.gen.ts`, `styles.css`, `components/ui/**`, and
`lib/utils.ts` retain their existing generator/design-system ownership.

## App And Layout Dependency Contract

| From | Allowed product imports | Forbidden imports |
|---|---|---|
| `app` | route tree, router/provider composition, reviewed facades | app imported by lower layers; product logic |
| root route | root layout facade, route/framework assets | feature internals, platform, another route implementation |
| `layout` | feature facades, domain, shared | routes, feature internals, platform, app |
| `app-status` | own internals, domain, platform, shared | layout, routes, app, another feature internal |
| `quick-navigation` | own internals, approved feature facades, domain, platform, shared | layout internals, routes, app, deep feature imports |

Cross-feature quick-navigation inputs must be the minimum reviewed facade exports
or values passed from app composition. Do not make services, traces, or runs
export raw wire contracts merely for the palette.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | app/layout/root/facade graph has no cycle, deep, route, or unknown edge |
| UI policy | `cargo xtask policy --only ui.architecture` | composition, only-Route, runtime, dashboard handoff, and facade rules pass |
| Test policy | `cargo xtask policy --only ui.tests` | app/layout/feature/route test topology and handoffs pass |
| Test matrix | `cargo xtask policy --only ui.test-matrix` | all shell risks and evidence IDs resolve once |
| Ratchets | `cargo xtask policy --only ui.ratchets` | root/router/shell/modules/functions/exports shrink; handoff exceptions reach zero |
| Focused tests | `cd ui && bun run --bun test:ci -- src/app/tests src/layout/tests src/features/app-status/tests src/features/quick-navigation/tests src/routes/tests/root-route.test.tsx` | non-zero owner-specific tests pass without diagnostics |
| All UI tests | `cd ui && bun run --bun test:ci` | complete suite passes under Bun; no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @shell` | fixture shell/root/nav/theme/fallback contracts pass |
| Real stack | `cd ui && bun run test:browser:full -- --grep @shell` | exact non-zero managed-stack shell row passes |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @shell && bun run test:browser:a11y -- --grep @shell && bun run test:browser:visual -- --grep @shell` | non-zero cross/mobile/a11y/visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | dataset/screen/spec/locator ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | separate shell reservation consumed once; lifecycle rules pass |
| Breadth policy | `cargo xtask policy --only ui.browser-breadth` | engine/device/a11y/golden ownership passes |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 with zero warnings/errors |
| Build | `cd ui && bun run build` | generated tree current; root/direct/SPA behavior unchanged |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | managed stack and selected breadth lanes pass |

All JS/TS/browser commands use exact locked tools through Bun. Oxc-backed xtask
and the one Playwright stack are authoritative. Node, foreign package managers,
ESLint, another source graph/runner, response interception, manual generated
edits, or implicit installs are forbidden.

## Shell Real-Stack Contract

`ui/tests/e2e/full-stack/shell.spec.ts` owns the exact Plan-145 `@shell` reserved
row. Reuse Plan 145's public-OTLP stable entity IDs and public readiness
predicates. Prove the managed product reaches the root shell, health status
reflects the public server boundary, primary/workspace navigation reaches the
seeded feature routes through visible links, active state follows navigation,
and direct deep-link refresh retains the root document/shell.

For dynamic dashboards, use the public UI/GraphQL setup established by Plan 137,
navigate into `/dashboards`, and prove the exact facade-driven item appears,
cancels cleanly on navigation, retains prior items across the characterized
failure/leave behavior, and preserves inline error semantics. Do not duplicate
dashboard CRUD assertions, Plan 150 overview series assertions, or Plan 145's
generic ingest-to-surface `@storage` walk.

**Verify:** `@shell` selects one or more Plan-143 rows only, uses one-worker
managed GreptimeDB plus isolated Turso, performs no direct storage access or
response interception, and leaves clean process/port/data ownership.

## Shell Browser Breadth

This plan owns every `@shell` Plan-146 row. Run root readiness, navigation,
sidebar collapse, active state, dynamic dashboard entries, command palette,
quick ID navigation, theme persistence, pending/error/not-found boundaries, and
direct refresh in the matrix-selected Firefox/WebKit projects. Cover genuine
touch/mobile drawer/sidebar behavior, long labels, overflow, tap targets, and
focus restoration on both mobile projects. Run axe plus keyboard order,
shortcuts, dialog focus trap/Escape/restoration, accessible names/status, and
reduced motion. Maintain only canonical shell states tied to declared layout risk.

**Verify:** cross/mobile/a11y/visual rows are non-zero and unique, use semantic
locators, and pass without broad masking, coordinate-only assertions, response
interception, or an unguarded golden update.

## Scope

**In scope:**

- Root route/document, framework router adapter, app composition, root layout,
  shell, navigation registry/icon, command palette UI, theme, and global fallbacks.
- Decoded app-status and quick-navigation API/model/facade ownership.
- Exact Plan-137 `loadDashboardNavigation({ signal })` facade consumption and
  deletion of its shell-query and unused-export handoff exceptions.
- App/layout/app-status/quick-navigation/root source tests and their Plan-129 rows.
- Shell dataset/screen/fixture contract, exact delegated full-stack spec, and
  matrix-owned cross/mobile/accessibility/visual evidence.
- Targeted old-file deletion and ratchets for the responsibilities above only.

**Out of scope:**

- Overview source/tests/E2E (Plan 150) and runtime-metrics/story/time-range/
  PageHeader ownership (Plan 149).
- Moving, editing, or repairing completed Plan-134-142 feature internals beyond
  consuming their already-published facades.
- Generic residual `components`, `components/console`, `hooks`, or `lib` sweep;
  repository-wide ledger closure; final docs; or deletion of ambiguous stale
  paths (Plan 151).
- Query/cache changes (133), live/poll optimization (147), route/lazy/chunk/
  minifier/source-map/bundle work (148), backend/API/product redesign, auth, or
  new navigation destinations.
- Internal npm packages, project references, another browser stack, Node, foreign
  package managers, manual route-tree/shadcn edits, or visual redesign.

## Git Workflow

- Stay on the single active branch; never create another branch or PR.
- Land app/router/root, app-status/quick-navigation, shell/dashboard handoff,
  tests, browser evidence, and old-path cleanup as separate green commits.
- Serialize `ui/test-matrix.json`, `ratchet.toml`, shared E2E catalogs/config, and
  generated route-tree updates. Re-read the active branch before every patch.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable update under repository policy.

## Steps

### Step 0: Prove prerequisites and freeze root/shell behavior

Confirm Plans 134-142, 145, 146, 149, 150, 152, and 153 are complete and their source,
browser, full-stack, and breadth evidence is green. Resolve all final feature
facades through Plan 100's live ledger. Record router creation/preload/scroll and
boundaries; root head/document/scripts/hydration/theme; shell navigation/active/
collapse behavior; dashboard trigger/request/abort/retention/error; health request
and labels; command shortcut/open/reset/requests/order/limits/navigation/failure;
fallbacks; and all accessibility/visual markers.

Require each remaining root/router/shell/nav/theme/fallback/quick-jump legacy
test or private import to have an exact Plan-129 handoff owned by Plan 143.
Confirm Plan 145 reserves exactly `full-stack/shell.spec.ts` under `@shell` and
that it is distinct from `@storage` and Plan 150's `@overview` row.

**Verify:** focused legacy tests and shell browser contracts select non-zero
evidence and pass; policies reject missing, duplicate, wrong-file, or already-
implemented shell reservations and any absent Plan-137 handoff.

### Step 1: Establish app and thin root composition

Move router/provider/composition creation into `app`. Keep `ui/src/router.tsx`
only if required by TanStack Start and reduce it to delegation. Preserve route
tree registration, preload stale time, scroll restoration, default boundaries,
context/provider lifetime, direct refresh, and SPA behavior. Do not introduce
QueryClient or change cache timing.

Move root layout composition behind the reviewed layout facade. Reduce
`routes/__root.tsx` to root file-route/head/document/boundary wiring and export
only `Route`. Preserve HTML/head/styles/scripts/hydration/theme/outlet exactly.

**Verify:** app/root tests, architecture, typecheck, build, direct-refresh, and
SPA shell cases pass. No lower layer imports app and the root route imports no
feature internal or platform module.

### Step 2: Extract app-status and quick-navigation data owners

Create each named `.graphql` operation and checked-in generated sibling through
Plan 152 under its feature API owner. Decode `unknown`, map once to readonly values, and map
expected failures once. Preserve app-status request method/body/count/timing,
abort, healthy/offline classification, tolerance, labels, and endpoint display.

For quick navigation, preserve service/recent trace/run request shapes/counts,
limits, order, open-only fetch, abort/failure tolerance, ID classification,
static page filtering, candidate labels/icons, close/reset behavior, and exact
navigation targets/search. Publish minimal explicit facades; layout receives
domain values/actions, never raw DTOs or documents.

**Verify:** API/model tests cover valid/null/malformed/error/abort and exact
requests/projections; command-palette data tests cover every ID kind, ordering,
limits, failure, cancellation, and navigation target.

### Step 3: Move shell, navigation, command palette, theme, and fallbacks

Move shell/nav/icon/theme/fallback components and command-palette UI/lifecycle to
layout. Consume only explicit app-status, quick-navigation, completed feature,
Plan-149 shared/capability, and Plan-137 dashboard facades. Model async shell
state as exhaustive discriminated unions without changing visible states.

Preserve sidebar groups/order/icons/labels, active state, collapsible behavior,
dashboard subitems and limit, command shortcut/dialog/focus, status pill, theme,
content dimensions/scrolling, fallbacks, and all current text/accessibility.

**Verify:** layout and browser `@shell` tests pass; architecture reports only
approved layout-to-facade edges and no direct transport, route, app, deep feature,
or feature-to-layout import.

### Step 4: Consume the dashboard navigation handoff exactly

Replace the shell's raw dashboard GraphQL implementation with Plan 137's exact
`loadDashboardNavigation({ signal })` and `DashboardNavigationItem` exports.
Preserve dashboard-path-only activation, raw uncached minimal selection,
AbortSignal lifecycle, ignored `AbortError`, retained prior items, leaving-path
behavior, error clearing/display, item order/limit, and navigation links.

After source and tests are green, delete both the Plan-137 shell-query exception
and temporary unused-export exception in the same commit. Do not add another
allowlist, compatibility wrapper, layout transport call, or cached list reuse.

**Verify:** targeted shell tests and architecture policy prove exact facade use;
request-count/cancel/retention/error characterization passes; both exceptions and
the duplicate shell query are absent.

### Step 5: Move source tests and delete owned old paths

Move router tests under `app/tests`, shell/composition/theme/fallback tests under
`layout/tests`, app-status and quick-navigation tests under their features, and
root URL/document/boundary tests under `routes/tests`. Split PageHeader and
overview assertions to their already-completed Plan-149/150 owners rather than
moving them here. Preserve stable matrix IDs and observable assertions; remove
private imports and Plan-129 handoffs atomically.

Delete only the old source/test paths owned by this plan after Oxc proves every
caller moved. Ratchet root route to at most 150 logical lines, handwritten
modules to 300, test scenarios to 500, functions/components/hooks to 60, and
complexity to 12 cyclomatic/15 cognitive. Remove all Plan-143 migration and
Plan-137 handoff exceptions.

**Verify:** focused/all tests and architecture/test/ratchet policies pass; no
old shell/nav/theme/fallback/quick-jump path or private root export remains.

### Step 6: Materialize shell browser evidence

Implement shell dataset, screen, and fixture-backed contracts using Plan 144's
existing extension points. Convert the exact Plan-145 `@shell` reservation into
the Shell Real-Stack Contract without copying IDs or `@storage` assertions. Add
only matrix-required cross/mobile/a11y/visual cases through Plan 146's projects
and guarded canonical golden process.

**Verify:** all shell browser commands and policies select non-zero unique rows,
pass in their declared projects, and leave no Node, process, network, state, or
artifact leak. `@overview`, `@storage`, and feature-owned rows are unchanged.

### Step 7: Close this migration and hand off verification

Rebuild only this plan's ownership slice from the live Oxc graph. Require every
app/layout/app-status/quick-navigation/root/shell source, test, and browser row
to have one owner; remove all temporary reexports and plan-specific exceptions.
Do not sweep unrelated generic buckets or rewrite final project documentation.
Record any remaining repository-wide item as Plan-151 verification input.

Run every command twice from clean state. The second run must not change routes,
matrix data, ratchets, reports, or goldens.

**Verify:** all commands exit zero twice, `git diff --check` is clean, and Plan
151 can consume a zero-exception app/layout/shell ownership slice.

## Test Plan

- App/router tests for one router per composition, generated registration,
  preload/scroll/default boundaries, provider lifetime, direct refresh, and SPA.
- Root-route tests for exact head/document/styles/scripts/hydration/theme/outlet,
  pending/error/not-found wiring, and only-Route export.
- App-status API/model tests for exact request, valid/null/malformed/error/abort,
  classification, labels, and tolerated failure.
- Quick-navigation API/model tests for exact requests/limits/order, all ID kinds,
  candidates, failures, cancellation, filtering, labels, and targets.
- Layout tests for navigation order/active state/collapse, dashboard lifecycle,
  command shortcut/dialog/focus/reset, theme, status, fallbacks, and content shell.
- Fixture browser contracts for root/shell/nav/theme/fallback/palette states.
- One exact managed-stack shell scenario consuming Plan 145's reserved row.
- Matrix-selected Firefox/WebKit/mobile/touch/axe/keyboard/focus/visual evidence.
- Oxc policy negatives for app reverse imports, layout transport/route/deep imports,
  duplicate dashboard query, stale handoff, private root export, and duplicate E2E ID.

## Done Criteria

- [ ] App owns router/provider composition and no lower layer imports app.
- [ ] `router.tsx` is absent or a framework-required thin delegate; root route
  exports only `Route` and preserves exact document/SPA behavior.
- [ ] Layout owns shell/nav/palette/theme/fallback composition and imports only
  approved feature/domain/shared facades.
- [ ] App-status and quick-navigation decode/map once, expose minimal facades, and
  preserve all request, failure, projection, and navigation behavior.
- [ ] Layout uses Plan 137's exact dashboard navigation facade with the raw/path/
  abort/prior-items/error contract unchanged; duplicate query and both exceptions
  are deleted.
- [ ] Plan-149 and Plan-150 owners are only consumed, never moved or duplicated;
  completed feature internals were not edited.
- [ ] Source tests use final app/layout/feature/route topology with stable matrix
  IDs, no private route import, and no Plan-143 legacy handoff.
- [ ] The exact Plan-145 `@shell` reservation is one non-empty
  `full-stack/shell.spec.ts` owner distinct from `@storage` and `@overview`.
- [ ] Shell contract/cross/mobile/accessibility/visual rows are uniquely owned,
  non-empty, and green.
- [ ] No Query/cache, live-performance, bundle, product/API, or generic residual
  sweep work landed.
- [ ] All commands pass twice from clean state with no generated drift.

## STOP Conditions

Stop and report if:

- any dependency plan or its forced-Bun/browser/full-stack/breadth evidence is
  incomplete or red;
- a completed feature lacks a stable facade needed by layout or requires a deep
  import/change to its implementation;
- Plan 137 lacks the exact dashboard navigation exports/exception handoff or its
  raw/path/abort/prior-items/error contract has changed;
- Plan 152's generator/handoff cannot represent a frozen app-status or quick-
  navigation GraphQL operation, or Plan 153 cannot validate a current non-
  GraphQL browser value; discovering a new environment/message consumer requires
  a separate plan;
- Plan 145 lacks one exact unimplemented `@shell` reservation or it overlaps
  `@storage`/`@overview` ownership;
- root/router/shell/nav/theme/fallback/request/accessibility/visual behavior has
  materially drifted before movement;
- architecture requires layout-to-platform, layout-to-route, feature-to-layout,
  app reverse import, feature deep import, cycle, or broad exception;
- preserving behavior requires overview or Plan-149 capability movement,
  completed-feature edits, Query/cache work, live optimization, bundle/chunk
  work, backend/API/product redesign, another browser stack, or generated edits;
- final cleanup discovers ambiguous or unowned generic code that belongs in Plan
  151 rather than this migration; or
- a required gate fails twice after one reasonable correction.

## Maintenance And Removal

Future root/shell changes update their app/layout or feature facade, source tests,
browser evidence, matrix rows, and ratchets together. Layout never regains direct
transport or feature-internal access. Dashboard navigation remains the exact
Plan-137 facade contract until a separately approved behavior change.

Delete this plan and its README row only after all owned old paths/tests/exports,
Plan-129 handoffs, Plan-137 exceptions, and shell reservations are resolved and
every done criterion and command is green.
