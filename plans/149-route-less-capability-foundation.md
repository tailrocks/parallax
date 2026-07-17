# Plan 149: Establish route-less UI capabilities before feature moves

> **Executor instructions**: Run this move-only foundation after Plans 100, 129,
> 152, and 153 are complete. Give the current runtime metric, story, time-range, and
> page-header code permanent owners and explicit facades before any product
> feature migration starts. Preserve every request, cache call, polling and
> visibility transition, URL value, callback, rendered state, accessible name,
> and visual output. Update consumers only to replace imports; do not restructure
> their feature code. Do not add browser infrastructure, Query, live-data
> optimization, or bundle work. Stop rather than create a second capability or a
> compatibility barrel.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/components/metric-strip.tsx ui/src/components/runtime-snapshot.tsx ui/src/components/console/story-timeline.tsx ui/src/components/console/range-picker.tsx ui/src/components/page-header.tsx ui/src/components/__tests__/metric-strip.test.tsx ui/src/components/__tests__/shell.test.tsx ui/src/components/console/__tests__/story-timeline.test.tsx ui/src/components/console/__tests__/range-picker.test.tsx ui/src/routes ui/src/features ui/src/domain ui/src/shared ui/test-matrix.json ratchet.toml`
> Reconcile Plan-100 lower-layer moves and Plan-152 handoff rows
> against the live ownership ledger. Stop if any named capability has already
> acquired a different permanent owner or its observable contract has changed.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 152, 153
- **Category**: TypeScript / route-less capabilities / architecture foundation
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — remaining deps Plan 152 + Plan 153 (checked 2026-07-17)

### Dependency resolution (2026-07-17)

| Dep | State | Evidence |
|---|---|---|
| 100 | **DONE** (retired) | Plan file gone; `090ea92b` architecture control plane; domain/time-range + ownership ledger live |
| 129 | **DONE** (retired) | Plan file gone; README “Plan 129 DONE (2026-07-17)”; `34174e31` Vitest foundation |
| 152 | **OPEN** | `plans/152-graphql-contract-foundation.md` present; README TODO; uncommitted WIP under `ui/codegen.ts`, `ui/src/platform/graphql/**`, `ui/src/features/dashboards/**` |
| 153 | **OPEN** | `plans/153-runtime-boundary-foundation.md` present; README TODO; uncommitted WIP under `ui/src/platform/{external-values,visibility,url,storage,sse}/**` |

### Full-scope blockers

Full Plan 149 cannot land until **both 152 and 153 are retired on `main`**:

1. **Plan 152** must supply the generator/transport and runtime-metric handoff so
   `features/runtime-metrics/api/runtime-metrics.graphql` + generated sibling can
   be created without a parallel decoder or raw `graphql()` string path.
2. **Plan 153** must supply final visibility/cancellation and non-GraphQL URL
   owners so `use-runtime-metrics` and any range/search edges consume permanent
   platform facades (not provisional `@/lib/use-visible` / ad-hoc URL decode).

### Independent work (not landed this pass)

These steps do **not** require 152/153 product APIs and can proceed once the
working tree is clean of concurrent 152/153 WIP (ownership ledger scans all
live `ui/src` files; untracked platform files fail `ui.architecture` and
serialize with `ratchet.toml`):

- Step 1 domain: `domain/story/story-beat.ts`, `domain/runtime-metrics/runtime-metric.ts`
  (`domain/time-range` already final from Plan 100)
- Step 3 presentation: `features/story`, `features/time-range`, `shared/components/page-header`
- Step 4 import-only consumers for story / time-range / page-header
- Step 5 tests/matrix/ratchet for those owners only

**Not independent:** Step 2 runtime-metrics GraphQL + hook + MetricStrip/
RuntimeSnapshotCard orchestration (needs 152 transport + 153 visibility).

### Re-entry

When 152 + 153 are retired on `main` (plan files deleted, README rows gone,
platform owners committed): re-run Step 0 drift check + prerequisite gates, then
execute Steps 1–6 in full, evidence twice green, retire this plan + README row.

## Why This Matters

Runtime metrics, story presentation, range selection, and page headings are
consumed by several routes. Leaving them at root component paths until the final
closure makes later feature plans depend on owners that do not yet exist. It
also encourages copies or temporary deep imports when those plans run in
parallel.

This plan publishes the exact owners first. Later feature plans consume stable
facades and can remain move-only. App, layout, overview, browser infrastructure,
cache behavior, live algorithms, and bundle delivery remain separate rollback
units.

## Fixed Decisions

1. `features/runtime-metrics` owns `MetricStrip`, `RuntimeSnapshotCard`, the
   runtime metric API/schema/mapper/error boundary, and their orchestration.
   `domain/runtime-metrics` owns only framework-neutral readonly metric values.
2. `features/story` owns `StoryTimeline` and its typed presentation contract.
   `domain/story` owns only the framework-neutral `StoryBeat` value.
3. `features/time-range` owns `RangePicker` and its presentation contract.
   `domain/time-range` owns the canonical pure resolved-range and preset values
   when Plan 100 has not already placed them.
4. `shared/components/page-header.tsx` owns `PageHeader`. It is product-neutral,
   receives typed title/back/action content, and imports no route, feature,
   domain, platform, or app/layout implementation.
5. Plan 152 owns the generated runtime-metric GraphQL document/schema and
   transport. Plan 153 owns visibility/cancellation and non-GraphQL URL values.
   Preserve their actual names and behavior; never create a parallel decoder or
   raw generic JSON assertion.
6. Runtime metric behavior is unchanged: the same metric names, scope precedence,
   request fields/variables/count, raw/cache choice, initial fetch, visibility
   pause, live interval, abort/stale-result handling, empty/error suppression,
   unit conversion, grouping, ordering, labels, and charts remain observable.
7. Story ordering, tones, links, time windows, empty text, keys, and formatting
   remain exact. Time-range preset/custom calculations, local date boundaries,
   clock reads, calendar behavior, callback values, labels, and popover behavior
   remain exact. Page-header layout, back behavior, actions, and accessible
   heading semantics remain exact.
8. Consumer changes are import-only. Plans 134-142 and 150 may later move their
   surrounding code, but this plan neither restructures those consumers nor
   changes their tests except for import paths required by the facade switch.
9. Source-owned tests use `features/<owner>/tests/**`,
   `domain/<owner>/tests/**`, or `shared/tests/**`. URL/search behavior remains in
   route tests. No new `__tests__` tree or test-only production export is allowed.
10. Plan 133 owns Query/cache changes, Plan 147 owns polling/live performance,
    and Plan 148 owns lazy/chunk/bundle work. This plan preserves their baselines.

## Target Ownership

```text
ui/src/
  domain/
    runtime-metrics/
      runtime-metric.ts
      tests/runtime-metric.test.ts
    story/
      story-beat.ts
      tests/story-beat.test.ts
    time-range/
      resolved-range.ts
      tests/resolved-range.test.ts
  features/
    runtime-metrics/
      api/
        runtime-metrics.graphql
        runtime-metrics.generated.ts
        load-runtime-metrics.ts
        runtime-metrics-mapper.ts
      model/runtime-metrics-error.ts
      components/
        metric-strip.tsx
        runtime-snapshot-card.tsx
      hooks/use-runtime-metrics.ts
      tests/
        api/runtime-metrics-api.test.ts
        components/metric-strip.test.tsx
        components/runtime-snapshot-card.test.tsx
        integration/use-runtime-metrics.test.tsx
      index.ts
    story/
      components/story-timeline.tsx
      tests/components/story-timeline.test.tsx
      index.ts
    time-range/
      components/range-picker.tsx
      tests/components/range-picker.test.tsx
      index.ts
  shared/
    components/page-header.tsx
    tests/components/page-header.test.tsx
```

Use Plan-100 paths when its ledger names a more precise domain or platform file.
The tree above is not permission to duplicate an existing range, clock, format,
GraphQL, visibility, or runtime-environment owner. Empty directories are
forbidden.

## Ownership And Import Contract

| Capability | Public facade | Allowed consumers | Forbidden ownership |
|---|---|---|---|
| Runtime metrics | `@/features/runtime-metrics` | services, issues, runs, traces, overview when needed | route-local DTOs, copied charts, direct transport in consumers |
| Story | `@/features/story` | runs and traces | copied timeline, route-owned `StoryBeat` |
| Time range | `@/features/time-range` | route features and overview | duplicate picker or range implementation |
| Page header | `@/shared/components/page-header` or the exact Plan-100 shared facade | any feature/route through typed props | layout/nav internals or feature-specific behavior |

Feature facades export only the reviewed component and domain input types needed
by external consumers. Documents, schemas, hooks, mappers, errors, and internal
components are not public. Handwritten wildcard exports and deep feature imports
fail policy.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | exact owners and facade edges; no cycle, deep import, or unknown file |
| UI policy | `cargo xtask policy --only ui.architecture` | route-less feature, domain, shared, runtime, and facade rules pass |
| Test policy | `cargo xtask policy --only ui.tests` | separated owner tests and exact legacy handoffs pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | old paths shrink and no size/export/import exception grows |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/runtime-metrics src/features/story src/features/time-range src/domain/runtime-metrics src/domain/story src/domain/time-range src/shared/tests/components/page-header.test.tsx` | non-zero capability tests pass without diagnostics |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass under Bun; no Node descendant |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 with zero warnings/errors |
| Production build | `cd ui && bun run build` | unchanged routes/rendering and no server-only client edge |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |

All JavaScript and TypeScript commands use exact lock-local tools through Bun
with auto-install disabled. Oxc-backed xtask is the only source graph and policy
oracle. Node, foreign package managers, ESLint, another formatter/parser, manual
generated edits, or an internal npm package are forbidden.

## Scope

**In scope:**

- The five current production components named in the drift check and their
  exact legacy tests.
- Canonical runtime metric, story, and resolved-range domain contracts not
  already placed by Plan 100.
- One explicit facade for each route-less feature and one product-neutral
  page-header shared export.
- Import-only changes for every existing consumer, including the exact removal
  of old-path imports and compatibility rows.
- Capability-owned unit/component/integration tests, matrix rows, ownership
  ledger entries, and structural ratchets.

**Out of scope:**

- App/router/root/layout/shell/navigation/theme/fallbacks, app status, quick
  navigation, or overview ownership (Plans 143 and 150).
- Any product feature restructuring, URL/search redesign, GraphQL/backend change,
  visual redesign, or new capability.
- Playwright fixtures, datasets, screens, projects, CI, managed-stack cases,
  browser breadth, or golden authoring.
- Query/cache behavior (133), live/poll algorithm or interval changes (147), and
  lazy/chunk/minifier/source-map/bundle behavior (148).
- Replacing Plan-100 platform GraphQL, visibility, clock, formatting, storage,
  download, or SSE owners.

## Git Workflow

- Stay on the single active branch; never create another branch or PR.
- Land domain contracts, runtime metrics, story/time-range/page-header, consumer
  import switches, and legacy cleanup as separate green commits.
- Serialize `ui/test-matrix.json`, `ratchet.toml`, and shared consumer import
  commits. Re-read the active branch before each shared-file patch.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable update under repository policy.

## Steps

### Step 0: Freeze behavior and resolve Plan-100 owners

Confirm Plans 100, 129, 152, and 153 are complete and their required gates are
green. Plan 152 supplies the generator/handoff, not a pre-created product file.
Run the drift check and this prerequisite-only subset:

```bash
cargo xtask arch
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build
cargo xtask ci --fast
```

Do not run the target-tree focused command until Steps 1-5 create and move its
files; zero selection is intentionally fatal. Inventory every consumer and
legacy test.
Record runtime metric documents/fields/scope precedence/request count/cache
choice, five-second live interval, initial fetch, visibility/abort behavior,
empty/error rendering, units/grouping/charts; story ordering/tone/link windows;
range preset/custom/clock/calendar behavior; and page-header props/DOM/actions.

Resolve Plan-152 generator/transport and runtime-metric handoff rows, Plan-153
visibility/URL paths, and Plan-100 clock/range/format/shared/test-support paths.
Require every remaining legacy test/private import to have an exact Plan-129
handoff owned by Plan 149.

**Verify:** the complete legacy suite and prerequisite subset pass; the
ownership ledger assigns every source/test exactly once; missing or conflicting
ownership is a STOP condition before movement.

### Step 1: Establish the pure domain contracts

Move only framework-neutral runtime metric, story beat, and resolved-range values
that do not already have a canonical Plan-100 owner. Preserve field optionality,
readonly semantics, nanosecond strings, units, kind/severity values, preset keys,
and exact range calculations. Do not move React, router, Date/browser access,
GraphQL documents, or display formatting into `domain`.

**Verify:** domain tests cover valid/empty/boundary values and exact moved
functions; architecture proves domain imports no React, TanStack, platform,
feature, browser, or transport module.

### Step 2: Move runtime metric decoding and orchestration

Create the named runtime-metrics `.graphql` operation and checked-in generated
sibling through Plan 152. Decode `unknown` once and map once to readonly domain values. Extract the current
request/effect into a cohesive hook or API adapter while preserving request
shape/count, raw/cache selection, run-over-service scope, live `to` calculation,
initial visible fetch, visibility pause, five-second interval, abort generation,
failure-to-empty behavior, and no-points suppression.

Move `MetricStrip` and `RuntimeSnapshotCard` presentation without altering chart
data, units, labels, grouping, ordering, dimensions, text, or empty rendering.
Publish only the reviewed public components and input/domain types.

**Verify:** API/integration tests cover valid/null/malformed/error/abort, exact
requests, visibility/live timer cleanup, stale completion, and zero/partial/full
panels. Component tests prove current units/grouping/empty/chart semantics.

### Step 3: Move story, time-range, and page-header presentation

Move `StoryTimeline` behind the story facade, preserving order, timestamp range,
tones, error/log/trace link selection, five-second log windows, keys, empty text,
badges, duration, and accessibility. Move `RangePicker` behind the time-range
facade, preserving preset and custom local-day boundaries, calendar state,
clock reads, labels, callbacks, and popover interactions.

Move `PageHeader` to its product-neutral shared owner. Keep its public props
minimal and typed; it must not import navigation registries or infer feature
behavior.

**Verify:** focused component tests pass with fake clock/timezone where required;
shared policy proves PageHeader contains no Parallax domain or upper-layer import.

### Step 4: Switch consumers through explicit facades

Replace every old component import with the exact new facade/shared export. Make
no other production change in consumer files. Preserve consumer props and domain
values exactly; adapt types at the capability boundary rather than copying a
type into each feature. Do not move route code, rename an export for convenience,
or add a temporary wildcard barrel.

Update the machine edge ledger with the exact approved consumers. Remove an old
path only after `rg` and Oxc resolution prove no caller remains.

**Verify:** `cargo xtask arch` reports only approved facade edges, the scoped diff
for every consumer is import/type-only, all UI tests pass, and production build
behavior is unchanged.

### Step 5: Move tests and delete legacy owners

Move capability behavior tests to their final owner paths while preserving stable
matrix IDs and assertions. Split the page-header assertions from the legacy shell
test without moving shell behavior. Remove Plan-129 handoff rows atomically with
their legacy file/import. Delete old production files and temporary reexports
only after all callers and tests use final owners.

Set exact shrink-only ratchets: handwritten module at most 300 logical lines,
test scenario at most 500, function/component/hook at most 60, cyclomatic at most
12, cognitive at most 15, and facades restricted to their reviewed exports.

**Verify:** old paths and legacy rows are absent, focused/all tests pass, and
architecture/test/ratchet policies have no Plan-149 migration exception.

### Step 6: Complete the foundation handoff

Record the final facade paths and exact consumer edge list in Plan-100's durable
machine ledger and the existing agent placement policy. Do not claim final
repository documentation closure; Plan 151 owns the live-tree rewrite after all
feature/app/layout moves.

Run every command twice from a clean state. The second run must not modify a
generated file, matrix, ratchet, or tracked source.

**Verify:** all commands exit zero twice, `git diff --check` is clean, and Plans
134-142 plus 150 can resolve every named capability without a deep/old-path import.

## Test Plan

- Runtime metric API/schema/mapper tests for exact document, scope precedence,
  valid/null/malformed/error/abort, values, units, and ordering.
- Runtime metric lifecycle tests for initial fetch, hidden/visible, live/non-live,
  interval, abort, stale completion, failure, and cleanup.
- Metric strip and runtime snapshot component tests for empty/partial/full data,
  grouping, unit conversion, labels, ordering, and accessible chart composition.
- Story domain/component tests for empty, every kind/tone/severity, links, log
  windows, timestamp range, duration, keys, and ordering.
- Time-range domain/component tests for every preset, custom local-day range,
  incomplete selection, current clock, labels, callback, and popover behavior.
- Page-header component tests for heading, back descriptor, actions, layout, and
  absence of navigation/feature coupling.
- Oxc policy fixtures for deep imports, wildcard facades, duplicate domain types,
  route/platform access, product code in shared, and stale old paths.

## Done Criteria

- [ ] Every named source and legacy test has exactly one final owner and no
  Plan-149 handoff remains.
- [ ] `runtime-metrics`, `story`, and `time-range` have explicit minimal facades;
  external consumers use no deep or old-root import.
- [ ] PageHeader is product-neutral under shared and imports no upper/domain layer.
- [ ] Runtime values decode once and map once; no duplicate document/schema/DTO or
  generic JSON assertion exists.
- [ ] Requests, cache calls, timers, visibility, aborts, values, URLs/callbacks,
  rendered output, accessibility, and visuals match the Step-0 baseline.
- [ ] Consumer production diffs outside owned capability files are import/type-only.
- [ ] Tests use final owner topology, retain stable matrix IDs, and import no
  private route implementation.
- [ ] Architecture reports the exact approved capability edges with zero cycle,
  deep import, wildcard facade, unknown file, or stale compatibility path.
- [ ] Query/cache, live-performance, bundle, app/layout/overview, and Playwright
  behavior were not changed.
- [ ] Every command passes twice from clean state with no generated diff.

## STOP Conditions

Stop and report if:

- Plan 100, 129, 152, or 153 is incomplete or its architecture/runtime/test evidence
  is red;
- Plan 100 already assigned a named capability to a conflicting permanent owner;
- Plan 152's generator/handoff cannot represent the frozen runtime-metric
  GraphQL operation or Plan 153 lacks its visibility/URL mechanism;
- preserving behavior requires changing a request, cache, timer, visibility,
  URL/search value, callback, chart, text, or product contract;
- a consumer cannot switch through a facade without restructuring its feature;
- a domain value would require React, TanStack, browser, transport, or feature code;
- PageHeader cannot remain product-neutral;
- Query/cache work, live optimization, bundle work, app/layout/overview movement,
  browser infrastructure, another package, or manual generated edits appear
  necessary; or
- a required gate fails twice after one reasonable correction.

## Maintenance And Removal

Future capability changes update the domain contract, runtime schema/mapper/error,
facade, tests, matrix rows, and ratchets together. New consumers require an exact
reviewed facade edge; copying a component or deep-importing an internal is not a
shortcut.

Delete this plan and its README row only after all old files/tests/imports and
Plan-149 exceptions are gone, the permanent facades are durable, and every done
criterion and command is green.
