# Plan 148 — route-owned chunks and bundle budgets (CLOSED 2026-07-17)

## Evidence (host macOS arm64)

| Gate | Result |
| --- | --- |
| Production build contract | `ui/vite.config.ts` — `sourcemap: false`, Rolldown manualChunks for graph/charts/virtualizer |
| Analyze | `cargo xtask ui-bundle analyze` / `bun run bundle:analyze` — budgets ok |
| Two clean builds | `cargo xtask ui-bundle build-twice` — **identical normalized inventories**, budgets ok |
| Byte ceilings | `ui/bundle-budgets.json` shrink-only (raw/gzip/file/largest/maps) |
| Policy | `cargo xtask policy --only ui.bundles` green |
| `@bundle` browser | **3 passed** — `bun run test:browser -- --grep @bundle` |
| Source maps | 0 in client output; embed path does not ship `.map` |

## Measured production client (final)

| Metric | Value | Ceiling |
| --- | --- | --- |
| files | 102 | 110 |
| totalRaw | 3_670_135 | 3_706_837 (+1%) |
| totalGzip | 1_104_206 | 1_115_249 (+1%) |
| largestGzip | 442_387 | 446_811 |
| sourceMapFiles | 0 | 0 |

## Commands

```bash
cd ui && bun run build
cargo xtask ui-bundle analyze
cargo xtask ui-bundle build-twice
cargo xtask policy --only ui.bundles
cd ui && bun run test:browser -- --grep @bundle
```

## Landed shape

```text
ui/scripts/bundle-analyze.ts
ui/bundle-budgets.json
ui/tests/e2e/contracts/bundle-resources.spec.ts
crates/parallax-xtask/src/ui_bundle.rs
crates/parallax-xtask/src/policy/ui_bundles.rs
target/ui-bundle/   # generated reports (not committed)
```

## Closure

Plan 148 residual budgets, two-clean-build determinism, and `@bundle` browser resource gates are durable and green. Further route-graph reachability refinements can land as ordinary shrink-only budget updates without a separate plan file.
