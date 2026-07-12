# Plan 134: Migrate investigations behind a strict feature facade

> **Executor instructions**: Follow this plan in order and preserve the shipped
> investigations behavior exactly while changing ownership. Start only after
> plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 are complete and green. Add the new
> feature
> modules before switching callers; then delete the old owners. Routes must
> export only `Route`, and every route or other feature must import
> investigations through `@/features/investigations`. Do not introduce TanStack
> Query or change cache semantics here; plan 133 owns that migration. Stop on a
> listed STOP condition instead of inventing a second architecture.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/investigations.index.tsx 'ui/src/routes/investigations.$investigationId.tsx' ui/src/lib/investigations.ts ui/src/lib/__tests__/investigations.test.ts ui/src/components/console/pin-button.tsx ui/src/routes/issues.* ui/src/routes/traces.* ui/src/routes/runs.* ui/src/lib/api.ts ui/test-matrix.json ui/tests/e2e ratchet.toml`
> Reconcile every changed responsibility against the current-state ledger below.
> Stop if a current owner or public behavior no longer matches this plan.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 100, 129, 132, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / feature migration / architecture
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

Investigations is split across two route modules, a generic library file, a
console component, shared API types, and three unrelated feature routes. Those
files mix wire contracts, persistence encoding, navigation, mutations, error
state, and presentation. The split creates route-to-implementation coupling and
makes pin behavior a de facto global component without a reviewed owner.

This plan gives investigations one feature boundary, a small explicit facade,
decoded API adapters, pure model functions, feature-owned UI and tests, and a
declared cross-feature pin contract. It is an ownership migration, not a product
redesign or cache rewrite.

## Current State

At the planned baseline:

- `ui/src/routes/investigations.index.tsx` is 205 lines. It owns the list loader,
  cached GraphQL document, list/empty/error rendering, create/delete mutations,
  cache invalidation, and create-to-detail navigation.
- `ui/src/routes/investigations.$investigationId.tsx` is 282 lines. It owns the
  cached detail loader and `notFound()` decision, editable pin/window/notes
  state, save/delete mutations, cache invalidation, list navigation, and detail
  presentation.
- `ui/src/lib/investigations.ts` is 155 lines. It owns the versioned persisted
  state shape, invalid-state fallback, 100-pin cap, parse/serialize functions,
  window search conversion, pin append logic, and typed range-aware hrefs.
- `ui/src/components/console/pin-button.tsx` is 198 lines. It lazily lists
  investigations when opened, creates or updates one, appends the current page
  as a pin, invalidates router cache, and presents loading/error/success states.
- `ui/src/routes/issues.$fingerprint.tsx`, `ui/src/routes/traces.$traceId.tsx`,
  and `ui/src/routes/runs.$runId.tsx` deep-import that pin component. These are
  the required declared edges `issues -> investigations`,
  `traces -> investigations`, and `runs -> investigations`.
- `ui/src/lib/api.ts` owns the wire-facing `Investigation` type mixed with many
  unrelated product contracts.
- `ui/src/lib/__tests__/investigations.test.ts` covers state fallback/capping,
  pin href construction, and window parameter conversion under the forbidden
  final `__tests__/` convention. No focused component contract owns pin loading,
  create/update failure, or duplicate/cap behavior through the UI.

Behavior that must remain unchanged:

- list URL `/investigations/` and detail URL
  `/investigations/$investigationId`;
- create navigates to the created detail; detail delete returns to the list;
- a missing detail raises the existing TanStack not-found boundary;
- invalid or unsupported stored state falls back to the current empty version;
- serialization, pin order, duplicate behavior, 100-pin cap, notes, window
  values, and generated telemetry links remain byte/URL compatible;
- list create stays disabled for a trim-empty name but sends the original
  untrimmed name, while pin-button create continues to trim before saving;
- appending at the 100-pin cap continues to retain the first 100 and drop the
  newly appended pin; a custom window still requires both bounds; notes remain
  plain textarea content rather than rendered HTML;
- list/detail reads retain the current `graphqlCached` freshness behavior,
  while mutations retain raw `graphql` behavior and exact invalidation timing;
- router invalidation after a mutation still does not clear the current
  module-global 15-second query-string-keyed GraphQL cache;
- the pin popover still loads candidates only when opened through raw uncached
  GraphQL, ignores a late completion after cleanup, and does not gain request
  abortion as collateral behavior;
- existing loading, empty, mutation-pending, success, and error text remain
  observable; and
- issue, trace, and run pin actions keep their labels, target kind, current URL,
  range search, navigation, and post-save feedback.

## Target Ownership

Create a directory only when adding its first real file. Use this final shape:

```text
ui/src/features/investigations/
  api/
    investigations-list.graphql
    investigations-list.generated.ts
    investigation-pin-options.graphql
    investigation-pin-options.generated.ts
    investigation-detail.graphql
    investigation-detail.generated.ts
    investigation-save.graphql
    investigation-save.generated.ts
    investigation-delete.graphql
    investigation-delete.generated.ts
    investigation-api.ts
  model/
    investigation.ts
    investigation-state.ts
    investigation-state-schema.ts
    investigation-error.ts
  components/
    investigations-page.tsx
    investigation-card.tsx
    create-investigation-dialog.tsx
    investigation-detail-page.tsx
    investigation-window.tsx
    investigation-pins.tsx
    investigation-notes.tsx
    pin-button.tsx
  hooks/
    use-investigation-draft.ts
    use-pin-to-investigation.ts
  tests/
    api/investigation-api.test.ts
    model/investigation-state.test.ts
    components/investigations-page.test.tsx
    components/investigation-detail-page.test.tsx
    integration/pin-button.test.tsx
  index.ts
ui/src/routes/tests/
  investigations-routes.test.tsx
ui/tests/e2e/
  datasets/investigations.ts
  screens/investigations-screen.ts
  contracts/investigations.spec.ts
  full-stack/investigations.spec.ts
  accessibility/investigations-accessibility.spec.ts
  mobile/investigations-mobile.spec.ts
  visual/investigations.visual.spec.ts
  visual/goldens/
    investigations-list.png
    investigations-detail.png
    investigations-pin.png
    investigations-edit-error.png
```

Plan 144/146 may already have created the investigations pilot files above.
Extend those exact files in place; do not create a second feature dataset,
screen, contract, or golden namespace.

Responsibilities are fixed:

- Each `.graphql` file contains one globally unique named Plan-152 operation,
  uses variables only, and has its checked-in `.generated.ts` sibling. The
  generated operation schema parses GraphQL `unknown`; the pin popover's partial
  list selection is a separate operation and is never asserted as the broader
  list/detail result.
- `investigation-state-schema.ts` instantiates Plan 153 for the persisted state
  string/JSON boundary and preserves current version/recovery behavior.
- `investigation-api.ts` exposes distinct decoded cached route-list/detail,
  uncached pin-candidate list, save, and delete operations. It maps transport/
  schema failures once to `InvestigationError`; selecting the platform's
  existing cached or raw transport is explicit per operation, but the feature
  creates no cache of its own. It owns no React state, navigation, or display
  text.
- `investigation.ts` owns readonly domain values and the sole wire-to-domain
  mapper. Delete the duplicate type from `ui/src/lib/api.ts` when no caller
  remains.
- `investigation-state.ts` owns pure state construction, parse, serialize,
  window search, href, and pin transforms. Preserve the stored version and
  compatibility behavior.
- `investigation-error.ts` owns an exhaustive Result-shaped expected-failure
  union. Distinguish at least transport, invalid response, load, save, and
  delete failures without using message text as control flow. Malformed or
  unknown-version persisted investigation state retains its current silent
  empty-state recovery and is not promoted to a user-visible load error.
- page components own user-visible rendering and event composition. They accept
  typed values/callbacks; they do not parse raw GraphQL envelopes.
- `pin-button.tsx` is the feature-owned public cross-feature control. Its
  lifecycle may stay in a hook or component; stateless operations remain pure.
- `index.ts` uses explicit named value/type exports only. Export only the page
  contracts, route loader adapters needed by routes, `PinButton`, and the pin
  input types required by approved consumers. No `export *` and no internal
  document/schema export.

Prefer pure functions and readonly modules. A class is allowed only if a real
resource lifecycle or invariant-bearing mutable identity is demonstrated in a
short comment and test. This migration has no known class requirement.

Final structural ratchets are exact: route module <=150 logical lines,
handwritten TS/TSX module <=300, test scenario file <=500, function/component/
hook <=60, cyclomatic complexity <=12, and cognitive complexity <=15. An
unchanged oversized move does not pass. Any inherited exception is exact,
expiring, and shrink-only.

## Dependency And Route Contract

- Routes may import only `@/features/investigations`, TanStack route APIs,
  shared/domain contracts, and route-local URL composition.
- Both investigation route files export only `Route`. Move every currently
  exported component/helper into the feature before switching tests.
- Issues, traces, and runs import `PinButton` and its public input types only
  from the feature facade. They cannot import `api`, `model`, or `components`
  paths directly.
- Add the three exact approved facade edges to plan 100's live architecture
  manifest/ratchet. No other feature may acquire an investigations dependency.
- Use `getRouteApi` in deep UI only if route state is unavoidable; never import
  a route definition into the feature.
- The detail page currently derives its back descriptor from layout-owned
  navigation. Pass `PageHeaderBack` from route composition and render
  `PageHeader` only through `@/shared/components/page-header`, as finalized by
  plan 149; never add a feature-to-layout import.
- Pin inputs that carry the current telemetry window use plan 149's minimum
  reviewed resolved-range input type through `@/features/time-range`. Keep
  persisted-window parsing, URL construction, and other pure range transforms at
  plan 100's canonical domain owner. Investigations has no baseline
  `RangePicker`; do not add one during this ownership move.
- Imports from plan 149 facades are explicit named value/type imports only. Do
  not deep-import their internals, use wildcard barrels, copy their components,
  or defer a completed investigations capability import to plan 143.
- Keep current `graphqlCached` reads and raw mutations through the platform
  adapter. Do not create `queries/`, query keys, `QueryClient`,
  `ensureQueryData`, or dual cache ownership. Plan 133 performs that later.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Architecture | `cargo xtask arch` | only approved edges; no cycle, route deep import, or unclassified file |
| UI policy | `cargo xtask policy --only ui.architecture` | route/facade/runtime/test topology passes |
| Test policy | `cargo xtask policy --only ui.tests` | matrix and `tests/` ownership pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | size/export/import baselines shrink or hold |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/investigations src/routes/tests/investigations-routes.test.tsx` | selected non-empty suite passes |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass with no unexpected diagnostic |
| Browser contract | `cd ui && bun run test:browser -- --grep @investigations` | registered feature cases pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @investigations` | non-zero managed GreptimeDB + Turso cases pass |
| Browser breadth | `cd ui && bun run test:browser:cross -- --grep @investigations && bun run test:browser:a11y -- --grep @investigations && bun run test:browser:visual -- --grep @investigations` | non-zero cross/mobile/a11y/visual rows pass |
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

`ui/tests/e2e/full-stack/investigations.spec.ts` owns the `@investigations`
managed-stack row. Reuse plan 145's public-OTLP trace/run/issue identities, then
create an investigation, pin a real telemetry page, edit window/notes, and
verify update/delete and persistence across a fresh BrowserContext through the
visible UI and public GraphQL postcondition. The project remains one worker and
uses managed GreptimeDB plus isolated Turso; it never reads database files,
direct-writes metadata, or intercepts browser responses.

**Verify**: `cd ui && bun run test:browser:full -- --grep @investigations`
selects at least one plan-134 matrix row and passes with the real-stack runtime
manifest, bounded readiness predicates, and clean process/data teardown.

## Feature Browser Breadth

This plan owns every `@investigations` row that consumes plan 146's projects.
Extend the existing pilot rather than duplicating it: run list/detail and CRUD/
pin/note workflows in Firefox and WebKit; cover both routes, dialogs/popovers,
long names/notes, and destructive confirmation on the two mobile device
projects; run axe plus keyboard/focus/Escape/restoration checks for every
interactive state; and maintain canonical list, detail, pin, and edit/error
visual states where the matrix names layout risk.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @investigations && bun run test:browser:a11y -- --grep @investigations && bun run test:browser:visual -- --grep @investigations` selects non-zero owned rows and passes without broad masking, response interception, or an exception outside plan 146's schema.

## Scope

In scope:

- the four current investigations production owners, their shared API type,
  their tests, both route adapters, and the three `PinButton` import sites;
- feature runtime schemas, domain mapping, typed expected errors, and facade;
- exact architecture/matrix/ratchet entries needed by this feature;
- plan 144 investigations dataset/screen/contracts, plan 145 feature-owned
  full-stack spec, and plan 146 accessibility/mobile/visual files, extending
  pilots rather than creating parallel owners; and
- deletion of obsolete files/exports after all callers move.

Out of scope:

- TanStack Query, cache-key, TTL, or hydration changes (plan 133), live-data
  algorithm changes (plan 147), and bundle/performance work (plan 148);
- issue, trace, or run feature restructuring beyond changing the facade import;
- GraphQL schema/backend changes, new pin kinds, a persisted-state version bump,
  visual redesign, notes semantics, markdown rendering, or URL changes;
- shell/navigation and shared real-stack/browser-project infrastructure; this
  plan still owns investigations-specific managed-stack, cross-browser/mobile/
  accessibility/visual rows;
- any `__tests__/` directory, new catch-all `utils.ts`/`types.ts`, internal npm
  package, route implementation export, Node runtime, or alternate package
  manager.

## Git Workflow

- Stay on the current single branch. Never create another branch or PR.
- Plan 134 is the sole writer for the import-only `PinButton` facade switch in
  `issues.$fingerprint.tsx`, `runs.$runId.tsx`, and `traces.$traceId.tsx`. Plans
  139, 140, and 142 must each declare plan 134 as a hard dependency and cannot
  start their route migrations before this plan publishes the green facade-
  switch commit. After that commit, ownership returns to those plans and they
  must preserve the facade import. Plan 134 changes no other line in those three
  routes and leaves no old-path compatibility component or re-export shim.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and `ui/src/lib/api.ts` are serialized
  feature-scoped commits. Re-read the current file, require no uncommitted
  writer, patch only investigations rows/type, commit green, then hand off. Do
  not regenerate or replace another feature's content.
- Commit focused green ownership slices and push each durable update.
- Use Conventional Commits and exactly one required agent-product trailer.
- Do not combine unrelated feature migrations in these commits.

## Steps

### Step 0: Reconcile prerequisites and freeze behavior

Confirm plans 100, 129, 132, 144, 145, 146, 149, 152, and 153 have retired from
the active index with their durable implementation artifacts present. Re-run their live
architecture, test, browser, storage, and breadth gates rather than treating a
deleted plan file as completion evidence. Run the drift check and this exact
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

Do not run focused target paths or
`--grep @investigations` commands until Steps 2-4 create/register them; zero
selection is intentionally fatal. Inventory the exact GraphQL requests, cache
keys/TTL behavior, invalidation calls, URLs/search values, stored state fixtures,
user-visible states, and current pilot matrix IDs. Add missing Vitest
characterization before moving production code.

Confirm Plan 152's generator and exact investigations handoff rows are green,
and Plan 153 exposes the final search/persisted-value mechanisms. Plan 152 does
not pre-create these product operations; this plan creates them from the frozen
requests using its exact template.

Plan 129's completed state must contain an exact expiring topology exception for
`ui/src/lib/__tests__/investigations.test.ts`, owned by plan 134 and removed in
Step 4. If plan 129 already moved it, reconcile this ledger to the live final
path and never recreate the legacy directory. If the old file remains but plan
129 claims zero exceptions, stop because the prerequisite graph is inconsistent.

Plan 145 must reserve the `@investigations` managed-stack stable IDs for
`full-stack/investigations.spec.ts`; its shared full-stack specs may provide
seed/readiness infrastructure but cannot already assert the same investigation
workflow. Move an existing reserved row instead of duplicating it.

Update `ui/test-matrix.json` so list, detail/not-found, create/update/delete,
invalid saved state, pin existing/create, cap/duplicate, window/notes, and
cross-feature pin risks have stable owners. Reuse plan 144 pilot IDs and data;
do not duplicate the scenario under new IDs.

**Verify**: `cargo xtask policy --only ui.tests` and the complete baseline UI
suite pass; the matrix has no duplicate, orphan, or missing investigations row.

### Step 1: Extract model, schema, and decoded API

Create the target `model/` and `api/` files. Move pure state logic first with
byte-for-byte fixtures for valid v1 data, malformed JSON, unknown version,
empty pins, 100/101 pins, duplicate pins, every pin kind, range search, and
notes/window round trips. Create each named operation and generated sibling
through Plan 152, decode list/detail/save/delete results from `unknown`, map once
to readonly domain values, and return exhaustive typed expected failures. Use
Plan 153 for persisted state/search values. Keep the old routes calling
compatibility exports until the new contracts are green.

**Verify**: run the focused API/model tests, typecheck, and
`cargo xtask policy --only ui.architecture`; malformed envelopes must fail in
the schema adapter rather than entering components.

### Step 2: Extract feature presentation and pin orchestration

Move list, detail, dialog/card, and pin UI into the target component files.
Split any component/hook over 60 logical lines by cohesive responsibility; do
not merely paste the 205/282/198-line owners into renamed files. Represent
load/mutation state exhaustively so impossible boolean combinations disappear,
while retaining all current text, accessibility names, disabled states,
navigation, and error recovery.

Add focused component/integration tests for empty/loading/error, create/delete,
detail save/delete/not-found handoff, lazy popover load, pin to existing, create
and pin, failure/retry, and router invalidation. Use semantic queries and
`userEvent.setup()`.

**Verify**: focused component/integration tests pass with no console/network
escape, and UI ratchets report every new module/component within budget.

### Step 3: Publish the facade and switch routes/consumers

Create the explicit `index.ts`. Convert both routes to thin URL/search/loader/
boundary/composition adapters and remove every export except `Route`. Switch
issues, traces, and runs to the facade import, then register only those three
cross-feature edges. Preserve detail `notFound`, create/delete navigation,
route-generated URLs, range search, loader data shape, current loading/error
boundaries, and cache/invalidation behavior.

Before touching the three consumer routes, confirm plans 139, 140, and 142 have
plan 134 in their live dependency rows and no uncommitted changes in them. Make
one import-only handoff commit and notify those owners after it is green; do not
combine their later feature refactors or leave a compatibility shim.

Move route tests to `ui/src/routes/tests/investigations-routes.test.tsx`. They
must exercise generated route behavior/public facade contracts without reading
`Route.options.component` or importing a private route symbol.

**Verify**: architecture, focused route tests, full typecheck, and production
build pass. `rg -n '@/features/investigations/' ui/src --glob '!features/investigations/**'`
returns no deep import, and the two route files export only `Route`.

### Step 4: Complete browser contracts and remove old owners

Extend plan 144's investigations dataset/screen/contracts to cover the matrix
rows owned here: list/detail, create/edit/delete, pin existing/create, notes,
invalid detail, empty, and recoverable error. Preserve fixture-based public
HTTP/GraphQL behavior and semantic locators; do not intercept happy-path
responses.

Implement the Feature Real-Stack Contract and Feature Browser Breadth sections
in the exact target files, register each non-empty project row once, and keep
shared plan 145/146 fixtures read-only. Do not duplicate the existing pilot.

Delete `ui/src/lib/investigations.ts`, its `__tests__` file, the old console
pin component, and the investigation type in `ui/src/lib/api.ts` only after
`rg` proves no caller remains. Remove compatibility exports in the same step.

**Verify**: focused Vitest, fixture-backed/full-stack/breadth
`@investigations` commands and policies, all UI tests, and `cargo xtask arch`
pass; searches for deleted paths and old deep imports return no matches.

### Step 5: Lock the final ratchets and run all gates

Update exact route/module/function/complexity/public-export baselines after the
old files are gone. Routes must be at most 150 logical lines; handwritten
modules at most 300; functions/components/hooks at most 60; complexity remains
within 12 cyclomatic/15 cognitive. Record any pre-existing exception with exact
scope, owner, expiry, and shrink-only value; do not add a broad exemption.

**Verify**: run every command in the Commands table twice from clean state. A
second run must not change generated files, matrix data, or snapshots.

## Test Plan

- Model: persisted-state compatibility, malformed/unknown versions, pin kinds,
  cap/duplicate/order, URL/range construction, notes/window round trip.
- API: valid/empty/null/malformed list/detail and save/delete transport/schema/
  domain-error projection.
- Components: list/detail empty/loading/error/mutations/accessibility and pin
  lazy load/create/update/failure/retry feedback.
- Routes: list/detail URLs, params, detail not-found, navigation, loader/error
  boundaries, and unchanged cache behavior without private route imports.
- Browser: deterministic investigation CRUD, notes, pins from representative
  feature pages, persistence across navigation, empty/error/invalid detail.
- Real stack: pin/note/CRUD persistence against public-OTLP identities, managed
  GreptimeDB, isolated Turso, and a fresh BrowserContext.
- Facade/type: only reviewed symbols are public; approved consumers compile and
  internal deep imports fail policy fixtures.

## Done Criteria

- [ ] All investigations production, model, API, test, and public UI ownership
  lives under `ui/src/features/investigations`, thin routes, or approved E2E
  directories.
- [ ] Both route modules export only `Route`; every external caller uses the
  explicit facade and only three approved feature edges exist.
- [ ] External data is decoded from `unknown` once, mapped once, and expected
  failures use the feature-owned exhaustive error union.
- [ ] URLs/search, persisted state, loading/empty/error text, navigation,
  not-found, mutations, invalidation, and cache behavior match baseline.
- [ ] All test bodies use `tests/`; the old `__tests__`, library, API type, and
  console component owners are deleted.
- [ ] Vitest/matrix/Playwright evidence covers every named risk and every command
  passes twice.
- [ ] Investigations-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and
  canonical visual rows are non-empty and green.
- [ ] The feature-owned `@investigations` managed-stack row is non-empty, uses
  only public boundaries, and passes with clean teardown.
- [ ] Architecture and size/complexity/export ratchets are green and shrink-only.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`/
  `PageHeaderBack` or reviewed `@/features/time-range` input facade is absent or
  incompatible; do not copy a legacy component, deep-import an internal path,
  add a wildcard export, or defer import repair to plan 143;
- any prerequisite is incomplete or plan 144's investigations pilot cannot be
  reconciled to one dataset/matrix owner;
- plan 145 lacks the delegated `@investigations` reservation or shared managed-
  stack infrastructure, or Step 4 cannot turn that reservation into a non-empty
  public-boundary row with clean one-worker teardown;
- plan 145's shared specs already own the same investigation stable ID/behavior
  and cannot hand it off without duplicate matrix ownership;
- plan 129 is marked complete while the legacy investigation test remains
  without the exact plan 134 expiring topology exception;
- drift changes a URL, GraphQL field, stored-state version, pin kind, cache
  contract, or cross-feature consumer not described here;
- preserving behavior requires a new backend/schema contract or simultaneous
  Query/cache migration;
- a route must export an implementation symbol, a consumer requires a deep
  feature import, or an undeclared fourth feature dependency appears;
- plans 139, 140, or 142 have concurrent uncommitted edits in a PinButton
  consumer route, omit plan 134 from their dependency rows, or cannot accept
  the post-handoff facade import without an old-path shim;
- decoding cannot occur at the API boundary without an unsafe assertion;
- a move would leave both old and new owners, an `__tests__` directory, or an
  oversized pasted module under a permanent exception; or
- any required gate fails twice after one reasonable correction.

## Maintenance And Removal

Future investigation contract changes update the runtime schema, domain mapper,
typed errors, facade, matrix rows, Vitest tests, and deterministic browser data
in one change. New pin consumers require a reviewed exact facade edge; they do
not import internals. Plan 133 may later replace the preserved cache calls, but
must keep this public facade and behavior evidence.

Delete this plan and its index row only after all done criteria are green, old
owners are removed, and durable tests/policy/matrix entries contain the lasting
contract.
