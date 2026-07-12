# Plan 150: Move overview into one bounded feature

> **Executor instructions**: Move only the overview implementation after the
> architecture, runtime-contract, test, browser, and route-less capability
> foundations are green. Preserve the `/` route, search/range behavior, request
> document/variables/count/cache calls, loader and error boundaries, all summary
> and series calculations, sample/onboarding behavior, links, accessible output,
> and visuals. Consume Plan 149 facades rather than moving or copying their code.
> Materialize Plan 145's exact `@overview` reservation in
> `full-stack/overview.spec.ts`; do not duplicate the `@storage` discovery flow.
> Do not change app/layout/shell, Query/cache, live algorithms, or bundle behavior.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/index.tsx ui/src/routes/__tests__/-overview.test.tsx ui/src/components/console/top-movers.tsx ui/src/components/console/__tests__/top-movers.test.tsx ui/src/features/overview ui/src/features/runtime-metrics ui/src/features/time-range ui/src/shared/components/page-header.tsx ui/test-matrix.json ui/tests/e2e/datasets/overview.ts ui/tests/e2e/screens/overview-screen.ts ui/tests/e2e/contracts/overview.spec.ts ui/tests/e2e/full-stack/overview.spec.ts ui/tests/e2e/accessibility/overview-accessibility.spec.ts ui/tests/e2e/mobile/overview-mobile.spec.ts ui/tests/e2e/visual/overview.visual.spec.ts ui/tests/e2e/visual/goldens ratchet.toml`
> Reconcile Plan-152 handoff rows, Plan-129 handoffs, Plan-145 reservation IDs,
> and Plan-149 facade paths with the live ledger. Stop on an ownership conflict or
> observable overview drift before editing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / overview / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The overview route is a large mixed owner for data access, range state, series
transforms, charts, top movers, recent records, onboarding, and presentation.
Its tests import route implementation exports because no feature boundary exists.

Plan 149 removes the cross-feature capability blocker first. This plan can then
move overview as one behavior-preserving feature, leave a thin file route, and
publish complete unit, fixture-backed, real-stack, cross-browser, mobile,
accessibility, and visual evidence without absorbing shell or final-closure work.

## Fixed Decisions

1. `features/overview` owns the overview GraphQL document/schema/decoded adapter,
   readonly models, pure transforms, presentation, feature tests, and explicit
   facade. `routes/index.tsx` retains only URL/search/loader/boundary/composition.
2. The Plan-152-generated operation/schema parses GraphQL `unknown`;
   one mapper creates readonly overview values. Overview search instantiates
   Plan 153's mechanism with its own schema and unchanged defaults. Do not
   duplicate a generated document, schema, DTO, or raw cast.
3. Preserve the exact previous/current range calculation, request fields,
   variables, request count/order, `graphqlCached` choice, loader timing, route
   error behavior, and sample-data contract. Plan 133 changes cache ownership later.
4. Preserve signal and latency series, band clamping, totals, visible-series
   toggles, brush windows, top-mover thresholds/ranking/sentences, recent issue
   and trace links, loading/empty/error/onboarding/sample states, all text,
   accessible names, geometry, and responsive visuals.
5. Range selection, runtime metrics, and page headings come only from Plan 149's
   facades. Overview may consume their public contracts but cannot deep-import,
   re-export, or take ownership of them.
6. The route exports only `Route`. It imports only the overview facade plus
   approved route/domain/shared contracts, never an overview internal, platform,
   another route, app, or layout.
7. Source tests live under `features/overview/tests/**` or `routes/tests/**`.
   Playwright files use the one Plans 132/144-146 stack and stable matrix IDs.
8. `full-stack/overview.spec.ts` consumes Plan 145's exact `@overview` reserved
   row. It reuses the public-OTLP seed/readiness owner and asserts overview-specific
   values; it does not repeat the `@storage` cross-route discovery scenario.
9. Plan 143 owns app/layout/shell, Plan 151 owns final architecture verification,
   Plan 147 owns live performance, and Plan 148 owns chunks/bundles.

## Target Ownership

```text
ui/src/features/overview/
  api/
    overview.graphql
    overview.generated.ts
    load-overview.ts
    overview-mapper.ts
  model/
    overview-summary.ts
    overview-series.ts
    overview-range.ts
    service-movers.ts
    overview-error.ts
  components/
    overview-page.tsx
    overview-signal-trends.tsx
    overview-latency-trends.tsx
    overview-top-movers.tsx
    overview-recent-issues.tsx
    overview-slowest-traces.tsx
    overview-onboarding.tsx
  tests/
    api/overview-api.test.ts
    model/overview-series.test.ts
    model/service-movers.test.ts
    components/overview-page.test.tsx
  index.ts
ui/src/routes/
  index.tsx
  tests/overview-route.test.tsx
ui/tests/e2e/
  datasets/overview.ts
  screens/overview-screen.ts
  contracts/overview.spec.ts
  full-stack/overview.spec.ts
  accessibility/overview-accessibility.spec.ts
  mobile/overview-mobile.spec.ts
  visual/overview.visual.spec.ts
  visual/goldens/
```

Preserve Plan-152 generator-owned names when different. Additional modules must
name a cohesive responsibility and satisfy structural limits; generic `types`,
`helpers`, `utils`, or `common` buckets are forbidden.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | overview facade and route edges pass with no cycle/deep/unknown owner |
| UI policy | `cargo xtask policy --only ui.architecture` | decoder, facade, runtime, and only-Route rules pass |
| Test policy | `cargo xtask policy --only ui.tests` | feature/route tests and matrix handoffs pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/module/function/export rows shrink without exception growth |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/overview/tests src/routes/tests/overview-route.test.tsx` | non-zero overview tests pass without diagnostics |
| All UI tests | `cd ui && bun run --bun test:ci` | complete suite passes under Bun; no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @overview` | fixture-backed overview states pass |
| Real stack | `cd ui && bun run test:browser:full -- --grep @overview` | exact non-zero managed-stack overview row passes |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @overview && bun run test:browser:a11y -- --grep @overview && bun run test:browser:visual -- --grep @overview` | non-zero cross/mobile/a11y/visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | dataset/screen/spec/matrix ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | reserved row consumed once; storage/lifecycle rules pass |
| Breadth policy | `cargo xtask policy --only ui.browser-breadth` | engine/device/a11y/golden ownership passes |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 with zero warnings/errors |
| Build | `cd ui && bun run build` | route tree current; `/` and SPA behavior unchanged |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | managed stack and selected breadth lanes pass |

All JavaScript/TypeScript/browser commands use exact locked tools through Bun.
Oxc-backed xtask and the single Playwright stack are authoritative. No Node,
foreign package manager, ESLint, second browser runner, response interception,
manual generated edit, or implicit install is allowed.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/overview.spec.ts` owns the exact Plan-145 `@overview`
reserved row. Reuse Plan 145's stable public-OTLP trace, log, metric, service,
issue, and run identities and bounded public readiness predicates. Open `/`,
assert overview-specific totals and populated signal/latency series, verify the
real recent issue and slow-trace links carry the expected stable IDs, and change
the range through visible controls to prove overview request/result wiring.

Do not repeat Plan 145's generic ingest-to-surface discovery or broad cross-route
walk. Do not direct-write native tables, read database files, intercept browser
responses, use fixed sleeps, or create metadata outside public UI/GraphQL setup.
The project remains one worker with managed GreptimeDB plus isolated Turso.

**Verify:** the exact `@overview` selection is non-zero, the reserved row becomes
implemented rather than duplicated, all assertions use visible UI/public typed
postconditions, and cleanup releases every process, port, and data owner.

## Feature Browser Breadth

This plan owns all `@overview` Plan-146 rows. Run populated, sample/onboarding,
empty, recoverable-error, range, series-toggle, brush, mover, and recent-link
contracts in the engines named by the matrix. Cover charts/cards/tables, long
service names, dense data, tap controls, text containment, and overflow on the
two mobile device projects. Run axe plus keyboard/focus/name/reduced-motion
checks. Maintain only canonical overview states whose matrix rows identify a
real layout risk.

**Verify:** cross/mobile/a11y/visual commands select non-zero owned rows and pass
without broad masking, coordinate-only assertions, response interception, or a
baseline update outside Plan 146's guarded canonical environment.

## Scope

**In scope:**

- `ui/src/routes/index.tsx`, overview-owned top-mover code, and their exact legacy
  source tests.
- Overview document/schema/adapter/mapper/error, pure range/series/mover models,
  presentation components, explicit facade, and thin `/` route.
- Overview source/route tests and stable matrix/ratchet ownership.
- Exact overview dataset, screen, fixture-backed contract, delegated full-stack
  spec, cross/mobile/accessibility/visual specs, and owned goldens.
- Import-only consumption of Plan-149 time-range/runtime/page-header facades.

**Out of scope:**

- App/router/root/layout/shell/nav/theme/fallbacks, app-status, quick-navigation,
  or shell browser evidence (Plan 143).
- Runtime-metrics, time-range, story, or page-header ownership changes (Plan 149).
- Generic residual bucket sweep or final documentation/ledger closure (Plan 151).
- Query/cache changes (133), live algorithm/performance changes (147), and
  route/chunk/minifier/source-map/bundle changes (148).
- Backend/API/URL/search redesign, new overview functionality, visual redesign,
  another browser/test stack, internal npm packages, or generated/shadcn edits.

## Git Workflow

- Stay on the one active branch; never create another branch or PR.
- Land model/API, presentation, route/test, browser evidence, and final cleanup as
  separate reviewable green commits.
- Serialize matrix, ratchet, E2E catalog, and generated route-tree updates with
  other active feature plans.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable update under repository policy.

## Steps

### Step 0: Prove prerequisites and freeze overview behavior

Confirm Plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete and
green. Plan 152 supplies the generator/handoff, not a pre-created product file.
Run the drift check and this prerequisite-only subset:

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

Do not run target overview paths or any `--grep @overview` command until Steps
3-5 create and register them; zero selection is intentionally fatal. Resolve
Plan-152's generator and exact overview handoff row, Plan-153 search decoding,
Plan-149 facade paths, and every Plan-129 overview handoff. Record URL/search
examples, current/previous ranges, exact request document/variables/count/cache
choice, series/bands/sample outputs, toggles/brush, movers, recent links,
loading/error/empty/onboarding text, accessibility markers, and visual geometry.

Confirm Plan 145 has exactly one reserved `@overview` row naming
`full-stack/overview.spec.ts`, and Plan 144/146 reserve the overview contract and
breadth IDs without implementing duplicate cases.

**Verify:** legacy overview Vitest tests select non-zero cases and pass; the
shared browser list/foundation and matrix policies pass; the exact `@overview`
rows remain reserved and non-selectable until Steps 3-5 create their files.
Policy rejects a missing, duplicate, wrong-file, or prematurely implemented
reservation.

### Step 1: Extract pure overview models

Move overview summary, range, signal/latency series, clamping, totals, sample-data,
visible-series, brush, and service-mover logic into cohesive readonly model
modules. Reuse Plan-100/149 domain functions only when behavior is identical;
do not duplicate range or formatting code.

**Verify:** model tests preserve every boundary, order, threshold, ranking,
sentence, sample/empty case, and link input. Ratchets show the route and old
top-mover source shrinking.

### Step 2: Move the decoded overview API

Create the named overview `.graphql` operation and checked-in generated sibling
through Plan 152. Parse `unknown`, map once to readonly domain values, and map transport/decode failures
to an exhaustive overview error while preserving the existing route boundary.
Keep exact document, variables, request count/order, `graphqlCached` call, and
sample fallback. No component or route may own raw GraphQL, wire DTOs, or casts.

**Verify:** API tests cover valid, null, malformed, GraphQL error, abort, exact
request/cache calls, and mapper outputs; typecheck and runtime policy pass.

### Step 3: Split presentation and publish the facade

Move page, signal/latency charts, top movers, recent issues/traces, and onboarding
into cohesive components. Consume Plan-149 RangePicker, runtime metric, and
PageHeader contracts where currently used. Preserve all controls, text, links,
cards/tables, chart semantics, responsive geometry, sample labels, accessible
names, and loading/error/empty/onboarding behavior.

Publish explicit route entry/load/search and presentation contracts only. Do not
export documents, schemas, internal components, or a wildcard barrel.

**Verify:** focused source/component tests, format, lint, and typecheck pass with
no interaction, request, accessibility, or visual delta. Do not select any
`@overview` browser row here; Step 5 creates and verifies all browser evidence.

### Step 4: Thin the route and move source tests

Reduce `routes/index.tsx` to route creation, search/loader dependencies, boundary
selection, and overview composition. It exports only `Route` and imports only the
overview facade plus approved route/domain/shared contracts.

Move feature behavior tests under `features/overview/tests/**` and URL/search/
loader/error/navigation tests under `routes/tests/overview-route.test.tsx`.
Preserve stable matrix IDs/assertions, remove private route imports, delete old
tests and implementation exports, and remove Plan-129 rows atomically.

**Verify:** architecture/test/ratchet policies pass, the route is at most 150
logical lines, and searches find no old overview private export or test path.

### Step 5: Materialize all overview browser evidence

Implement the dataset, screen, and fixture contract for every current overview
state using Plan-144 extension rules. Implement the exact Feature Real-Stack
Contract by converting, not copying, the Plan-145 reservation. Add only the
cross/mobile/a11y/visual rows required by the matrix and generate reviewed
goldens solely through Plan 146's canonical update path.

Run every fixture-backed, full-stack, cross-browser, mobile, accessibility, and
visual `@overview` selection only after all files and matrix rows exist.

**Verify:** all overview browser commands and policies select non-zero unique
IDs, pass in their declared projects, and leave no process, network, state, or
artifact leak. `@storage` ownership remains unchanged.

### Step 6: Delete old owners and lock the feature

Delete `components/console/top-movers.tsx`, its legacy test, the old overview
route test, all overview route implementation exports, and temporary reexports
after Oxc proves no caller. Remove exact migration exceptions and update
shrink-only route/module/function/complexity/export ratchets.

Run the complete command table twice from clean state. The second run must not
change generated routes, matrix rows, reports, or goldens.

**Verify:** every command exits zero twice, `git diff --check` is clean, and the
live ledger assigns all overview sources/tests/evidence exactly once.

## Test Plan

- API tests for exact request/cache behavior and valid/null/malformed/error/abort
  decoding and mapping.
- Model tests for ranges, previous window, signal/latency merging, clamping,
  totals, samples, toggles, brush values, and all mover thresholds/ranks/text.
- Component tests for populated/sample/loading/error/empty/onboarding, charts,
  brush/toggles, movers, recent links, controls, and accessibility.
- Route tests for exact `/`, search normalization, loader dependencies, error
  boundary, direct navigation, and SPA navigation through public behavior.
- Fixture browser contracts for all named overview states and links.
- One exact real-stack overview scenario using Plan-145 public seeds/readiness.
- Matrix-selected Firefox/WebKit/mobile/touch/axe/keyboard/focus/visual evidence.
- Policy negatives for private route export/import, deep facade access, duplicate
  reservation, response interception, broad masking, and stale golden ownership.

## Done Criteria

- [ ] The `/` route exports only `Route`, retains exact URL/search/loader behavior,
  and is at most 150 logical lines.
- [ ] All overview API/model/component ownership lives under `features/overview`
  and external imports use its explicit facade.
- [ ] Every boundary value decodes once and maps once with exhaustive overview
  errors and no duplicate schema/DTO/cast.
- [ ] Requests, cache calls, calculations, states, text, links, accessibility,
  geometry, and visuals match the frozen baseline.
- [ ] Plan-149 capabilities are consumed only through their public facades and
  were not moved or copied.
- [ ] Source tests use final feature/route topology with stable matrix IDs and no
  private route implementation import.
- [ ] The exact Plan-145 `@overview` reservation is one non-empty
  `full-stack/overview.spec.ts` row and `@storage` was not duplicated.
- [ ] Overview fixture, cross-browser, mobile, accessibility, and visual rows are
  non-empty, uniquely owned, and green.
- [ ] No app/layout/shell, Query/cache, live-performance, or bundle behavior changed.
- [ ] Every command passes twice from clean state with no generated drift.

## STOP Conditions

Stop and report if:

- a prerequisite or forced-Bun/browser/full-stack/breadth gate is incomplete/red;
- Plan 152's generator/handoff cannot represent the frozen overview GraphQL
  operation or Plan 153 lacks the search decoder mechanism;
- Plan 149 lacks an exact required facade or a deep/old-path import is necessary;
- URL/search/request/cache/calculation/render/accessibility/visual behavior has
  materially drifted before movement;
- Plan 145 lacks one exact unimplemented `@overview` reservation or its
  `@storage` foundation already asserts the same overview-specific workflow;
- preserving behavior requires app/layout/shell work, Query/cache changes, live
  optimization, bundle/chunk work, backend/API/product redesign, or another feature;
- browser evidence requires response interception, fixed sleeps, broad masking,
  another runner, Node, or an unguarded golden update;
- structural limits require arbitrary fragmentation; or
- a required gate fails twice after one reasonable correction.

## Maintenance And Removal

Future overview changes update its document/schema/mapper/error, facade, tests,
browser evidence, matrix rows, and ratchets together. Cross-feature capabilities
stay behind Plan-149 facades; new overview behavior does not expand those owners.

Delete this plan and its README row only after the old sources/tests/exports and
reservation rows are resolved, all final owners are durable, and every done
criterion and command is green.
