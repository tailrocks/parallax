# Plan 141: Move logs and the reusable log table into one feature

> **Executor instructions**: Create the canonical logs feature and its reviewed
> facade without changing `/logs`, URL/search serialization, GraphQL request and
> mutation behavior, live-tail lifecycle/order/caps, stale-page guards, table
> identity/virtualization, document-sheet behavior, loading/empty/error states,
> or cache ownership. Keep generic EventSource/visibility/timer mechanics at the
> Plan-153 platform owner; logs owns only log schemas, mapping, state and
> orchestration. Publish the minimum decoded log/table contract required by plan
> 140. TanStack Query and cache correction belong to plan 133.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/logs.tsx ui/src/components/logs-table.tsx ui/src/hooks/use-live-stream.ts ui/src/routes/__tests__/-logs.test.tsx ui/src/hooks/__tests__/use-live-stream.test.tsx ui/src/features/logs ui/test-matrix.json ratchet.toml`
> Plans 100, 129, and 149 may already have relocated the generic live-stream
> platform hook/test and route-less capability imports. Follow their ownership
> ledger. STOP if log behavior or the platform contract differs materially from
> this inventory.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / logs / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The logs route and table contain 1,354 lines spanning search, queries, saved-view
mutations, live-tail state, paging/race guards, charting, column serialization,
virtualization, document parsing and presentation. The generic live-stream hook
is mixed into the same ad hoc structure, while run detail imports the log model
and table directly from a root component. This plan creates one logs owner, one
platform SSE boundary, and one narrow facade for future consumers.

## Current Paths And Responsibilities

| Current path | Current responsibility at `e3e7997` | Required final owner |
|---|---|---|
| `ui/src/routes/logs.tsx` | Search schema, loader/query construction, saved views, context window, paging/live state, histogram and menus | Thin route plus `features/logs` |
| `ui/src/components/logs-table.tsx` | Log wire-like shape, column/severity helpers, stable keys, virtualization, rows and document sheet | Logs model/components |
| `ui/src/hooks/use-live-stream.ts` | Generic EventSource, visibility pause/reconnect, buffering and flush lifecycle | Plan-153 platform SSE facade, not logs internals |
| `ui/src/routes/__tests__/-logs.test.tsx` | Search/window/columns/severity, table virtualization/identity/keyboard/context and saved views | Feature behavior under `features/logs/tests/**`; route contracts under `routes/tests/` |
| `ui/src/hooks/__tests__/use-live-stream.test.tsx` | Generic EventSource lifecycle tests | Plan-153 platform test owner; log-specific frame/orchestration cases move under logs tests |
| Plan-152 GraphQL generator/handoff | Runtime validation template for queries and saved-view mutations | Create named operations and generated siblings under `features/logs/api/` |
| Plan-153 SSE/search/JSON mechanism | URL/saved-state decoding and live-frame entry | Instantiate with log-owned schemas/mapping; do not duplicate platform code |

At baseline the route exports saved-view/search/window helpers and menus; the
root table exports `LogDoc`, column/severity helpers and `LogsTable`. The final
route exports only `Route`; all external log consumers import reviewed facade
names, never these old paths.

## Fixed Behavior And Ownership

1. Preserve the exact `/logs` route, `q/service/sev/range/from/to/live/cols/
   anchor` search acceptance, serialization, defaults and navigation behavior.
2. Preserve request operations/variables/count and current raw-versus-cached
   selection for initial, live-zero-list, around-anchor, histogram, load-older,
   save and delete actions. Do not add Query keys, invalidation or cache changes.
3. Preserve page size, histogram step, context window, stale-generation guard,
   saved-view order/state, live URL parameters, frame flush, incoming order,
   buffer caps, reconnect labels and current malformed-frame outcome.
4. Plan 152 schemas parse all GraphQL responses as `unknown`. Log-owned schemas
   instantiate Plan 153 for search, saved-view state, attribute/resource JSON,
   and live frames. A logs mapper produces one readonly `LogRecord` domain shape
   and stable UI identity once.
5. `model/logs-error.ts` owns discriminated load/page/view/live failures. Preserve
   loader-fatal, inline older/view errors, platform reconnect status and current
   visible text/boundaries.
6. The generic EventSource hook remains under `platform`; `use-live-logs.ts`
   supplies log URL construction, frame decoding, identity and feature state. Do
   not fork the generic hook or expose it through the logs facade.
7. Publish only the decoded `LogRecord` contract and reusable `LogsTable`
   capability needed by run detail, plus stable log feature entry contracts.
   Consumers cannot import log API/model/component internals.
8. Preserve row DOM identity, virtualization threshold, anchor state, keyboard
   activation, selected document/search, trace/run links and raw copy output.
9. Use pure functions/readonly values. React hooks own lifecycle/mutable identity;
   no class or module singleton is expected.

## Plan 149 Capability Contract

- Logs imports `PageHeader` from `@/shared/components/page-header` and
  `RangePicker` plus only its minimum reviewed resolved-range input contract from
  `@/features/time-range`.
- Use explicit named value/type imports only. Do not deep-import plan 149
  internals, use wildcard barrels, copy a legacy capability into logs, or defer
  a completed log capability import to plan 143.
- Plan 152 owns GraphQL/cache, Plan 153 owns generic SSE/visibility/search/JSON,
  and Plan 100 retains clock/format/pure-range and other technical/domain foundations.
  Logs owns typed values and feature orchestration only.

## Target Tree

```text
ui/src/features/logs/
  api/
    logs-search.graphql
    logs-search.generated.ts
    load-logs.ts
    load-older-logs.ts
    log-saved-views-list.graphql
    log-saved-views-list.generated.ts
    log-saved-view-save.graphql
    log-saved-view-save.generated.ts
    log-saved-view-delete.graphql
    log-saved-view-delete.generated.ts
    save-log-view.ts
    delete-log-view.ts
    live-log-schema.ts
    logs-mapper.ts
  model/
    log-record.ts
    logs-search.ts
    logs-search-schema.ts
    log-window.ts
    log-columns.ts
    log-severity.ts
    log-document-fields.ts
    saved-log-view.ts
    logs-error.ts
  components/
    logs-page.tsx
    logs-histogram.tsx
    logs-table.tsx
    virtualized-logs-table.tsx
    log-row.tsx
    log-document-sheet.tsx
    log-column-menu.tsx
    saved-log-views-menu.tsx
  hooks/
    use-live-logs.ts
    use-log-pagination.ts
    use-saved-log-views.ts
  tests/
    api/logs-api.test.ts
    api/live-log-contract.test.ts
    model/logs-search-window.test.ts
    model/log-columns-severity.test.ts
    model/log-document-fields.test.ts
    components/logs-page.test.tsx
    components/logs-table.test.tsx
    components/log-document-sheet.test.tsx
    integration/live-logs.test.tsx
  index.ts
ui/src/routes/
  logs.tsx
  tests/logs-route.test.tsx
ui/tests/e2e/
  datasets/logs.ts
  screens/logs-screen.ts
  contracts/logs.spec.ts
  full-stack/logs.spec.ts
  accessibility/logs-accessibility.spec.ts
  mobile/logs-mobile.spec.ts
  visual/logs.visual.spec.ts
  visual/goldens/
    logs-search.png
    logs-populated-table.png
    logs-document-sheet.png
    logs-empty.png
    logs-live-error.png
```

Plan 152 provides the generator/template and handoff rows, not these product
files. This plan creates each named operation and exact `.generated.ts` sibling;
live/search/attribute schemas separately instantiate Plan 153. The Plan-153
platform live-stream path is deliberately absent from this feature tree.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | no route/deep/cycle/unknown edge; approved future runs facade use only |
| UI architecture | `cargo xtask policy --only ui.architecture` | log facade, platform SSE, route export, runtime decode and client rules pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/table/functions/exports shrink without new exception |
| Test ownership | `cargo xtask policy --only ui.tests` | all log matrix IDs resolve below logs tests; generic SSE keeps platform owner |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/logs/tests` | non-zero log tests pass without diagnostics |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0, zero warnings/errors |
| Unit suite | `cd ui && bun run --bun test:ci` | all tests pass under Bun; no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @logs` | non-zero fixture-backed log rows pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @logs` | non-zero managed GreptimeDB + Turso log rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross -- --grep @logs` | non-zero Firefox/WebKit/mobile log rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y -- --grep @logs` | non-zero axe/keyboard/focus log rows pass |
| Visual | `cd ui && bun run test:browser:visual -- --grep @logs` | non-zero canonical log visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | log matrix/spec/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | log storage/seed/lifecycle ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | log engine/mobile/a11y/visual ownership passes |
| Build | `cd ui && bun run build` | exit 0, route tree current, no server module in client chunks |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | log real-stack and breadth lanes are green |

Every JS/TS command is exact-lock and Bun-forced with auto-install disabled.
Oxc-backed xtask policy is authoritative for imports, runtime suffixes, exports,
test topology and ratchets. Node/foreign managers/ESLint/a second graph are not
fallbacks.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/logs.spec.ts` owns plan 145's delegated, non-empty
`@logs` row. Seed deterministic run/trace-correlated log records through public
OTLP using `datasets/logs.ts`; wait on a named public log predicate; then drive
search/severity/service filtering, anchor context, document-sheet fields,
trace/run links, and saved-view create/delete through `screens/logs-screen.ts`.
Prove the saved view survives a fresh BrowserContext through Turso. Do not
repeat plan 145's distinct `@storage` discovery or live-transport scenario.

Run one worker against managed GreptimeDB plus an isolated Turso database. Use
only public OTLP, GraphQL, and UI boundaries with bounded readiness predicates;
never write/read database internals, intercept browser responses, or use fixed
sleeps.

**Verify**: `cd ui && bun run test:browser:full -- --grep @logs` selects at least
one plan-141 row and passes with the real-stack runtime manifest and clean
process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@logs` row that consumes plan 146's projects. Run search/
filter/context, paging, saved views, document sheet, live/reconnect, row links,
and virtualization in Firefox and WebKit. Cover dense/long log values, column
selection, sheet/menu controls, touch scrolling, and overflow on both mobile
device projects. Run axe plus keyboard/focus/Escape/restoration checks and keep
canonical search, populated table, document sheet, empty, and error/live visual
states.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @logs && bun run test:browser:a11y -- --grep @logs && bun run test:browser:visual -- --grep @logs` selects non-zero breadth rows and passes without response interception, broad row masking, or fixed timing sleeps.

## Scope

**In scope:**

- The log route, root log table, and every log-specific API/model/component/hook
  responsibility listed above.
- Log-specific consumption of Plan-149 page-header/time-range facades and
  Plan-153 platform SSE/visibility/search/JSON and Plan-100 format/pure-range/chart owners, plus cleanup
  of the old generic hook compatibility path when all callers already use the
  platform owner.
- New `features/logs/**`, separated log tests, log matrix and ratchet rows.
- Explicit facade exports for `LogRecord` and `LogsTable` used by plan 140.
- Feature-owned logs dataset/screen/contract/full-stack/accessibility/mobile/
  visual/golden files and their non-empty plan 144-146 matrix rows.
- Tool-generated route-tree refresh through build only.

**Out of scope:**

- Generic EventSource/visibility/timer implementation or its platform-owned
  contract tests after Plan 153; do not pull them into logs.
- Run feature implementation; Plan 140 consumes this facade later.
- TanStack Query/cache/freshness/invalidation and old cache deletion (plan 133),
  live algorithm/page-size/threshold/timer optimization (plan 147), or bundle
  work (plan 148).
- Backend/API/URL/search changes, new log features, visual redesign, generated/
  shadcn manual edits, other features, catch-all
  modules, project references/internal packages or unnecessary classes.
- Shared plan 144-146 Playwright configuration, fixtures, reporters, lifecycle,
  CI, matrix schema, and browser infrastructure; consume them read-only.

## Git Workflow

- Stay on the single active branch; never create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any shared generated registry/config
  are serialized feature-scoped commits. Re-read current content, require no
  uncommitted writer, change only logs rows, land green, then hand off. Never
  regenerate or replace another feature's content.
- Land model/API, table/components, feature hooks, and route/test closure as
  separate reviewable green changes.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable green update under repository policy.

## Steps

### Step 0: Freeze log behavior and the platform/facade boundary

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

Do not run focused target paths or any `--grep @logs` command until Step 5
creates and registers those files; zero selection is intentionally fatal.
Resolve Plan 149's final page-header/time-range facades, Plan 152's log/saved-
view GraphQL contracts, Plan 153's SSE/visibility/search/JSON paths, and Plan
100's format/pure-range paths.
Record URL/search round trips, every request document/variable/count/cache
choice, context/paging generation guard, saved-view state, live URL/flush/order/
cap/reconnect behavior, table threshold/keys/selection and browser markers.
Reserve reviewed facade names for `LogRecord` and `LogsTable` so Plan 140 can
consume them without a deep import.

Require every logs/log-table `__tests__` path and private route import to have an
exact plan-129 legacy handoff owned by plan 141. Generic platform SSE handoffs
remain with plan 100's recorded owner. This plan completes all log capability
imports and leaves no import repair for plan 143. Stop on a missing, wildcard,
expired, or differently owned row; delete each row when its owner moves it.

Confirm plan 145 reserves `@logs` for
`ui/tests/e2e/full-stack/logs.spec.ts` and its shared `@storage` specs retain only
foundation log discovery/live-transport behavior. Consume the feature
reservation without duplicating either foundation stable ID or scenario.

**Verify**: every prerequisite command above exits 0, the generic platform SSE
tests and legacy handoffs are green/exact, and the delegated logs row is reserved
but not yet required to select a feature spec.

### Step 1: Extract the decoded log model

Move log record/saved-view shapes, search parse/serialize, context window,
histogram step, columns, severity, stable identity and document-field projection
into cohesive model modules. Runtime-decoded attribute/resource objects feed
document projection; do not retain component `JSON.parse` casts. Preserve raw
copy serialization and field order.

Move existing pure assertions into model tests and add malformed decoded-object,
duplicate/unknown columns, severity boundaries, observed timestamp skew, empty
identifiers and stable-key cases.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/logs/tests/model && bun run typecheck`
must pass; ratchets show the route/table shrinking without a new exception.

### Step 2: Move log API and live-frame adapters

Place canonical initial/context/older/saved-view operations and schemas under
logs API. Implement decoded adapters/mappers with the typed logs error union.
Build the log-specific live-frame decoder and URL builder on top of the platform
SSE lifecycle. Preserve request sequence/count/cache selection and mutation/local
state behavior exactly.

No raw GraphQL string, manual interpolation/escaping, generic JSON cast,
duplicated log wire type, or second SSE decoder remains in route/components.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/logs/tests/api && bun run typecheck`
must cover valid/null/malformed/error/cancel requests/frames and exact variables,
request counts, saved mutations and error projections.

### Step 3: Split the table and page orchestration

Split page, histogram, table, virtualization, row, document sheet and menus into
the target components. Move live/pagination/saved-view React lifecycle into the
three hooks with discriminated states and current generation/cancellation refs.
Keep DOM keys, virtualization threshold, selection/focus/keyboard behavior,
visible columns, trace/run links, anchor context, saved ordering and every
loading/empty/error/live label exact.

Compose `PageHeader` and `RangePicker` only through the final plan 149 facades;
use Plan 153's SSE/visibility mechanisms and Plan 100's formatting/pure-domain
owners for resolved range behavior.

Do not optimize incoming sort/buffer work or change memoization in this move.
Components/functions must meet 60 lines and new modules 300 unless an exact
pre-existing shrink-only row remains.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/logs/tests/components src/features/logs/tests/integration && bun run check && bun run lint`
must pass with fake-clock/live cleanup and unchanged browser geometry/behavior.

### Step 4: Publish the facade and thin the route

Create an explicit `features/logs/index.ts`. Export route entry/load/search
contracts plus the reviewed `LogRecord` type and `LogsTable` component for plan
140. Do not export API schemas/documents, feature hooks, internal table pieces,
or wildcard barrels.

Reduce `routes/logs.tsx` to file-route creation, search/loader wiring, boundary
selection and top-level feature composition. It exports only `Route`, imports
only the feature facade/lower route primitives, and contains no raw request,
decoder, table, chart, state machine or test helper.

**Local verification:**
`cargo xtask arch && cargo xtask policy --only ui.architecture && cd ui && bun run build`
must prove only-`Route` export, exact `/logs` route, facade-only imports, platform
SSE ownership and reviewed public export set.

### Step 5: Close tests, matrix, ratchets and compatibility paths

Move log API/model/component/live tests into `features/logs/tests/**` and exact
URL/search/loader/boundary/navigation contracts into
`routes/tests/logs-route.test.tsx`. Preserve matrix IDs/assertions, remove
route-private imports, and delete the old route test. Generic EventSource
lifecycle tests stay at the Plan-153 platform test owner. Do not create another
`__tests__` tree.

Create or extend the exact feature-owned `datasets/logs.ts`,
`screens/logs-screen.ts`, fixture contract, full-stack, accessibility, mobile,
visual, and named golden files in the Target Tree. Consume plan 145's reserved
`@logs` row, register each feature matrix ID/project once, and make every grep-
scoped selection non-empty. Shared plans 144-146 fixtures, configuration,
reporters, lifecycle code, and infrastructure remain read-only; generic live-
transport coverage stays with the plan 145 `@storage` foundation.

Ratchet the route to 150 lines, modules to 300, functions/components/hooks to 60
and complexity to 12/15. Delete resolved old table/route/export/test rows and all
temporary compatibility exports after Plan-153 platform callers are canonical.

**Local verification:** run the command table twice. Every `--grep @logs`
selection must be non-zero, `git diff --check` must be clean, and scoped status
must contain only allowed files.

## Test Plan

- `tests/api/logs-api.test.ts`: initial/context/older/saved-view valid/null/
  malformed/error/cancel, exact operations/variables/count and typed errors.
- `tests/api/live-log-contract.test.ts`: valid/malformed/empty frames and decode
  before state mutation.
- `tests/model/logs-search-window.test.ts`: all search keys, saved round trip,
  context, histogram step and custom/live transitions.
- `tests/model/log-columns-severity.test.ts`: columns round trip/dedup/unknown and
  all severity/fatal boundaries.
- `tests/model/log-document-fields.test.ts`: field order, attribute/resource
  decoding, observed skew, raw output and missing identifiers.
- `tests/components/logs-page.test.tsx`: filters, histogram, paging, saved views,
  loading/empty/error and live labels.
- `tests/components/logs-table.test.tsx`: below/above virtualization threshold,
  stable row identity on prepend, custom links, anchor and keyboard activation.
- `tests/components/log-document-sheet.test.tsx`: selection/search/copy/context,
  trace/run links and close/reset behavior.
- `tests/integration/live-logs.test.tsx`: platform visibility/reconnect/flush,
  feature frame order/cap, generation reset and cleanup.
- `routes/tests/logs-route.test.tsx`: exact URL/search/loader/boundary/client
  navigation through public route behavior.
- Fixture browser: deterministic search/context/paging/saved-view/document/live
  states and row links through `@logs`.
- Real stack: public-OTLP log search/context/document links and Turso saved-view
  persistence against managed engines.
- Browser breadth: selected Firefox/WebKit/mobile behavior, axe/keyboard/focus,
  and named canonical log visuals.

## Done Criteria

- [ ] `/logs` exports only `Route`, preserves exact URL/search/loader behavior,
  and is at or below 150 logical lines.
- [ ] All log API/model/component/orchestration lives under `features/logs`; the
  generic EventSource lifecycle remains at the platform owner.
- [ ] The explicit facade exposes only reviewed entry contracts plus decoded
  `LogRecord` and `LogsTable`; run consumers need no deep import.
- [ ] Every payload/frame is decoded and mapped once with typed log errors; no
  generic JSON cast, duplicate schema or raw request remains in UI components.
- [ ] Requests/cache calls, paging guards, live order/caps/timing, saved views,
  table identity/virtualization/keyboard and browser behavior match baseline.
- [ ] Log feature tests live under `features/logs/tests/**`, route contracts under
  `routes/tests/`, platform SSE tests retain their lower-layer owner, and no old
  private-route test remains.
- [ ] Logs-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and canonical
  visual rows are non-empty and green.
- [ ] The feature-owned `@logs` managed-stack row is non-empty, uses public
  OTLP/GraphQL/UI boundaries, and passes against GreptimeDB + Turso.
- [ ] Architecture/tests/ratchets and all Bun unit/browser/build/aggregate gates
  pass twice with no Node.
- [ ] No unnecessary class, catch-all module, wildcard facade, Query/cache change,
  manual generated edit or unrelated feature change exists.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`
  or `@/features/time-range` `RangePicker` facade/input contract is absent or
  incompatible; do not copy a legacy component, deep-import internals, add a
  wildcard export, or defer import repair to plan 143;
- prerequisites or forced-Bun/browser log evidence are incomplete/red;
- plan 145 lacks the delegated `@logs` reservation/shared managed-stack
  infrastructure, or Step 5 cannot make it a non-empty public-boundary row with
  clean one-worker teardown;
- a shared `@storage` discovery/live spec or another feature owns the same logs
  stable ID/scenario, or the reservation points at a different file;
- feature browser evidence requires editing shared plans 144-146 fixtures,
  configuration, lifecycle, reporters, CI, or matrix schema;
- Plan 153 did not establish one generic SSE/visibility owner or another feature
  still depends on the old hook in a way the facade graph cannot preserve;
- Plan 152's generator/handoff cannot represent a frozen log/saved-view GraphQL
  operation or Plan 153 lacks the search/SSE/JSON mechanism;
- baseline request/cache/search/paging/saved/live/table behavior has drifted;
- a stable decoded log/table facade cannot serve run detail without exposing
  internals or creating a cycle;
- preserving behavior requires Query/cache/backend/product/live-performance,
  another feature, manual generated/shadcn or broad exception changes;
- structural limits require arbitrary fragmentation; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Required Deletions

Future log boundaries update document/schema/mapper/error, feature facade, tests,
matrix and ratchets together. The decoded log/table facade is a reviewed public
surface; additions require an actual independent consumer. Plan 133 may add log
query modules and change cache ownership without moving this feature.

Delete before retiring this plan:

- `ui/src/routes/__tests__/-logs.test.tsx`;
- `ui/src/components/logs-table.tsx` after the feature component/facade is the
  sole owner;
- every log implementation export from `ui/src/routes/logs.tsx`;
- `ui/src/hooks/use-live-stream.ts` and its old-path test only if Plan 153's
  canonical platform replacements exist and every caller already migrated;
- every temporary log/table/live old-path reexport; and
- every completed log migration exception/ledger row.

Never delete the canonical platform SSE source/tests. Delete this plan and README
row only after all required deletions and done criteria are durable and green.
