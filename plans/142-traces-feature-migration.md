# Plan 142: Move trace search, analysis, and inspection into one feature

> **Executor instructions**: Decompose both trace routes and every trace-specific
> model/component/library into one bounded feature. Preserve `/traces/` and
> `/traces/$traceId`, search/range/tab/view behavior, request documents/variables/
> count, cache calls, live-tail lifecycle/order, list paging/filtering, waterfall
> selection and keyboard behavior, inspector/GraphQL/RPC/story/compare/critical-
> path behavior, all loading/empty/error states, links and visual output. Consume
> Plan-149 route-less UI capabilities, Plan-152 GraphQL, Plan-153 external-value,
> and Plan-100 remaining technical/pure-domain capabilities through their exact facades. Do not implement
> Query/cache changes; plan 133 owns them. Stop on a missing runtime contract,
> cyclic boundary or behavior drift rather than improvising.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- 'ui/src/routes/traces.index.tsx' 'ui/src/routes/traces.$traceId.tsx' ui/src/routes/__tests__/-trace-graphql.test.tsx ui/src/routes/__tests__/-trace-links.test.tsx ui/src/routes/__tests__/-trace-rpc.test.tsx ui/src/routes/__tests__/-trace-view-modes.test.tsx ui/src/routes/__tests__/-traces-search.test.tsx ui/src/lib/trace-tree.ts ui/src/lib/graphql-trace.ts ui/src/lib/rpc-trace.ts ui/src/components/console/trace-waterfall.tsx ui/src/components/console/field-explorer.tsx ui/src/components/console/attribute-compare.tsx ui/src/components/console/evidence-gaps.tsx ui/src/components/console/graphql-operation.tsx ui/src/components/console/rpc-stream.tsx ui/src/features/traces ui/test-matrix.json ratchet.toml`
> Plans 100/129/149 may relocate lower-layer imports, route-less capabilities,
> and tests. Reconcile their ledger, but STOP if a trace product/API contract
> changed.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 134, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / traces / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — upstream UI foundation, investigations, and browser plans are incomplete

## Contract reconciliation (2026-07-17)

Plans 156/157 rename the correlation contract before this plan starts: the
inspector's resource/span attributes carry `cli.invocation.id` (legacy
`parallax.run.id` support is removed entirely), trace→run links
become trace→`/invocations/$invocationId`, and PRODUCER/CONSUMER job spans
(`job.id`/`job.type`) plus `background.cycle` roots are expected span shapes.
Re-characterize the baseline at the post-157 head; "run" reads as
"invocation". See plans/156-unified-cli-observability-contract.md.

## Why This Matters

The trace routes contain 2,252 lines, including the 1,500-line detail hotspot.
They own list/search/live behavior, GraphQL requests, trace tree/window/skew,
waterfall rendering, GraphQL and RPC reconstruction, critical-path and compare
actions, inspector parsing, linked traces, logs, story and presentation. Three
trace model libraries and multiple console components are detached from their
domain owner. Tests therefore import many route internals. This plan establishes
one trace facade and removes those route/test coupling points without changing
behavior.

## Current Paths And Responsibilities

| Current path | Current responsibility at `e3e7997` | Required final owner |
|---|---|---|
| `ui/src/routes/traces.index.tsx` | Search/sort/page/range args, baseline comparison, live spans, field explorer, attribute compare and table | Thin route plus trace list modules |
| `ui/src/routes/traces.$traceId.tsx` | Detail load, JSON/event parsing, tree/window/skew, critical/compare state, summary/waterfall/GraphQL/RPC/log/story/metrics/inspector UI | Thin route plus trace detail modules |
| `ui/src/lib/trace-tree.ts` | Ordering, window/position, tree/error ancestors, service lanes and skew detection | Trace model |
| `ui/src/lib/graphql-trace.ts` | GraphQL operation/field reconstruction and self-time aggregation | Trace model |
| `ui/src/lib/rpc-trace.ts` | RPC event/message reconstruction, outcomes and messaging summary | Trace model |
| `ui/src/components/console/trace-waterfall.tsx` | Virtualized tree/errors/lanes waterfall, minimap and keyboard selection | Trace component |
| `ui/src/components/console/field-explorer.tsx` | Trace-list field/value exploration | Trace component |
| `ui/src/components/console/attribute-compare.tsx` | Trace baseline attribute comparison | Trace component |
| `ui/src/components/console/evidence-gaps.tsx` | Trace evidence gap card | Trace component |
| `ui/src/components/console/graphql-operation.tsx` | Reconstructed GraphQL operation card | Trace component |
| `ui/src/components/console/rpc-stream.tsx` | RPC stream/message timeline | Trace component |
| `ui/src/components/console/span-kind.tsx` | Waterfall span-kind metadata/chip | Trace component; split its current shared-kit assertions |
| Trace-specific route/lib/component tests | Search, view modes, links, compare, RPC, GraphQL, tree, waterfall and panels | Feature behavior under `features/traces/tests/**`; route contracts under `routes/tests/` |
| Plan-149 page-header/time-range/runtime-metrics/story facades | `PageHeader`, `RangePicker`, `MetricStrip`, `StoryTimeline`, and their minimum readonly inputs | Consume through explicit final facades; do not copy or deep-import |
| Plan-152 GraphQL/cache and Plan-153 SSE/search/JSON contracts | Technical runtime-boundary capability | Consume through canonical owners; do not move into traces |
| Plan-152 GraphQL generator/handoff | List/detail/critical/compare runtime template | Create named operations and generated siblings under `features/traces/api/` |

The detail route currently exports numerous test-only components and types. The
final list and detail route files export only `Route`.

## Fixed Behavior And Ownership

1. Preserve exact `/traces/` and `/traces/$traceId` paths, list search keys,
   sort mapping, page reset, range links, detail tab/view parsing and defaults.
2. Preserve list/detail/service-only/attribute-baseline/critical-path/compare
   operation documents, variables, request count/order, and current
   `graphqlCached`/`graphql` choice. Query/cache/freshness remain unchanged.
3. Preserve list live URL, platform visibility/reconnect/flush, buffer reset,
   ordering and cap. Decode frames before mutation but do not optimize algorithms.
4. Plan 152 schemas parse all GraphQL operation results. Trace-owned schemas
   instantiate Plan 153 for search, live frames, span attributes/resource/events/
   links, and protocol event JSON. Trace API/model mappers create readonly
   domain values once; components never parse wire JSON.
5. `model/traces-error.ts` owns discriminated list/detail/live/critical/compare
   failures. Preserve fatal route loaders, reconnect state, inline critical/
   compare errors, not-found and all visible error messages/boundaries.
6. Trace tree, GraphQL reconstruction and RPC reconstruction are pure trace model
   modules. Preserve order, cycle termination, skew thresholds, N+1 merge,
   deadline/cancel labels and messaging summaries exactly.
7. Feature consumers use `@/features/traces`; trace internals may use cohesive
   relative imports. No route implementation, app/layout, another feature's
   internals, or old generic lib/component path is imported.
8. `PageHeader`, `RangePicker`, `MetricStrip`, and `StoryTimeline` remain at
   Plan-149 owners. `PinButton` remains at Plan 134's investigations facade.
   GraphQL/cache remain at Plan 152, generic SSE/visibility/search/JSON at Plan
   153, and clock/format/pure-range contracts at Plan-100 owners. Traces supplies typed inputs and
   composition only.
9. Use readonly data and pure functions. React hooks/refs own real component
   lifecycles; no stateless service/model class or module singleton is allowed.

## Plan 149 Capability Contract

- Traces imports `PageHeader` from `@/shared/components/page-header`,
  `RangePicker` from `@/features/time-range`, `MetricStrip` plus only its minimum
  readonly inputs from `@/features/runtime-metrics`, and `StoryTimeline` plus its
  minimum readonly story-beat input from `@/features/story`.
- Use explicit named value/type imports only. Do not deep-import plan 149
  internals, use wildcard barrels, copy a legacy capability into traces, or
  defer a completed trace capability import to plan 143.
- Plan 152 owns GraphQL/cache, Plan 153 owns SSE/visibility/search/JSON, and Plan
  100 retains clock/format/pure-range and other technical/domain foundations.
  Plan 134 remains the sole `PinButton` facade owner.

## Target Tree

```text
ui/src/features/traces/
  api/
    traces-list.graphql
    traces-list.generated.ts
    load-traces.ts
    trace-detail.graphql
    trace-detail.generated.ts
    load-trace-detail.ts
    trace-critical-path.graphql
    trace-critical-path.generated.ts
    load-trace-critical-path.ts
    trace-compare.graphql
    trace-compare.generated.ts
    compare-traces.ts
    live-trace-schema.ts
    traces-mapper.ts
  model/
    trace-summary.ts
    trace-span.ts
    traces-search.ts
    traces-search-schema.ts
    trace-tree.ts
    trace-window.ts
    trace-skew.ts
    trace-events.ts
    trace-inspector.ts
    trace-diff.ts
    graphql-operations.ts
    rpc-streams.ts
    traces-error.ts
  components/
    traces-page.tsx
    traces-table.tsx
    trace-field-explorer.tsx
    trace-attribute-compare.tsx
    trace-detail-page.tsx
    trace-summary.tsx
    trace-view-mode.tsx
    trace-waterfall.tsx
    trace-span-kind.tsx
    trace-critical-path.tsx
    trace-clock-skew.tsx
    trace-evidence-gaps.tsx
    trace-graphql-operations.tsx
    trace-rpc-streams.tsx
    trace-logs.tsx
    trace-compare-sheet.tsx
    trace-compare-result.tsx
    trace-linked-edges.tsx
    trace-inspector.tsx
    trace-inspector-events.tsx
    trace-inspector-links.tsx
  hooks/
    use-live-traces.ts
    use-trace-critical-path.ts
    use-trace-compare.ts
  tests/
    api/traces-api.test.ts
    api/live-trace-contract.test.ts
    model/traces-search.test.ts
    model/trace-tree.test.ts
    model/graphql-operations.test.ts
    model/rpc-streams.test.ts
    model/trace-inspector.test.ts
    components/traces-page.test.tsx
    components/trace-waterfall.test.tsx
    components/trace-detail-page.test.tsx
    components/trace-graphql-operations.test.tsx
    components/trace-rpc-streams.test.tsx
    components/trace-compare-links.test.tsx
    integration/live-traces.test.tsx
  index.ts
ui/src/routes/
  traces.index.tsx
  traces.$traceId.tsx
  tests/traces-routes.test.tsx
ui/tests/e2e/
  datasets/traces.ts
  screens/traces-screen.ts
  contracts/traces.spec.ts
  full-stack/traces.spec.ts
  accessibility/traces-accessibility.spec.ts
  mobile/traces-mobile.spec.ts
  visual/traces.visual.spec.ts
  visual/goldens/
    traces-list.png
    traces-waterfall.png
    traces-inspector.png
    traces-graphql-rpc.png
    traces-empty.png
    traces-error.png
```

Plan 152 provides the generator/template and handoff rows, not these product
files. This plan creates each named operation and exact `.generated.ts` sibling;
live/search/attribute schemas separately instantiate Plan 153. Split additional inspector/component
files only for cohesive responsibilities and structural limits, never into
generic `types`, `helpers`, `utils` or `common` buckets.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | no route/deep/cycle/old-path/unknown-owner edge |
| UI architecture | `cargo xtask policy --only ui.architecture` | trace facade, runtime decoder, route export and platform boundary pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/lib/component/function/export rows shrink; no exception grows |
| Test ownership | `cargo xtask policy --only ui.tests` | every trace matrix ID resolves below trace feature tests |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/traces/tests` | non-zero trace tests pass without diagnostics |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0, zero warnings/errors |
| Unit suite | `cd ui && bun run --bun test:ci` | all tests pass under Bun, no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @traces` | non-zero fixture-backed trace rows pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @traces` | non-zero managed GreptimeDB + Turso trace rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross -- --grep @traces` | non-zero Firefox/WebKit/mobile trace rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y -- --grep @traces` | non-zero axe/keyboard/focus trace rows pass |
| Visual | `cd ui && bun run test:browser:visual -- --grep @traces` | non-zero canonical trace visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | trace matrix/spec/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | trace storage/seed/lifecycle ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | trace engine/mobile/a11y/visual ownership passes |
| Build | `cd ui && bun run build` | exit 0, route tree current, trace lazy boundaries retained |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | trace real-stack and breadth lanes are green |

All JS/TS gates use exact lock-local Bun with installation disabled. Oxc-backed
xtask is authoritative for imports, exports, cycles, runtime reachability and AST
ratchets. Node, foreign package managers, ESLint/plugins, a second parser graph,
or direct generated edits are forbidden.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/traces.spec.ts` owns plan 145's delegated, non-empty
`@traces` row. Seed two deterministic multi-span traces with GraphQL and RPC
semantic events, links, errors, and resource attributes through public OTLP
using `datasets/traces.ts`; wait on named public trace predicates; then drive
list/search/detail navigation, waterfall selection, inspector events/links,
GraphQL/RPC reconstruction, and the existing compare action through
`screens/traces-screen.ts`. Do not repeat plan 145's distinct `@storage`
discovery/cross-route or live-transport smoke.

Run one worker against managed GreptimeDB plus an isolated Turso database. Use
only public OTLP, GraphQL, and UI boundaries with bounded readiness predicates;
never write/read database internals, intercept browser responses, or use fixed
sleeps.

**Verify**: `cd ui && bun run test:browser:full -- --grep @traces` selects at
least one plan-142 row and passes with the real-stack runtime manifest and clean
process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@traces` row that consumes plan 146's projects. Run trace
list/detail/search/live, view modes, waterfall/selection, inspector, GraphQL/RPC,
compare/critical, and cross-feature links in Firefox and WebKit. Cover dense
waterfalls, minimap, tabs/sheets, long attributes/events, touch scrolling, and
overflow on both mobile device projects. Run axe plus full keyboard/focus/
Escape/restoration checks and keep canonical list, waterfall, inspector,
GraphQL/RPC, empty, and error visual states with deterministic data.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @traces && bun run test:browser:a11y -- --grep @traces && bun run test:browser:visual -- --grep @traces` selects non-zero breadth rows and passes without response interception, broad waterfall masking, coordinate-only assertions, or fixed sleeps.

## Scope

**In scope:**

- Both trace routes and every trace-specific responsibility/path listed above.
- Trace-specific portions of existing component/lib tests, including span-kind
  assertions currently mixed into the console kit test.
- New `features/traces/**`, explicit facade, separated trace tests, trace matrix
  and ratchet rows.
- Feature-owned traces dataset/screen/contract/full-stack/accessibility/mobile/
  visual/golden files and their non-empty plan 144-146 matrix rows.
- Imports/composition against Plan-149 canonical page-header/time-range/runtime-
  metrics/story facades, Plan-152 GraphQL/cache, Plan-153 SSE/visibility/search/
  JSON, Plan-100 clock/format/pure-range owners, and Plan-134 investigations facade without moving those
  implementations.
- Tool-generated route-tree refresh through normal build only.

**Out of scope:**

- TanStack Query/cache/freshness/invalidation and `graphqlCached` deletion (plan
  133), live/waterfall/list capacity or timer optimization (plan 147), and
  bundle/route chunk redesign (plan 148).
- Backend GraphQL or semantic changes, new trace views, new comparison/critical
  algorithms, visual redesign, another feature,
  shadcn/generated manual edits, project references/internal packages, generic
  buckets or unnecessary classes.
- Shared plan 144-146 Playwright configuration, fixtures, reporters, lifecycle,
  CI, matrix schema, and browser infrastructure; consume them read-only.
- Taking ownership of Plan-149 story/runtime-metrics/time-range/page-header,
  Plan-152 GraphQL, Plan-153 SSE/visibility/search/JSON, or Plan-100 clock/
  format/pure-range contracts.

## Git Workflow

- Work on the single active branch only; do not create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any shared generated registry/config
  are serialized feature-scoped commits. Re-read current content, require no
  uncommitted writer, change only traces rows, land green, then hand off. Never
  regenerate or replace another feature's content.
- Land pure models, API adapters, list, detail-analysis components, inspector,
  and route/test closure in separate reviewable green changes.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable green update under repository policy.

## Steps

### Step 0: Freeze all trace contracts

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

Do not run focused target paths or any `--grep @traces` command until Step 7
creates and registers those files; zero selection is intentionally fatal.
Resolve Plan 152's generator and trace/list/detail/critical/compare handoff rows, Plan
153's search/SSE/embedded-JSON paths, Plan 149's
final page-header/time-range/runtime-metrics/story facades, plan 134's
investigations facade, and Plan-100 technical/pure-domain facades. Record
route/search examples, every request document/
variable/count/cache choice, live URL/order/cap/reset, not-found/error states,
tree/skew/GraphQL/RPC outputs, waterfall modes/selection/keyboard/minimap,
compare/critical actions, inspector caps/expansion, links, story/metric
composition and browser markers. Map every existing trace test ID.

Require every trace-owned route/lib/component `__tests__` path and private route
import to have an exact plan-129 legacy handoff owned by plan 142. Stop on a
missing, wildcard, expired, or differently owned row; delete each row when its
test/import moves.

Confirm plan 145 reserves `@traces` for
`ui/tests/e2e/full-stack/traces.spec.ts` and its shared `@storage` specs retain
only foundation trace discovery/cross-route/live-transport behavior. Consume
the feature reservation without duplicating a foundation stable ID or scenario.

**Verify**: every prerequisite command above exits 0, every trace legacy handoff
is exact, and the delegated traces row is reserved but not yet required to
select a feature spec.

### Step 1: Move pure trace models first

Move trace summary/span/search, tree/window/skew, event/inspector decoding,
diff-format transforms, GraphQL operation reconstruction and RPC streams into
cohesive model modules. Preserve algorithms exactly before splitting for size.
Replace route-local JSON parsing with canonical decoded values/mappers. Add the
typed trace error union without moving UI state yet.

Move existing library/search assertions into feature model tests. Preserve cycle
termination, stable ordering, zero-duration drawing, skew thresholds, N+1 field
merge, partial errors, RPC filtering/outcomes and messaging summaries.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/traces/tests/model && bun run typecheck`
must pass all moved cases; `cargo xtask policy --only ui.ratchets` must show old
lib and route rows shrinking.

### Step 2: Move decoded trace API adapters

Place canonical list, detail, critical-path, compare and live operations/schemas
under trace API. Implement one mapper per wire result into readonly trace models.
Keep service-only and baseline attribute list paths, detail combined query, and
on-demand critical/compare requests behaviorally exact. Map errors into the trace
union while preserving fatal loader versus inline/reconnect boundaries.

No raw query, string interpolation/escaping, generic `as T`, route/component
`JSON.parse`, duplicate event/span type, or second schema remains.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/traces/tests/api && bun run typecheck`
must cover valid/null/malformed/error/cancel requests/frames and exact documents,
variables, count, order and cache calls.

### Step 3: Move trace list and live orchestration

Extract traces page/table, search patching, field explorer and attribute compare.
Create `use-live-traces.ts` over the canonical platform SSE facade with decoded
frames and current reset/order/cap behavior. Preserve pagination, filters/sorts,
range links, baseline window, row identity, loading and empty states.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/traces/tests/components/traces-page.test.tsx src/features/traces/tests/integration/live-traces.test.tsx && bun run check && bun run lint`
must pass with fake-clock visibility/reconnect/cleanup and unchanged browser list
behavior.

### Step 4: Move waterfall, protocol analysis, and detail orchestration

Move waterfall/span-kind, summary/view/skew/evidence, GraphQL/RPC sections,
trace logs, story/metric composition, critical and compare hooks/components.
Represent critical and compare async state as discriminated unions while
preserving current interactions and messages. Compose `StoryTimeline` and
`MetricStrip` only through plan 149's explicit facades; keep their implementations
at those owners.

Preserve waterfall tree/errors/lanes rows, keyboard/minimap selection,
highlighting, clock-skew warning, GraphQL selection, RPC messages/outcomes,
critical totals and comparison tables exactly. No performance algorithm change
belongs in this move.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/traces/tests/components && bun run check && bun run lint`
must pass all waterfall/GraphQL/RPC/compare/link cases and browser detail parity.

### Step 5: Split and move the inspector

Extract inspector composition, event/link lists, key/value/resource/attribute/
log sections and code/copy presentation. Use decoded span event/resource/link
models; preserve display order, list cap/expand behavior, selected whole trace/
span behavior, error/grpc labels, linked-trace resolution and search-bearing
links. Avoid one replacement inspector monolith.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/traces/tests/model/trace-inspector.test.ts src/features/traces/tests/components/trace-detail-page.test.tsx && bun run typecheck`
must pass inspector invalid/empty/capped/expanded/link/log/error cases.

### Step 6: Publish the facade and thin both routes

Create explicit `features/traces/index.ts` exports for route entry/load/search
contracts and only reviewed cross-feature trace contracts. Do not export private
inspector/waterfall/API internals or wildcard barrels.

Reduce both routes to file-route creation, params/search/loader wiring,
boundaries and top-level feature composition. Each exports only `Route`, imports
the trace facade, and contains no implementation state/query/parser/helper. No
route-to-route or deep feature import remains.

**Local verification:**
`cargo xtask arch && cargo xtask policy --only ui.architecture && cd ui && bun run build`
must prove exact route IDs/search bindings, only-`Route` exports, facade-only
imports, no cycle/server leak and retained automatic route splitting.

### Step 7: Close tests, matrix, ratchets and old paths

Move trace API/model/component/live test bodies under `features/traces/tests/**`
and exact URL/search/loader/boundary/navigation contracts under
`routes/tests/traces-routes.test.tsx`. Preserve stable matrix IDs/assertions and
remove private route imports. Extract only trace/span-kind cases from mixed
generic kit tests; do not move unrelated shared cases. Delete old trace-specific
tests and sources after consumers switch. Do not create another `__tests__` tree.

Create or extend the exact feature-owned `datasets/traces.ts`,
`screens/traces-screen.ts`, fixture contract, full-stack, accessibility, mobile,
visual, and named golden files in the Target Tree. Consume plan 145's reserved
`@traces` row, register each feature matrix ID/project once, and make every grep-
scoped selection non-empty. Shared plans 144-146 fixtures, configuration,
reporters, lifecycle code, and infrastructure remain read-only; shared
discovery/live-transport smoke stays under the `@storage` foundation.

Ratchet routes to 150 lines, new modules to 300, functions/components/hooks to
60, complexity to 12/15, and facade exports to the exact reviewed list. Remove
resolved old-path/size/export/assertion/test exceptions; add no new exception.

**Local verification:** run every command twice. Every `--grep @traces`
selection must be non-zero, `git diff --check` must be clean, and scoped status
must contain only allowed files.

## Test Plan

- API tests: exact list/detail/critical/compare/live valid/null/malformed/error/
  cancel documents, variables, request counts and typed errors.
- Search/list tests: garbage/default values, every sort/filter/page/range patch,
  detail links, service-only and baseline argument mapping.
- Tree/GraphQL/RPC tests: preserve every existing ordering/cycle/skew/window,
  field-tree/N+1/error and RPC/message/outcome/messaging case.
- Inspector model tests: invalid/empty attributes/resources/events/links, stable
  order, cap/expand, status and selected-span projections.
- List component/live tests: table, filters, paging, field explorer, attribute
  compare, loading/empty and stream reset/order/cap/reconnect/cleanup.
- Waterfall/detail tests: modes, keyboard/minimap, critical highlight, skew,
  evidence, story/metrics/logs, not-found and accessible semantics.
- GraphQL/RPC/compare/link tests: presence/absence, selection, messages, deadline/
  cancel, added/removed/changed, linked resolution and range links.
- `routes/tests/traces-routes.test.tsx`: exact URLs/search/loaders/boundaries and client navigation
  through public route behavior.
- Fixture browser: deterministic list/detail/search/view/waterfall/inspector/
  GraphQL/RPC/compare states through `@traces`.
- Real stack: public-OTLP multi-trace waterfall/inspector/protocol/compare
  behavior against managed GreptimeDB plus isolated Turso.
- Browser breadth: selected Firefox/WebKit/mobile behavior, axe/keyboard/focus,
  and named canonical trace visuals.

## Done Criteria

- [ ] Both trace routes export only `Route`, retain exact URL/search/loader
  contracts, and are at or below 150 logical lines.
- [ ] All trace-specific API/model/components/hooks and former trace libs/console
  components live under `features/traces` with an explicit facade.
- [ ] Every payload/frame/embedded JSON value is decoded once and mapped once;
  typed trace errors preserve fatal/reconnect/inline/not-found behavior.
- [ ] Requests/cache, list/live, tree/skew, waterfall, inspector, GraphQL/RPC,
  compare/critical, links/story/metrics and browser behavior match baseline.
- [ ] Trace feature tests live under `features/traces/tests/**`, route contracts
  under `routes/tests/`, and no private route import/old trace test remains.
- [ ] Traces-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and canonical
  visual rows are non-empty and green.
- [ ] The feature-owned `@traces` managed-stack row is non-empty, uses public
  OTLP/GraphQL/UI boundaries, and passes against GreptimeDB + Turso.
- [ ] Architecture/tests/ratchets and all Bun unit/browser/build/aggregate gates
  pass twice with no Node and retained route splitting.
- [ ] No unnecessary class, generic bucket, wildcard/deep export, duplicate
  schema, Query/cache/performance change, manual generated edit or unrelated
  feature change exists.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`,
  `@/features/time-range` `RangePicker`, `@/features/runtime-metrics`
  `MetricStrip`, or `@/features/story` `StoryTimeline` facade/input contract is
  absent/incompatible; do not copy a legacy component, deep-import internals,
  add a wildcard export, or defer import repair to plan 143;
- prerequisites or forced-Bun/browser trace evidence are incomplete/red;
- plan 145 lacks the delegated `@traces` reservation/shared managed-stack
  infrastructure, or Step 7 cannot make it a non-empty public-boundary row with
  clean one-worker teardown;
- a shared `@storage` discovery/live spec or another feature owns the same traces
  stable ID/scenario, or the reservation points at a different file;
- feature browser evidence requires editing shared plans 144-146 fixtures,
  configuration, lifecycle, reporters, CI, or matrix schema;
- Plan 152's generator/handoff cannot represent a frozen trace GraphQL boundary,
  Plan 153's search/SSE/JSON mechanism is absent, or an embedded payload cannot be
  decoded without an API/product decision;
- request/cache/search/live/tree/waterfall/inspector/protocol/compare/critical or
  visual behavior has drifted materially before movement;
- a trace model/component requires another feature's internals or a cyclic/deep
  import that composition cannot remove;
- preserving behavior requires Query/cache/backend/new-algorithm/performance,
  manual generated/shadcn, Playwright infrastructure or another feature changes;
- a route-less story/runtime-metric/page-header/time-range owner conflicts with
  Plan 149's ledger, or a technical SSE/visibility/clock/format/pure-range owner
  conflicts with Plan 100's ledger;
- structural limits require arbitrary fragmentation; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Required Deletions

Future trace behavior updates canonical documents/schemas/mappers/errors, pure
models, facade, tests/matrix and ratchets together. Plan 133 may add trace query
modules and change cache ownership without moving this feature again.

Delete before retiring this plan:

- all five old trace route test files named in the drift check;
- `ui/src/lib/trace-tree.ts`, `ui/src/lib/graphql-trace.ts`,
  `ui/src/lib/rpc-trace.ts` and their old tests;
- the seven trace-specific console sources named in Current Paths and their old
  tests, plus the old `span-kind.tsx` path after its cases are separated;
- every trace implementation export from both route files;
- every temporary trace old-path reexport; and
- every completed trace migration exception/ledger row.

Do not delete Plan-149 story/runtime-metrics/time-range/page-header owners or
Plan-100 technical/pure-domain owners. Delete this plan and README row only
after required deletions and all done criteria are durable and green.
