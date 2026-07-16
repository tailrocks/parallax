# Plan 140: Move runs, sessions, and live observation into one feature

> **Executor instructions**: Move the run list/detail implementation into a
> bounded feature after the logs facade exists. Preserve `/runs/` and
> `/runs/$runId`, search/tab/range behavior, two-stage detail loading, current
> cache calls, live SSE/poll timing and ordering, runtime-snapshot bounds, lazy
> bundle behavior, download output, all loading/empty/error states, and rendered
> behavior. Use only the Plan-141 logs facade for log records/table capability.
> Do not implement TanStack Query or change cache ownership; plan 133 owns that.
> Stop rather than weakening runtime decoding or creating a cross-feature deep
> import.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/runs.index.tsx 'ui/src/routes/runs.$runId.tsx' ui/src/routes/__tests__/-runs.test.tsx ui/src/components/console/agent-session.tsx ui/src/components/console/__tests__/agent-session.test.tsx ui/src/components/live-stream-panel.tsx ui/src/components/runtime-snapshot.tsx ui/src/features/runs ui/test-matrix.json ratchet.toml`
> Plans 100, 129, 141, and 149 intentionally change lower-layer, log, and
> route-less capability paths. Reconcile those through their facades; STOP if
> the run contract itself has changed.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 134, 141, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / runs / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — upstream UI foundation, logs, investigations, and browser plans are incomplete

## Contract reconciliation (2026-07-17)

Plans 156/157 replace the runs surface this plan was written to move: the
routes become `/invocations` + `/invocations/$invocationId`, the GraphQL
fields and SSE params are renamed (`runId` → `invocationId`), the run/
observed-run merge moves server-side, and new tabs (sessions/screens/actions,
jobs/cycles, conversations) exist. At execution time this plan is re-baselined
as the **invocations feature migration**: same migration discipline
(behavior-preserving move behind `features/invocations` with explicit
facades), applied to the post-157 surface. Re-run Step 0 characterization
against that head; every "run" reference below reads as "invocation"; the
agent-session model reads as the `gen_ai.*` conversations model. See
plans/157-cli-invocation-observability-ui.md and the Unified CLI
Observability note in plans/README.md.

## Why This Matters

The run routes contain 1,216 lines and currently combine list normalization,
range filtering, two-stage GraphQL loading, live log/span streams, polling,
runtime metrics, agent sessions, evidence bundle fetching/downloading, tables,
and presentation. Detail code also imports implementation and types from the
list route. The reusable logs table is another undeclared deep dependency. This
plan creates one runs facade and one explicit `runs -> logs facade` edge without
changing behavior.

## Current Paths And Responsibilities

| Current path | Current responsibility at `e3e7997` | Required final owner |
|---|---|---|
| `ui/src/routes/runs.index.tsx` | Run/observed-run merge, duration/status/range filtering, search, list table/status badge | Thin route plus `features/runs` list modules |
| `ui/src/routes/runs.$runId.tsx` | Detail and snapshot loaders, live SSE/polling, lazy bundle, stats/issues/traces/logs/story/runtime/session UI and download | Thin route plus `features/runs` |
| `ui/src/routes/__tests__/-runs.test.tsx` | Merge/range, list/detail, download and runtime-bound tests | Feature behavior under `features/runs/tests/**`; route contracts under `routes/tests/` |
| `ui/src/components/console/agent-session.tsx` | Agent step/session model and timeline card | Runs model/component |
| `ui/src/components/console/__tests__/agent-session.test.tsx` | Step links/token/error rendering | Runs component tests |
| `ui/src/components/live-stream-panel.tsx` | Run-only stream summary and live event stack at baseline | Runs components |
| `ui/src/components/runtime-snapshot.tsx` | Runtime metric chart consumed by runs and services | Consume Plan-149 `@/features/runtime-metrics` facade; do not claim it as run-only |
| Plan-141 logs facade | Stable decoded log record and `LogsTable` capability | Only approved product-feature dependency; route-less capability edges remain with Plan 149 |
| Plan-149 page-header/time-range/runtime-metrics/story facades | Route-less UI capabilities and minimum readonly inputs | Consume through explicit final facades; do not duplicate or deep-import |
| Plan-153 SSE/visibility/search/frame contracts plus Plan-100 clock/download/format/pure-range contracts | Technical platform and pure domain lower layers | Consume through canonical owners; do not duplicate |

At baseline the detail route imports `RunStatusBadge`, `durationNs`, and `RunRow`
from the list route. The final routes never import each other and export only
`Route`.

## Fixed Behavior And Ownership

1. Keep exact `/runs/` and `/runs/$runId` paths, run ID params, list search keys,
   detail `tab`/range search behavior, link propagation and defaults.
2. Preserve list request and detail's current two-stage request sequence: run
   metadata first, then traces/logs/story/runtime/session using the derived
   snapshot lower bound. Preserve request documents, variables, count,
   `graphqlCached`/`graphql` selection, and lazy bundle timing.
3. Preserve live endpoints, enable/visibility behavior, 250 ms platform flush,
   10-second run polling, tolerated transient poll failures, buffer caps/order,
   stream status and DOM identity. This plan decodes frames but does not optimize
   live algorithms.
4. Plan 152 schemas parse run, observed-run, detail, runtime, session, and
   bundle GraphQL payloads as `unknown`. Run-owned schemas instantiate Plan
   153's mechanism for search and log/span live frames. Feature API mappers
   produce readonly domain values once. Log wire/domain types come only from
   Plan 141's facade.
5. `model/runs-error.ts` owns discriminated list/detail/live/bundle failures.
   Preserve current fatal loader, tolerated poll, stream-status, and visible
   bundle error boundaries.
6. The `runs -> logs` dependency is explicit and limited to reviewed facade
   exports for the decoded log model and log table. No log internals, route file,
   raw document fields or decoder may be imported.
7. `PageHeader`, `RangePicker`, runtime metric presentation, and story timeline
   remain at the owners established by Plan 149. SSE/visibility/search/frame
   mechanisms remain at Plan 153; clock/download/format/pure-range contracts
   remain at Plan-100 owners.
   Runs owns its adapters/orchestration, not copies of shared infrastructure.
8. Prefer pure functions and readonly state. Use a class only for real lifecycle
   or invariant-bearing mutable identity; React effects/refs own the current
   stream, poll, lazy-load and cancellation lifecycles.

## Plan 149 Capability Contract

- Runs imports `PageHeader` from `@/shared/components/page-header`, `RangePicker`
  from `@/features/time-range`, `MetricStrip` and `RuntimeSnapshotCard` plus their
  minimum readonly inputs from `@/features/runtime-metrics`, and `StoryTimeline`
  plus its minimum readonly story-beat input from `@/features/story`.
- Use explicit named value/type imports only. Do not deep-import plan 149
  internals, use wildcard barrels, copy a legacy capability into runs, or defer
  a completed run capability import to plan 143.
- Plan 152 owns GraphQL/cache, Plan 153 owns SSE/visibility/search/frame
  mechanisms, and Plan 100 retains clock/download/format/pure-range and other
  technical/domain foundations.

## Target Tree

```text
ui/src/features/runs/
  api/
    runs-list.graphql
    runs-list.generated.ts
    load-runs.ts
    run-detail.graphql
    run-detail.generated.ts
    load-run-detail.ts
    run-poll.graphql
    run-poll.generated.ts
    poll-run.ts
    run-bundle.graphql
    run-bundle.generated.ts
    load-run-bundle.ts
    run-live-schemas.ts
    runs-mapper.ts
  model/
    run-record.ts
    run-row.ts
    runs-search.ts
    runs-search-schema.ts
    run-duration.ts
    run-runtime-window.ts
    run-live-event.ts
    agent-session.ts
    runs-error.ts
  components/
    runs-page.tsx
    runs-table.tsx
    run-status-badge.tsx
    run-detail-page.tsx
    run-stats.tsx
    run-live-panel.tsx
    run-agent-session-card.tsx
    run-runtime-section.tsx
    run-related-issues.tsx
    run-related-traces.tsx
    run-bundle-card.tsx
  hooks/
    use-run-live-observation.ts
    use-run-bundle.ts
  tests/
    api/runs-api.test.ts
    api/run-live-contract.test.ts
    model/run-row.test.ts
    model/run-runtime-window.test.ts
    model/agent-session.test.ts
    components/runs-page.test.tsx
    components/run-detail-page.test.tsx
    components/run-agent-session-card.test.tsx
    integration/run-live-observation.test.tsx
  index.ts
ui/src/routes/
  runs.index.tsx
  runs.$runId.tsx
  tests/runs-routes.test.tsx
ui/tests/e2e/
  datasets/runs.ts
  screens/runs-screen.ts
  contracts/runs.spec.ts
  full-stack/runs.spec.ts
  accessibility/runs-accessibility.spec.ts
  mobile/runs-mobile.spec.ts
  visual/runs.visual.spec.ts
  visual/goldens/
    runs-list.png
    runs-detail-session.png
    runs-bundle.png
    runs-empty.png
    runs-error.png
```

Plan 152 provides the generator/template and handoff rows, not these product
files. This plan creates each named operation and exact `.generated.ts` sibling;
live/search schemas separately instantiate Plan 153. `run-runtime-section.tsx` composes
`RuntimeSnapshotCard` through plan 149's `@/features/runtime-metrics` facade; it
does not move the cross-feature primitive into runs.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | no route-to-route, deep logs, cycle, or unknown-owner edge |
| UI architecture | `cargo xtask policy --only ui.architecture` | runs facade, approved logs edge, runtime/client and only-Route rules pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | run routes/modules/functions/exports shrink without new exception |
| Test ownership | `cargo xtask policy --only ui.tests` | all run/session/live matrix IDs resolve under run tests |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/runs/tests` | non-zero run tests pass with no diagnostics |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0, zero warnings/errors |
| Unit suite | `cd ui && bun run --bun test:ci` | all tests pass under Bun; no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @runs` | non-zero fixture-backed run rows pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @runs` | non-zero managed GreptimeDB + Turso run rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross -- --grep @runs` | non-zero Firefox/WebKit/mobile run rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y -- --grep @runs` | non-zero axe/keyboard/focus run rows pass |
| Visual | `cd ui && bun run test:browser:visual -- --grep @runs` | non-zero canonical run visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | run matrix/spec/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | run storage/seed/lifecycle ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | run engine/mobile/a11y/visual ownership passes |
| Build | `cd ui && bun run build` | exit 0, generated routes current, no server-only client reachability |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | run real-stack and breadth lanes are green |

Use only exact lock-local Bun commands with installation disabled. Oxc-backed
xtask is the source parser/resolver/ratchet authority. Node, foreign package
managers, ESLint/plugins, a second graph, or direct browser tooling are forbidden.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/runs.spec.ts` owns plan 145's delegated, non-empty
`@runs` row. Seed a deterministic run-correlated trace, logs, metrics, and agent
session attributes through public OTLP using `datasets/runs.ts`; wait on named
public run and related-signal predicates; then drive list/detail navigation,
the two-stage related trace/log/runtime/session presentation, lazy bundle load,
and the existing bundle download through `screens/runs-screen.ts`. Do not repeat
plan 145's distinct `@storage` discovery or live-transport cases, and consume
only plan 141's public log/table facade.

Run one worker against managed GreptimeDB plus an isolated Turso database. Use
only public OTLP, GraphQL, and UI boundaries with bounded readiness predicates;
never write/read database internals, intercept browser responses, or use fixed
sleeps.

**Verify**: `cd ui && bun run test:browser:full -- --grep @runs` selects at
least one plan-140 row and passes with the real-stack runtime manifest, the
owned download assertion, and clean process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@runs` row that consumes plan 146's projects. Run list/
detail, tabs/range, session state, live status, runtime snapshot, lazy bundle,
and bundle download in Firefox and WebKit. Cover log/session tables, tabs,
long commands, download action, and overflow on both mobile device projects.
Run axe plus keyboard/focus/Escape/restoration checks and maintain canonical list,
detail/session, bundle, empty, and error visual states.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @runs && bun run test:browser:a11y -- --grep @runs && bun run test:browser:visual -- --grep @runs` selects non-zero breadth rows and passes without response interception, broad masking, or duplicate log capability.

## Scope

**In scope:**

- Both run routes and all run-specific API/model/component/hook responsibilities.
- Agent-session source/test and the run-only live panel source.
- Run composition/import updates for Plan-149 page-header/time-range/runtime-
  metrics/story facades, Plan-100 SSE/visibility/clock/download/format/pure-range
  owners, and the Plan-141 log facade/table.
- New `features/runs/**`, separated run tests, run matrix rows and ratchets.
- Feature-owned runs dataset/screen/contract/full-stack/accessibility/mobile/
  visual/golden files and their non-empty plan 144-146 matrix rows.
- Normal tool-generated route-tree update, never a manual edit.

**Out of scope:**

- Any Plan-141 logs implementation, schema, table behavior or test edit; consume
  its facade only.
- Reassigning Plan-149 runtime-metrics/story/time-range/page-header capability
  implementations or Plan-100 platform SSE/visibility/clock/download and pure-
  domain implementations.
- TanStack Query/cache/freshness/invalidation and `graphqlCached` deletion (plan
  133), live buffer/performance or polling optimization (plan 147), bundle
  format, or download redesign.
- Backend/API/URL changes, new run/session features, visual redesign, generated/
  shadcn manual edits, other features, internal
  packages/project references, catch-all modules or class-per-file ceremony.
- Shared plan 144-146 Playwright configuration, fixtures, reporters, lifecycle,
  CI, matrix schema, and browser infrastructure; consume them read-only.

## Git Workflow

- Work only on the active branch; never create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any shared generated registry/config
  are serialized feature-scoped commits. Re-read current content, require no
  uncommitted writer, change only runs rows, land green, then hand off. Never
  regenerate or replace another feature's content.
- Land model/API, live/bundle hooks, components, and route/test closure as
  independently reviewable green changes.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable green update as repository policy requires.

## Steps

### Step 0: Freeze run and cross-feature contracts

Confirm plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete. Confirm plan 141 is
also complete and exposes exactly the decoded log record/table capability needed
by run detail. Run the drift check and this exact prerequisite-only subset:

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

Do not run focused target paths or any `--grep @runs` command until Step 5
creates and registers those files; zero selection is intentionally fatal. Record
the approved `runs -> logs` facade edge. Resolve plan 149's final page-header,
time-range, runtime-metrics, and story facades; Plan 152's run/bundle GraphQL
contracts; Plan 153's SSE/visibility/search/frame paths; and Plan 100's clock/
download/format/pure-range paths. Record
URLs/search, list/detail/bundle/poll/SSE request sequence/count/
cache choice, timers, visibility behavior, buffer ordering/caps, snapshot bounds,
errors and browser markers.

Require every runs/session `__tests__` path and private route import to have an
exact plan-129 legacy handoff owned by plan 140. Stop on a missing, wildcard,
expired, or differently owned row; delete each row when its test/import moves.

Confirm plan 145 reserves `@runs` for
`ui/tests/e2e/full-stack/runs.spec.ts` and its shared `@storage` specs retain only
foundation run discovery/live-transport behavior. Consume the feature
reservation without duplicating either foundation stable ID or scenario.

**Verify**: every prerequisite command above exits 0, the logs facade and legacy
handoffs are exact, and the delegated runs row is reserved but not yet required
to select a feature spec.

### Step 1: Extract the readonly run model

Move run/observed-run/detail/trace/live/session shapes, merge and range overlap,
duration/status, search parsing/patching, detail-row mapping, snapshot lower-bound
logic, live event projection and typed errors into cohesive model files. Inject
the canonical clock into pure calculations where Plan 100 provides it while
preserving current default time behavior. Remove the detail route's imports from
the list route by sharing internal feature model/component modules.

Move existing assertions into model tests. Cover CLI/external merge precedence,
running/finished/external durations, overlapping ranges, invalid nanosecond input
contract, snapshot fallback, status tones and agent token/error steps.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/runs/tests/model && bun run typecheck`
must pass; architecture must show no route-to-route edge and ratchets must shrink.

### Step 2: Move decoded list/detail/live/bundle API ownership

Place canonical named documents/schemas under runs API ownership. Keep the two
detail stages separate and preserve derived bounds. Implement adapters/mappers
for list, detail/rest, poll and bundle. Move log/span SSE frame decoding to
canonical schemas and return typed domain batches; do not let `JSON.parse` plus
assertion enter feature state. Keep the generic EventSource lifecycle in the
Plan-153 platform facade and use the Plan-141 log domain model.

Map errors into the run union while retaining fatal loader, tolerated poll,
stream status and visible bundle boundaries exactly.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/runs/tests/api && bun run typecheck`
must cover valid/null/malformed/error/abort frames and requests. Step-0 request
sequence/count/cache/timing remains exact.

### Step 3: Extract run lifecycles and presentation

Move live observation and bundle fetching into the two hooks. Represent each
async state with discriminated unions; keep effect/ref cancellation, polling,
visibility, stream reconnect and stale bundle guards exact. Do not introduce a
class or module singleton.

Split list/table/status and detail/stat/session/live/runtime/related/bundle
presentation into target components. The log section imports `LogsTable` and log
model only from `@/features/logs`. The runtime/story/metric/download sections
compose plan 149's runtime-metrics/story facades and plan 100's download facade.
Page chrome and range selection compose the plan 149 page-header/time-range
facades. Preserve DOM keys, accessible names, live order/caps, links/search,
bundle text/download filename and all empty/loading/error states.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/runs/tests/components src/features/runs/tests/integration && bun run check && bun run lint`
must pass, including fake-clock stream/poll/bundle cleanup and browser parity.

### Step 4: Publish the facade and thin both routes

Publish explicit value/type exports in `features/runs/index.ts`: only route
entry/load/search contracts and reviewed public run contracts/components. Keep
`runs -> logs` in the machine dependency map with exact exports/reason/owner.
Wildcard exports and consumer deep imports fail.

Reduce route files to route creation, params/search/loader wiring, boundaries and
composition. Both export only `Route`, import the runs facade, never import each
other, and never import logs/platform internals.

**Local verification:**
`cargo xtask arch && cargo xtask policy --only ui.architecture && cd ui && bun run build`
must prove unchanged route IDs, only-`Route` exports, facade-only edges, no cycle,
and no server-only module in client chunks.

### Step 5: Close tests, matrix, ratchets and legacy paths

Move run/agent-session/live feature tests to `features/runs/tests/**` and exact
URL/search/tab/range/loader/error/navigation contracts to
`routes/tests/runs-routes.test.tsx`. Remove private route imports and exercise
route behavior through public APIs. Preserve matrix IDs/assertions and delete old
test files. Do not create another `__tests__` tree.

Create or extend the exact feature-owned `datasets/runs.ts`,
`screens/runs-screen.ts`, fixture contract, full-stack, accessibility, mobile,
visual, and named golden files in the Target Tree. Consume plan 145's reserved
`@runs` row, register each feature matrix ID/project once, and make every grep-
scoped selection non-empty. Shared plans 144-146 fixtures, configuration,
reporters, lifecycle code, and infrastructure remain read-only. Keep the owned
bundle-download assertion inside the run specs rather than changing shared
download diagnostics.

Ratchet routes to 150 lines, new modules to 300, functions/components/hooks to
60 and complexity to 12/15. Remove resolved route-to-route, old-path, export,
size, assertion and test-layout rows; no new exception may grow.

**Local verification:** run every command twice. Every `--grep @runs` selection
must be non-zero, `git diff --check` must be clean, and scoped status must
contain only allowed files.

## Test Plan

- `tests/api/runs-api.test.ts`: exact list/two-stage detail/poll/bundle requests,
  valid/null/malformed/error/cancel and typed error boundaries.
- `tests/api/run-live-contract.test.ts`: log/span valid/malformed/oversized/empty
  frames and secret-safe diagnostics before state mutation.
- `tests/model/run-row.test.ts`: merge precedence, range overlap, durations,
  statuses, filtering/search and deterministic clock cases.
- `tests/model/run-runtime-window.test.ts`: valid start, fallback bound and exact
  query range calculation.
- `tests/model/agent-session.test.ts`: step kinds, links, tokens, errors, empty and
  truncated session behavior.
- `tests/components/runs-page.test.tsx`: rows/filters/links/status/durations and
  empty behavior.
- `tests/components/run-detail-page.test.tsx`: not-found, stats, related data,
  session/runtime/story/log table and bundle loading/error/download.
- `tests/integration/run-live-observation.test.tsx`: visibility/reconnect, timers,
  poll tolerance, ordering/caps, cleanup, stream state and stable identity.
- `routes/tests/runs-routes.test.tsx`: exact URLs/search/tab/range/load/error
  and client navigation contracts.
- Fixture browser: deterministic run list/detail/tab/live/session/runtime/bundle
  states and navigation through `@runs`.
- Real stack: public-OTLP run correlation, related signals/runtime/session, and
  lazy bundle download against managed GreptimeDB plus isolated Turso.
- Browser breadth: selected Firefox/WebKit/mobile behavior, axe/keyboard/focus,
  and named canonical run visuals.

## Done Criteria

- [ ] Both run routes export only `Route`, retain exact paths/search contracts,
  import no route implementation, and are at or below 150 lines.
- [ ] Run list/detail/API/live/session/bundle ownership lives under
  `features/runs`; external consumers use its explicit facade.
- [ ] The only cross-feature edge is the approved `runs -> logs` facade subset;
  no log deep import or duplicate log type/schema exists.
- [ ] Runtime payloads/frames are decoded once and mapped once with typed run
  errors preserving fatal/tolerated/visible boundaries.
- [ ] Request/cache/timer/visibility/order/cap/snapshot/bundle/download and UI
  behavior match the baseline; no Query/cache change landed.
- [ ] Run/session/live feature tests live under `features/runs/tests/**`, route
  contracts live under `routes/tests/`, and no old/private-route test remains.
- [ ] Runs-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and canonical
  visual rows are non-empty and green.
- [ ] The feature-owned `@runs` managed-stack row is non-empty, uses public
  OTLP/GraphQL/UI boundaries, and passes against GreptimeDB + Turso.
- [ ] Architecture/tests/ratchets and all Bun unit/browser/build/aggregate gates
  pass twice with no Node.
- [ ] No unneeded class, catch-all module, wildcard export, manual generated edit
  or out-of-scope feature change exists.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`,
  `@/features/time-range` `RangePicker`, `@/features/runtime-metrics`
  `MetricStrip`/`RuntimeSnapshotCard`, or `@/features/story` `StoryTimeline`
  facade/input contract is absent/incompatible; do not copy a legacy component,
  deep-import internals, add a wildcard export, or defer import repair to plan
  143;
- Plan 141's facade lacks the exact decoded log/table capability or requires a
  deep import/change to its implementation;
- prerequisites or forced-Bun/browser run evidence are incomplete/red;
- plan 145 lacks the delegated `@runs` reservation/shared managed-stack
  infrastructure, or Step 5 cannot make it a non-empty public-boundary row with
  clean one-worker teardown;
- a shared `@storage` discovery/live spec or another feature owns the same runs
  stable ID/scenario, or the reservation points at a different file;
- feature browser evidence requires editing shared plans 144-146 fixtures,
  configuration, lifecycle, reporters, CI, or matrix schema;
- request order/count/cache choice, live timing/order/caps, snapshot/bundle or
  URL/search behavior has drifted before movement;
- Plan 152's generator/handoff cannot represent a frozen run/session/bundle
  GraphQL operation or Plan 153 lacks the search/SSE/frame mechanism;
- preserving behavior requires Query/cache/live-performance/backend/product
  changes, another feature edit, or a second SSE/log decoder;
- runtime-metrics/story/page-header/time-range ownership conflicts with plan
  149's ledger, or download/SSE/visibility/clock/pure-range ownership conflicts
  with plan 100's ledger and cannot be consumed through the exact facade;
- architecture becomes cyclic or structural limits require arbitrary splitting;
  or
- a required gate fails twice after a reasonable correction.

## Maintenance And Required Deletions

Future run work updates the canonical schemas/mappers/errors, run facade, logs
edge, tests/matrix and ratchets together. Plan 133 may add run query modules and
replace cache ownership without moving these files.

Delete before retiring this plan:

- `ui/src/routes/__tests__/-runs.test.tsx`;
- `ui/src/components/console/agent-session.tsx` and its old test;
- `ui/src/components/live-stream-panel.tsx` after its run-owned replacements are
  the only consumers;
- every run implementation export and route-to-route import from both routes;
- every temporary run old-path reexport; and
- every completed run migration exception/ledger row.

Do not delete plan 149's canonical runtime-metrics or story capability owners.
Delete this plan and its README row only after all required deletions and done
criteria are durable and green.
