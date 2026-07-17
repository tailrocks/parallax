# Plan 144 — Playwright product contracts and required CI gate

**Date:** 2026-07-17  
**Status:** complete — local Chromium contracts 7/7 green; required CI job
`browser-contracts` added to `ci-required`  
**Depends on:** 094, 101, 128, 129, 132 (all complete)

## What landed

1. **Typed seed/reset facade** in `parallax-test-support::browser`
   (`DatasetId`, scenario manifests, `reset_and_seed`, investigation snapshots,
   postcondition checks). `MemoryStore::clear()` supports isolation.
2. **Composition seam harness** —
   `parallax-server` example `browser_contracts_serve` injects `MemoryStore` +
   Turso at `start_with_capabilities`, serves the built UI through a loopback
   proxy (optional one-shot GraphQL failure injection), and exposes a private
   control plane for reset/snapshot. Never a product storage mode.
3. **Xtask orchestration** — `cargo xtask browser-contracts-serve`.
4. **Product Playwright fixtures** — `productTest` resets dataset before the
   page opens; diagnostics fail unexpected console/network/dialog events.
5. **Shell + investigations pilot contracts** under
   `ui/tests/e2e/contracts/` with semantic locators and matrix IDs
   `@pw-shell-*` / `@pw-investigations-*`.
6. **Matrix inventory** — every shipped feature surface owns a
   `playwright/contracts` row (implemented or reserved with `delivery_plan`).
7. **Policy** — `cargo xtask policy --only ui.browser-contracts`.
8. **Required CI job** — `browser-contracts` on UI/Rust path changes; member of
   `ci-required` (skipped when paths irrelevant → aggregate success).

## Commands (verification)

```bash
cd ui && bun ci
cd ui && bunx --bun --no-install playwright install --with-deps chromium
cd ui && bun run build
cargo build --locked -p parallax-server --example browser_contracts_serve
cargo xtask policy --only ui.browser-contracts
cargo xtask policy --only ui.browser-foundation
cargo xtask policy --only ui.tests
cd ui && bun run test:browser:list
cd ui && bun run test:browser
cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build
mise exec -- actionlint .github/workflows/*.yml
```

## Fixed decisions preserved

- Bun-only `@playwright/test` 1.61.1; no Node runner, no direct
  `playwright`/`playwright-core` packages.
- In-memory adapter is test-harness composition only.
- No happy-path `page.route()` substitution.
- Chromium contracts lane is the required gate; cross-browser is plan 146;
  real Greptime/Turso stack is plan 145.
