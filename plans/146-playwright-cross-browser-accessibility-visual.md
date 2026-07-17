# Plan 146: Establish cross-browser, mobile, accessibility, and visual Playwright gates

> **Executor instructions**: Extend the fixture-backed Playwright stack without
> changing product structure or duplicating feature scenarios. First prove the
> exact locked Bun/Playwright combination launches and tears down every selected
> engine on macOS and Linux. Then add project, accessibility, keyboard, mobile,
> and canonical visual infrastructure with shell plus one representative pilot.
> Plans 134-143 and 150 add their owned breadth using these projects. If an
> engine or dependency fails the no-Node compatibility gate, mark this plan
> BLOCKED; do not substitute another runner or silently claim partial parity.
> Start after plan 145 so the shared Playwright config, scripts, matrix schema,
> and CI workflow have one writer at a time; preserve its full-stack project.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- .github/workflows/ci.yml ui/package.json ui/bun.lock ui/playwright.config.ts ui/test-matrix.json ui/tests/e2e ui/src/styles.css crates/parallax-xtask ratchet.toml`
> Reconcile plan 132/144's fixture, reporter, project, and artifact contracts
> before editing. Preserve one test identity across projects.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 101, 132, 144, 145
- **Category**: tests / compatibility / accessibility / visual
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: IN PROGRESS — 145 Playwright config ownership landed; begin engine/a11y/visual projects (2026-07-17)

## Why This Matters

A Chromium desktop contract alone does not validate Firefox/WebKit behavior,
touch input, actual mobile device characteristics, keyboard focus, runtime
accessibility, or layout pixels. A resized desktop viewport is also not a mobile
contract. Parallax's dense tables, virtualized timelines, menus, dialogs,
charts, theme behavior, and responsive shell need distinct evidence classes so
a passing screenshot does not masquerade as behavioral or accessibility proof.

This plan provides those reusable evidence lanes. Feature plans add cases at
the same time they migrate each surface; plan 143 owns remaining shell breadth
and plan 150 owns overview breadth.

## Fixed Decisions

1. Playwright projects explicitly own Chromium, Firefox, WebKit, mobile/touch,
   accessibility, and canonical visual execution. There is no generic
   "all browsers" shell script with hidden defaults.
2. Browser engines come only from the locked `@playwright/test` browser
   revision and are installed explicitly through Bun-forced, no-install
   commands. System browsers and mutable latest downloads are forbidden.
3. Before enabling a project, prove config/spec loading, workers, launch,
   interaction, reports, artifacts, teardown, and process ancestry under Bun on
   macOS and Linux. No Node child, hang, leaked process, or hidden flag is allowed.
4. `@axe-core/playwright` is the only additional browser test integration and
   is exact-pinned only after plan 101 passes Bun runtime, peer, lifecycle,
   integrity, license, and unused-dependency checks. Do not add ESLint,
   `eslint-plugin-playwright`, an alpha Oxlint JavaScript plugin, or a second axe
   wrapper.
5. Runtime accessibility scans supplement, not replace, semantic locators,
   keyboard navigation, focus order/restoration/trapping, Escape behavior,
   accessible names, reduced motion, and manual review.
6. Mobile projects use real Playwright device descriptors with viewport,
   screen, device scale, user agent, touch, and mobile context. A viewport-only
   project is named responsive desktop and cannot satisfy a mobile row.
7. Visual goldens are produced only in one digest-pinned Linux image with fixed
   browser/font/locale/timezone/color/reduced-motion inputs. Developer machines
   may inspect diffs but cannot author canonical baselines.
8. Required tests have zero status-clearing retries. Visual thresholds start at
   exact match where stable; any non-zero threshold is measured, scoped,
   owner/reason/expiry bound, and shrink-only.
9. Every breadth row uses stable `lane_owner: playwright/breadth` and its final
   feature/layout `scenario_owner`. Plans 134-143/150 appear only as temporary
   `delivery_plan`; materialization clears that field and never transfers or
   collapses durable ownership.

## Target Projects And Files

```text
ui/playwright.config.ts
ui/tests/e2e/
  accessibility/
    shell-accessibility.spec.ts
    investigations-accessibility.spec.ts
  mobile/
    shell-mobile.spec.ts
    investigations-mobile.spec.ts
  visual/
    shell.visual.spec.ts
    investigations.visual.spec.ts
    goldens/                    # canonical reviewed outputs only
  fixtures/
    accessibility-fixture.ts
    visual-fixture.ts
  support/
    accessibility-exceptions.ts
    visual-manifest.ts
```

Required project identities:

| Project | Purpose |
|---------|---------|
| `contracts-chromium` | plan 144's deterministic behavioral gate |
| `cross-firefox` | selected behavior parity in Firefox desktop |
| `cross-webkit` | selected behavior parity in WebKit desktop |
| `mobile-chromium` | touch/mobile behavior on Chromium device descriptor |
| `mobile-webkit` | touch/mobile behavior on WebKit device descriptor |
| `accessibility-chromium` | deterministic axe and keyboard/focus cases |
| `visual-chromium-linux` | canonical reviewed screenshots only |

Do not repeat every contract in every project. `ui/test-matrix.json` declares
which risk needs which project; shared contract code is reused only when the
same observable assertion is valid across engines.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Exact install | `cd ui && bun ci` | frozen lock, no lifecycle scripts |
| Browsers | `cd ui && bunx --bun --no-install playwright install --with-deps chromium firefox webkit` | locked engines installed explicitly |
| Cross-browser/mobile | `cd ui && bun run test:browser:cross` | selected Firefox/WebKit/mobile cases pass |
| Accessibility | `cd ui && bun run test:browser:a11y` | axe plus keyboard/focus cases pass |
| Visual check | `cd ui && bun run test:browser:visual` | canonical comparison passes; no baseline write |
| Policy | `cargo xtask policy --only ui.browser-breadth` | engine, device, axe, exception, golden, and matrix rules pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build` | all exit 0 |
| Workflow | `mise exec -- actionlint .github/workflows/*.yml` | exit 0 |

The canonical update command must be a separate explicit maintainer action such
as `bun run test:browser:visual:update` which refuses to run outside the pinned
environment and produces a review manifest. It is never called by normal CI.

## Scope

In scope:

- Exact-engine Bun compatibility matrix and explicit browser provisioning.
- Playwright desktop/mobile/accessibility/visual projects and typed fixtures.
- Exact-pinned `@axe-core/playwright` after dependency policy passes.
- Shell plus investigations pilot cases, matrix/project rules, canonical visual
  environment, golden review/update protocol, and path-aware CI lanes.
- Failure artifacts and structured accessibility/visual exceptions.

Out of scope:

- Repeating the complete feature suite here; plans 134-143 and 150 add their
  breadth.
- Real GreptimeDB/Turso behavior (plan 145).
- Product redesign or using visual diffs to approve intended design changes.
- Unit/component coverage, query/cache/live/bundle work, Node, direct browser
  packages, another test runner, browser response interception, or CSS/XPath.

## Git Workflow

- Stay on the one active branch; do not create a branch or PR.
- Land compatibility evidence, projects/accessibility, and canonical visual/CI
  changes as separate green commits.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Prove every engine and dependency under Bun

Using the locked plan 132 harness, extend the macOS/Linux compatibility matrix
for Firefox and WebKit: explicit install, list, launch, semantic click/type,
download, trace/screenshot/video, one/multiple workers, timeout, failure,
cancel, reports, and teardown. Repeat for the two selected device descriptors.
Record exact browser revisions and process trees.

Evaluate exact `@axe-core/playwright` through plan 101 before installing it.
Prove import/config/scan/report execution through Bun with no Node child or
lifecycle download. Keep the dependency absent if the matrix fails.

**Verify**: all required engine/dependency rows pass twice on macOS/Linux. Any
hang, crash, hidden variable, Node child, missing platform binary, or leak marks
this plan BLOCKED and preserves a minimal reproduction.

### Step 1: Add explicit projects and matrix selection

Add the project table above with fixed locale/timezone/color/reduced-motion and
separate output directories. Use approved Playwright device descriptors, not
hand-copied partial settings. Add matrix fields for `engines`, `mobile`,
`accessibility`, and `visual`; validate that each value maps to an existing
project and every selected case is discovered.

Keep `contracts-chromium` as the required behavioral source. Cross/mobile/a11y/
visual projects select only declared cases and fail when their selection is
empty. Report one stable case ID across project repetitions.

**Verify**: project list and positive matrix fixtures pass; unknown engine,
viewport-only mobile claim, duplicate stable ID, empty selection, or project
without distinct scenario/lane owners fails policy.

### Step 2: Add runtime accessibility and keyboard/focus contracts

Create a typed accessibility fixture that runs axe only after the page reaches
the scenario's stable state. Scan shell/navigation and the investigations list,
detail, create/edit dialog, destructive confirmation, pin, and note states.
Test keyboard reachability/order, visible focus, focus trap and restoration,
Escape close, accessible names/descriptions, live-status behavior, and reduced-
motion behavior separately.

Exceptions use exact rule, locator/state, owner, reason, created date, expiry,
and removal condition. Missing, expired, broadened, or no-longer-observed
exceptions fail. Never disable a rule globally to handle one component.

**Verify**: known violation fixtures fail with stable diagnostics; exact active
exception passes; stale/expired/broadened exceptions fail; keyboard/focus tests
pass in Chromium and the selected cross engine where matrix risk requires it.

### Step 3: Add genuine mobile/touch behavior

Test shell navigation, responsive overflow, dialogs/drawers, tables/timelines,
tap target interaction, virtualized scrolling, orientation where supported,
and text containment using the two mobile projects. The pilot must prove touch
input and mobile user agent/device settings are active, not merely a small
viewport.

Assert user-visible behavior and absence of horizontal page overflow. Do not
encode exact element coordinates except in a focused geometry helper with an
explicit tolerance and risk reason.

**Verify**: device-settings assertions, shell/investigations scenarios, narrow/
wide text fixtures, and an intentional overflow/touch-negative fixture behave
as expected on both mobile projects.

### Step 4: Establish canonical visual comparison

Define a digest-pinned Linux runner/container with exact browser revision,
fonts, viewport/device scale, locale, timezone, theme, reduced motion, and
animation/clock stabilization. Capture only named stable states. Dynamic IDs,
times, cursors, and telemetry values must come from deterministic fixtures;
masking is a narrow reviewed exception, not the default.

Check in reviewed shell and investigations desktop/mobile/dark/light goldens
only where each variant catches a distinct layout risk. Store a manifest with
case ID, project, dimensions, theme, browser revision, environment digest,
threshold, and source commit. The update command refuses a dirty tree, wrong
environment, missing review reason, or mass deletion/addition outside scope.

**Verify**: clean comparison passes; intentional one-pixel, font, overflow,
missing-golden, stale-manifest, wrong-environment, and unauthorized-update
fixtures fail with expected/actual/diff artifacts.

### Step 5: Add distinct path-aware CI gates

Add cross/mobile, accessibility, and canonical visual jobs selected by UI/test/
style/config/lock/workflow inputs. Explicitly install locked engines. Reuse the
established Bun package cache but do not add a browser cache without measured
proof. Upload redacted traces/screenshots/videos/reports/diffs on failure.

Aggregate required cadence explicitly: accessibility and canonical visual are
required for selected changes; cross/mobile are required according to the
approved matrix/cadence and always run on main/schedule. A skipped irrelevant
job is distinguishable from a selected zero-test job. Baseline updates require
review and never occur in CI.

**Verify**: actionlint/path fixtures pass; intentional engine, axe, keyboard,
mobile, and pixel failures turn the appropriate job red; zero-selection fails;
normal irrelevant skips and selected success aggregate correctly.

## Test Plan

- macOS/Linux no-Node compatibility and cleanup for Firefox/WebKit/device/axe.
- Matrix/project discovery, non-empty selection, and stable-ID repeat tests.
- Axe rule, exact/stale/expired exception, keyboard, focus, Escape, name, and
  reduced-motion cases.
- Touch/device-settings, responsive overflow, table/timeline, dialog/drawer, and
  narrow/long-text mobile cases.
- Canonical environment, golden manifest, pixel diff, threshold, update guard,
  and artifact tests.
- CI path, install, skip, zero-test, failure, aggregate, and redaction cases.

## Done Criteria

- [ ] Firefox, WebKit, both mobile projects, and axe pass the exact-version Bun
  matrix on macOS/Linux without Node, hangs, or leaks.
- [ ] Projects and matrix encode distinct behavioral, mobile, accessibility,
  and visual evidence with no empty or falsely named coverage.
- [ ] Shell/investigations pilot states pass runtime axe, keyboard/focus, genuine
  mobile/touch, and selected cross-engine cases.
- [ ] Canonical goldens can be compared reproducibly and updated only through a
  guarded reviewed path in the pinned environment.
- [ ] Distinct CI jobs fail the owning evidence class and publish redacted
  diagnostic artifacts without silently updating baselines.
- [ ] Every command in this plan passes twice from clean state.

## STOP Conditions

Stop and report if:

- any engine or axe integration requires Node, hidden flags, implicit install,
  another runner, or leaks a process;
- a viewport-only configuration is the only way to claim mobile coverage;
- accessibility requires a global rule disable or visual stability requires
  broad masking;
- canonical output cannot be reproduced in the pinned environment;
- CI would write/approve goldens automatically or a retry would clear status;
- artifacts expose secrets or unredacted telemetry; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

Each feature plan declares its actual cross/mobile/accessibility/visual risks in
the matrix instead of multiplying all tests across all projects. Browser,
device, axe, font, or environment upgrades are one reviewed compatibility and
golden-regeneration unit.

Delete this plan and its README row only after compatibility, projects, pilot
breadth, exceptions, canonical visual protocol, CI lanes, and negative fixtures
are durable and green. Plans 143/150 materialize shell/overview rows, and plan
151 later proves every shipped surface has its required final matrix evidence.
