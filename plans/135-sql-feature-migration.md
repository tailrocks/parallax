# Plan 135: Migrate the SQL workspace behind decoded feature boundaries

> **Executor instructions**: Follow this plan in order. Preserve the current SQL
> URL, statement editing, read-only execution, schema browser, history, snippets,
> result links, loading/error states, and request behavior while changing code
> ownership. Start only after plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are
> complete and green. Add model/API/component owners before switching the route, then delete
> route implementation exports and old tests. Do not add TanStack Query or a
> second cache; plan 133 owns server-state caching. This structural plan must not
> invent a SQL download/export action that is absent from the baseline.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/sql.tsx ui/src/routes/__tests__/-sql.test.tsx ui/src/routes/__tests__/-final-sweep.test.tsx ui/src/lib/api.ts ui/src/platform ui/test-matrix.json ui/tests/e2e ratchet.toml`
> Compare live exports, storage behavior, GraphQL requests, and user-visible SQL
> actions with the current-state ledger. Any mismatch is a STOP condition until
> this plan is revised.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / feature migration / architecture
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The entire SQL workspace lives in one 684-line route. That file owns search
validation, GraphQL strings, schema/result decoding, local persistence,
snippets, navigation targeting, editor focus, async state, and all presentation.
Tests compensate by importing implementation exports from the route and reading
private route component options.

This plan leaves the route as a URL adapter, places untrusted data behind
runtime schemas, makes result/history transforms pure, routes browser storage
through the platform boundary, and publishes a reviewed SQL facade. It retains
the current read-only product contract and explicitly excludes a new export or
download feature.

## Current State

At `e3e7997`:

- `ui/src/routes/sql.tsx` is 684 lines and exports `Route`, `EXAMPLES`,
  `SqlCellTarget`, `targetForCell`, `SqlResultsTable`, `SqlResultBody`, and
  `SnippetsMenu` so tests can reach private implementation.
- The `/sql` route validates one optional string search value, `query`. It
  initializes the editor from `query`, otherwise the first example, and updates
  the editor when a later non-empty search query changes.
- `EXAMPLES` contains the shipped read-only queries and GreptimeDB native table
  names. Those strings are product contracts; this migration must not rename
  native tables or broaden SQL capabilities.
- `HISTORY_KEY` is `parallax.sql.history`. `loadHistory` reads `localStorage`
  directly, returns `[]` for absent/malformed/non-array values, and execution
  writes a de-duplicated most-recent-first list capped at 20. It currently
  trusts any JSON array as `string[]`; non-string members can fail later menu
  rendering. Step 0 must characterize mixed arrays rather than silently filter
  or reject them during this structural move.
- `targetForCell` and result helpers parse JSON row strings, identify trace,
  run, issue, and service cells, build typed destination targets, and fall back
  to an empty result for malformed row JSON. Result rendering preserves column
  order, row count, truncation notice, and links only recognized identifiers.
- Mount effects issue raw GraphQL requests for `information_schema.columns` and
  SQL-page saved views. Non-JSON rows, non-arrays, and rows with falsey first
  three cells are skipped. Truthy non-string cells currently pass that guard
  and may fail later presentation, so Step 0 must characterize them rather than
  silently tightening the boundary. Schema load failures use the main inline
  error; snippet failures use their separate inline error.
- Query execution uses raw `graphql`, measures with `performance.now`, clears
  the prior result on failure, records history only on success, and keeps the
  current running/elapsed/error presentation. Command/Control+Enter and the Run
  button execute the exact editor statement.
- Snippet list/save/delete uses the saved-view GraphQL contract with page
  `/sql`; selecting a snippet replaces the editor statement, save trims the
  name, and delete removes the returned ID from local state.
- Identifier insertion replaces the textarea selection, restores focus on the
  next animation frame, and otherwise appends when the ref is unavailable.
- `ui/src/routes/__tests__/-sql.test.tsx` imports result helpers/components from
  the route and builds a private router. It covers cell targets, result links,
  truncation, and snippet select/save/delete.
- At the baseline, `ui/src/routes/__tests__/-final-sweep.test.tsx` imports
  `EXAMPLES` and private route component state and mixes native-table/editor
  SQL cases with dashboard cases. Plan 129 must mechanically move its SQL cases
  into the existing `ui/src/routes/__tests__/-sql.test.tsx`, preserve IDs/
  assertions/import behavior, move dashboard cases separately, and delete the
  mixed file before this plan starts.

The baseline has no SQL download/export button and no use of `Blob`,
`URL.createObjectURL`, or a download helper in `sql.tsx`. This structural
migration preserves that absence. Direct `localStorage` remains forbidden in
the feature; it moves behind plan 100's platform storage adapter. Plan 100's
existing platform download facade remains the only allowed mechanism for a
future separately approved export, but this plan does not import, call, or
modify it.

Behavior that must remain unchanged includes `/sql?query=...`, search parsing,
initial/default statement, focus/selection, keyboard execution, exact examples,
schema/snippet request timing, loading/error separation, malformed-row fallback,
result links/truncation, elapsed display, history key/order/cap, saved-view page,
and raw uncached GraphQL request behavior. Query/cache migration is deferred to
plan 133.

## Target Ownership

Create only files with real responsibilities:

```text
ui/src/features/sql/
  api/
    sql-schema.graphql
    sql-schema.generated.ts
    sql-execute.graphql
    sql-execute.generated.ts
    sql-snippets-list.graphql
    sql-snippets-list.generated.ts
    sql-snippet-save.graphql
    sql-snippet-save.generated.ts
    sql-snippet-delete.graphql
    sql-snippet-delete.generated.ts
    sql-result-schema.ts
    sql-api.ts
    sql-history-repository.ts
  model/
    sql-result.ts
    sql-row.ts
    sql-cell-target.ts
    sql-history.ts
    sql-snippet.ts
    sql-examples.ts
    sql-error.ts
  components/
    sql-page.tsx
    sql-editor.tsx
    sql-schema-browser.tsx
    sql-results-table.tsx
    sql-result-body.tsx
    snippets-menu.tsx
    save-snippet-dialog.tsx
  hooks/
    use-sql-workspace.ts
  tests/
    api/sql-api.test.ts
    api/sql-history-repository.test.ts
    model/sql-result.test.ts
    model/sql-history.test.ts
    components/sql-editor.test.tsx
    components/sql-results-table.test.tsx
    components/snippets-menu.test.tsx
    integration/sql-workspace.test.tsx
  index.ts
ui/src/routes/tests/
  sql-route.test.tsx
ui/tests/e2e/
  datasets/sql.ts
  screens/sql-screen.ts
  contracts/sql.spec.ts
  full-stack/sql.spec.ts
  accessibility/sql-accessibility.spec.ts
  mobile/sql-mobile.spec.ts
  visual/sql.visual.spec.ts
  visual/goldens/
    sql-workspace.png
    sql-populated-result.png
    sql-error.png
    sql-snippet-dialog.png
```

Ownership rules:

- Each `.graphql` file contains one globally unique named variables-only
  operation and one checked-in Plan-152-generated sibling. The generated result
  schema validates outer fields and retains each JSON-scalar `rows` element as
  `unknown`. `sql-result-schema.ts` and `sql-row.ts` instantiate Plan 153 for the
  one operation-specific nested parse during
  domain mapping: any non-string result row, malformed JSON string, or parsed
  non-array maps to the current empty-cell row; schema-discovery rows retain the
  separately characterized skip/accept behavior. Use operation-specific
  schemas so a partial response is never asserted as a broader shared DTO. A
  non-array outer `rows` field or other invalid outer envelope remains a
  failure.
- `sql-api.ts` exposes `loadSqlSchema`, `runSql`, `loadSqlSnippets`,
  `saveSqlSnippet`, and `deleteSqlSnippet`. It maps decoded values once to
  domain values and projects transport/schema failures to `SqlError`. It owns
  no React state, browser storage, cache, focus, or navigation.
- `sql-history-repository.ts` composes the pure history functions with
  `@/platform/browser-storage`. It preserves the exact key and wire shape. The
  feature cannot call global `localStorage`.
- model modules own readonly values and pure row parse, link targeting,
  history normalization, examples, and snippet transforms. Do not create
  catch-all `types.ts` or `utils.ts`.
- `sql-error.ts` defines an exhaustive Result-shaped expected-failure union
  with operation context. At minimum distinguish transport, invalid response,
  schema discovery, query execution, history persistence, and snippet list/
  save/delete. Error message text is presentation, never control flow.
- `use-sql-workspace.ts` may own the real effect/focus/request lifecycle and an
  exhaustive workspace state. Read elapsed time through plan 100's monotonic
  platform clock contract while preserving the displayed milliseconds; do not
  retain direct `performance.now()` access. The hook must not become a bag of
  pure helpers.
- components own presentation and semantic interaction only. Keep each
  function/component/hook under the 60-line target through cohesive splits.
- `index.ts` has explicit named exports for the route-facing page/contract and
  any stable cell-target type used outside SQL. It cannot export documents,
  schemas, repositories, private components, or `export *`.

Prefer pure functions and readonly data. No class is expected. A class is
permitted only for a demonstrated lifecycle or invariant-bearing mutable
identity with focused tests; grouping SQL helpers is not justification.

Final structural ratchets are exact: route module <=150 logical lines,
handwritten TS/TSX module <=300, test scenario file <=500, function/component/
hook <=60, cyclomatic complexity <=12, and cognitive complexity <=15. An
unchanged oversized move does not pass. Any inherited exception is exact,
expiring, and shrink-only.

## Platform Storage Boundary

- History storage is feature policy plus a platform mechanism: feature code
  owns key, JSON shape, order, dedupe, cap, and fallback; the platform adapter
  owns access to browser storage and unavailable/security-error behavior.
- Inject or pass the platform storage contract so model/API tests run without
  globals. Step 0 must characterize read/write failure through a throwing
  storage fixture, including whether result, elapsed time, history, and inline
  error change. Preserve that observed behavior here; decoupling a storage
  failure from query success requires a separate approved behavior change.
- Feature and route code must contain no `localStorage`, `sessionStorage`,
  `performance.now`, `Blob`, `URL.createObjectURL`, temporary anchor click, or
  filesystem API.
- The existing plan 100 platform download facade is the sole future browser
  mechanism. Do not import/call/modify it or create a SQL serializer, adapter,
  export button, filename/format contract, or browser matrix row in this plan.

## Plan 149 Capability Contract

- The SQL page renders `PageHeader` only from
  `@/shared/components/page-header`, using explicit named imports from the final
  plan 149 facade. It does not import layout/navigation internals.
- SQL has no baseline range picker, runtime metric, or story capability. Do not
  add unused imports from those plan 149 facades or introduce new UI behavior to
  make the dependency visible.
- Plan 100 remains the owner of GraphQL transport, storage, monotonic clock,
  download, formatting, and other technical/pure-domain contracts consumed here.
- Do not deep-import a plan 149 feature, use a wildcard barrel, copy a legacy
  capability into SQL, or defer a completed SQL capability import to plan 143.

## Route And Cache Contract

- `ui/src/routes/sql.tsx` retains only `validateSearch`, route declaration,
  route/search composition, and a small feature-page render. It exports only
  `Route`.
- The route imports only `@/features/sql`, TanStack route APIs, and allowed
  domain/shared types. Tests cannot read `Route.options.component`.
- Feature code must not import a route. Pass typed search values/callbacks or use
  `getRouteApi` only when composition cannot supply the contract without a
  cycle.
- Preserve raw, uncached GraphQL request timing. Do not add a `queries/`
  directory, `QueryClient`, query keys, `ensureQueryData`, prefetching, TTL, or
  request deduplication. Those are plan 133 responsibilities.
- Result links are URL contracts, not imports from traces/runs/issues/services.
  No cross-feature code dependency is needed.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Architecture | `cargo xtask arch` | no cycle, deep import, route export, or unclassified file |
| UI policy | `cargo xtask policy --only ui.architecture` | facade/platform/runtime topology passes |
| Test policy | `cargo xtask policy --only ui.tests` | matrix and `tests/` ownership pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | route/module/function/export budgets shrink or hold |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/sql src/routes/tests/sql-route.test.tsx` | selected non-empty suite passes |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass without unexpected diagnostics |
| Browser contract | `cd ui && bun run test:browser -- --grep @sql` | registered SQL product contracts pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @sql` | non-zero managed GreptimeDB + Turso cases pass |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @sql && bun run test:browser:a11y -- --grep @sql && bun run test:browser:visual -- --grep @sql` | non-zero cross/mobile/a11y/visual rows pass |
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

`ui/tests/e2e/full-stack/sql.spec.ts` owns the `@sql` managed-stack row. Query
the native telemetry tables populated through plan 145's public OTLP seed,
verify schema discovery and expected result links/empty/error states, save and
delete a snippet through the UI, and prove the snippet survives a fresh
BrowserContext through Turso. History must survive reload/navigation only in
the original BrowserContext's local storage and must be absent in a genuinely
fresh context. Keep SQL read-only and do not add a download action. The test
uses public GraphQL/UI surfaces, bounded readiness, one worker, managed
GreptimeDB, and isolated Turso only.

**Verify**: `cd ui && bun run test:browser:full -- --grep @sql` selects at least
one plan-135 matrix row and passes with no direct table insert, database-file
read, response interception, fixed sleep, or leaked process/data directory.

## Feature Browser Breadth

This plan owns every `@sql` row that consumes plan 146's projects. Run editor,
safe execution, result links, history, snippets, schema browser, empty/error,
and keyboard execution in Firefox and WebKit. Cover editor focus/selection,
wide/long result tables, menus/dialogs, and overflow on both mobile device
projects. Run axe plus complete keyboard/focus/Escape/restoration checks. Keep
canonical workspace, populated-result, error, and snippet-dialog visual states;
do not add a download control merely to create coverage.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @sql && bun run test:browser:a11y -- --grep @sql && bun run test:browser:visual -- --grep @sql` selects non-zero owned rows and passes with no response interception, broad masking, or unplanned product action.

## Scope

In scope:

- `ui/src/routes/sql.tsx`, SQL-owned slices of both current test files, SQL
  shared API contracts if any, new feature/route tests, and exact matrix/E2E/
  ratchet entries across plans 144, 145, and 146;
- runtime decoding, domain mapping, typed failures, pure result/history logic,
  component/orchestration splits, explicit facade, and thin route;
- browser storage migration through the plan 100 platform adapter.

Out of scope:

- Query/cache changes (plan 133), live-data algorithms (plan 147), bundle/
  performance work (plan 148), SQL backend grammar/safety changes, write SQL,
  pagination/streaming, syntax highlighting, a new editor library, visual
  redesign, schema changes, or changes to native telemetry table names;
- restructuring destination features behind result links;
- shared plan 145/146 real-stack/browser-project infrastructure; this plan
  still owns SQL-specific managed-stack and breadth rows/files;
- plan 129's dashboard legacy handoff file; plan 137 owns it;
- direct browser/platform globals, SQL export/download product work or a
  feature-specific download abstraction, changes to plan 100's existing
  platform download facade, Node, a foreign package manager, internal npm
  packages, new `__tests__/`, or route implementation exports.

## Git Workflow

- Stay on the current single branch; never create another branch or PR.
- Plan 129 owns the mechanical mixed-file split. After that prerequisite, plan
  135 owns only `ui/src/routes/__tests__/-sql.test.tsx`; plan 137 owns only its
  separate dashboard handoff. The two feature plans may then execute in
  parallel and never write the same legacy test file.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any surviving SQL slice in
  `ui/src/lib/api.ts` are serialized feature-scoped commits. Re-read the current
  file, require no uncommitted writer, patch only SQL rows/types, commit green,
  then hand off. Do not regenerate or replace another feature's content.
- Commit focused green slices and push every durable update.
- Use Conventional Commits and exactly one required agent-product trailer.

## Steps

### Step 0: Reconcile prerequisites, shared ownership, and product contracts

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

Do not run focused target paths or `--grep @sql` commands until Steps 3-5
create/register them; zero selection is intentionally fatal. Confirm the final
plan 149 page-header facade, the live plan 100 platform storage facade name,
Plan 152's generator/transport and exact SQL handoff rows, and Plan 153's
search/storage/JSON mechanisms. Plan 152 does not pre-create these product
operations; this plan creates them from the frozen requests using its exact
template. Confirm
plan 129 deleted
`ui/src/routes/__tests__/-final-sweep.test.tsx`, moved every SQL case/ID into
`ui/src/routes/__tests__/-sql.test.tsx`, and handed dashboard cases to a
different exact file. Do not recreate the mixed file.

Plan 129's completed state must retain one exact expiring exception for the SQL
private-route imports in `-sql.test.tsx`, owned by plan 135 and removed in Step
4. If that file remains without the exception, stop because the prerequisite
graph is inconsistent.

Plan 145 must reserve the `@sql` managed-stack stable IDs for
`full-stack/sql.spec.ts`; its shared specs may provide seed/readiness
infrastructure but cannot already assert the same SQL/snippet workflow. Move an
existing reserved row instead of duplicating it.

Inventory exact search normalization, examples, GraphQL text/fields/timing,
schema-row tolerance (including truthy non-string cells), result JSON/link/
truncation behavior, history wire data, throwing storage read/write behavior,
mixed-array history members, focus/keyboard behavior, errors, and snippet
mutations. Confirm the browser matrix has no SQL export/download row because the
product has no such action. If safe typing of mixed history requires a new
recovery behavior, stop for an explicit product decision and record it in the
matrix before implementation.

Add/update stable `ui/test-matrix.json` ownership for search/editor, safe query
execution, valid/malformed/empty/truncated results, link targets, history,
schema browser, snippets, and storage failure. Do not create duplicate browser
cases.

**Verify**: test-matrix policy and the complete baseline UI suite pass;
`test ! -e ui/src/routes/__tests__/-final-sweep.test.tsx` succeeds, all SQL IDs
are in `-sql.test.tsx`, and no SQL/dashboard test has two owners.

### Step 1: Extract pure model and persistence policy

Move examples, result/row parsing, cell target mapping, snippet values, and
history policy into target model files. Add golden examples and exhaustive
tests for absent/malformed/non-array/mixed-member history, duplicates, 20/21
entries, order, storage write failure, non-string/malformed/non-array row
payloads, empty cells, recognized IDs, unknown columns, column order, row count,
and truncation.

Create `sql-history-repository.ts` over the platform storage contract. Keep the
key and serialized array compatible. Its typed failure is consumed so the UI
matches the Step 0 characterization; do not assume storage and query errors are
currently independent.

**Verify**: focused model/repository tests pass, typecheck is green, and
`rg -n 'localStorage|sessionStorage|performance\.now|\bBlob\b|createObjectURL' ui/src/features/sql ui/src/routes/sql.tsx`
returns no matches.

### Step 2: Generate named operations, decode nested JSON, and map typed errors

Create one named `.graphql` document and checked-in generated sibling per schema
discovery, SQL execution, and saved-view list/save/delete operation. Generated
schemas parse outer transport output from `unknown`; the Plan-153 nested row
schema maps once into feature domain values and preserves the exact characterized
schema-row acceptance/skip behavior. If runtime safety requires
rejecting a truthy non-string cell that currently reaches presentation, stop
for an explicit behavior decision instead of silently broadening "malformed".
Remove unsafe generic assertions for moved operations. Return/throw only at the
platform edge, then project once to the feature's discriminated expected-
failure contract.

**Verify**: API tests cover valid, empty, malformed outer envelope, non-string/
malformed/non-array row fallback, GraphQL error, cancellation classification
where supported, and each mutation; focused tests and architecture policy pass.

### Step 3: Split the workspace into feature components and lifecycle

Build the target editor, schema browser, result table/body, snippet menu/dialog,
page, and focused hook. Model async UI as exhaustive states without independent
booleans that allow impossible combinations. Preserve mount request timing,
separate snippet/main errors, selection/focus behavior, keyboard shortcut,
disabled labels, examples/history order, elapsed timing, result links, and
truncation text.

Do not paste the 684-line route into `sql-page.tsx`; every module and function
must meet its final structural budget. Add semantic component/integration tests
using the shared plan 129 harness and `userEvent.setup()`.

**Verify**: focused component/integration tests, lint, typecheck, and UI ratchets
pass with no unexpected network/console/timer diagnostic.

### Step 4: Publish the facade, thin the route, and migrate tests

Create explicit `index.ts` exports. Convert `sql.tsx` to a route adapter that
exports only `Route`, validates the same `query`, and composes the public page.
Move the complete post-plan-129 `-sql.test.tsx` handoff to the target feature/
route test files. Tests use public contracts/shared router builders, not route
implementation exports or `Route.options.component`. Do not touch plan 137's
separate dashboard handoff.

Delete old private route exports after callers move. No route, layout, or other
feature may deep-import SQL internals.

**Verify**: focused tests, architecture, typecheck, and build pass;
`rg -n '^export (const|function|type|interface)' ui/src/routes/sql.tsx` reports
only `export const Route`, and the old SQL test files/imports are absent.

### Step 5: Complete browser evidence and cleanup

Add the SQL dataset/screen/contracts assigned by plan 144 for editor search
initialization, keyboard/button execution, successful/empty/truncated/error
results, deep links, history persistence, snippet CRUD, and schema browser. Use
public surfaces, semantic locators, deterministic seed, and observable results;
no response interception, fixed sleeps, or invented export action.

Implement the Feature Real-Stack Contract and Feature Browser Breadth sections
in the exact target files, register each non-empty project row once, and keep
shared plan 145/146 fixtures read-only.

Delete obsolete shared API types/compatibility exports only after `rg` proves
no caller. Update exact route/module/function/complexity/export/direct-global
ratchets and matrix rows.

**Verify**: fixture-backed/full-stack/breadth `@sql` commands and policies, full
Vitest, architecture, test policy, and ratchets pass; deleted symbols/paths and
forbidden globals have zero hits.

### Step 6: Run the complete final gate twice

Run every Commands-table entry twice from clean state. The second run must not
change generated route code, lock data, browser artifacts, or tracked files.

**Verify**: all commands exit 0 twice and `git diff --check` reports no errors.

## Test Plan

- Model: native-table examples, row decode, all cell-target variants, malformed
  fallback, result column/order/count/truncation, history parse/dedupe/cap/order.
- API: valid/malformed/error schema discovery, SQL result, saved-view list/save/
  delete, exact page and field contracts, error projection.
- Platform integration: unavailable/corrupt/mixed-member/write-failing storage
  preserves the characterized behavior and exact key.
- Components/integration: search/default statement, selection insertion/focus,
  Command/Control+Enter, running/success/error/elapsed, schema expand, examples,
  history, snippets, result links/truncation, accessibility.
- Route: optional/invalid `query`, direct navigation and later search update,
  pending/error composition, only public facade use.
- Browser: deterministic safe execution, result states/links, history, snippet
  CRUD, schema behavior, and proof that no unplanned export control appears.
- Real stack: safe native-table query/schema/result behavior, Turso snippet
  persistence across a fresh context, and local history scoped to the original
  context.

## Done Criteria

- [ ] SQL production code has one feature owner and the route exports only
  `Route` while importing only the public facade.
- [ ] All external data is decoded from `unknown`, mapped once, and expected
  failures use `SqlError` without message-based control flow.
- [ ] History uses the platform storage adapter with exact key/shape/order/cap;
  feature/route code has no direct storage/download/browser globals.
- [ ] URL/search, examples, request timing/cache behavior, focus/keyboard,
  results/links/truncation, snippets, loading, and errors match baseline.
- [ ] Tests live only under `tests/`, do not import route internals, the plan 129
  `-sql.test.tsx` handoff is removed, and dashboard handoff files are untouched.
- [ ] No SQL export/download action, SQL-specific abstraction, or browser matrix
  row is added; plan 100's platform download facade remains untouched.
- [ ] SQL-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and canonical
  visual rows are non-empty and green.
- [ ] The feature-owned `@sql` managed-stack row is non-empty, uses only public
  boundaries, and passes with clean teardown.
- [ ] Vitest, browser, build, Oxc, xtask architecture/test/ratchet, and aggregate
  gates pass twice.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or `@/shared/components/page-header` does not expose a
  compatible explicit `PageHeader` contract; do not copy the legacy header,
  deep-import internals, add a wildcard export, or defer import repair to plan
  143;
- a prerequisite is incomplete, the platform adapter contracts are absent,
  Plan 152's generator/handoff cannot represent a frozen SQL operation, or Plan
  153 lacks the search/storage/JSON mechanism;
- plan 145 lacks the delegated `@sql` reservation or shared managed-stack
  infrastructure, or Step 5 cannot turn that reservation into a non-empty
  public-boundary row with clean one-worker teardown;
- plan 145's shared specs already own the same SQL/snippet stable ID/behavior and
  cannot hand it off without duplicate matrix ownership;
- plan 129 is marked complete while a SQL legacy/private-route test remains
  without its exact plan 135 expiring topology exception;
- plan 129 leaves the mixed `-final-sweep.test.tsx`, loses a SQL case/ID, or
  hands any one legacy file to both plans 135 and 137;
- drift changes SQL write-safety, native-table examples, URL/search, storage
  format, result links, GraphQL operations, loading/error, or request timing;
- migration requires Query/cache work, a backend/schema change, direct browser
  globals, unsafe response assertions, or a route implementation export;
- the observed storage failure path cannot be reproduced through the platform
  repository without an explicit product behavior change;
- runtime-safe mixed history handling has no explicit decision even though it
  differs from the baseline's unchecked-array behavior;
- an oversized copy, `__tests__`, duplicate owner, deep import, or permanent
  broad policy exception would remain; or
- a required verification fails twice after one reasonable correction.

## Maintenance And Removal

Future SQL operations add a named document, runtime schema, one domain mapper,
typed error cases, matrix evidence, and tests together. Persistence policy stays
in the feature while mechanisms stay in platform. Any future download/export
format requires its own product plan and platform-boundary design. Plan 133 may
later replace request/cache behavior but must keep the facade and
characterization evidence.

Delete this plan and its index row only after all old route/test owners are
removed, every done criterion is green, and
durable tests/policies/matrix entries own the lasting contract.
