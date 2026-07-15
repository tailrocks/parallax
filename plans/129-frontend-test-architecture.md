# Plan 129: Build deterministic Vitest characterization and test ownership

> **Executor instructions**: Establish a genuinely Bun-run unit/component/route
> baseline before moving production code. Test public behavior and contracts,
> keep test bodies outside production modules, and make unexpected runtime
> diagnostics fail. This plan does not implement browser automation; plans 132
> and 144-146 consume its durable risk matrix and own the browser layers. Do not
> move a feature-coupled test before its production owner exists. Record every
> remaining legacy path/private import as an exact expiring handoff to plans
> 134-143, 149, or 150 so this prerequisite cannot deadlock its consumers.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/package.json ui/bun.lock ui/vite.config.ts ui/tsconfig.json ui/bunfig.toml`
> If test counts, script runtime ancestry, or production owners changed, update
> the current-state inventory and `ui/test-matrix.json` before editing tests.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 094, 101, 128
- **Category**: TypeScript / React / unit and integration testing
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: IN PROGRESS

## Why This Matters

Parallax has substantial UI tests, but the current green result is not yet a
valid Bun-only baseline. At `e3e7997`, the ordinary `bun run test:ci` path can
follow a Node shebang. The forced form `mise exec -- bun run --bun test:ci`
loads 41 files but fails 17 suites at `ui/src/lib/range.ts:12` because `z.object`
is undefined; only 24 files and 100 tests pass. The ordinary path also emits
unimplemented `scrollTo()` diagnostics. Starting the TypeScript refactor or
Playwright adoption from that state would let runtime-loader and harness defects
masquerade as application regressions.

Tests also duplicate router construction, `matchMedia`, and browser shims, use
low-level `fireEvent` broadly, and import implementation exports from route
files. There is no durable machine-readable map from product risk to unit,
component, route, and future browser evidence.

## Current Evidence

- `ui/package.json` exposes `test` and `test:ci` as bare Vitest commands; plan
  094 must make every script recursively Bun-run with installation disabled.
- `ui/vite.config.ts` selects `src/**/*.test.{ts,tsx}` but defines no shared
  setup file, diagnostic policy, or coverage ownership.
- Router builders are repeated across route/component tests, including
  `-overview.test.tsx`, `-services.test.tsx`, `-issues.test.tsx`, and
  `command-palette.test.tsx`.
- `window.matchMedia` is independently replaced in several files.
- `fireEvent` is used for ordinary clicks, typing, and keyboard input in route
  and component tests.
- Route tests import `OverviewContent`, `ServiceDetailContent`,
  `IssueDetailContent`, trace components, loaders, and types directly from route
  modules. Their owning plans 134-143, 149, and 150 must remove those test-only
  exports while moving production code and tests together.
- Current route coverage omits a complete durable matrix for ecosystem,
  investigations, cross-route navigation, cache/reconnect, and all failure
  states.
- 2026-07-15: the mandatory forced-Bun baseline passes all 41 files and 175
  tests with no runtime diagnostic output. The first ownership slice deletes
  the mixed `-final-sweep.test.tsx`, moves its two SQL cases into the existing
  `-sql.test.tsx`, and moves its six dashboard cases into the sole authorized
  `-dashboards.test.tsx` legacy file without changing assertions. The split
  exposed and fixed SQL's order-dependent DOM leak by adding explicit cleanup.
  Both focused files pass 11 tests; TypeScript, Oxfmt, and diff hygiene pass.
- 2026-07-15: added Bun-installed `@testing-library/user-event` 14.6.1 and
  began the semantic-interaction migration with the dashboard create flow,
  using one `userEvent.setup()` session. The shared setup now owns global React
  cleanup plus deterministic `scrollTo` and `matchMedia` shims instead of
  relying on file ordering. The full forced-Bun baseline remains 41 files / 175
  tests; TypeScript, both Oxlint lanes, Oxfmt, and diff hygiene pass locally.
- 2026-07-15: introduced the versioned `ui/test-matrix.json` with exact entries
  for all 41 current files and all 175 named tests. Every row has a stable ID,
  product/risk owner, Vitest lane/layer, environment, and one path-specific
  expiring legacy handoff. `cargo xtask policy --only ui.tests` now discovers
  source tests independently and rejects schema drift, duplicate IDs, unknown
  owners/layers, missing or empty files, mismatched test-name sets, and broad
  handoffs. Positive real-repository validation and a stale-name negative
  fixture pass; both policy tests, strict xtask Clippy, formatting, and diff
  hygiene are green locally.
- 2026-07-15: centralized the remaining browser harness state. Fifteen test
  files no longer redefine `scrollTo`/`matchMedia`, and shell/command-palette
  no longer own duplicate `ResizeObserver`/`scrollIntoView` shims. The shared
  setup now provides those contracts plus jsdom's missing empty
  `Element.getAnimations()` behavior; the latter fixed four Base UI
  post-cleanup exceptions exposed by the stricter global lifecycle. The
  `ui.tests` policy now rejects local shim duplication, test bodies under
  harness-only `src/test`, unrecorded legacy topology, and final `/tests/`
  files that retain legacy handoffs. Its negative fixture exercises both stale
  IDs and shim duplication. All 41 files / 175 tests pass under forced Bun with
  zero unhandled errors; TypeScript, both Oxlint lanes, Oxfmt, focused xtask
  tests, strict xtask Clippy, policy, and diff hygiene pass locally.

## Fixed Test Topology

```text
ui/src/
  app/tests/                public composition/router contracts
  layout/tests/             shell/navigation/theme/global-boundary tests
  features/<feature>/
    tests/
      api/                 runtime contract and adapter tests
      model/               pure transform/state tests
      components/          user-visible component behavior
      integration/         feature orchestration with typed harnesses
  domain/**/tests/         cross-feature pure domain tests
  platform/**/tests/       transport/browser adapter contract tests
  shared/**/tests/         product-neutral unit/component tests
  routes/tests/            URL/search/loader/boundary contracts only
  test/                    setup, fixtures, builders; no test bodies
ui/tests/
  harness/                 tests of setup/builders/diagnostic policy
  e2e/                     one Playwright tree; lane/scenario owners are separate
ui/test-matrix.json        durable risk-to-evidence manifest
```

`tests/` is the only final source-owned test directory name. Test bodies stay
separate from production files and below their owning feature/layer so moves
cannot orphan coverage. `ui/tests/harness/` is the sole exception for testing
test infrastructure itself; it is not product-test ownership. Empty taxonomy
directories are forbidden.

Plan 129 establishes this final rule but does not pretend the final feature
owners already exist. At plan-129 completion, a legacy `__tests__` path or
private route import may remain only through an exact `ui/test-matrix.json`
handoff containing current file/test IDs, imported symbol, destination owner,
removal plan 134-143, 149, or 150, created date, and expiry at that plan's
completion. No wildcard path, generic owner, new legacy test, or broadened
private import is allowed, except the single mechanical dashboard split fixed
below. Each owner deletes its handoff rows atomically with the move; plan 151
proves the legacy set reaches zero.

`ui/test-matrix.json` is schema-versioned and machine-read by xtask. Each row
contains a stable ID, product surface, risk/failure mode, `scenario_owner`,
`lane_owner`, optional `delivery_plan`, test layer, test ID/file, required
environment, status, prerequisite, and optional exact legacy handoff.
`scenario_owner` is a stable architecture ID such as `features/logs` or
`layout/shell`; `lane_owner` is a stable lane ID such as
`playwright/contracts`, `playwright/full-stack`, `playwright/breadth`,
`performance/live`, or `performance/bundle`. Numeric plans are temporary and
appear only in `delivery_plan` while reserving/materializing a row; that field is
cleared when delivery completes. Plan 132 owns base runner/config outside
product rows and is enforced by architecture policy rather than represented by
a fake scenario owner. The durable owner fields cannot collapse or transfer
when a reserved row becomes executable. Only those owners or the exact active
`delivery_plan` may update a row.

## Test-Layer Contract

| Layer | Owns | Must not own |
|-------|------|--------------|
| Model/domain unit | Pure transforms, state machines, exhaustive variants, schema round trips | React/router/network behavior |
| Platform/API contract | Valid/malformed GraphQL/SSE/JSON, cancellation, error mapping, network escape | Feature presentation |
| Component | Visible behavior, accessibility semantics, user interactions | Private implementation details or real external services |
| Feature integration | Router/provider/query seams, mutation orchestration, cache isolation extension point | Whole-product cross-route flow |
| Route contract | URL/search round trip, `loaderDeps`, pending/error/not-found, SSR shell/client navigation | Exported private route components |
| Playwright E2E | Real browser, cross-route flow, storage stack, screenshots/a11y; 132/144-146 own infrastructure lanes and 134-143/150 own product scenarios | Unit-level edge combinations or an ambiguous single-owner row |
| Type test | Facade, generated contract, inference and intentional compile failures | Runtime behavior |

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Forced-Bun baseline | `cd ui && bun run --bun test:ci` | 41/41 current files and all current tests pass; no Node descendant |
| Typecheck | `cd ui && bun run typecheck` | exit 0, no errors |
| Lint | `cd ui && bun run lint` | exit 0, zero warnings |
| Format | `cd ui && bun run check` | exit 0 |
| Focused test | `cd ui && bun run --bun test:ci -- <path>` | selected non-empty tests pass |
| Test policy | `cargo xtask policy --only ui.tests` | final/new topology plus exact legacy handoffs, matrix, diagnostics, and anti-pattern rules pass |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |

All commands use plan 094's checked-in `bunfig.toml`, exact lock-local
executables, and disabled auto-install. A command that succeeds by spawning
Node is a failure even when test assertions pass.

Use Testing Library's semantic query priority and `userEvent.setup()` for normal
interactions. Keep Vitest as the unit/component/route runner; do not replace it
with `bun:test`, Jest, or Playwright component testing. Plans 132 and 144-146 own
the separate black-box browser boundary.

## Scope

In scope:

- Current test inventory, exact legacy handoffs, new owner-correct tests,
  `ui/src/test/**`, and `ui/tests/harness/**`.
- `ui/vite.config.ts`, `ui/package.json`, and `ui/bun.lock` for the exact Vitest,
  Testing Library, and `@testing-library/user-event` test foundation.
- `ui/test-matrix.json` and its schema/xtask validator.
- Test-file Oxlint policy and Oxc-backed structural rules without ESLint or
  alpha JavaScript plugins.
- Characterization required before plan 100 establishes the layer graph, plan
  149 moves shared capabilities, and plans 134-143/150 move features.

Out of scope:

- Production feature/route/capability movement (plans 134-143, 149, and 150).
- Feature-coupled `__tests__` relocation and private route export deletion
  (plans 134-143, 149, and 150); this plan records their exact handoff only.
- Runtime schema/type implementation (plans 152 and 153). This plan freezes
  current wire examples as `unknown` characterization data and must not import
  either future runtime decoder.
- Playwright compatibility/foundation, browser CI, screenshots, and real-stack
  E2E (plans 132 and 144-146).
- TanStack Query implementation; provide only a cache-builder extension point
  that plan 133 fills when Query is installed and first used.
- Replacing Vitest with `bun:test`, running Node, or adding ESLint/Jest.
- A global coverage percentage with no named risk owner.

## Git Workflow

- Stay on the single active branch in `AGENTS.md`; do not create a branch or PR.
- Keep the forced-Bun baseline repair, harness, characterization waves, and
  legacy-handoff policy in independently reviewable green commits.
- Use Conventional Commits, DCO, and exactly one agent-product trailer. Example:
  `test(ui): centralize router harness`.
- Push each durable green update.

## Steps

### Step 0: Require the real Bun baseline

Consume plan 094's forced-Bun repair. Reproduce on supported macOS and Linux:

1. `bun run --bun test:ci` loads all 41 current files and all current tests;
2. the Zod `z.object` loader failure is gone without changing application
   schemas or falling back to Node;
3. process ancestry contains Bun but no Node executable;
4. missing Bun, missing lockfile, implicit install, and zero selected tests fail;
5. warnings, unhandled rejections, and orphan processes are captured.

Record exact Bun/Vite/Vitest/Zod versions and the root cause/fix in durable test
policy. Do not hide the defect by excluding the 17 suites or changing imports
through assertions.

**Verify**: forced-Bun macOS/Linux baseline is green twice from clean state and
the process-tree negative fixture fails when a Node child is injected.

### Step 1: Create the durable risk matrix

Inventory every current test and every product surface. At minimum include:

- shell, navigation, command palette, theme, responsive layout, 404 and route
  fallbacks;
- overview;
- issues list/detail/status and links;
- traces list/detail/search/view modes/compare/links/GraphQL/RPC;
- logs search/context/saved views/live/reconnect;
- services list/detail;
- runs list/detail/live/bundle;
- dashboards list/detail/create/edit/delete/widgets;
- investigations list/detail/create/delete/pins/notes;
- ecosystem graph/navigation;
- SQL safety/history/snippets/results;
- GraphQL/SSE valid, malformed, empty, error, cancel, and reconnect states; and
- URL/search round trips, SSR shell, client navigation, and current cache/
  Refresh/invalidation behavior.

For each risk, record existing meaningful assertions or a missing test. Do not
count a snapshot, rendered line, or test-file existence as behavioral evidence.

**Verify**: xtask rejects duplicate IDs, unknown owners/layers, missing test
files/IDs, empty required statuses, and an omitted product catalog surface.

### Step 2: Build one deterministic harness

Add `ui/src/test/` with:

- global setup/cleanup;
- typed `renderApp`/router/history builders;
- GraphQL and SSE valid/malformed characterization builders from frozen current
  wire examples; builders return `unknown` and import no plan-152 or plan-153
  production schema/decoder;
- per-test cache-reset extension point;
- fixed UTC clock/timezone and deterministic IDs/data;
- `matchMedia`, ResizeObserver, scroll, visibility, theme, and reduced-motion
  support;
- unexpected console warning/error, page/runtime error, unhandled rejection,
  network escape, pending timer, and update-after-cleanup failure hooks.

Extend `ui/vite.config.ts` test discovery to include both source-owned
`src/**/*.test.{ts,tsx}` files and `tests/harness/**/*.test.{ts,tsx}` while
explicitly excluding `tests/e2e/**`. Playwright specs must never be collected by
Vitest, and harness tests must fail zero-selection like every other owned set.

Expected diagnostics require an exact test-local assertion. A global substring
allowlist is forbidden. Fix the current `scrollTo()` warning through a faithful
harness implementation or exact test ownership, not by suppressing console
output.

Place its separated self-tests under `ui/tests/harness/`; `ui/src/test/` contains
no test body.

**Verify**: harness self-tests intentionally trigger every diagnostic and prove
isolation/cleanup across two sequential router instances.

### Step 3: Standardize user interaction and assertions

Add the latest stable mutually compatible `@testing-library/user-event` through
Bun under plan 101 policy. Use `userEvent.setup()` and semantic role/name/label
queries for normal user behavior. Retain `fireEvent` only for an exact primitive
not modeled by user-event, such as low-level resize, pointer, virtualization, or
SSE mechanics, with a short test-local reason.

Ban real sleeps, shared mutable fixture objects, large DOM snapshots,
snapshot-only behavior tests, conditional/focused tests, tests with no meaningful
assertion, and blanket retries. Await observable outcomes instead of manually
flushing framework internals.

Enable stable native Oxlint Vitest/React rules. Enforce Playwright invariants
later through plans 132/144-146 using stable config, runtime checks, and
Oxc-backed policy; do not install an ESLint Playwright plugin or alpha Oxlint
JavaScript plugin.

**Verify**: negative fixtures fail each anti-pattern/rule and the complete
forced-Bun suite stays green.

### Step 4: Characterize routes and feature boundaries

Close every currently shipped row needed by plan 100. Convert a route test to
generated route APIs/public feature contracts only when the required production
facade already exists without moving feature code. Otherwise preserve its
current assertion, adopt the shared harness where possible, and record the exact
legacy file/private symbol/destination/removal-plan handoff rather than inventing
another test facade or production export.

Characterize exact current behavior, including known stale-cache behavior, so
plan 133 can change it intentionally later. Characterization does not bless a
known bug as permanent; the matrix marks it as current behavior plus owning
follow-up plan.

**Verify**: every plan 100 catalog surface and named state has green evidence or
an exact handoff/blocking prerequisite; there is no generic "add during
refactor" row and every remaining private import resolves to one plan 134-143,
149, or 150.

### Step 5: Enforce new topology and publish the feature handoff

Move only tests whose final owner already exists independently of plans 134-143,
149, and 150,
such as test-harness self-tests and any settled platform/domain/shared owner.
Preserve stable matrix IDs and assertions. New tests use the final `tests/`
topology immediately, except the exact temporary dashboard file created by the
mixed-file ownership split below. Do not relocate a feature test into an empty
placeholder directory, export a route private for testing, or duplicate a test
in old/new paths.

For every remaining `__tests__` file and private route import, persist the exact
handoff fields above. Add policy fixtures proving the row is path/symbol/plan
specific, cannot grow, and fails when the target plan/file is absent, the expiry
is reached, or the old path disappears without removing the row.

Mechanically split any legacy test file containing multiple future feature
owners before publishing handoffs. In particular, move every SQL case/ID from
`ui/src/routes/__tests__/-final-sweep.test.tsx` into the existing
`ui/src/routes/__tests__/-sql.test.tsx`, move every dashboard case/ID into new
`ui/src/routes/__tests__/-dashboards.test.tsx`, and delete the mixed file.
Preserve test IDs, assertions, fixtures, and current private imports exactly;
this is an ownership split, not a production move or cleanup. Assign the SQL
file to plan 135 and dashboard file to plan 137. No handoff file may name two
removal plans. Register `-dashboards.test.tsx` as the sole allowed new legacy
path with exact file/test IDs, plan-137 owner, creation in this step, expiry at
plan 137 completion, and mandatory deletion in plan 137 Step 5. Policy fixtures
must reject a second created legacy path, any added test ID/import after the
split, a broadened scope, or survival past that expiry.

**Verify**: policy finds no test body in production or `src/test`, no unrecorded
legacy path/private route import, no wildcard/expired/orphan handoff, and no
matrix orphan. Every new/moved test is in final topology, and every retained
legacy file has one future feature owner; the dashboard split is the only
created legacy path and is byte-for-byte behavior-equivalent to its source
cases.

### Step 6: Add risk-based evidence and structural ratchets

Use branch/condition evidence for the strict boundary schemas, state machines,
and high-risk transforms named by the matrix. If a Bun-compatible coverage
provider is evaluated, keep it report-only until file selection, source maps,
generated exclusions, and repeatability are proven. Do not assume Vitest V8
coverage under Bun or adopt a vanity repository-wide threshold.

Ratchet duplicate harness construction, test file/function size, unjustified
`fireEvent`, snapshots, sleeps, diagnostic allowlists, skipped/focused tests,
private route imports, legacy handoffs, and missing matrix ownership. Legacy
counts and exact symbol sets may only shrink until plan 151 reaches zero.

**Verify**: deliberate growth/stale-row/unknown-owner fixtures fail, targeted
tests pass, and `cargo xtask ci --fast` is green twice.

## Test Plan

- Forced-Bun process ancestry, missing-runtime/lock, implicit-install, zero-test,
  and Zod/Vitest loader regression fixtures on macOS/Linux.
- Harness self-tests for console warnings/errors, rejection, network escape,
  leaked timers/state/cache, missing cleanup, and duplicate router state.
- Semantic query/user-event cases and one justified low-level event case.
- GraphQL/SSE valid/malformed/empty/error/cancel/reconnect contract tests.
- Search/loader/SSR/client-navigation/error/pending/not-found route contracts.
- Test-matrix schema, stable-ID, coverage-catalog, path/test-ID, and orphan
  validation, including exact legacy-handoff shrink/expiry cases.
- Native Oxlint Vitest/React plus Oxc policy negative fixtures.
- Topology/size/private-import/snapshot/sleep/skip/focus/diagnostic ratchets.

## Done Criteria

- [ ] `bun run --bun test:ci` is genuinely Bun-run and green on macOS/Linux;
  passing through Node cannot satisfy the gate.
- [ ] All current test files/tests run, zero-test selection fails, and unexpected
  diagnostics are zero.
- [ ] `ui/test-matrix.json` maps every shipped product risk/surface to stable
  meaningful evidence and survives plan retirement.
- [ ] One typed deterministic harness replaces duplicated router/browser shims.
- [ ] Normal interactions use user-event and semantic queries; every remaining
  `fireEvent` has a precise low-level reason.
- [ ] Every new/moved test uses the single final `tests/` topology and no test
  body lives in production or `ui/src/test`, except the exact plan-137 dashboard
  split whose path/IDs/imports are frozen and expiring.
- [ ] Every remaining legacy `__tests__` path/private route import has one exact
  shrink-only handoff to plans 134-143, 149, or 150; there is no unrecorded or
  wildcard debt.
- [ ] Stable Vitest/React lint and structural ratchets catch intentional defects
  without ESLint or alpha JavaScript plugins.
- [ ] Risk-based evidence covers named critical branches without relying on a
  hollow global percentage.
- [ ] Typecheck, format, lint, forced-Bun tests, test policy, and fast aggregate
  commands pass.

## STOP Conditions

Stop and report; do not improvise if:

- forced-Bun Vitest remains red or requires Node, a foreign package manager,
  live external network, or an application schema weakening;
- a diagnostic can be removed only through a broad global allowlist;
- characterization reveals an unsettled product/cache/search contract with no
  owner;
- characterization would require exporting a new route internal or changing
  production behavior instead of recording an exact existing handoff;
- a dependency requires an unreviewed lifecycle script or mutable install;
- coverage cannot map reliably to handwritten sources under Bun; or
- a targeted/full gate fails twice after a reasonable correction.

## Maintenance And Removal

Every future UI change updates `ui/test-matrix.json` when it adds/removes a risk,
surface, or test ID. Reviewers should reject hidden Node execution, duplicated
harnesses, private route imports, and tests that encode implementation details
instead of observable behavior.

Delete this plan and its README row after the forced-Bun baseline, durable matrix,
harness, characterization, final-topology rule, exact feature handoffs, and
structural policy are green. Legacy handoff rows remain as executable debt owned
by plans 134-143, 149, and 150 and must shrink to zero in plan 151. Keep
`ui/test-matrix.json` and the executable policy as implementation artifacts for
plans 100 and 132-151.
