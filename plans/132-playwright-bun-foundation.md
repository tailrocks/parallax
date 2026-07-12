# Plan 132: Establish a Bun-only Playwright test foundation

> **Executor instructions**: Make Playwright Test the only browser E2E runner,
> but never violate Bun-only runtime policy. Run the exact compatibility matrix
> first. If Playwright spawns Node, hangs, leaks workers, requires a hidden
> compatibility flag, or fails supported macOS/Linux, mark this plan BLOCKED
> with a minimal reproduction. Do not fall back to Node and do not build a
> second browser driver.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- AGENTS.md ui/package.json ui/bun.lock ui/bunfig.toml ui/tsconfig.json ui/playwright.config.ts crates/parallax-xtask ratchet.toml`

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 094, 101, 128, 129
- **Category**: TypeScript / Playwright / test infrastructure
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: TODO

## Why This Matters

Parallax has no real-browser runner. Playwright provides browser contexts,
semantic locators, web-first assertions, projects, fixtures, reporters, traces,
screenshots, and videos needed by the later browser plans. Its current stable
source contains an explicit Bun loader path, but Bun is not a formally
documented Playwright runtime and upstream compatibility history includes hangs
and worker leaks. A hello-world on one machine is insufficient for repository
adoption.

This plan proves the exact runtime boundary and builds only the reusable
foundation. Plan 144 adds fixture-backed product flows and required CI; plan 145
adds GreptimeDB + Turso full-stack coverage; plan 146 adds cross-browser/mobile/
accessibility/visual breadth.

## Fixed Decisions

1. Exact lock-local `@playwright/test` is the sole browser framework.
2. Direct `playwright`, direct `playwright-core`, Node runtime, npm/pnpm/yarn,
   and a custom Rust CDP test framework are forbidden.
3. Bun is forced with `bunx --bun --no-install`; package auto-install and
   dependency lifecycle browser downloads are disabled.
4. Browser installation is a separate explicit command using the locked runner.
5. Rust xtask may start/seed/stop Parallax processes but owns no browser locator,
   assertion, context, project, reporter, trace, or screenshot logic.
6. Vitest + Testing Library remains the unit/component/route runner. Playwright
   component testing is not adopted.
7. Playwright-specific static invariants use stable config plus plan 095's
   Rust/Oxc AST policy. Do not add ESLint, `eslint-plugin-playwright`, or alpha
   Oxlint JavaScript plugins.

## Target Foundation

```text
ui/
  playwright.config.ts
  tests/e2e/
    fixtures/
      test.ts
      diagnostics.ts
      dataset.ts
    screens/
      shell-screen.ts
    smoke/
      foundation.spec.ts
    support/
      manifests.ts
```

Only the shell/readiness smoke belongs here. Product route/flow tests belong to
plan 144.

## Required Scripts

```json
{
  "scripts": {
    "test:browser:list": "bunx --bun --no-install playwright test --list",
    "test:browser:foundation": "bunx --bun --no-install playwright test --project=foundation-chromium"
  }
}
```

The final project/script set grows in plans 144-146. Scripts remain lock-local,
no-install, and no-Node.

## Configuration Contract

`ui/playwright.config.ts` is strict TypeScript included in typecheck/lint/format.
It must define:

- one `testDir` under `ui/tests/e2e`;
- `forbidOnly` in CI and zero-test/list failure;
- `failOnFlakyTests` when a diagnostic retry is enabled;
- bounded global/action/navigation/expect/webServer timeouts;
- localhost-only `baseURL`;
- UTC, `en-US`, reduced-motion, fixed color-scheme defaults;
- failure-only screenshot/video and retry trace policy;
- line + HTML locally and blob/JUnit machine reports in CI-compatible mode;
- one foundation Chromium project; and
- a `webServer` command that delegates process lifecycle to xtask and never
  reuses an unknown local server.

Use typed Bun environment access. Add `@types/bun` only if plan 101 proves it is
needed and compatible. Node type declarations do not authorize Node execution.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Install | `cd ui && bun ci` | exact lock, reviewed/disabled lifecycle scripts |
| Browser install | `cd ui && bunx --bun --no-install playwright install --with-deps chromium` | package-matched browser installed; no package auto-install |
| Inventory | `cd ui && bun run test:browser:list` | non-zero stable-ID list |
| Foundation | `cd ui && bun run test:browser:foundation` | smoke passes, diagnostics clean |
| Type/lint/format | `cd ui && bun run typecheck && bun run lint && bun run check` | exit 0 |
| Policy | `cargo xtask policy --only ui.browser-foundation` | runtime/dependency/config/locator/artifact rules pass |

## Scope

In scope:

- Exact stable `@playwright/test`, matching transitive core/browser revision,
  optional `@types/bun`, and their lock/policy evidence.
- Bun-only scripts, base config, foundation fixtures, shell screen, smoke,
  reports, and process/artifact self-tests.
- Minimal xtask server/readiness/cleanup contract required for the smoke.

Out of scope:

- Full route/cross-feature contract suite and required CI (plan 144).
- GreptimeDB + Turso real-stack browser flows (plan 145).
- Firefox/WebKit/mobile/accessibility/visual projects (plan 146).
- Product feature/capability refactoring (plans 134-143, 149, and 150).
- `@axe-core/playwright` until plan 146 first uses it.

## Git Workflow

- Stay on the single active branch; do not create a branch or PR.
- Land compatibility evidence, locked dependency/config, fixtures, and smoke as
  separate green changes.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Prove exact Playwright-on-Bun compatibility

Resolve latest stable Bun and Playwright under plan 101. In an isolated fixture
with `type: "module"`, prove on macOS and Linux with Node absent:

- TypeScript config/spec/custom fixture load without hidden flags;
- `--list`, one worker, multiple workers, pass, failure, retry-pass/flaky,
  timeout, and zero-test behavior;
- Chromium explicit install and launch;
- webServer readiness/teardown on success, failure, timeout, and cancel;
- line/HTML/blob/JUnit reports and trace/screenshot/video artifacts;
- process ancestry contains Bun, allowed shell/browser/Cargo/Parallax processes,
  but no Node executable; and
- two clean runs leave no worker/browser/server zombie, temp data, or occupied
  port.

Do not require Playwright UI mode. Headed/debug/UI commands are optional only
after their own no-Node proof.

**Verify**: persist positive/negative matrix evidence. Any failed required row
sets this plan BLOCKED; no fallback is authorized.

### Step 1: Add the single locked dependency and scripts

Add exact `@playwright/test` through Bun. Prove no direct duplicate Playwright
package and exact runner/core revision alignment. Keep install scripts disabled;
browser provisioning remains the explicit command. Add only the two foundation
scripts and verify missing dependency/browser cannot trigger auto-install.

**Verify**: dependency/integrity/license/lifecycle/runtime policy and intentional
direct-package/version-mismatch negative fixtures pass.

### Step 2: Add strict configuration and policy fixtures

Create the configuration contract above. Add plan 095/Oxc rules for focused/
skipped tests, missing assertions, unknown stable IDs, CSS/XPath and arbitrary
sleep use, config/project drift, zero selection, and files outside the E2E tree.
Use stable config/runtime rules rather than an unsupported lint plugin.

**Verify**: typecheck/lint/format and every negative config/spec fixture pass.

### Step 3: Build typed base fixtures and diagnostics

Create a typed `test` export with:

- fresh BrowserContext/page per test;
- deterministic dataset ID input but no product-specific seed yet;
- automatic console warning/error, `pageerror`, request-failure/external-network,
  unhandled dialog/download, and server-process diagnostic capture;
- setup/cleanup in the same fixture; and
- failure attachments using unique `testInfo.outputPath` locations.

Screen objects use semantic role/name/label/text locators. A screen object exists
only with real reuse and cannot hide broad assertions, sleeps, CSS selectors, or
data setup.

**Verify**: self-tests intentionally trigger every diagnostic and prove isolated
cleanup across sequential and parallel cases.

### Step 4: Add minimal xtask lifecycle and foundation smoke

Extend xtask to start the already-built Parallax UI/server test harness on
declared ports, narrate startup, wait for `/health`, publish a sanitized runtime
manifest, and terminate all children on success/failure/cancel. It does not
drive the browser.

Add one smoke that opens the root shell, asserts a stable accessible heading/
navigation marker and URL, and proves zero unexpected diagnostics. Do not add
product route flows in this plan.

**Verify**: lifecycle/occupied-port/tamper/timeout/cleanup tests and the
foundation project pass twice from clean state on macOS/Linux.

## Test Plan

- Exact-version Bun/Playwright module/config/worker/browser/webServer/reporter/
  artifact/process/cleanup compatibility matrix.
- Node/direct-package/version/implicit-install/lifecycle negative fixtures.
- Strict config, stable-ID, no-focus/skip, no-CSS/XPath/sleep, and zero-test
  policy fixtures.
- Base fixture isolation and automatic diagnostic self-tests.
- xtask readiness/success/failure/cancel/occupied-port/cleanup cases.
- One accessible root-shell foundation smoke.

## Done Criteria

- [ ] Exact stable Playwright runs through Bun on macOS/Linux with no Node,
  hidden flag, hang, crash, zombie, or implicit install.
- [ ] `@playwright/test` is the only browser framework/direct package.
- [ ] Scripts/config are strict, lock-local, non-empty, and policy tested.
- [ ] Typed fixtures isolate contexts/state and fail unexpected diagnostics.
- [ ] xtask owns process lifecycle only and the foundation smoke is green.
- [ ] Typecheck, lint, format, inventory, foundation, and policy commands pass
  twice from clean state.

## STOP Conditions

Stop and report if:

- the exact stable matrix needs Node, a hidden compatibility variable, a foreign
  package manager, implicit install, or lifecycle browser download;
- config/workers/browser/webServer/reporters hang, crash, leak, or diverge on
  macOS/Linux;
- a second browser runner/direct package appears necessary;
- unexpected diagnostics require a broad allowlist; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

Upgrade Bun, Playwright, and browser revisions as one reviewed compatibility
unit and rerun the full matrix. New browser capabilities build on this one
fixture/config stack.

Delete this plan and README row only after the compatibility packet, dependency,
config, fixtures, lifecycle, smoke, and policies are durable and green. If
blocked, keep only the exact reproduction and upstream trigger.
