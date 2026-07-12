# Plan 139: Move issues and stacktrace ownership into one feature

> **Executor instructions**: Refactor the issue list, issue detail, and
> stacktrace parser as one behavior-preserving feature migration. Preserve the
> `/issues/` and `/issues/$fingerprint` contracts, URL/search behavior, request
> order/count, best-effort correlation behavior, status and bucket interactions,
> loading/empty/error states, cache calls, and rendered output. Do not change
> cache ownership or introduce TanStack Query; plan 133 owns that separately.
> Run the local verification after every step and stop on any STOP condition.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/routes/issues.index.tsx 'ui/src/routes/issues.$fingerprint.tsx' ui/src/routes/__tests__/-issues.test.tsx ui/src/lib/stacktrace.ts ui/src/lib/__tests__/stacktrace.test.ts ui/src/features/issues ui/test-matrix.json ratchet.toml`
> Plans 100/129/149 may already have moved lower-layer imports, route-less
> capabilities, or harness code. Follow their ledger, but STOP if issue requests,
> states, or user behavior changed.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 100, 129, 132, 134, 144, 145, 146, 149, 152, 153
- **Category**: TypeScript / issues / feature migration
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: TODO

## Why This Matters

The two issue routes contain 1,383 lines spanning search, queries, decoded data,
trend/tag transforms, mutation state, virtualized tables, stacktrace parsing,
runtime context, breadcrumbs, and presentation. The 166-line stacktrace parser
is issue-domain logic stranded under generic `lib`. Tests import route internals
instead of a reviewed feature owner. This plan moves those responsibilities into
one explicit facade while preserving every contract.

## Current Paths And Responsibilities

| Current path | Current responsibility at `e3e7997` | Required final owner |
|---|---|---|
| `ui/src/routes/issues.index.tsx` | Issue search/filter/sort, list loader, tag/trend helpers, virtualized table and sparkline | Thin list route plus `features/issues` |
| `ui/src/routes/issues.$fingerprint.tsx` | Detail/correlation loader, status mutation, bucket request race guard, trend/stack/context/tags/breadcrumbs/occurrences UI | Thin detail route plus `features/issues` |
| `ui/src/lib/stacktrace.ts` | Rust/Python/V8/Go/Java frame parsing, app/library classification, structured-frame count | `features/issues/model/stacktrace.ts` |
| `ui/src/routes/__tests__/-issues.test.tsx` | List/detail rendering, range links, frames and breadcrumbs | Feature behavior under `features/issues/tests/**`; route contracts under `routes/tests/` |
| `ui/src/lib/__tests__/stacktrace.test.ts` | Parser language and raw-fallback cases | `features/issues/tests/model/stacktrace.test.ts` |
| Plan-152 GraphQL generator/handoff | Runtime validation template for list/detail/correlation/mutation/bucket payloads | Create named operations and generated siblings under `features/issues/api/` |
| Plan-153 search/JSON mechanism | URL search plus embedded resource/tag JSON | Instantiate with issue-owned schemas and existing fallbacks |

At baseline the list route exports `MiniSparkline`, `IssueRow`, `IssuesData`,
`IssuesSearch`, search/tag helpers and `IssuesContent`; the detail route exports
its loader and `IssueDetailContent`. The final route modules export only `Route`.

## Fixed Behavior And Ownership

1. Keep the exact `/issues/` and `/issues/$fingerprint` route paths, fingerprint
   parameter handling, search keys/defaults, range propagation, and sort/filter
   semantics.
2. Preserve list/detail/correlation/bucket/status request documents, variables,
   order, count, abort/race behavior, `graphql` versus `graphqlCached` choice, and
   Router invalidation behavior. Plan 133 later changes cache ownership.
3. Trace correlation remains best effort: an unavailable/aged-out trace must not
   prevent issue detail rendering. Preserve the current empty resource,
   breadcrumb, run-link, and release fallback semantics.
4. Plan-152-generated schemas parse every GraphQL operation once. Issue-owned
   schemas use Plan 153's mechanism for URL search and embedded correlation
   resource/tag JSON. API mappers create readonly issue domain values once;
   route/components never parse wire JSON or use a generic cast.
5. `model/issues-error.ts` owns discriminated list/detail/correlation/mutation/
   bucket failure variants. Adapters map transport/decode failures once while
   preserving current thrown boundaries, best-effort catches, and visible error
   text.
6. The stacktrace parser is issue-owned pure model logic. Its language behavior,
   raw passthrough, app/library classification, frame order, and structured count
   do not change.
7. External imports use the explicit issues facade. No route implementation,
   app/layout module, or another feature's internal file is imported.
8. Use pure functions and readonly data. A class is allowed only for real mutable
   identity/lifecycle; the existing bucket request identity remains a React ref,
   not a new service class.

## Plan 149 Capability Contract

- Issues imports `PageHeader` from `@/shared/components/page-header`,
  `RangePicker` from `@/features/time-range`, and `MetricStrip` plus only its
  minimum readonly inputs from `@/features/runtime-metrics`.
- Use explicit named value/type imports only. Do not deep-import plan 149
  internals, use wildcard barrels, copy a legacy capability into issues, or
  defer a completed issue import to plan 143.
- Plan 152 owns GraphQL/cache behavior, Plan 153 owns search/embedded-JSON
  decoding, and Plan 100 retains formatting, pure range, and other technical/
  domain foundations. Issues supplies typed inputs and
  composition only.

## Target Tree

```text
ui/src/features/issues/
  api/
    issues-list.graphql
    issues-list.generated.ts
    load-issues.ts
    issue-detail.graphql
    issue-detail.generated.ts
    load-issue-detail.ts
    issue-status.graphql
    issue-status.generated.ts
    set-issue-status.ts
    issue-occurrences.graphql
    issue-occurrences.generated.ts
    load-issue-occurrences.ts
    issues-mapper.ts
  model/
    issue-summary.ts
    issue-detail.ts
    issues-search.ts
    issues-search-schema.ts
    issue-trend.ts
    issue-tags.ts
    stacktrace.ts
    issues-error.ts
  components/
    issues-page.tsx
    issues-table.tsx
    issue-sparkline.tsx
    issue-detail-page.tsx
    issue-trend-chart.tsx
    issue-stacktrace-card.tsx
    issue-context-sections.tsx
    issue-tags-table.tsx
    issue-breadcrumbs.tsx
    issue-occurrences.tsx
  hooks/
    use-issue-actions.ts
  tests/
    api/issues-api.test.ts
    model/issues-search.test.ts
    model/issue-trend-tags.test.ts
    model/stacktrace.test.ts
    components/issues-page.test.tsx
    components/issue-detail-page.test.tsx
    components/issue-stacktrace-card.test.tsx
  index.ts
ui/src/routes/
  issues.index.tsx
  issues.$fingerprint.tsx
  tests/issues-routes.test.tsx
ui/tests/e2e/
  datasets/issues.ts
  screens/issues-screen.ts
  contracts/issues.spec.ts
  full-stack/issues.spec.ts
  accessibility/issues-accessibility.spec.ts
  mobile/issues-mobile.spec.ts
  visual/issues.visual.spec.ts
  visual/goldens/
    issues-list.png
    issues-detail.png
    issues-stacktrace.png
    issues-status-error.png
    issues-empty.png
```

Plan 152 provides the generator/template and handoff rows, not these product
files. This plan creates each named operation and exact `.generated.ts` sibling;
the feature search/embedded-JSON schemas separately instantiate Plan 153.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | no route/deep/cycle/unknown-owner edge |
| UI architecture | `cargo xtask policy --only ui.architecture` | issue facade, route-only exports, decoder and runtime rules pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | issue route/module/function/export rows shrink; no exception grows |
| Test ownership | `cargo xtask policy --only ui.tests` | issue/stacktrace matrix IDs resolve only below feature tests |
| Focused tests | `cd ui && bun run --bun test:ci -- src/features/issues/tests` | non-zero issue tests pass without diagnostics |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0, zero warnings/errors |
| Unit suite | `cd ui && bun run --bun test:ci` | all tests pass under Bun, no Node descendant |
| Browser contract | `cd ui && bun run test:browser -- --grep @issues` | non-zero fixture-backed issue rows pass |
| Real-stack browser | `cd ui && bun run test:browser:full -- --grep @issues` | non-zero managed GreptimeDB + Turso issue rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross -- --grep @issues` | non-zero Firefox/WebKit/mobile issue rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y -- --grep @issues` | non-zero axe/keyboard/focus issue rows pass |
| Visual | `cd ui && bun run test:browser:visual -- --grep @issues` | non-zero canonical issue visual rows pass |
| Browser contract policy | `cargo xtask policy --only ui.browser-contracts` | issue matrix/spec/fixture ownership passes |
| Real-stack policy | `cargo xtask policy --only ui.browser-full-stack` | issue storage/seed/lifecycle ownership passes |
| Browser breadth policy | `cargo xtask policy --only ui.browser-breadth` | issue engine/mobile/a11y/visual ownership passes |
| Build | `cd ui && bun run build` | exit 0, generated route tree current |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | issue real-stack and breadth lanes are green |

All tooling is exact-lock, auto-install-disabled, and Bun-only. Oxc-backed xtask
policy owns source parsing/resolution/ratchets. Do not introduce Node, a foreign
package manager, ESLint, a JavaScript lint plugin, or a second source graph.

## Feature Real-Stack Contract

`ui/tests/e2e/full-stack/issues.spec.ts` owns plan 145's delegated, non-empty
`@issues` row. Seed a correlated error span with deterministic fingerprint,
resource context, breadcrumb events, and stacktrace attributes through public
OTLP using `datasets/issues.ts`; wait on the named public issue predicate; then
drive list filtering, detail navigation, parsed stack/context/breadcrumb output,
occurrence-bucket selection, and trace/service links through
`screens/issues-screen.ts`. Do not repeat plan 145's distinct `@storage` issue-
status persistence proof.

Run one worker against managed GreptimeDB plus an isolated Turso database. Use
only public OTLP, GraphQL, and UI boundaries with bounded readiness predicates;
never write/read database internals, intercept browser responses, or use fixed
sleeps.

**Verify**: `cd ui && bun run test:browser:full -- --grep @issues` selects at
least one plan-139 row and passes with the real-stack runtime manifest and clean
process/port/data teardown.

## Feature Browser Breadth

This plan owns every `@issues` row that consumes plan 146's projects. Run issue
list/detail, filters/search, status mutation, bucket selection, stacktrace, and
trace/service context links in Firefox and WebKit. Cover virtualized rows,
stacktrace expansion, long error text, status controls, and overflow on both
mobile device projects. Run axe plus keyboard/focus/Escape/restoration checks
and maintain canonical list, detail, empty, status-error, and stacktrace visual
states.

**Verify**: `cd ui && bun run test:browser:cross -- --grep @issues && bun run test:browser:a11y -- --grep @issues && bun run test:browser:visual -- --grep @issues` selects non-zero breadth rows and passes without response interception, broad masking, or order-dependent mutation state.

## Scope

**In scope:**

- Both issue routes and every issue-specific responsibility listed above.
- `ui/src/lib/stacktrace.ts` and its tests.
- New `ui/src/features/issues/**`, explicit facade, and separated issue tests.
- Issue/stacktrace `ui/test-matrix.json` and exact `ratchet.toml` rows.
- Feature-owned issues dataset/screen/contract/full-stack/accessibility/mobile/
  visual/golden files and their non-empty plan 144-146 matrix rows.
- Tool-generated route-tree refresh only through the normal build.

**Out of scope:**

- Query/cache/invalidation/freshness changes and `graphqlCached` deletion (plan
  133), mutation redesign, or an unplanned polling optimization.
- Backend GraphQL changes, new issue workflow/statuses, fingerprint semantics,
  visual redesign, new parser languages, or parser correctness changes unrelated
  to movement.
- Moving Plan-149 page-header/time-range/runtime-metrics capabilities or
  Plan-100 formatting/pure-range/technical owners, modifying shadcn or generated
  files by hand, or another feature.
- Shared plan 144-146 Playwright configuration, fixtures, reporters, lifecycle,
  CI, matrix schema, and browser infrastructure; consume them read-only.
- Catch-all modules, internal packages/project references, or classes without a
  lifecycle/invariant-bearing identity.

## Git Workflow

- Use the single active branch only; do not create a branch or PR.
- Feature-local files may be built in parallel, but edits to
  `ui/test-matrix.json`, `ratchet.toml`, and any shared generated registry/config
  are serialized feature-scoped commits. Re-read current content, require no
  uncommitted writer, change only issues rows, land green, then hand off. Never
  regenerate or replace another feature's content.
- Keep model/parser, API adapters, components/actions, and route/test closure in
  separate reviewable green changes.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable green update under repository policy.

## Steps

### Step 0: Freeze issue behavior and evidence

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

Do not run focused target paths or any `--grep @issues` command until Step 5
creates and registers those files; zero selection is intentionally fatal.
Resolve Plan 152's generator and exact issue handoff rows, Plan 153's search/JSON
paths, Plan 149's final
page-header/time-range/runtime-metrics facades, and plan 100's technical/pure-
domain facade paths. Record list/detail/correlation/bucket/status request sequence and
count, URL/search round trips, sort/filter/range behavior, missing issue, aged-
out trace, mutation errors, in-flight bucket replacement, stacktrace variants,
and browser markers.

Require every issues/stacktrace `__tests__` path and private route import to have
an exact plan-129 legacy handoff owned by plan 139. Stop on a missing, wildcard,
expired, or differently owned row; delete each row when its test/import moves.

Confirm plan 145 reserves `@issues` for
`ui/tests/e2e/full-stack/issues.spec.ts` and its shared `@storage` spec retains
only the distinct issue-status persistence proof. Consume the feature
reservation without duplicating that stable ID or mutation scenario.

**Verify**: every prerequisite command above exits 0, the legacy handoffs are
exact, and the delegated issues row is reserved but not yet required to select
a feature spec.

### Step 1: Extract the issue model and stacktrace parser

Move issue summary/detail/event/trend/search/tag/resource/breadcrumb shapes and
pure transforms into named model files. Move `stacktrace.ts` intact first, then
split only where responsibility and size require it. Preserve parser output byte
for byte for existing cases. Add the typed error union without changing route or
component error handling yet.

Move tests, not copies, into model test ownership. Add malformed tag/resource
cases through feature-owned Plan-153 schemas rather than assertions.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/issues/tests/model && bun run typecheck`
must pass all parser/search/trend/tag cases; `cargo xtask policy --only ui.ratchets`
must show shrink-only source rows.

### Step 2: Move decoded issue API adapters

Move the existing canonical list, detail, correlation, status, and bucket
operations/schemas under issue API ownership. Implement one mapper per distinct
wire result and typed error projection. Preserve the correlation request's
best-effort failure boundary and the bucket request identity/race guard. Route and
component code may receive only decoded domain values and typed action results.

No raw query, manual interpolation/escaping, `JSON.parse` cast, generic `as T`, or
duplicate schema remains in an issue route/component.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/issues/tests/api && bun run typecheck`
must cover valid/null/missing/malformed/error/abort, aged-out correlation, status
failure, and stale bucket response. Step-0 request sequence/count/cache calls must
remain exact.

### Step 3: Split issue components and action orchestration

Move list page/table/sparkline and detail page/chart/stack/context/tag/breadcrumb/
occurrence sections into the target component files. Move status and bucket
orchestration into `use-issue-actions.ts` only if it is a genuine React lifecycle
owner; keep its state as a discriminated union so impossible loading/error/value
combinations are not representable. Preserve accessible names, virtualization
thresholds, row identity, scroll behavior, library-frame toggle, links, and all
current visual/empty/loading/error states.

**Local verification:**
`cd ui && bun run --bun test:ci -- src/features/issues/tests/components && bun run check && bun run lint`
must pass and the fixture browser baseline must show no rendering/interaction
delta.

### Step 4: Publish the facade and thin both routes

Create an explicit `features/issues/index.ts`. Export only the route components,
loaders/search contract needed by route adapters and stable issue contracts used
outside the feature. No wildcard or accidental generated/heavy export is allowed.

Reduce both route files to route creation, params/search/loader wiring, boundary
selection, and composition. Each route exports only `Route`, imports the feature
through `@/features/issues`, and imports no route implementation or feature
internal path.

**Local verification:**
`cargo xtask arch && cargo xtask policy --only ui.architecture && cd ui && bun run build`
must prove both unchanged route IDs, facade-only imports, only-`Route` exports,
and no implementation retained by route chunks.

### Step 5: Close tests, matrix, ratchets, and old paths

Move issue API/model/component tests under `features/issues/tests/**` and exact
URL/search/loader/error/navigation contracts under
`routes/tests/issues-routes.test.tsx`. Preserve stable matrix IDs, remove private
route imports, and delete the two old test files. Exercise route behavior through
the public router/feature facade. Do not create another `__tests__` tree.

Create or extend the exact feature-owned `datasets/issues.ts`,
`screens/issues-screen.ts`, fixture contract, full-stack, accessibility, mobile,
visual, and named golden files in the Target Tree. Consume plan 145's reserved
`@issues` row, register each feature matrix ID/project once, and make every grep-
scoped selection non-empty. Shared plans 144-146 fixtures, configuration,
reporters, lifecycle code, and infrastructure remain read-only.

Ratchet both routes to 150 lines, all new handwritten modules to 300, functions/
components/hooks to 60, and complexity to 12/15. Remove old exact rows as their
paths disappear; no new size, export, assertion, suppression, import, or test
topology exception may be added.

**Local verification:** run the command table twice. Every `--grep @issues`
selection must be non-zero, `git diff --check` must be clean, and scoped status
must contain only this plan's allowed files.

## Test Plan

- `tests/api/issues-api.test.ts`: list/detail/correlation/status/bucket valid,
  null, malformed, error, cancel, best-effort and stale-response behavior.
- `tests/model/issues-search.test.ts`: garbage/default search, every sort/filter,
  range preservation, clearing and request-variable mapping.
- `tests/model/issue-trend-tags.test.ts`: trend totals/delta, top tags, malformed/
  empty tags, context grouping, and breadcrumb truncation/order.
- `tests/model/stacktrace.test.ts`: every existing language, compact forms,
  app/library classification, raw fallback, empty input and structured count.
- `tests/components/issues-page.test.tsx`: virtualization boundary, sparkline,
  filters/sorts, links, loading and both empty states.
- `tests/components/issue-detail-page.test.tsx`: not-found, correlation fallback,
  status success/error, bucket race, links, runtime context and occurrences.
- `tests/components/issue-stacktrace-card.test.tsx`: structured/raw/empty frames,
  culprit highlight and library toggle.
- `routes/tests/issues-routes.test.tsx`: exact URLs/search/loaders/error
  boundaries and navigation through public route behavior.
- Fixture browser: deterministic issue list/detail/filter/status/bucket/
  stacktrace/link states through `@issues`.
- Real stack: public-OTLP correlated error, parsed stack/context/breadcrumbs,
  occurrence bucket, and trace/service links against managed engines.
- Browser breadth: selected Firefox/WebKit/mobile behavior, axe/keyboard/focus,
  and named canonical issue visuals.

## Done Criteria

- [ ] Both issue routes export only `Route`, retain exact URLs/search semantics,
  and are at or below 150 logical lines.
- [ ] All issue API/model/component/action ownership, including stacktrace, lives
  under `features/issues` and external consumers use its explicit facade.
- [ ] Every external payload is decoded once and mapped once; typed issue errors
  preserve current fatal versus best-effort and visible-error behavior.
- [ ] Request order/count/cache calls, status invalidation, bucket race handling,
  loading/empty/error states, links, virtualization and browser behavior match.
- [ ] Issue/stacktrace feature tests live under `features/issues/tests/**`, route
  contracts live under `routes/tests/`, and no private route import/old test remains.
- [ ] Issues-owned Firefox/WebKit, mobile/touch, axe/keyboard/focus, and canonical
  visual rows are non-empty and green.
- [ ] The feature-owned `@issues` managed-stack row is non-empty, uses public
  OTLP/GraphQL/UI boundaries, and passes against GreptimeDB + Turso.
- [ ] Architecture/test/ratchet, Bun format/lint/typecheck/unit/browser/build and
  fast/full aggregate gates pass twice.
- [ ] No class ceremony, catch-all module, wildcard facade, duplicate schema,
  Query/cache change, generated manual edit, or unrelated feature change landed.

## STOP Conditions

Stop and report if:

- plan 149 is incomplete or its `@/shared/components/page-header` `PageHeader`,
  `@/features/time-range` `RangePicker`, or
  `@/features/runtime-metrics` `MetricStrip` facade/input contract is absent or
  incompatible; do not copy a legacy component, deep-import internals, add a
  wildcard export, or defer import repair to plan 143;
- prerequisites or forced-Bun/browser issue evidence are incomplete/red;
- plan 145 lacks the delegated `@issues` reservation/shared managed-stack
  infrastructure, or Step 5 cannot make it a non-empty public-boundary row with
  clean one-worker teardown;
- the shared `@storage` issue-status proof or another feature owns the same
  issues stable ID/scenario, or the reservation points at a different file;
- feature browser evidence requires editing shared plans 144-146 fixtures,
  configuration, lifecycle, reporters, CI, or matrix schema;
- the baseline request, URL/search, cache, mutation, race, best-effort, parser or
  visual behavior has drifted materially;
- Plan 152's generator/handoff cannot represent a frozen issue GraphQL boundary,
  or Plan 153 cannot support its search/embedded-JSON boundary;
- a parser move changes existing output or requires a new language policy;
- preserving behavior requires a backend/GraphQL/cache/product decision, another
  feature edit, manual generated/shadcn change, or broad graph exception;
- the facade/layer graph becomes cyclic or needs a deep feature import;
- structural limits require arbitrary file fragmentation; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Required Deletions

Future issue boundaries add/update document, runtime schema, mapper, typed error,
tests, matrix rows and facade in one change. Plan 133 may add `queries/` and
replace current cache/invalidation, but must preserve this ownership.

Delete before plan retirement:

- `ui/src/routes/__tests__/-issues.test.tsx`;
- `ui/src/lib/stacktrace.ts` and `ui/src/lib/__tests__/stacktrace.test.ts` after
  their feature-owned replacements are the only callers;
- every issue implementation export from both route files;
- every temporary old-path issue/stacktrace reexport; and
- every completed issue-specific migration exception/ledger row.

Delete this plan and README row only after these deletions and all done criteria
are durable and green.
