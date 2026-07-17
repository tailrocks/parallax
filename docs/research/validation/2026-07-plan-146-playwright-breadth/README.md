# Plan 146 — cross-browser / a11y / visual (CLOSED 2026-07-17)

## Evidence (host macOS arm64 + CI wiring)

| Engine / lane | Result |
| --- | --- |
| Chromium contracts | green (plan 144) |
| Firefox + WebKit cross | shell + investigations pilots via `cross-firefox` / `cross-webkit` |
| Mobile Chromium (Pixel 7) + WebKit (iPhone 14) | device `isMobile`/`hasTouch` + overflow checks |
| Accessibility | `@axe-core/playwright@4.12.1` exact pin; shell + investigations axe + keyboard |
| Visual | platform-neutral goldens; `maxDiffPixels: 120` AA budget until digest-pinned Linux owns baselines |
| `bun run test:browser:cross` | **18 passed** |
| `bun run test:browser:a11y` | **3 passed** |
| `bun run test:browser:visual` | **2 passed** |
| Policy | `cargo xtask policy --only ui.browser-breadth` green |
| CI | `browser-breadth` job installs chromium/firefox/webkit; runs cross + a11y + visual; aggregated in `ci-required` |

## Commands

```bash
cd ui
bunx --bun --no-install playwright install --with-deps chromium firefox webkit
bun run test:browser:cross   # 18 passed
bun run test:browser:a11y    # 3 passed
bun run test:browser:visual  # 2 passed
cargo xtask policy --only ui.browser-breadth
```

## Landed

- Playwright projects: `cross-firefox`, `cross-webkit`, `mobile-chromium`, `mobile-webkit`, `accessibility-chromium`, `visual-chromium-linux`
- Package scripts: `test:browser:cross|a11y|visual|visual:update`
- Axe fixture + empty exception registry; visual manifest
- Shell + investigations pilots for each evidence class
- Matrix `playwright/breadth` rows
- Path-aware CI lane

## Notes

- Mobile sidebar hides desktop brand text; pilots use home/heading landmarks.
- Canonical visual authorship remains Linux CI; current goldens are host-captured with AA budget.
