# Plan 132 — Bun-only Playwright foundation

**Status:** DONE (2026-07-17)  
**Host (primary):** Darwin arm64, Bun 1.3.14 via mise  
**Linux matrix:** Docker `oven/bun:1.3.14` (linux/arm64)  
**Playwright:** `@playwright/test@1.61.1` (lock-aligned `playwright` / `playwright-core` 1.61.1)  
**Chromium:** Playwright revision 1228 (Chrome for Testing 149.0.7827.55)

## Compatibility matrix (Step 0)

Isolated fixture: `/tmp/parallax-pw-bun-matrix` (`type: "module"`, `trustedDependencies: []`,
`bun install --ignore-scripts`, explicit `playwright install chromium`).

| Row | macOS | Linux (Docker) | Notes |
| --- | --- | --- | --- |
| TypeScript config/spec load | pass | pass | no hidden compatibility flags |
| `--list` | pass (5 tests) | pass | stable inventory |
| one worker pass | exit 0 | exit 0 | |
| multi worker pass | exit 0 | exit 0 | 2 workers / 2 tests |
| intentional failure | exit 1 + screenshot/video | not re-run | expected |
| retry → flaky pass | exit 0, “1 flaky” | not re-run | file-backed attempt counter |
| timeout | exit 1 | not re-run | expected |
| zero-test selection | exit 1, `No tests found` | not re-run | fail-closed |
| reporters line/html/blob/junit | present | present | junit.xml + blob zip |
| webServer readiness/teardown | pass | pass | Bun.serve health + Playwright stop |
| two clean runs / no port leak | port 4173 free | free | no leftover playwright workers |
| real Node executable | **absent** | **absent** | see ancestry note |

### Process ancestry note

`bunx --bun --no-install playwright` may show `comm=node` / argv0 `node` because of
Playwright’s `#!/usr/bin/env node` shebang, but `lsof` text mapping of that PID is
**`…/mise/installs/bun/1.3.14/bin/bun`** (Bun binary), not mise Node. Workers are
`bun …/playwright/lib/worker/workerProcessEntry.js`. Unrelated host Node processes
(e.g. chrome-devtools MCP) are not descendants of the test tree.

Linux Docker: pass suite exit 0; `ps` unavailable in image so ancestry sampled via
absence of `/bin/node` playwright processes in the run log.

## Repository foundation (Steps 1–4)

| Command | Result |
| --- | --- |
| `cd ui && bun ci` (frozen, ignore-scripts) | exact lock; no lifecycle browser download |
| `bunx --bun --no-install playwright install chromium` | package-matched browser |
| `bun run test:browser:list` | 1 foundation test, stable `@pw-foundation-shell` |
| `bun run test:browser:foundation` (run 1) | 1 passed (~3.3s) |
| `bun run test:browser:foundation` (run 2) | 1 passed (~1.5s) |
| `bun run typecheck && bun run lint && bun run check` | exit 0 |
| `cargo xtask policy --only ui.browser-foundation` | exit 0 |
| `cargo clippy -p parallax-xtask --locked -- -D warnings` | exit 0 |

### Delivered tree

```text
ui/playwright.config.ts
ui/tests/e2e/
  fixtures/{test,diagnostics,dataset}.ts
  screens/shell-screen.ts
  smoke/foundation.spec.ts
  support/manifests.ts
crates/parallax-xtask/src/browser_foundation.rs   # cargo xtask browser-foundation-serve
crates/parallax-xtask/src/policy/browser_foundation.rs
```

### Runtime contract

- Sole direct browser package: `@playwright/test@1.61.1`
- Scripts force Bun + no-install: `bunx --bun --no-install playwright …`
- `webServer` → `cargo xtask browser-foundation-serve` (static `ui/dist/client`,
  `/health`, GraphQL empty-product stub, runtime manifest under
  `ui/test-results/foundation-runtime.json`; `reuseExistingServer: false`)
- Semantic locators only; diagnostics fixture fails external network / console /
  pageerror / dialog / download
- Vitest remains unit/component/route runner; Playwright component testing not adopted

## Done criteria

- [x] Exact stable Playwright runs through Bun on macOS + Linux without real Node
- [x] `@playwright/test` only direct browser package
- [x] Scripts/config strict, lock-local, policy tested
- [x] Typed fixtures isolate context and fail unexpected diagnostics
- [x] xtask owns process lifecycle; foundation smoke green twice
- [x] typecheck/lint/format/list/foundation/policy pass
