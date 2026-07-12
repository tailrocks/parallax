# Plan 137: Migrate dashboards into decoded model and API boundaries

> **Executor instructions**: Follow this plan in order and preserve dashboard
> list/detail URLs, range search, persisted layouts, widget queries/charts,
> create/edit/delete flows, loading/error/not-found, and current cache behavior.
> Start only after plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete and green. Extract
> runtime schemas, pure layout/series functions, and decoded APIs before moving
> presentation. Routes must export only `Route` and import only the dashboards
> facade. Do not add TanStack Query; plan 133 owns cache migration, plan 147
> owns live-data algorithms, and plan 148 owns bundle/performance work. Stop
> rather than guess about legacy layout compatibility or shell ownership. Do not
> invent a dashboard export action that is absent from the baseline.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/dashboards.index.tsx 'ui/src/routes/dashboards.$dashboardId.tsx' ui/src/routes/__tests__/-final-sweep.test.tsx ui/src/routes/__tests__/-dashboards.test.tsx ui/src/components/parallax-shell.tsx ui/src/components/__tests__/shell.test.tsx ui/src/lib/api.ts ui/src/lib/range.ts ui/test-matrix.json ui/tests/e2e ratchet.toml`
> Compare live exports, layout parser/serializer, operations, chunking, search,
> cache/invalidation, chart output, shell query, and tests with the ledger below.
> Any mismatch must be resolved in this plan before editing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / feature migration / architecture
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

Dashboards spans two route modules totaling 1,005 lines. Those routes own wire
types, GraphQL generation, cached loading, persisted JSON compatibility, metric
metadata effects, series transforms, mutation orchestration, URL state, charts,
and presentation. The detail route deep-imports implementation exports from the
list route, while tests import both routes' private symbols.

This plan establishes one dashboards feature with operation-specific runtime
schemas, readonly domain values, compatibility-preserving layout encoding,
pure/bounded series logic, cohesive components/hooks, and a reviewed facade.
Thin routes retain URL/boundary composition. The migration does not redesign
dashboards, replace caching, or silently add export behavior.

## Current State

### List and create route

- `ui/src/routes/dashboards.index.tsx` is 525 lines. It declares `Dashboard`,
  `DashboardSearch`, and extensible `Widget`; constants `AGGS` and `CHARTS`;
  search/range helpers; layout parse/serialize; metric label/value effects;
  cached list/metric-name loading; list/delete; create dialog; and cards.
- `/dashboards/` accepts optional `range`, `from`, and `to`. Its local
  `searchString` accepts strings and finite numbers, converting numbers to
  strings. Detail links and create navigation use normalized range search; a
  preset removes stale custom bounds and valid paired bounds remain custom.
- The loader issues one `graphqlCached` request for dashboard id/name/layout/
  updated time plus metric names. List delete uses raw `graphql`, reports an
  inline delete error, and invalidates the router. Empty/list/card text,
  relative time, widget count, confirmation, and current links are observable.
- Create starts with one empty widget, permits more, filters widgets without a
  metric at save, disables create for a trim-empty name but sends the original
  untrimmed name/layout through `dashboardSave`, resets/closes, and navigates to
  the new detail with the current normalized range.
- `parseLayout` returns `[]` on invalid JSON/non-array, retains array objects
  whose `metric` is a string, and preserves unknown widget properties.
  `serializeWidgets` uses normal `JSON.stringify`. Unknown properties are a
  compatibility contract, not disposable data.
- `WidgetPicker` loads metric label names after metric selection and label
  values after group selection using raw GraphQL. Value lookup uses `fromNanos:
  "0"` and `toNanos = Date.now() * 1_000_000`; failures become empty option
  arrays and late completions are ignored. Existing selected group/filter values
  remain visible even when absent from the returned option list.

### Detail route and series

- `ui/src/routes/dashboards.$dashboardId.tsx` is 480 lines and deep-imports
  `WidgetPicker`, `emptyWidget`, layout functions, and `Widget` from the sibling
  route, creating a route-to-route implementation dependency.
- `/dashboards/$dashboardId` parses range search, includes every search field in
  `loaderDeps`, resolves the range, cached-loads dashboard plus metric names,
  throws `notFound()` for a null dashboard, parses layout, cached-loads series,
  and returns detail/chart data.
- `loadWidgetSeries` builds dynamic aliased `metricSeries` fields, uses absolute
  aliases `w0...`, sends no query for zero widgets, chunks at exactly 24 widgets
  to remain below GraphQL complexity limits, preserves result order across
  chunks, and treats a missing alias as an empty series. It currently includes
  name/range/aggregation and optional `groupBy`; do not add filter semantics in
  this structural migration.
- `toWidgetData` keeps at most five series groups, supplies current fallback
  group names, combines points by timestamp, formats time for the active range,
  and sorts by nanosecond `BigInt`. Chart rendering preserves line/area/bar,
  group colors/legend, point count, width, title fallback, and empty widgets.
- Detail edit keeps a draft, adds/removes/reorders widgets, saves layout through
  raw `dashboardSave`, exits edit and invalidates on success, cancels to the
  loader widgets, and shows inline mutation errors. Delete uses raw GraphQL and
  navigates to `/dashboards`. Range changes merge typed search. Back navigation
  currently comes from a layout-owned `navItem`.

### Tests and adjacent shell ownership

- At the baseline, `ui/src/routes/__tests__/-final-sweep.test.tsx` mixes
  dashboard and SQL tests. Its dashboard slices cover unknown-field/layout
  round trips, label choices, stale-bound removal, custom-range links/create
  navigation, and series loading through route internals. Plan 129 must
  mechanically move those cases and stable IDs to
  `ui/src/routes/__tests__/-dashboards.test.tsx`, move SQL separately, and
  delete the mixed file before this plan starts.
- Dashboard model/API/facade/widget coverage is incomplete: malformed operation
  envelopes, not-found, exact 24/25/48 chunk ordering, missing aliases, five/six
  groups, label failures/late completion, list/detail mutations, chart variants,
  and route boundaries lack focused owners.
- `ui/src/components/parallax-shell.tsx` independently raw-loads dashboard nav
  items only on dashboard paths and displays an inline navigation error;
  `ui/src/components/__tests__/shell.test.tsx` owns that behavior. Plan 143 owns
  layout/shell migration. Its request selects only `id`/`name`, uses raw
  uncached GraphQL with an `AbortSignal`, ignores `AbortError`, runs only while
  the pathname starts `/dashboards`, retains the prior items across non-
  dashboard paths, and clears only the inline error there. This plan exposes
  that exact decoded operation as `loadDashboardNavigation({ signal })` plus
  `DashboardNavigationItem` through the facade, but does not edit shell/tests.
  Keep one exact temporary shell-query exception assigned to plan 143.

The baseline has no dashboard export button or file-download implementation.
This structural migration preserves that absence. A future export action needs
its own product plan and cannot be added as collateral refactoring or claimed as
covered browser behavior.

## Behavior Preservation Contract

Preserve exactly:

- both route paths/trailing slash, list numeric/string search coercion, detail
  range schema, every loader dependency, range normalization/merge, and typed
  create/detail navigation;
- current cached list/detail/series requests, raw label/mutation requests, cache
  key/TTL/request-count behavior, mutation invalidation timing, and no dual cache;
- the current fact that router invalidation after a mutation does not itself
  clear the module-global 15-second query-string-keyed GraphQL cache;
- list/empty/delete/create/error/not-found states and user-visible text;
- layout invalid fallback, accepted legacy records, known and unknown property
  round trips, widget ordering, and filtered empty widgets on create;
- label/value request inputs, current-clock nanoseconds, empty-on-error, ignored
  late completion, and preservation of selected missing options;
- dynamic aliases, 24-widget chunks, order/missing-alias behavior, maximum five
  chart groups, timestamp sort/format, and chart/legend/width/title behavior;
- edit/add/remove/move/save/cancel/delete state, navigation, and errors; and
- shell's current dashboard navigation behavior until plan 143 consumes the
  facade under its separately owned migration.

## Target Ownership

Create only real files:

```text
ui/src/features/dashboards/
  api/
    dashboard-navigation.graphql
    dashboard-navigation.generated.ts
    dashboards-list.graphql
    dashboards-list.generated.ts
    dashboard-detail.graphql
    dashboard-detail.generated.ts
    dashboard-save.graphql
    dashboard-save.generated.ts
    dashboard-delete.graphql
    dashboard-delete.generated.ts
    dashboard-metric-names.graphql
    dashboard-metric-names.generated.ts
    dashboard-metric-labels.graphql
    dashboard-metric-labels.generated.ts
    dashboard-metric-values.graphql
    dashboard-metric-values.generated.ts
    dashboard-api.ts
    widget-series-operation.ts
    widget-series-schema.ts
    widget-series-api.ts
  model/
    dashboard.ts
    widget.ts
    dashboard-layout.ts
    dashboard-layout-schema.ts
    dashboard-range.ts
    widget-series.ts
    dashboard-error.ts
  components/
    dashboards-page.tsx
    dashboard-page.tsx
    dashboard-cards.tsx
    dashboard-create-dialog.tsx
    dashboard-editor.tsx
    widget-picker.tsx
    widget-chart.tsx
  hooks/
    use-dashboard-editor.ts
    use-widget-options.ts
  tests/
    api/dashboard-api.test.ts
    api/widget-series-api.test.ts
    model/dashboard-layout.test.ts
    model/widget-series.test.ts
    components/dashboard-cards.test.tsx
    components/dashboard-create-dialog.test.tsx
    components/widget-picker.test.tsx
    components/widget-chart.test.tsx
    integration/dashboard-workflows.test.tsx
    integration/dashboard-facade.test.ts
  index.ts
ui/src/routes/tests/
  dashboard-routes.test.tsx
ui/tests/e2e/
  datasets/dashboards.ts
  screens/dashboards-screen.ts
  contracts/dashboards.spec.ts
  full-stack/dashboards.spec.ts
  accessibility/dashboards-accessibility.spec.ts
  mobile/dashboards-mobile.spec.ts
  visual/dashboards.visual.spec.ts
  visual/goldens/
    dashboards-list.png
    dashboards-detail-chart.png
    dashboards-editor.png
    dashboards-empty.png
    dashboards-recoverable-error.png
```

Fixed ownership:

- Each static `.graphql` file contains one globally unique named variables-only
  operation and one checked-in Plan-152-generated sibling for navigation,
  list/detail, save/delete, or metric metadata. It does not render, navigate, or
  cache.
- Plan 152 already owns the sole bounded dynamic `widget-series-*` AST/schema/
  adapter exception. Extend it in place; never recreate a string builder.
- `dashboard-layout-schema.ts` instantiates Plan 153 for persisted widget JSON.
  It must accept the baseline's minimal string `metric`, safely
  classify known fields, and retain unknown fields for round-trip compatibility.
  Never use `as Widget` or discard extension properties.
- `dashboard-api.ts` exposes decoded list/detail/save/delete and metric metadata
  operations, maps boundary values once, and projects failures to
  `DashboardError`. It owns no React/router state or cache policy beyond calling
  the existing platform cached/raw operation selected by the caller contract.
  Metric-label-value bounds read plan 100's platform clock contract and retain
  the exact current `now * 1_000_000` nanosecond calculation; feature code does
  not call `Date.now()` directly.
- `loadDashboardNavigation({ signal })` in `dashboard-api.ts` uses its own
  operation-specific `{ dashboards { id name } }` schema and raw uncached
  transport. It must not call or reshape the cached list-plus-metric-names
  loader. Cancellation remains distinguishable so plan 143 can preserve the
  shell's `AbortError` behavior.
- `widget-series-api.ts` owns the bounded aliased/chunked requests and response
  alias decoding. It preserves 24 exactly and returns results in widget order.
- `dashboard.ts` and `widget.ts` own readonly domain values. Represent unknown
  persisted properties explicitly enough to reserialize them unchanged.
- `dashboard-layout.ts` owns pure parse/map/serialize/default/edit transforms.
  Runtime-safe handling may not tighten or silently normalize a legacy record
  without a characterized compatibility decision.
- `dashboard-range.ts` owns only dashboard-specific pure range-link adaptation;
  generic range parsing/merge remains in plan 100's domain owner.
- `widget-series.ts` owns pure maximum-five grouping, timestamp merge/sort,
  display-row conversion, and chart model construction.
- `dashboard-error.ts` is an exhaustive Result-shaped expected-failure union.
  Distinguish transport, invalid response, list/detail/not-found, series,
  metric metadata, canceled navigation load, save, and delete; do not branch on
  message strings. Invalid persisted layout JSON retains the current empty-
  layout recovery in the model instead of becoming a new user-visible load
  error.
- hooks own actual editor and cancellable/ignore-late request lifecycle only.
  Pure transforms stay in models. Components receive typed values/callbacks and
  never parse GraphQL/JSON.
- `index.ts` explicitly exports route-facing pages/loaders, stable dashboard/
  widget contracts, `loadDashboardNavigation`, and
  `DashboardNavigationItem` for plan 143. It does not expose documents,
  schemas, internal hooks/components, wildcard exports, or the platform
  transport itself.

Prefer pure functions and readonly values. No class is expected. A class is
allowed only for a real lifecycle or invariant-bearing mutable identity with a
focused test; query building, editor grouping, and chart conversion remain
functions/modules.

Final structural ratchets are exact: route module <=150 logical lines,
handwritten TS/TSX module <=300, test scenario file <=500, function/component/
hook <=60, cyclomatic complexity <=12, and cognitive complexity <=15. An
unchanged oversized move does not pass. Any inherited exception is exact,
expiring, and shrink-only.

## Route, Facade, Cache, And Layout Rules

- Both route modules export only `Route`, import only
  `@/features/dashboards` plus allowed route/domain/shared contracts, and own
  only search/params/loader dependency/boundary/composition behavior.
- Remove the sibling route import completely. Feature components and tests use
  feature internals within their owner or the explicit facade outside it.
- The feature must not import layout for `navItem`; route composition passes a
  typed `PageHeaderBack` descriptor and renders `PageHeader` only through plan
  149's `@/shared/components/page-header` facade.
- Render `RangePicker` only through plan 149's `@/features/time-range` facade.
  Plan 100 remains the owner of pure range parsing/merge/search, clock, and
  formatting; Plan 152 owns GraphQL transport/cache compatibility and Plan 153
  owns external search/layout JSON decoding.
- Use explicit named value/type imports from plan 149 facades. Do not deep-import
  internals, use wildcard barrels, copy capability implementations, or expect
  plan 143 to repair completed dashboard capability imports.
- Keep range search in the route/domain boundary and list/detail URL differences
  characterized. No feature import of a route definition is allowed.
- Preserve `graphqlCached` for list/detail/series and raw `graphql` for label/
  value/mutations. Do not create `queries/`, QueryClient, query keys,
  hydration/prefetch, or a second cache. Plan 133 owns the later transition.
- Plan 143 must import `loadDashboardNavigation` and
  `DashboardNavigationItem` from the facade, preserve raw/uncached selection,
  path trigger, abort, retained items, and error behavior, then delete the exact
  exception. Until then, `parallax-shell.tsx` retains that expiring duplicate-
  query exception; this plan does not modify shell/layout files. If the unused-
  facade ratchet counts these two handoff exports before plan 143 consumes them,
  record one exact plan 143-owned expiry/removal exception or serialize this
  facade commit immediately before plan 143. Never broaden the unused-export
  allowlist.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Architecture | `cargo xtask arch` | no route cycle/deep import/unknown edge/unclassified file |
| UI policy | `cargo xtask policy --only ui.architecture` | facade/runtime/layout/test boundaries pass |
| Test policy | `cargo xtask policy --only ui.tests` | matrix and `tests/` ownership pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/module/function/complexity/export budgets shrink or hold |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/dashboards src/routes/tests/dashboard-routes.test.tsx` | selected non-empty suite passes |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass without unexpected diagnostics |
| Browser contract | `cd ui && bun run test:browser -- --grep @dashboards` | registered dashboard product cases pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @dashboards` | non-zero managed GreptimeDB + Turso cases pass |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @dashboards && bun run test:browser:a11y -- --grep @dashboards && bun run test:browser:visual -- --grep @dashboards` | non-zero cross/mobile/a11y/visual rows pass |
| Browser policy | `cargo xtask policy --only ui.browser-contracts` | matrix/spec/locator/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | storage/seed/lifecycle/matrix ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | engine/mobile/a11y/visual rows and goldens pass policy |
| Format | `cd ui && bun run check` | exit 0 |
| Oxc lint | `cd ui && bun run lint` | zero warnings |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Production build | `cd ui && bun run build` | exit 0; generated routes/chunks current |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | real-stack and selected breadth lanes are green |

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/dashboards.spec.ts` owns the `@dashboards` managed-
stack row. Reuse plan 145's public-OTLP metric identity, wait for public metric
visibility, create a dashboard/widget, verify real chart points and label/group
options, edit/reorder/save, then open a fresh BrowserContext and prove Turso
persistence before delete. Keep the project one worker with managed GreptimeDB
plus isolated Turso; use UI/public GraphQL only and do not add export behavior.

**Verify**: `cd ui && bun run test:browser:full -- --grep @dashboards` selects at
least one plan-137 matrix row and passes with bounded readiness, no browser
response interception/direct database access, and clean lifecycle teardown.

## Feature Browser Breadth

This plan owns every `@dashboards` row that consumes plan 146's projects. Run
list/detail, create/edit/reorder/delete, range, label/value, and chart workflows
in Firefox and WebKit. Cover cards, editor controls, menus/dialogs, chart
containment, long metric/group labels, and reordering interactions on both
mobile device projects. Run axe plus keyboard/focus/Escape/restoration checks.
Keep canonical list, detail chart, editor, empty, and recoverable-error visual
states; do not invent an export action.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @dashboards && bun run test:browser:a11y -- --grep @dashboards && bun run test:browser:visual -- --grep @dashboards` selects non-zero owned rows and passes without broad chart masking, response interception, or an unplanned product control.

## Scope

In scope:

- both dashboard routes, their plan-129 dashboard test handoff, feature-owned
  wire/model/layout/series/error/UI responsibilities, facade and route tests;
- shared dashboard DTO cleanup where callers move, exact architecture/ratchet/
  matrix changes, plan 144 dashboard datasets/screens/contracts, plan 145
  full-stack spec, and plan 146 breadth files;
- an explicit facade contract usable by later plan 143 without a deep import.

Out of scope:

- `parallax-shell.tsx`, shell tests/navigation/layout migration (plan 143),
  except the exact temporary ownership record; plan 129's SQL test handoff
  (plan 135); Query/cache migration (plan 133), live-data
  algorithms (plan 147), and bundle/performance work (plan 148);
- backend/GraphQL schema changes, filter semantics not currently sent to series,
  dashboard sharing/import/export as new product features, new chart libraries,
  new aggregations/chart types, visual redesign, or persistence-version change;
- real-engine and browser-project infrastructure, Node, foreign package managers,
  internal npm packages, `__tests__`, catch-all modules, route implementation
  exports, unsafe DTO casts, or permanent broad exceptions.

## Git Workflow

- Stay on the current single branch. Never create another branch or PR.
- Plan 129 owns the mechanical mixed-file split. After that prerequisite, plan
  137 owns only `ui/src/routes/__tests__/-dashboards.test.tsx`; plan 135 owns
  only its separate SQL handoff. The feature plans may execute in parallel and
  never write the same legacy test file.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any surviving dashboard slice in
  `ui/src/lib/api.ts` are serialized feature-scoped commits. Re-read the current
  file, require no uncommitted writer, patch only dashboard rows/types, commit
  green, then hand off. Do not regenerate or replace another feature's content.
- Land decoded models/APIs, UI/facade/routes, and cleanup/evidence as focused
  green commits; push every durable update.
- Use Conventional Commits and exactly one required agent-product trailer.

## Steps

### Step 0: Prove prerequisites and freeze compatibility

Confirm plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete. Run the drift
check and this exact prerequisite-only subset:

```bash
cargo xtask arch
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cargo xtask policy --only ui.browser-contracts
cargo xtask policy --only ui.browser-full-stack
cargo xtask policy --only ui.browser-breadth
cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build
cd ui && bun run test:browser:list
cd ui && bun run test:browser:foundation
cargo xtask ui-browser full-stack preflight
cargo xtask ci --fast
```

Do not run focused target paths or `--grep @dashboards` commands until Steps
3-6 create/register them; zero selection is intentionally fatal. Confirm plan 129
deleted `ui/src/routes/__tests__/-final-sweep.test.tsx`, moved
every dashboard case/stable ID unchanged to
`ui/src/routes/__tests__/-dashboards.test.tsx`, and handed SQL cases to a
different exact file. Plan 137 owns only the dashboard handoff. Confirm plan
149's final page-header/time-range facades, Plan 100's pure-range/clock
contracts, Plan 152's GraphQL generator/transport and exact dashboard handoff
rows, and Plan 153's search/persisted-JSON mechanisms. Plan 152's dynamic
widget-series files already exist; create each static product operation here
through its exact template.

Plan 129's completed state must retain one exact expiring private-route/topology
exception for `-dashboards.test.tsx`, owned by plan 137 and removed in Step 5.
The shell test exception remains owned by plan 143. If either legacy test lacks
its exact owner/expiry/removal step, stop because the prerequisite graph is
inconsistent.

Plan 145 must reserve the `@dashboards` managed-stack stable IDs for
`full-stack/dashboards.spec.ts`; its shared storage-composition and seed/
readiness infrastructure cannot assert the dashboard workflow. Consume the
delegated row instead of duplicating it.
Record exact GraphQL operations, cached/raw split, request counts, URLs/search,
all visible states, navigation/invalidation, label clock/late-result behavior,
layout fixtures, series chunks/aliases, charts, and shell duplicate owner.

Characterize legacy layouts beyond the existing two tests: invalid JSON,
non-array, non-object, missing/non-string metric, missing/invalid known fields,
unknown nested fields, width, group/filter, and exact parse-serialize output.
Runtime safety must be grounded in these fixtures, not an assumption.

Add stable matrix ownership for list/detail/search, CRUD/edit/reorder, layout
compatibility, labels/late completion, series chunk/group/alias behavior,
and charts/empty/error/not-found. Confirm there is no export row or control.

**Verify**: test-matrix policy and all baseline UI tests pass;
`test ! -e ui/src/routes/__tests__/-final-sweep.test.tsx` succeeds, all dashboard
IDs are in `-dashboards.test.tsx`, and each risk has one owner.

### Step 1: Extract persisted layout and pure series models

Create readonly widget/dashboard values and pure layout functions. Preserve
unknown properties and exact valid-layout serialization. Safely classify known
fields without assertions. If malformed known fields currently reach rendering,
capture the baseline and implement a typed compatibility result; do not coerce
or discard silently.

Keep Plan 152's dynamic document/alias adapter unchanged and move only pure
series/domain mapping into bounded modules. Test
zero widgets; 1, 24, 25, 48, and 49 widgets; absolute aliases; missing alias;
input/result order; 0/1/5/6 groups; null/duplicate group names; unordered/
duplicate timestamps; large nanosecond sort; empty points; every chart model.

**Verify**: focused model tests and typecheck pass; golden layout fixtures
round-trip unknown fields and chunk tests prove exact request count/order.

### Step 2: Generate static operations and complete decoded APIs

Create one named `.graphql` document and checked-in generated sibling for every
static list/detail/save/delete/metric metadata operation. Parse generated output
from `unknown`, map once, and project typed errors. Consume Plan 152's existing
widget-series AST/alias decoder without a `Record<string, Series[]>` trust cast.
Preserve cached/raw selection, exact 24 chunking, fields, clock range, empty-on-
label-error behavior, and ignored late response lifecycle.

API tests cover valid/empty/null/malformed/error/cancellation classifications,
exact requests, not-found distinction, mutation results, label/value inputs, and
series alias failures. Do not hide malformed dashboard/detail/series data as an
empty state unless baseline explicitly does.

**Verify**: focused API tests, architecture policy, lint, and typecheck pass.

### Step 3: Split list/create/widget presentation

Create dashboards page/cards/create dialog/widget picker and focused options
hook. Preserve empty/list/delete errors, confirmations, relative/widget counts,
range links, create filtering/reset/navigation, labels/values/fallback options,
and all accessibility names. Keep late-completion guards or equivalent
cancellation ownership in the hook.

Use exhaustive async/mutation states rather than independent booleans. Split
cohesive responsibilities so every function/component/hook is at most 60 lines
and no handwritten file exceeds 300; do not paste the old 525-line route.

**Verify**: component/integration tests for list/create/delete/labels/late
completion/error/accessibility pass with fake clock and no unexpected diagnostic;
ratchets remain green.

### Step 4: Split detail editor and chart presentation

Create detail page/editor/chart and focused editor hook. Preserve loader-to-draft
identity behavior, add/remove/move boundaries, cancel reset, save/invalidate,
delete/navigation, empty widgets, range changes, chart variants/group legend,
wide layout, labels/counts, and inline errors. Pass a typed back descriptor from
route composition instead of importing layout.

Add component/integration tests for edit/add/remove/first-last move/no-op,
cancel/save failure/success, delete failure/success, no widgets, 1/5 groups,
line/area/bar, wide widget, and range callback.

**Verify**: focused detail/chart tests, lint, typecheck, and ratchets pass.

### Step 5: Publish the facade, thin both routes, and migrate tests

Create explicit `index.ts`. Convert both route files to search/params/loader/
not-found/composition adapters exporting only `Route`. Remove the sibling-route
import. Move the complete post-plan-129 `-dashboards.test.tsx` handoff into
feature/route tests using the shared harness and public contracts; leave plan
135's SQL handoff untouched. Add facade type/export tests and negative deep-
import policy fixtures.

Keep the shell query/test unchanged and record its exact plan 143 removal owner.
Publish and facade-test `loadDashboardNavigation({ signal })` with raw uncached
`id`/`name` semantics so plan 143 has a precise replacement contract.

**Verify**: architecture, focused route/facade tests, full typecheck, and build
pass. Both route files export only `Route`; route-to-route and external internal-
feature import searches return no matches.

### Step 6: Complete browser contracts and remove obsolete owners

Add deterministic plan 144 cases for list/empty/error, create/detail/not-found,
widget add/edit/reorder/remove/save/cancel, delete, range persistence, label/
value options, and line/area/bar rendering. Use public HTTP/GraphQL, semantic
locators, and observable assertions without interception, fixed sleeps, or an
invented export action.

Implement the Feature Real-Stack Contract and Feature Browser Breadth sections
in the exact target files, register each non-empty project row once, and keep
shared plan 145/146 fixtures read-only.

Delete obsolete route exports/shared DTOs only after `rg` proves no caller.
Update matrix and exact route/module/function/complexity/export/deep-import
ratchets. Retain only the scoped shell exception with plan 143 owner/expiry.

**Verify**: fixture-backed/full-stack/breadth `@dashboards` commands and
policies, all Vitest, architecture, test policy, and ratchet gates pass; old
route test imports and sibling route imports are gone.

### Step 7: Run the complete final gate twice

Run every Commands-table entry twice from clean state. The second run must not
change generated routes, matrix data, snapshots, or tracked files.

**Verify**: every command exits 0 twice and `git diff --check` is clean.

## Test Plan

- Layout/model: invalid/non-array/legacy records, known/unknown property round
  trips, defaults/edit/order, search normalization, exact serialization.
- Series/API: exact documents/fields/errors, zero and 24-boundary chunks,
  aliases/order/missing aliases, group cap/names, timestamp merge/sort/format,
  cached/raw request parity.
- Metadata: metric labels/values, current clock input, empty-on-error, ignored
  late completion, selected missing option preservation.
- Components/integration: list/empty/create/delete, cards/range, editor add/
  remove/move/save/cancel/delete/errors, every chart and empty data.
- Routes: list numeric/string search, detail search/loader deps, not-found,
  direct navigation, range changes, boundaries, only public facade imports.
- Facade/type: exact public export set; documents/schemas/hooks/components remain
  private and deep imports fail.
- Browser: deterministic dashboard CRUD/widget/range/chart/error flows and proof
  that no unplanned export control appears.
- Real stack: public-OTLP metric chart/label behavior and dashboard persistence
  against managed GreptimeDB plus isolated Turso.

## Done Criteria

- [ ] Dashboard API/model/UI/tests have one feature owner; both routes are thin,
  export only `Route`, and have no route-to-route or deep-feature imports.
- [ ] Every external envelope and alias response is decoded from `unknown` once
  and mapped once; persisted layout strings use the one model decoder with the
  characterized fallback; expected failures use `DashboardError`.
- [ ] Unknown legacy widget properties round-trip; characterized valid layouts,
  URLs/search, labels, chunks/groups, charts, states, navigation, and cached/raw
  behavior match baseline.
- [ ] Tests live only under `tests/`; the plan 129 `-dashboards.test.tsx` handoff
  is removed without changing SQL tests; facade/model/API/widget/browser
  evidence is green.
- [ ] Shell remains untouched with one exact plan 143 handoff; the facade
  exposes tested raw `loadDashboardNavigation({ signal })` and
  `DashboardNavigationItem` contracts with the current partial selection.
- [ ] No dashboard export/download action, abstraction, or browser matrix row is
  added by this structural migration.
- [ ] Dashboard-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and
  canonical visual rows are non-empty and green.
- [ ] The feature-owned `@dashboards` managed-stack row is non-empty, uses only
  public boundaries, and passes with clean teardown.
- [ ] Oxc, Vitest, browser, build, architecture/test/ratchet, and aggregate gates
  pass twice with shrink-only budgets.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/features/time-range` `RangePicker` or
  `@/shared/components/page-header` `PageHeader`/`PageHeaderBack` contract is
  absent/incompatible; do not copy a legacy component, deep-import internals,
  add a wildcard export, or defer import repair to plan 143;
- a prerequisite, runtime schema mechanism, range/platform owner, test matrix,
  or browser fixture is absent/red;
- plan 145 lacks the delegated `@dashboards` reservation or shared managed-stack
  infrastructure, or Step 6 cannot turn that reservation into a non-empty
  public-boundary row with clean one-worker teardown;
- plan 145's shared specs already own the same dashboard stable ID/behavior and
  cannot hand it off without duplicate matrix ownership;
- plan 129 is marked complete while a dashboard legacy/private-route test
  remains without its exact plan 137 or plan 143 expiring topology exception;
- plan 129 leaves the mixed `-final-sweep.test.tsx`, loses a dashboard case/ID,
  or hands one legacy file to both plans 135 and 137;
- plan 143 concurrently owns shell/facade lines needed here without a serialized
  handoff;
- legacy layout safety cannot preserve unknown properties/current valid output
  without an unsafe cast, silent data loss, or unapproved version migration;
- drift changes route/search, fields, cache/raw split, invalidation, chunk size,
  group cap, clock/late response behavior, charts, mutation, or navigation;
- implementation requires Query/cache migration, backend/schema changes, new
  filter/export semantics, route implementation exports, deep imports, or a
  feature-to-layout dependency;
- the dashboard-navigation handoff would require a broad/permanent unused-
  export exception or cannot be serialized immediately before plan 143;
- an oversized unchanged move, duplicate owner, `__tests__`, or broad permanent
  exception would remain; or
- a required verification fails twice after one reasonable correction.

## Maintenance And Removal

Future dashboard fields update named documents, generated result schemas,
feature-owned layout schemas, domain mappers,
typed errors, layout compatibility fixtures, matrix rows, and browser evidence
together. Persisted unknown properties remain protected until an explicit
versioned migration. Keep series query limits measured and tested at boundaries.
Plan 133 may later replace caching, and plan 143 must remove the shell exception
when it consumes this facade.

Delete this plan and its index row only after all done criteria are green, old
route/shared owners are removed, and durable tests/policies/matrix entries
retain the contract.
