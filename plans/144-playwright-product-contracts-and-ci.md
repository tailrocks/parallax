# Plan 144: Make fixture-backed Playwright product contracts a required CI gate

> **Executor instructions**: Start only after plan 132 proves the exact locked
> Playwright version runs through Bun on macOS and Linux without a Node process.
> Build the deterministic product-contract harness and required CI lane in this
> plan. Add only shell/navigation and one representative feature pilot here;
> plans 134-143 and 150 add independently owned feature/shell/overview scenarios
> before moving each owner. Run every verification command before proceeding. Stop on any listed
> STOP condition instead of adding a second browser runner or weakening Bun-only
> policy.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- .github/workflows/ci.yml .github/actions ui/package.json ui/bun.lock ui/playwright.config.ts ui/test-matrix.json ui/tests/e2e crates/parallax-server crates/parallax-test-support crates/parallax-xtask ratchet.toml`
> If plan 132 or the Rust workspace changed these owners, reconcile the paths
> with the live facades before editing. Do not recreate an old crate boundary.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 101, 128, 129, 132
- **Category**: tests / Playwright / CI
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: TODO

## Why This Matters

The baseline UI has ten navigable product surfaces but no real-browser test
lane. `.github/workflows/ci.yml:304-330` runs typecheck, lint, Vitest, and a
production build; `ci-required` at lines 407-416 has no browser dependency.
Existing route tests mock GraphQL and import route implementation symbols, so
they cannot prove browser navigation, accessible interaction, process cleanup,
or a production-shaped request boundary.

Plan 132 supplies the runner and a shell smoke. This plan turns that foundation
into a deterministic product-contract lane which later feature plans can extend
in parallel without inventing fixture, locator, artifact, or CI conventions.

## Fixed Decisions

1. `@playwright/test` remains the only browser framework and direct browser
   package. All commands are exact, lock-local, no-install, and Bun-forced.
2. Fixture-backed contracts use the internal in-memory adapter only as a
   test-harness dependency injected at server composition. It is never exposed
   as a product storage mode, CLI option, environment fallback, or release edge.
3. Tests seed state before navigation through a typed test-support facade. The
   browser still uses Parallax's public HTTP/GraphQL/SSE surfaces. Route-level
   `page.route()` happy-path response substitution is forbidden.
4. Every case has a stable ID recorded in `ui/test-matrix.json`, stable
   `scenario_owner`/`lane_owner`, temporary `delivery_plan`, dataset, state
   class, and required lane. Plans 132/144-146 deliver infrastructure; plans
   134-143/150 deliver product assertions/files. Numeric IDs never become durable
   owners, and owner fields never transfer merely because a reservation becomes
   executable.
5. Tests use role, accessible name, label, placeholder, and visible text
   locators plus web-first assertions. CSS/XPath selectors, arbitrary sleeps,
   fixed polling loops, and broad screen objects are policy failures.
6. Required contract CI uses Chromium, one clean retry-free status attempt, and
   isolated BrowserContexts. Cross-browser, touch/mobile, accessibility scans,
   and visual goldens belong to plan 146.
7. Unexpected console errors/warnings, page errors, failed requests, dialogs,
   downloads, external network, server exits, and leaked processes fail the
   owning test unless an exact expiring exception exists.
8. Browser installation is explicit. Do not cache browser binaries unless a
   measured download-versus-restore experiment proves a material win and the
   cache key includes runner/browser version, OS, architecture, and manifest.

## Target Ownership

```text
ui/
  playwright.config.ts
  test-matrix.json
  tests/e2e/
    fixtures/
      test.ts                 # plan 132 base fixture, extended here
      product-fixture.ts      # dataset/reset/readiness contract
    datasets/
      catalog.ts              # stable IDs and typed scenario manifests
      shell.ts
      investigations.ts       # representative pilot only
    screens/
      shell-screen.ts
      investigations-screen.ts
    contracts/
      shell.spec.ts
      investigations.spec.ts
    support/
      artifact-manifest.ts
      test-matrix.ts
crates/<current-test-support-owner>/
  ... typed browser seed/reset/readiness facade
crates/parallax-xtask/
  ... browser server lifecycle and manifest orchestration only
```

Plans 134-143 and 150 create the remaining feature-owned dataset, screen, and
contract files. Keep a screen object only where interactions are reused; assertions stay
visible in the scenario unless a small domain assertion has multiple callers.

## Product Contract Inventory

The durable matrix must reserve stable `scenario_owner` IDs for all independently
delivered groups even though this plan materializes only `shell.*` and the
investigations pilot. Those rows use `layout/shell` and
`features/investigations`; both use `playwright/contracts` as `lane_owner` and
144 only as temporary `delivery_plan`:

| Owner plan | Required browser behavior |
|------------|---------------------------|
| 134 | investigations list/detail/create/update/delete, pins, notes |
| 135 | SQL safe execution, result states, history, snippet CRUD, schema browser |
| 136 | ecosystem topology, empty/error state, trace/service navigation |
| 137 | dashboard list/detail/widget create/update/delete and chart states |
| 138 | services list/detail, filtering, RED/infra absence, exemplar links |
| 139 | issues list/detail, search/filter/status mutation, context links |
| 140 | runs list/detail, session state, runtime snapshot, bundle download |
| 141 | logs search/filter/context/saved views/live/reconnect |
| 142 | traces list/detail/compare, waterfall, GraphQL/RPC/story links |
| 143 | shell, theme, route failures, not-found and cross-route navigation |
| 150 | overview range, trends, movers, onboarding and recent-entity links |

Each feature plan must add happy, empty, loading where observable, recoverable
error, invalid URL/search, and destructive/mutation cases that exist for that
surface. Do not duplicate the same behavior in every browser engine or lane.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Exact install | `cd ui && bun ci` | frozen lock; no lifecycle install |
| Browser install | `cd ui && bunx --bun --no-install playwright install --with-deps chromium` | locked Chromium installed explicitly |
| Inventory | `cd ui && bun run test:browser:list` | stable non-zero case list matching the matrix |
| Contracts | `cd ui && bun run test:browser` | fixture-backed Chromium contracts pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build` | all exit 0 |
| Browser policy | `cargo xtask policy --only ui.browser-contracts` | matrix, locator, isolation, artifact, runtime, and no-interception rules pass |
| CI workflow | `mise exec -- actionlint .github/workflows/*.yml` | exit 0 |
| Fast aggregate | `cargo xtask ci --fast` | contract lane included and green when selected |

Plan 132's `test:browser:list` must list every configured project. This plan
adds `test:browser` as the stable required local/CI interface; later plans add
`test:browser:full` and `test:browser:cross` without changing its meaning.

## Scope

In scope:

- Typed deterministic browser dataset/reset/readiness support behind an
  internal test-only server composition seam.
- Playwright product fixture, stable matrix loader, shell contract, and one
  investigations pilot proving mutation and persistence within the fixture.
- Contract-lane scripts/config/reporters, path-aware CI selection, artifact
  upload, and `ci-required` aggregation.
- Parser-backed policy for stable IDs, semantic locators, assertions, forbidden
  interception/sleeps/focus/skip, and one-to-one matrix/spec ownership.

Out of scope:

- Feature/capability restructuring or remaining flows (plans 134-143, 149, and
  150).
- Real GreptimeDB/Turso behavior (plan 145).
- Firefox, WebKit, touch/mobile, axe, or screenshot baselines (plan 146).
- Query/cache behavior (plan 133), live-data performance (plan 147), bundle
  performance (plan 148), product redesign, or production fixture endpoints.
- Node, another package manager, ESLint, a Playwright lint plugin, an alpha
  Oxlint JavaScript plugin, direct `playwright`/`playwright-core`, or Rust CDP.

## Git Workflow

- Stay on the one active branch; do not create a branch or PR.
- Land the seed facade, product fixture/pilot, and CI gate as separate green
  commits so failures can be attributed.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Freeze the browser contract manifest

Extend `ui/test-matrix.json` with stable IDs for the complete table above. Each
row records `scenario_owner`, `lane_owner`, `delivery_plan`, feature, risk,
dataset ID, state class, required lane, and existing Vitest characterization
IDs. Reject missing or collapsed owner fields, an owner incompatible with the
lane/product inventory, duplicate IDs, unknown lanes, and a spec that is absent
from the matrix. Negative fixtures also reject transferred stable owners, a
terminal/stale or unindexed `delivery_plan`, and incompatible owner/lane pairs.

Record the exact selected UI/Rust/CI paths that must trigger the contract job.
At minimum include UI source/config/lock/test files, GraphQL/API/server/test-
support/xtask code, Cargo manifests/lock, mise, and the CI workflow itself.

**Verify**: `cargo xtask policy --only ui.test-matrix` reports every shipped
surface assigned once with one infrastructure lane and one product scenario
owner, no duplicate or orphan row, and the negative fixtures fail for missing,
collapsed, duplicate, transferred, or unknown ownership.

### Step 1: Build a typed fixture-backed server contract

At the live test-support owner, define a versioned scenario manifest containing
fixed UTC timestamps, trace/span/run/fingerprint/service IDs, deterministic
metric/log/span rows, Turso-like metadata entities, mutation preconditions, and
expected postconditions. Seed/reset APIs are Rust-internal calls used by xtask,
not HTTP product endpoints.

Start the server with the test adapter injected at composition, allocate all
possible application ports dynamically, and emit a sanitized runtime manifest
containing base URL, dataset ID, process IDs, and artifact directory. A reset
must verify the requested dataset identity and leave no state from the prior
test. Do not expose a `memory` storage config or compile the adapter into a
release graph.

**Verify**: Rust tests prove deterministic same-seed output, different-dataset
isolation, reset after success/failure/cancel, release-graph exclusion, occupied
port behavior, and no product configuration path to the adapter.

### Step 2: Extend the automatic product fixture

Extend plan 132's typed Playwright fixture to request one dataset per test,
start/reuse only the owned harness according to config, reset before the page is
opened, attach the runtime/dataset manifests, and assert post-test cleanup. Do
not share mutable state across tests or depend on test order.

Treat unexpected browser/server diagnostics as failures. Allow an exception only
through a structured record with exact event matcher, owner, reason, expiry, and
linked matrix ID. Prove that a stale exception and an unused exception fail.

**Verify**: sequential and parallel self-tests intentionally contaminate state,
crash the server, occupy a port, emit every diagnostic class, and prove the
fixture reports the original failure while cleaning all owned resources.

### Step 3: Add shell and investigations pilot contracts

Add shell contracts for root readiness, primary/workspace navigation, direct
deep-link refresh, invalid route/not-found behavior, theme persistence, and a
fixture-controlled recoverable API failure. Add an investigations pilot which
lists a seeded investigation, opens detail, creates/edits/deletes a record, pins
an item, writes a note, navigates away/back, and verifies persisted fixture
state through the UI and typed postcondition facade.

Use semantic locators and observable assertions only. Do not assert class names,
DOM nesting, React internals, raw GraphQL text, animation timing, or screenshot
pixels here.

**Verify**: the two spec files pass twice in opposite order with one worker and
then the approved parallel count; the same dataset IDs produce the same report
inventory and no external request.

### Step 4: Make contract CI path-aware and required

Add a dedicated `browser-contracts` job. Install Rust and Bun through mise,
restore the established Rust/Bun caches, run `bun ci`, explicitly install the
locked Chromium with system dependencies, build required binaries/UI once, and
run `bun run test:browser` with zero status-clearing retries. Do not install
Node or use npm/npx.

Upload blob/JUnit plus redacted failure traces, screenshots, video, console/
network metadata, runtime/dataset manifests, and server logs when a failure or
cancel occurs. Give artifacts bounded retention and names keyed by run/attempt/
shard. Add the job to the stable `ci-required` aggregate. A skipped job caused
by an irrelevant path must aggregate as success; a selected missing/zero-test
job must fail.

**Verify**: actionlint passes; synthetic path cases select and skip exactly as
declared; intentional failing/zero-test fixtures make `ci-required` red; a
normal contract run makes it green.

### Step 5: Publish the extension contract for feature plans

Document in `ui/AGENTS.md` and test policy: where datasets, screens, feature
specs, and assertions go; how IDs enter the matrix; how mutation postconditions
are checked; which diagnostics are automatic; and which commands every feature
plan runs. Add a template fixture/spec that is compiled and policy-checked but
not counted as a test.

**Verify**: copy the template into a temporary policy fixture, register one
synthetic case, and prove inventory/test execution succeeds; omit each required
field in negative fixtures and prove the policy rejects it.

## Test Plan

- Rust seed/reset/readiness, determinism, isolation, release-edge, port, and
  lifecycle tests.
- Matrix parser tests for complete, duplicate, orphan, missing-owner, unknown-
  lane, and stale-row cases.
- Playwright fixture tests for state leakage and every automatic diagnostic.
- Shell route/navigation/deep-link/theme/failure contracts.
- Investigations CRUD/pin/note pilot with UI and typed postconditions.
- CI path selection, zero-test, failure artifact, skip, aggregate, and no-Node
  process-tree fixtures.

## Done Criteria

- [ ] `ui/test-matrix.json` assigns every shipped browser behavior to one
  product scenario owner and one infrastructure lane owner, and every
  implemented test to one stable ID.
- [ ] Fixture-backed tests use an internal injected test adapter without adding
  a product mode or browser network response stubs.
- [ ] Shell and investigations pilot contracts are deterministic and pass in
  changed order and approved parallel execution.
- [ ] `bun run test:browser` is a stable Bun-only required Chromium contract
  command with zero selected-test ambiguity.
- [ ] Browser CI is path-aware, uploads redacted failure evidence, and is part
  of `ci-required`.
- [ ] Policy rejects forbidden locators, sleeps, interception, focus/skip,
  orphan IDs, broad exceptions, hidden Node, and a second runner.
- [ ] Every command in this plan passes twice from a clean state.

## STOP Conditions

Stop and report if:

- plan 132 is blocked or any contract command starts Node;
- deterministic fixture setup requires a production memory-storage mode or a
  release dependency on the in-memory adapter;
- tests require happy-path browser response interception or shared ordered state;
- a feature cannot name stable seed and observable postcondition contracts;
- required CI cannot distinguish selected, intentionally skipped, and missing/
  zero-test states;
- browser artifacts expose telemetry bodies or secrets after redaction; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

Every new or changed product surface updates its matrix row, deterministic
dataset, browser contract, and path classification in the same change. Keep the
contract lane small and deterministic; real engines and broad browser/visual
coverage remain separate lanes.

Delete this plan and its README row only after the seed contract, shell/pilot
suite, policy, path-aware required CI job, aggregate behavior, and artifacts are
durable and green.
