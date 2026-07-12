# Plan 129: Build a deterministic feature-owned frontend test architecture

> **Executor instructions**: Characterize current behavior before plan 100
> moves routes. Keep Vitest under Bun, test public feature/route behavior, and
> make unexpected browser/runtime errors fail. Do not chase a global coverage
> percentage or rewrite production behavior to simplify tests.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 093, 128
- **Category**: TypeScript / React / testing
- **Planned at**: `a1d8bf82`, 2026-07-12
- **Status**: TODO

## Why

The UI has 41 separate test files and 175 passing tests, but router creation,
`matchMedia`, and browser shims are duplicated; the passing run emits unhandled
browser warnings. Current interaction tests use low-level `fireEvent` broadly,
and no plan maps high-risk routes/boundaries to meaningful characterization
before the feature refactor. Browser smoke has no deterministic Bun command,
data/clock/theme contract, viewport set, or artifact policy.

## Scope

- Risk/critical-path test map and feature-owned test taxonomy.
- Shared typed Vitest/Testing Library setup and router/API/cache-isolation
  builders that plan 100 extends when QueryClient first lands.
- User-level interaction rules, runtime error failure, and async determinism.
- Route/search/loader/SSR contracts and deterministic browser smoke harness.
- Risk-based coverage evidence and test structural ratchets.
- Native/type-aware Oxlint test-file policy supplied by plan 131 and extended
  with boundary rules by plan 128.

Out of scope:

- Moving production features/routes, owned by plan 100.
- Type/runtime schema implementation, owned by plan 128.
- Replacing Vitest with `bun:test` or running any tool through Node.
- Pixel-perfect visual redesign or a blanket global coverage target.

## Steps

### Step 1: Map risk to existing evidence

Extend plan 093's inventory into a matrix of critical user path, data/runtime
boundary, high-churn route, state machine, expected failure modes, current test
IDs, assertion quality, and missing characterization. Prioritize ingest/live
views, traces, issues, runs, dashboards, SQL safety, search URLs, GraphQL errors,
and reconnect/cache behavior.

Record current warnings, console output, unhandled rejections, timers, random/
wall-clock use, network escape, router builders, and browser shim duplication.
Do not count snapshots or lines as behavioral coverage.

### Step 2: Create one deterministic unit/component harness

Add `ui/src/test/` setup with typed render, router, GraphQL/SSE fixture,
cache-reset extension point, clock/timezone, `matchMedia`, ResizeObserver,
scroll, theme, and reduced-motion helpers. Every test receives isolated current
caches and cleanup. Do not add an unused Query dependency here; plan 100 adds a
QueryClient builder in the same slice that installs and first uses Query.

Unexpected console errors/warnings, unhandled rejections, network calls,
pending timers, and state updates after cleanup fail. Allowing an expected
diagnostic requires an exact scoped assertion. Preserve `test:ci` no-tests
failure.

### Step 3: Establish test ownership and interaction rules

Define the final mapping to
`features/<feature>/{api,model,queries,components}`. Existing tests stay in their
current separate owner until plan 100 creates that feature, then move with the
public facade; this plan does not create empty feature directories. Route
contract tests use route APIs/search schemas and do not import private route
components. Pure transforms/state machines and runtime schemas get table tests;
components use semantic role/name queries and `userEvent.setup()`.

Migrate touched `fireEvent` use to user-event. Retain it only for an exact
low-level event not modeled by user-event, with a short reason. Ban large DOM
snapshots, snapshot-only interaction tests, real sleeps, shared mutable data,
and blanket retries.

### Step 4: Characterize routes before plan 100

Before this plan retires, close the current named matrix for root/overview,
logs, traces, services, runs, issues, dashboards, and SQL: URL/search round
trips, loader dependencies, pending/error/not-found behavior, SPA root-shell
render, client navigation, current mutation/Refresh behavior, and SSE visibility
where applicable. Tests use TanStack route contracts and public behavior, never
exported private components. There is no "plan 100 prerequisite" escape row:
all currently existing behavior in the matrix is characterized here. Plan 100
adds Query/move-specific cases before each wave and moves the already-green
tests to feature entries without weakening coverage.

### Step 5: Define the browser contract

Use a repository-owned Rust xtask harness speaking the Chrome DevTools Protocol
to a separately version/digest-pinned Chrome-for-Testing/Chromium binary. Do not
add Puppeteer/Playwright or assume their install documentation guarantees Bun
runtime support. Apply plan 101's policy to select a latest-stable Rust CDP
implementation whose full graph passes native-TLS/no-rustls, license, platform,
and browser-provisioning fixtures; if none passes, STOP rather than hand a Node
process to CI. No dependency lifecycle script may
download or run a browser. Do not promote experimental `Bun.WebView` while its
official API is marked experimental.

The stable repository/CI entry is `bun run test:browser`, which delegates to
`cargo xtask ui-browser test`; explicit `bun run test:browser:update` delegates
to its review-only update mode and never runs in CI or as a side effect. The
xtask starts the declared test-only Parallax server on assigned ports, serves
the production build, waits for `/health`, seeds fixed test-support data, drives
CDP, and owns browser/server shutdown even on failure. Missing route cases or a
zero-test selection fail.

Use deterministic seeded API data, UTC/fixed clock, `en-US`, repository fonts,
light/dark theme fixtures, reduced motion, device scale factor 1, and exact
desktop `1440x900` and mobile `390x844` viewports. Each route starts from an
isolated seed and clean browser context. CDP geometry assertions require no
unexpected horizontal overflow, clipped target, or overlap among named layout
regions. Platform-keyed screenshot comparison uses the pinned browser/fonts and
an explicit initial threshold of at most 0.1% pixels differing by more than
8/255 per channel; any threshold change is a separate shrink-only policy
change. Store actual/diff/DOM/console/network artifacts under
`target/ui-browser/<platform>/<route>/<viewport>/` on failure.

This plan covers the current root plus logs, traces, issues, runs, dashboards,
and SQL route smoke set. Plan 100 adds a case before each later feature wave.
Assert readiness, route marker, no overlap/clipping/overflow, no hydration or
console error, no unexpected network request, and no server-only client-bundle
leak. Executable-path ancestry may contain Bun, Cargo/xtask, the declared
Parallax server, and the browser process family (renderer/GPU/zygote/crashpad),
but no Node or undeclared runtime.

### Step 6: Add risk-based evidence and ratchets

Use branch/condition evidence for high-risk schemas/state machines and touched
hotspots. If instrumented coverage is useful, spike an Istanbul-compatible path
under Bun; do not assume Vitest V8 coverage support. Report uncovered named
risks, not a vanity global percentage.

Ratchet test file/function size, duplicate harness construction, unsupported
fireEvent uses, snapshots, sleeps, and unexpected diagnostic allowlists. New
tests must belong to a feature/shared/app/route contract owner.

Enable Oxlint's native Vitest rules on test files and stable native React Hooks
rules on test components/hooks. Every selected rule has a negative test-file
fixture, and overrides cannot suppress promise, focused/skipped-test,
conditional-test, or hook-order defects.

Vitest/Vite commands consume plan 094's bunfig run/auto-install policy and exact
lock-local binaries. Process ancestry fails Node, implicit install, and mutable
`@latest` even before the browser harness begins.

## Test Plan

- Harness self-tests intentionally producing console errors, rejection,
  network escape, leaked timer/cache, and missing cleanup.
- Semantic query/user-event fixtures and justified low-level event fixture.
- Search/loader/SSR/client-navigation/error-boundary contracts.
- GraphQL/SSE valid and malformed fixture integration with plan 128 schemas.
- Browser repeatability across two clean runs, exact route/viewport inventory,
  geometry/screenshot threshold, artifact-path, readiness, and tamper fixtures.
- Process-tree and blocked-lifecycle fixtures proving only Bun, Cargo/xtask, the
  declared Parallax server, and browser process family are present; Node or an
  undeclared executable fails.
- Native Oxlint Vitest/React negative fixtures for the test file classes.
- Baseline-update/tamper, no-test, duplicate-harness, snapshot, sleep, and
  ratchet negative fixtures.

## Done Criteria

- [ ] Every current critical/high-churn UI path in the named matrix maps to
  meaningful green tests; no case is deferred as a plan 100 prerequisite.
- [ ] One typed deterministic harness replaces duplicated router/browser setup.
- [ ] Unexpected console/rejection/network/timer failures make CI red.
- [ ] User interactions use semantic queries/user-event except reasoned exact
  low-level cases.
- [ ] Route tests do not require private route implementation exports.
- [ ] Browser smoke is Bun-invoked, deterministic, responsive, and artifacted.
- [ ] Browser smoke owns exact routes, viewports, readiness, seed, geometry,
  screenshot threshold, update mode, process ancestry, and no-test failure.
- [ ] Native Oxlint Vitest/React rules catch their intentional test defects.
- [ ] Risk-based evidence covers touched critical branches without a hollow
  global target.
- [ ] Test structural/diagnostic ratchets cannot grow silently.

## STOP Conditions

- Test setup needs Node, a foreign package manager, live external network,
  unreviewed rustls/CDP dependencies, or mutable shared user data.
- Characterization reveals an unsettled product/cache/search contract; assign
  it before encoding the wrong behavior.
- A test requires exporting route internals or changing production semantics.
- Browser baselines vary under fixed inputs and the cause is not understood.
- Coverage cannot run reliably under Bun or reports generated files as product
  quality; keep named risk evidence instead.

## Remove When

Delete this plan and row when the shared harness, ownership rules, route
characterization, deterministic browser smoke, and risk-based gates are green.
