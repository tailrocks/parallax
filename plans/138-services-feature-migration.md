# Plan 138: Move services into one bounded feature

> **Executor instructions**: Perform a move-only services refactor after the
> architecture, characterization, runtime-contract, and fixture-backed browser
> prerequisites are green. Preserve the `/services` and `/services/$service`
> URLs, search parsing, GraphQL requests and variables, loader timing, loading/
> empty/error rendering, links, cache calls, and visual behavior. Do not install
> TanStack Query or change cache ownership; plan 133 owns that later. Run each
> local verification before continuing. If a STOP condition occurs, stop and
> report instead of inventing another layer or compatibility path.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/services.tsx 'ui/src/routes/services.$service.tsx' ui/src/routes/__tests__/-services.test.tsx ui/src/features/services ui/test-matrix.json ratchet.toml`
> Plans 100, 129, and 149 intentionally relocate lower-layer imports,
> route-less capabilities, and test support. Reconcile those paths through their
> ownership ledger, but STOP if the service
> behavior or API contract differs from the current-state inventory below.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / services / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The service list and detail routes total 1,397 lines and currently own wire
requests, decoded shapes, URL state, service-domain transforms, charts, tables,
and presentation. Their tests import route implementation exports because no
feature owner exists. This plan creates one services facade and decomposes those
responsibilities without changing product or cache behavior.

## Current Paths And Responsibilities

| Current path | Current responsibility at `e3e7997` | Required final owner |
|---|---|---|
| `ui/src/routes/services.tsx` | Service list search, loader, catalog merge, sorting, table, links, empty/loading UI | Thin route plus `features/services` list modules |
| `ui/src/routes/services.$service.tsx` | Service detail loader, RED series/exemplar/release transforms, identity/runtime cards, charts and recent traces | Thin route plus `features/services` detail modules |
| `ui/src/routes/__tests__/-services.test.tsx` | List/detail transforms, links, range propagation, charts and exemplar interaction | Feature behavior under `features/services/tests/**`; route contracts under `routes/tests/` |
| Plan-152 GraphQL generator/handoff | Runtime validation template for list/detail responses | Create named operations and generated siblings under `features/services/api/` |
| Plan-153 search mechanism | Unknown-first URL search decoding | Instantiate with service-owned schemas/defaults; do not duplicate platform code |
| Plan-149 page-header/time-range/runtime-metrics facades | `PageHeader`, `RangePicker`, `RuntimeSnapshotCard`, and their minimum reviewed inputs | Consume only through the explicit final facades; do not copy or deep-import them |
| Plan-152 GraphQL/cache plus Plan-100 format/pure-range contracts | Technical platform and pure domain capability | Consume through canonical owners; do not move them into services |

The baseline route exports include `ServiceSummary`, `ServicesData`,
`ServicesSearch`, search/sort/merge helpers, `ServicesIndexContent`,
`ServiceDetailData`, RED/exemplar helpers, and `ServiceDetailContent`. The final
route files export only `Route`.

## Fixed Behavior And Ownership

1. `/services` and `/services/$service` remain the exact file-route contracts.
2. Existing search keys, accepted invalid-input behavior, custom/preset range
   propagation, sorting defaults, encoded service links, loader request count,
   and GraphQL operation/variable values do not change.
3. Current `graphqlCached` versus `graphql` selection and freshness behavior stay
   unchanged. Query keys, QueryClient, invalidation, polling, and cache repair are
   plan 133 work.
4. Plan 152's generated operation schema parses GraphQL `unknown` once. A
   services API mapper converts the decoded wire result once into readonly
   services domain values. Service search instantiates Plan 153's mechanism
   with a service-owned schema and exact existing defaults.
5. `features/services/model/services-error.ts` owns a discriminated services
   error union. API adapters map transport/decode failures once while preserving
   the current thrown `Error` boundary and visible message. A missing service
   remains the current nullable domain result and existing empty state.
6. Feature code imports lower layers or another feature only through reviewed
   facades. No services module imports a route implementation, `app`, `layout`,
   or another feature's internals.
7. Prefer pure named modules and readonly values. Add a class only if a real
   lifecycle or invariant-bearing mutable identity exists; none is expected.

## Plan 149 Capability Contract

- Services imports `PageHeader` from `@/shared/components/page-header`,
  `RangePicker` from `@/features/time-range`, and `RuntimeSnapshotCard` plus only
  its minimum readonly inputs from `@/features/runtime-metrics`.
- Those imports are explicit named value/type imports. External consumers do not
  deep-import facade internals, use wildcard barrels, copy a legacy capability,
  or wait for plan 143 to repair completed service imports.
- Plan 152 owns GraphQL transport/cache behavior, Plan 153 owns search decoding,
  and Plan 100 retains formatting, pure range, and other technical/domain foundations. The service
  feature supplies typed inputs and composition only.

## Target Tree

```text
ui/src/features/services/
  api/
    services-list.graphql
    services-list.generated.ts
    load-services.ts
    service-detail.graphql
    service-detail.generated.ts
    load-service-detail.ts
    services-mapper.ts
  model/
    service-summary.ts
    service-detail.ts
    services-search.ts
    services-search-schema.ts
    services-table-model.ts
    service-red-series.ts
    services-error.ts
  components/
    services-page.tsx
    services-table.tsx
    service-detail-page.tsx
    service-identity-card.tsx
    service-release-strip.tsx
    service-red-charts.tsx
    service-recent-traces.tsx
  tests/
    api/services-api.test.ts
    model/services-search.test.ts
    model/services-table-model.test.ts
    model/service-red-series.test.ts
    components/services-page.test.tsx
    components/service-detail-page.test.tsx
  index.ts
ui/src/routes/
  services.tsx
  services.$service.tsx
  tests/services-routes.test.tsx
ui/tests/e2e/
  datasets/services.ts
  screens/services-screen.ts
  contracts/services.spec.ts
  full-stack/services.spec.ts
  accessibility/services-accessibility.spec.ts
  mobile/services-mobile.spec.ts
  visual/services.visual.spec.ts
  visual/goldens/
    services-list.png
    services-detail.png
    services-empty.png
    services-error.png
```

Plan 152 provides the generator/template and handoff rows, not these product
files. This plan creates each named operation and exact `.generated.ts` sibling;
`services-search-schema.ts` separately instantiates Plan 153.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | no cycle, forbidden edge, route-to-route import, deep consumer import, or unclassified services file |
| UI architecture | `cargo xtask policy --only ui.architecture` | services facade, route-only export, runtime boundary, and test topology pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/module/function/export rows shrink and no exception grows |
| Test ownership | `cargo xtask policy --only ui.tests` | services feature and route-contract matrix ownership pass |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/services/tests src/routes/tests/services-routes.test.tsx` | non-zero selected services tests pass with no unexpected diagnostic |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 and zero warnings/errors |
| Unit suite | `cd ui && bun run --bun test:ci` | all tests pass under Bun; no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @services` | non-zero fixture-backed services rows pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @services` | non-zero managed GreptimeDB + Turso services rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross -- --grep @services` | non-zero Firefox/WebKit/mobile services rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y -- --grep @services` | non-zero axe/keyboard/focus services rows pass |
| Visual | `cd ui && bun run test:browser:visual -- --grep @services` | non-zero canonical services visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | services matrix/spec/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | services storage/seed/lifecycle ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | services engine/mobile/a11y/visual ownership passes |
| Production build | `cd ui && bun run build` | exit 0; generated route tree current; URLs unchanged |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | services real-stack and breadth lanes are green |

All JavaScript/TypeScript commands resolve exact lock-local tools under Bun with
auto-install disabled. Node, npm, pnpm, yarn, ESLint, and a second formatter or
parser are not valid fallbacks. Oxc-backed xtask policy is the authoritative
source graph and structural oracle.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/services.spec.ts` owns plan 145's delegated, non-empty
`@services` row. Seed two related services, RED signals, one recent trace, and a
metric exemplar through public OTLP using `datasets/services.ts`; wait on named
public GraphQL predicates; then drive list search/range, encoded detail
navigation, RED charts, recent-trace/exemplar links, and the current absent-
infrastructure state through `screens/services-screen.ts`. Plan 145's distinct
`@storage` discovery/link smoke remains shared foundation evidence and is not
reimplemented here.

Run one worker against managed GreptimeDB plus an isolated Turso database. Use
only public OTLP, GraphQL, and UI boundaries with bounded readiness predicates;
never write/read database internals, intercept browser responses, or use fixed
sleeps.

**Verify**: `cd ui && bun run test:browser:full -- --grep @services` selects at
least one plan-138 row and passes with the real-stack runtime manifest and clean
process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@services` row that consumes plan 146's projects. Run
service list/detail, search/range, empty/error, exemplar/trace links, and absent
infrastructure-band behavior in Firefox and WebKit. Cover dense tables, charts,
long service metadata, tap navigation, and overflow on both mobile device
projects. Run axe plus keyboard/focus checks and maintain canonical list, detail,
empty, and error visual states with deterministic data.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @services && bun run test:browser:a11y -- --grep @services && bun run test:browser:visual -- --grep @services` selects non-zero breadth rows and passes without broad chart masking or response interception.

## Scope

**In scope:**

- The two current services route modules and their service-owned responsibilities.
- `ui/src/features/services/**` creation and explicit facade.
- Migration of existing services tests into `features/services/tests/**`.
- Service rows in `ui/test-matrix.json` and exact services rows in `ratchet.toml`.
- Feature-owned services dataset/screen/contract/full-stack/accessibility/mobile/
  visual/golden files and their non-empty plan 144-146 matrix rows.
- Tool-generated `routeTree.gen.ts` refresh only when the normal build changes it;
  never edit generated code manually.

**Out of scope:**

- TanStack Query, cache keys/invalidation/freshness, `graphqlCached` deletion, or
  request-count changes; plan 133 owns them.
- Backend GraphQL/schema changes, URL/search redesign, new service functionality,
  chart redesign, live-data optimization, or metric stub completion.
- Reassigning Plan-100 technical/domain ownership or Plan-149 route-less
  capability ownership, editing shadcn primitives, creating internal packages/
  project references, or adding a generic `types.ts`, `helpers.ts`, `common.ts`,
  or `utils.ts` bucket.
- Other feature source/tests.
- Shared plan 144-146 Playwright configuration, fixtures, reporters, lifecycle,
  CI, matrix schema, and browser infrastructure; consume them read-only.

## Git Workflow

- Stay on the single active branch in `AGENTS.md`; do not create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any shared generated registry/config
  are serialized feature-scoped commits. Re-read current content, require no
  uncommitted writer, change only services rows, land green, then hand off.
  Never regenerate or replace another feature's content.
- Keep model/API extraction, component splits, and route/test closure as separate
  reviewable green changes.
- Use Conventional Commits, DCO, and exactly one agent-product trailer; push each
  durable green update as required by repository policy.

## Steps

### Step 0: Reproduce the service contract

Confirm plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete. Run the drift check
and this exact prerequisite-only subset:

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

Do not run focused target paths or any `--grep @services` command until Step 5
creates and registers those files; zero selection is intentionally fatal. Locate
Plan 152's generator and exact service handoff rows, Plan 153's search decoder,
Plan 149's final page-header, time-range, and runtime-metrics facades, Plan 100's technical/pure-domain
facades, and all service rows in `ui/test-matrix.json`. Record current URL/search
examples, GraphQL operation and variable values, request counts, custom-range
links, loading/error/empty states, and rendered browser markers before moving
code.

Require every services `__tests__` path and private route import to have an exact
plan-129 legacy handoff owned by plan 138. Stop on a missing, wildcard, expired,
or differently owned row; delete each row when its test/import moves.

Confirm plan 145 reserves `@services` for
`ui/tests/e2e/full-stack/services.spec.ts` and that shared `@storage` specs own
only foundation discovery/link behavior. Consume that reservation; do not add a
second stable ID for the same feature scenario.

**Verify**: every prerequisite command above exits 0, the legacy handoff is
exact, and the delegated services row is reserved but not yet required to select
a feature spec.

### Step 1: Extract the readonly services model

Move list/detail domain shapes, search parsing/patching, catalog merge, sorting,
error-rate calculations, URL construction, RED series calculations, latency
bands, exemplars, release windows, and the typed error union into cohesive model
modules. Keep timestamps as strings and range behavior exact. Do not move wire
DTOs into `model/` or invent a generic type bucket.

Add model tests by moving, not duplicating, the existing assertions. Include
garbage search values, every sort, catalog missing fields, encoded service names,
empty/zero RED series, latency band clamping, and exemplar marker ordering.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/services/tests/model && bun run typecheck`
must pass with non-zero tests; `cargo xtask policy --only ui.ratchets` must show
the original route rows shrinking.

### Step 2: Move decoded API adapters

Create the named list/detail operations and checked-in generated siblings under
service API ownership. Implement one list adapter and one detail adapter that call the
current Plan-152 transport/cache-preserving facade, parse `unknown`, map once to
the readonly model, and map failures to the services error union before restoring
the existing route error boundary. No raw query string, manual value escaping,
generic `as T`, duplicated schema, cache, or new request may remain in a route or
component.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/services/tests/api && bun run typecheck`
must cover valid, missing, null, malformed, GraphQL-error, abort, and not-found
cases. Recorded request documents, variables, count, and cache calls must match
Step 0 exactly.

### Step 3: Split service presentation by responsibility

Move the list page/table and detail page sections into the target components.
Keep page headings, navigation, accessible names, table columns, chart series,
release/identity/runtime sections, exemplar popovers, empty/loading/error states,
and link/search values unchanged. Consume only feature-internal modules, the
feature facade where external, Plan-149 route-less capability facades, and
Plan-100 technical/pure-domain facades. Compose `PageHeader`, `RangePicker`, and
`RuntimeSnapshotCard` through their exact public paths. Do not copy runtime,
range, formatting, chart, or shared UI implementations.

Each component/function must meet the 60-line target or have a pre-existing exact
shrink-only row. No handwritten module may exceed 300 logical lines after this
fully restructured feature.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/services/tests/components && bun run check && bun run lint`
must pass; component semantics and browser screenshots/geometry must match the
fixture-backed baseline.

### Step 4: Publish the facade and thin both routes

Create `features/services/index.ts` with explicit named value/type exports only.
Export the two route entry components/loaders/search contracts required by the
routes and only genuinely reviewed cross-feature contracts. Wildcard exports and
consumer deep imports fail policy.

Rewrite both route files as adapters containing route creation, parameter/search
binding, loader wiring, boundary selection, and top-level composition only.
Each route exports only `Route`; remove all implementation exports. Neither route
may import another route or a feature-internal path.

**Local verification:**
`cargo xtask arch && cargo xtask policy --only ui.architecture && cd ui && bun run build`
must prove facade-only imports, only-`Route` exports, both unchanged route IDs,
and no services implementation pulled into another route entry.

### Step 5: Finish separated tests and structural closure

Move service API/model/component behavior into `features/services/tests/**` and
URL/search/loader/error/navigation contracts into
`routes/tests/services-routes.test.tsx`. Switch private route imports to feature
public contracts or public router behavior and delete the old route test file.
Preserve stable matrix IDs/assertions; add only missing API mapper/error, facade,
and route boundary cases. Do not create another `__tests__` directory.

Create or extend the exact feature-owned `datasets/services.ts`,
`screens/services-screen.ts`, fixture contract, full-stack, accessibility,
mobile, visual, and named golden files in the Target Tree. Consume plan 145's
reserved `@services` row, register each feature matrix ID/project once, and make
every grep-scoped selection non-empty. Shared plans 144-146 fixtures,
configuration, reporters, lifecycle code, and infrastructure remain read-only.

Update exact ratchet rows: both routes must be at or below 150 logical lines;
all new handwritten modules at or below 300; functions/components/hooks at or
below 60; complexity at or below 12/15; no new facade export, assertion,
suppression, deep-import, or test-layout exception.

**Local verification:** run the complete command table twice. Every
`--grep @services` selection must be non-zero, `git diff --check` must be clean,
and `git status --short` must show no file outside this plan's scope.

## Test Plan

- `tests/api/services-api.test.ts`: valid/null/malformed/error/cancel list and
  detail payloads, one decode/map, exact operation/variables/request count.
- `tests/model/services-search.test.ts`: garbage/default/custom-range search,
  patching, encoded links, and every sort.
- `tests/model/services-table-model.test.ts`: catalog merge, missing metadata,
  error rate, stable ordering, and empty rows.
- `tests/model/service-red-series.test.ts`: totals/latest values/latency bands,
  zero/empty inputs, releases, and exemplar markers.
- `tests/components/services-page.test.tsx`: list table, loading, empty, links,
  range propagation, and accessible controls.
- `tests/components/service-detail-page.test.tsx`: found/not-found, identity,
  runtime, RED charts, release strip, recent traces, and exemplar interaction.
- `routes/tests/services-routes.test.tsx`: exact URLs/search/loader/error
  boundaries and client navigation using public route behavior.
- Fixture browser: deterministic list/detail/search/range, empty/error, and
  trace/exemplar link contracts through `@services`.
- Real stack: public-OTLP service/RED/exemplar/recent-trace behavior against
  managed GreptimeDB plus isolated Turso.
- Browser breadth: selected Firefox/WebKit/mobile behavior, axe/keyboard/focus,
  and named canonical service visuals.

## Done Criteria

- [ ] Both service routes export only `Route`, remain at their current URLs, and
  are at or below 150 logical lines.
- [ ] All service-owned API/model/component code lives under
  `features/services`, with explicit facade-only external imports.
- [ ] Every wire result is runtime-decoded once, mapped once, and owns a typed
  services error path without changing visible errors or not-found behavior.
- [ ] Search, requests, loading/error/empty states, cache calls, links, charts,
  and browser behavior match the recorded baseline.
- [ ] Feature tests live under `features/services/tests/**`, route contracts live
  under `routes/tests/`, and neither imports a private route implementation.
- [ ] Services-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and
  canonical visual rows are non-empty and green.
- [ ] The feature-owned `@services` managed-stack row is non-empty, uses public
  OTLP/GraphQL/UI boundaries, and passes against GreptimeDB + Turso.
- [ ] Oxc architecture, test, ratchet, format, lint, typecheck, unit, browser,
  build, and fast/full aggregate gates pass twice.
- [ ] No class, catch-all module, wildcard facade, duplicate schema, Query/cache
  change, or unrelated feature edit was introduced.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`,
  `@/features/time-range` `RangePicker`, or
  `@/features/runtime-metrics` `RuntimeSnapshotCard` facade/input contract is
  absent/incompatible; do not copy a legacy component, deep-import internals,
  add a wildcard export, or defer import repair to plan 143;
- a prerequisite is incomplete or the forced-Bun/browser services baseline is red;
- plan 145 lacks the delegated `@services` reservation/shared managed-stack
  infrastructure, or Step 5 cannot make it a non-empty public-boundary row with
  clean one-worker teardown;
- a shared `@storage` spec or another feature owns the same services stable ID/
  scenario, or the reservation points at a different file;
- feature browser evidence requires editing shared plans 144-146 fixtures,
  configuration, lifecycle, reporters, CI, or matrix schema;
- live URLs/search/request/cache/error/rendering behavior differs from this plan's
  inventory before the move;
- Plan 152's generator/handoff cannot represent a frozen service operation,
  Plan 153's search mechanism is absent, or a second decoder appears necessary;
- a service module requires a forbidden upper/deep/cyclic import and composition
  cannot remove it;
- preserving behavior appears to require changing GraphQL/backend contracts,
  Query/cache semantics, generated code by hand, or another feature;
- a new module/function cannot meet structural limits without arbitrary
  fragmentation; or
- a required local/full gate fails twice after a reasonable correction.

## Maintenance And Required Deletions

Future service changes enter through the explicit facade, update their decoded
schema/domain mapper/error, tests, matrix IDs, and ratchets together. Plan 133 may
later add `features/services/queries/` and change loader cache ownership; it must
not move these modules again.

Delete before retiring this plan:

- `ui/src/routes/__tests__/-services.test.tsx`;
- every service implementation export from both route files;
- every temporary old-path service reexport created during the move; and
- every service-specific migration exception/ledger row whose target now exists.

Delete this plan and its README row only after those deletions and all done
criteria are durable and green.
