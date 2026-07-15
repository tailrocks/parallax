# Plan 136: Migrate ecosystem topology into a bounded feature

> **Executor instructions**: Follow this plan step by step and preserve the
> `/ecosystem` search, loader, cache, delayed-pending, graph, empty/error, and
> navigation contracts exactly. Start only after plans 100, 129, 132, 144, 145,
> 146, 149, 152, and 153 are complete and green. Extract pure topology/domain code and decoded API
> adapters before moving presentation. Keep URL ownership in the thin route,
> publish only an explicit feature facade, and delete old console owners after
> callers switch. Do not adopt TanStack Query here; plan 133 owns cache changes.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/ecosystem.tsx ui/src/components/console/ecosystem-graph.tsx ui/src/components/console/__tests__/ecosystem-graph.test.tsx ui/src/components/console/hooks.ts ui/src/lib/range.ts ui/src/lib/api.ts ui/src/routes/__root.tsx ui/test-matrix.json ui/tests/e2e ratchet.toml`
> Compare live search normalization, loader fields/cache calls, graph layout,
> links, text, and pending timing with the ledger below. Stop on mismatch.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MEDIUM
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / feature migration / architecture
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — upstream UI foundation and browser plans are incomplete

## Why This Matters

Ecosystem is small enough to look tidy, but its responsibilities are spread
across a route, a console graph component, shared API DTOs, range helpers, and a
component test under `__tests__`. The graph also embeds a topology/layout
algorithm in React and relies on URL contracts with services and traces.

This plan gives the surface one decoded API boundary, readonly domain model,
pure deterministic layout, feature-owned UI/tests, and explicit facade while
keeping the route responsible for typed URL state. It establishes a compact
feature pattern without changing data, caching, graph appearance, or links.

## Current State

At the planned baseline:

- `ui/src/routes/ecosystem.tsx` is 100 lines. It validates optional string
  `range`, `from`, and `to`; includes all three in `loaderDeps`; resolves an
  absolute nanosecond range in the loader; issues one cached service-map query;
  and composes range navigation, delayed pending, skeleton, and graph.
- Its GraphQL query sends `maxTraces: 100` and selects the current node/edge
  names, counts, errors, duration, and trace-related fields. The request and
  current module-global query-string-keyed TTL cache behavior must not change.
- Search normalization follows the existing range contract: a recognized preset
  wins over stale `from`/`to`; valid paired bounds make a custom range; invalid
  or unknown input falls back to 24h. Range changes navigate with `replace: true`
  and clear obsolete preset/custom fields.
- Pending search navigation retains the current graph for 700ms, then renders
  the current six-row table skeleton until new data arrives. Initial pending and
  loader error use the existing root boundaries.
- `ui/src/components/console/ecosystem-graph.tsx` is 234 lines. Pure
  `count`, `edgeRate`, and `layoutNodes` logic is embedded beside SVG rendering.
  Layout includes endpoint names found only on edges, supplies zero/null
  defaults, sorts deterministically, assigns depth groups, and tolerates cyclic
  or disconnected input according to current behavior.
- Empty data displays `No service edges.`. The non-empty view exposes the
  current summary and SVG accessible label, node/edge colors and widths, call/
  error/p95 formatting, and service/trace links.
- Node links go to `/services/$service` with the complete current range. Edge
  labels go to `/traces` with the source service and complete range. These are
  cross-feature URL contracts, not code imports; ecosystem must not depend on
  services or traces internals/facades.
- `ui/src/lib/api.ts` owns the wire-facing service-map DTO among unrelated
  contracts.
- `ui/src/components/console/__tests__/ecosystem-graph.test.tsx` is 93 lines,
  owns a private router builder, and covers a happy A-to-B graph, 50% error,
  service link, and source-service trace link. There is no focused API, layout,
  route, pending, malformed, empty, or error coverage.

Behavior to preserve exactly:

- `/ecosystem` URL and optional `range/from/to` search shape, normalization,
  `loaderDeps`, replace navigation, and link search propagation;
- range resolution at loader invocation, exact `maxTraces: 100`, selected
  fields, cached request key/TTL/request count, and inherited error boundary;
- current graph retained for 700ms before the six-row skeleton;
- edge-only endpoint inclusion, sort/depth/layout geometry, zero-call error rate,
  colors, widths, labels, p95/count formatting, and accessible names;
- empty text and typed destinations for service nodes and source-filtered trace
  edges; and
- all current loading, empty, success, and recoverable/error observations.

## Target Ownership

```text
ui/src/features/ecosystem/
  api/
    service-map.graphql
    service-map.generated.ts
    service-map-api.ts
  model/
    service-map.ts
    service-map-layout.ts
    ecosystem-search-schema.ts
    ecosystem-error.ts
  components/
    ecosystem-page.tsx
    ecosystem-graph.tsx
    ecosystem-node.tsx
    ecosystem-edge.tsx
  tests/
    api/service-map-api.test.ts
    model/service-map-layout.test.ts
    components/ecosystem-graph.test.tsx
    integration/ecosystem-page.test.tsx
  index.ts
ui/src/routes/tests/
  ecosystem-route.test.tsx
ui/tests/e2e/
  datasets/ecosystem.ts
  screens/ecosystem-screen.ts
  contracts/ecosystem.spec.ts
  full-stack/ecosystem.spec.ts
  accessibility/ecosystem-accessibility.spec.ts
  mobile/ecosystem-mobile.spec.ts
  visual/ecosystem.visual.spec.ts
  visual/goldens/
    ecosystem-populated.png
    ecosystem-empty.png
    ecosystem-recoverable-error.png
```

Fixed responsibilities:

- `service-map.graphql` owns the single named variables-only Plan-152 operation
  with exact `maxTraces` and field selection; its checked-in generated sibling
  parses the complete result from `unknown`, including nullable/numeric behavior.
- `ecosystem-search-schema.ts` instantiates Plan 153 while preserving every
  accepted/defaulted range value and round trip.
- `service-map-api.ts` executes through the platform GraphQL/cache contract,
  maps once into domain values, and returns/projects a typed `EcosystemError`.
  It owns no URL, React, layout, or display logic.
- `service-map.ts` owns readonly `ServiceMap`, node, edge, and display-neutral
  domain values plus the sole wire-to-domain mapper. Remove the duplicate DTO
  from `ui/src/lib/api.ts` only after all consumers move.
- `service-map-layout.ts` owns pure count, rate, endpoint completion, current
  node sorting, edge-iteration-sensitive depth/group assignment, coordinate,
  width, and color-input calculations. Given the same ordered readonly map and
  dimensions, it returns the same readonly layout; this migration must not make
  cyclic topology independent of edge order.
- `ecosystem-error.ts` owns a discriminated Result-shaped expected-failure union
  covering at least transport, invalid response, and load failure. Message text
  is not control flow.
- `ecosystem-page.tsx` receives domain data, current range, pending state, and a
  typed range-change callback. It does not import route definitions or parse
  search.
- graph/node/edge components own accessible presentation and typed link
  creation. Split real responsibilities so no component exceeds 60 logical
  lines; do not make one-line wrapper theater.
- `index.ts` explicitly exports only `EcosystemPage`, the route-facing load
  contract, and stable domain/input types needed outside the feature. No
  documents, schemas, internals, or wildcard exports.

Prefer pure functions and readonly values. No class is expected. A class is
allowed only for a real lifecycle or invariant-bearing mutable identity with a
focused test; graph grouping or coordinates do not justify one.

Final structural ratchets are exact: route module <=150 logical lines,
handwritten TS/TSX module <=300, test scenario file <=500, function/component/
hook <=60, cyclomatic complexity <=12, and cognitive complexity <=15. An
unchanged oversized move does not pass. Any inherited exception is exact,
expiring, and shrink-only.

## Route, Range, And Cache Contract

- `ui/src/routes/ecosystem.tsx` remains the sole owner of TanStack `Route`,
  `validateSearch`, all search-affecting `loaderDeps`, typed navigation, and
  composition. It exports only `Route`.
- The route imports the ecosystem facade and approved domain/shared range and
  pending contracts only. The feature cannot import a route, layout, app, or
  another feature's internals.
- Plan 100 must have placed range parsing/search construction and delayed
  pending under their durable domain/shared owners. Do not duplicate those
  helpers in ecosystem to make imports pass.
- Render `RangePicker` only from `@/features/time-range` and `PageHeader`/
  `PageHeaderBack` only from `@/shared/components/page-header`, using the final
  plan 149 explicit facades. The feature supplies reviewed typed inputs; plan
  100 continues to own pure range construction, delayed pending, and formatting;
  Plan 152 owns GraphQL transport/cache compatibility and Plan 153 owns search
  decoding.
- Do not deep-import plan 149 internals, use wildcard barrels, copy these
  capabilities into ecosystem, or defer a completed ecosystem import to plan
  143.
- Service/trace destinations remain literal typed URL contracts. Record them in
  the matrix/architecture ledger, but do not add `ecosystem -> services` or
  `ecosystem -> traces` source dependencies.
- Retain the current `graphqlCached` behavior and stale/request-count
  characterization. Do not create `queries/`, query keys, `QueryClient`,
  `ensureQueryData`, hydration, prefetch, or a second cache. Plan 133 owns that
  later migration.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Architecture | `cargo xtask arch` | no cycle, deep import, route export, or unknown edge |
| UI policy | `cargo xtask policy --only ui.architecture` | facade/runtime/test topology passes |
| Test policy | `cargo xtask policy --only ui.tests` | matrix and `tests/` placement pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/module/function/complexity budgets shrink or hold |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/ecosystem src/routes/tests/ecosystem-route.test.tsx` | selected non-empty suite passes |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass without unexpected diagnostics |
| Browser contract | `cd ui && bun run test:browser -- --grep @ecosystem` | registered ecosystem cases pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @ecosystem` | non-zero managed GreptimeDB + Turso cases pass |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @ecosystem && bun run test:browser:a11y -- --grep @ecosystem && bun run test:browser:visual -- --grep @ecosystem` | non-zero cross/mobile/a11y/visual rows pass |
| Browser policy | `cargo xtask policy --only ui.browser-contracts` | matrix/spec/locator/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | storage/seed/lifecycle/matrix ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | engine/mobile/a11y/visual rows and goldens pass policy |
| Format | `cd ui && bun run check` | exit 0 |
| Oxc lint | `cd ui && bun run lint` | zero warnings |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Production build | `cd ui && bun run build` | exit 0; generated route/chunks current |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | real-stack and selected breadth lanes are green |

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/ecosystem.spec.ts` owns the `@ecosystem` managed-stack
row. Reuse plan 145's public-OTLP multi-service trace, wait on the named public
service-map predicate, open `/ecosystem`, verify the real edge/count/error/p95
state, change range, and follow the service and source-filtered trace links.
Use one worker with managed GreptimeDB plus isolated Turso and assert only UI or
public GraphQL state; never direct-write native tables or intercept responses.

**Verify**: `cd ui && bun run test:browser:full -- --grep @ecosystem` selects at
least one plan-136 matrix row and passes with bounded eventual visibility and
clean process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@ecosystem` row that consumes plan 146's projects. Run
populated/empty/error topology, range changes, and service/trace navigation in
Firefox and WebKit. Cover graph labels/edges, long service names, tap navigation,
and page overflow on both mobile device projects. Run axe plus keyboard/focus
checks for every reachable graph action. Keep canonical populated, empty, and
recoverable-error visual states using deterministic topology data.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @ecosystem && bun run test:browser:a11y -- --grep @ecosystem && bun run test:browser:visual -- --grep @ecosystem` selects non-zero owned rows and passes without coordinate-only assertions, broad masks, or intercepted happy paths.

## Scope

In scope:

- the ecosystem route, graph component, graph test, service-map API DTO, and
  exact range/pending imports needed after plan 100;
- feature API/schema/domain/layout/errors/components/facade and route tests;
- ecosystem rows/data/screens/contracts in the plan 129/144 matrix, plan 145
  full-stack spec, plan 146 breadth files, and exact architecture/ratchet
  updates; and
- removal of obsolete console/shared owners after all callers move.

Out of scope:

- TanStack Query/cache changes (plan 133), live-data algorithms (plan 147),
  bundle/performance work (plan 148), backend service-map algorithms or GraphQL
  schema, new graph layouts, interaction redesign,
  filtering, zoom/pan, animation, or visual restyling;
- services/traces feature refactors, real-stack/browser-project infrastructure,
  or shell boundary changes; this plan still owns ecosystem-specific managed-
  stack, cross-browser/mobile/accessibility/visual rows;
- duplication of range/pending helpers, `__tests__`, catch-all helper/type
  modules, deep feature imports, route implementation exports, Node, or another
  package manager.

## Git Workflow

- Stay on the current single branch; never create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and `ui/src/lib/api.ts` are serialized
  feature-scoped commits. Re-read the current file, require no uncommitted
  writer, patch only ecosystem rows/type, commit green, then hand off. Do not
  regenerate or replace another feature's content.
- Land decoded model/API, presentation/facade, and cleanup/evidence as focused
  green commits; push each durable update.
- Use Conventional Commits and exactly one required agent-product trailer.
- Do not combine services or traces migrations with this feature.

## Steps

### Step 0: Confirm foundations and freeze the observable contract

Confirm plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete. Verify plan
149 exposes the final page-header and time-range facades. Verify plan 100 has one
durable pure-range owner, delayed-pending owner, and architecture manifest.
Verify Plan 152's generator/handoff and Plan 153's search mechanism. Run the
drift check and this exact
prerequisite-only subset:

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

Do not run focused target paths or `--grep @ecosystem` commands until Steps 3-5
create/register them; zero selection is intentionally fatal. Capture exact
normalized search values, loader nanosecond inputs, GraphQL document/fields,
cache hit/request counts, 700ms transition, skeleton structure, graph geometry,
text, colors, links, and inherited pending/error behavior.

Plan 129's completed state must retain an exact expiring exception for
`ui/src/components/console/__tests__/ecosystem-graph.test.tsx`, owned by plan
136 and removed in Step 5. If plan 129 already moved it, reconcile this ledger
and never recreate the legacy path. If it remains without the exception, stop
because the prerequisite graph is inconsistent.

Plan 145 must reserve the `@ecosystem` managed-stack stable IDs for
`full-stack/ecosystem.spec.ts`. Its distinct `@storage` telemetry-discovery row
may prove that an edge appears and a cross-route link works, but cannot use an
`@ecosystem` ID or own the detailed graph/range/error workflow here. Consume the
delegated row instead of duplicating it.

Add stable matrix ownership for preset/custom/invalid search, cached load,
valid/malformed/empty data, edge-only/cyclic layout, delayed pending, error,
accessible graph, and service/trace navigation. Reserve plan 144 ecosystem
dataset/screen/spec IDs without duplicate cases.

**Verify**: test-matrix policy and the complete baseline UI suite pass; every
ecosystem risk has exactly one owner and one expected test layer.

### Step 1: Extract readonly domain, runtime schema, and pure layout

Create `service-map.ts`, `service-map-layout.ts`, and their tests. Move current
numeric/layout behavior without rounding, sorting, or geometry changes. Cover
empty maps, edge-only endpoints, zero calls, nullable durations, multiple
depths, disconnected nodes, cycles in at least two edge orders, current
order-sensitive output, error rates, and current width/color inputs.

Create the named `service-map.graphql` operation and checked-in generated sibling
through Plan 152, plus the domain mapper and Plan-153 search schema. External
GraphQL/search data enters as `unknown`, is parsed once, and becomes a readonly
domain value once. No `as ServiceMap`, generic trust cast, duplicate DTO, or
React type belongs in the model.

**Verify**: focused model/schema tests and typecheck pass; repeated identical
ordered input is deterministic, while golden cyclic fixtures preserve the
baseline output for each tested edge order.

### Step 2: Extract the cache-preserving decoded API

Implement `service-map-api.ts` through Plan 152's platform adapter. Preserve
`maxTraces: 100`, all fields,
absolute range inputs, query-string cache key, TTL, and current error timing.
Project transport/schema errors once into `EcosystemError`; do not hide malformed
data as an empty graph.

Add API tests for valid/empty/malformed/null/numeric edge cases, exact variables
and fields, transport failure, cancellation classification where supported, and
cache hit/miss/request count parity.

**Verify**: focused API tests and architecture policy pass; current request-count
characterization remains unchanged.

### Step 3: Build feature presentation within structural budgets

Move page/graph presentation into the target files and split node/edge rendering
by cohesive ownership. Pass current range and navigation callbacks from route
composition. Preserve the 700ms current-content delay/skeleton switch, empty
text, summary, accessible names, node/edge styling, p95/count/rate formatting,
and typed link search.

Migrate the existing graph assertions and add component/integration coverage
for empty/edge-only/error/pending transition/accessibility and every link. Use
the shared plan 129 harness, fake clock for 699/700ms, semantic queries, and
`userEvent.setup()`; no real sleep or private router builder.

**Verify**: focused component/integration tests, lint, and ratchets pass with no
unexpected console/network/timer diagnostic.

### Step 4: Publish the facade and thin route

Create explicit `index.ts`. Convert `ecosystem.tsx` to a route adapter that owns
search validation, complete `loaderDeps`, loader invocation, replace navigation,
and feature composition while exporting only `Route`. Feature code must not
import `Route` or layout. If the page currently needs a layout-owned back/nav
descriptor, pass `PageHeaderBack` from route composition and render
`PageHeader` through plan 149's `@/shared/components/page-header` facade; do not
add a feature-to-layout edge.

Move route tests to `ui/src/routes/tests/ecosystem-route.test.tsx`. Test URL
round trips, preset/custom/invalid normalization, all `loaderDeps`, replace
navigation, loader/cache behavior, delayed pending, root error, and typed links
through public behavior. Do not import private route symbols.

**Verify**: architecture, focused route tests, typecheck, and build pass. The
route exports only `Route`, and external deep-import searches return no matches.

### Step 5: Complete browser contracts and delete obsolete owners

Add fixture-backed plan 144 ecosystem cases for populated/empty/recoverable
error topology, preset/custom range changes, delayed loading where observable,
direct deep link, and service/trace navigation. Use semantic locators, stable
data, and public HTTP/GraphQL; no interception or fixed sleeps.

Implement the Feature Real-Stack Contract and Feature Browser Breadth sections
in the exact target files, register each non-empty project row once, and keep
shared plan 145/146 fixtures read-only.

Delete the console graph/test and the service-map DTO from `ui/src/lib/api.ts`
only after `rg` proves no caller. Remove compatibility re-exports and update
matrix plus exact route/module/function/complexity/export ratchets.

**Verify**: focused Vitest, fixture-backed/full-stack/breadth `@ecosystem`
commands and policies, all UI, architecture, test policy, and ratchet gates
pass; old paths and deep imports have zero matches.

### Step 6: Run the final gate twice

Run every Commands-table entry twice from clean state. The second run must not
modify route generation, matrix content, snapshots, or other tracked files.

**Verify**: all commands exit 0 twice and `git diff --check` is clean.

## Test Plan

- Model: endpoint completion, deterministic sorting/depth/coordinates, cycles,
  disconnected/edge-only/empty input, zero calls, rates, width/color inputs.
- API: exact operation/variables, valid/empty/malformed/null data, numeric
  boundary mapping, transport/error projection, cache request-count parity.
- Components: summary/empty/accessibility, nodes/edges/p95/rates, service and
  source-filtered trace links with full range, 699/700ms pending transition.
- Route: search validation/round trip, every loader dependency, absolute range,
  replace navigation, initial pending/root error, cached load.
- Browser: populated/empty/error, range changes, direct load, service and trace
  navigation using deterministic fixture data.
- Real stack: public-OTLP multi-service edge visibility, range, and service/
  trace navigation against managed GreptimeDB plus isolated Turso.
- Facade/type: reviewed exports compile; route/internal deep imports fail policy.

## Done Criteria

- [ ] Ecosystem production/API/model/tests have one feature owner; the old
  console/shared owners are deleted.
- [ ] The route exports only `Route`, owns typed URL state, and imports only the
  explicit facade plus allowed domain/shared contracts.
- [ ] GraphQL data is parsed from `unknown` once, mapped once, and failures use
  the feature-owned exhaustive error contract.
- [ ] Search/loader/cache/request count, 700ms pending/skeleton, graph geometry,
  text/accessibility, empty/error, and service/trace links match baseline.
- [ ] Tests live only under `tests/`, cover every named matrix risk, and browser
  cases use the plan 144 fixture without response interception.
- [ ] No source dependency on services/traces was added; only URL contracts
  remain.
- [ ] Ecosystem-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and
  canonical visual rows are non-empty and green.
- [ ] The feature-owned `@ecosystem` managed-stack row is non-empty, uses only
  public boundaries, and passes with clean teardown.
- [ ] Oxc, Vitest, build, browser, architecture/test/ratchet, and aggregate gates
  pass twice.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/features/time-range` `RangePicker` or
  `@/shared/components/page-header` `PageHeader`/`PageHeaderBack` contract is
  absent/incompatible; do not copy a legacy component, deep-import internals,
  add a wildcard export, or defer import repair to plan 143;
- a prerequisite, test matrix, runtime schema mechanism, range owner, pending
  owner, or platform cache adapter is absent/red;
- plan 145 lacks the delegated `@ecosystem` reservation or shared managed-stack
  infrastructure, or Step 5 cannot turn that reservation into a non-empty
  public-boundary row with clean one-worker teardown;
- plan 145's shared specs use an `@ecosystem` stable ID or own the detailed
  graph/range/error workflow instead of the distinct `@storage` smoke;
- plan 129 is marked complete while the legacy ecosystem test remains without
  its exact plan 136 expiring topology exception;
- drift changes search normalization, `loaderDeps`, `maxTraces`, selected fields,
  cache key/TTL/request count, 700ms timing, graph layout, text, or links;
- range/pending ownership requires duplication or a feature import of route,
  layout, services, or traces;
- malformed data cannot be rejected at the API boundary without unsafe casts;
- preserving behavior requires Query/cache migration, a backend/schema change,
  or browser response interception;
- a route implementation export, deep import, `__tests__`, oversized unchanged
  move, duplicate owner, or broad permanent exception would remain; or
- any required command fails twice after one reasonable correction.

## Maintenance And Removal

Future service-map fields change the named operation, runtime schema, mapper,
domain value, error/test cases, and matrix evidence together. Layout changes
remain pure and golden-tested. New destinations remain typed URL contracts
unless an independently reviewed source dependency is genuinely needed. Plan
133 may later replace the preserved cache path without changing the facade or
observable characterization.

Delete this plan and its index row only after every done criterion is green,
old owners are removed, and durable tests/policy/matrix records own the lasting
contract.
