# Plan 146 — cross-browser / a11y / visual (host probe, 2026-07-17)

## Depends on

Plan 145 shared Playwright config writer — still open. Plan 132/144 Chromium contracts green.

## Host probe notes

| Engine | Status |
| --- | --- |
| Chromium (locked Playwright) | installed for contracts |
| Firefox / WebKit | not yet installed via plan 146 explicit Bun install |
| `@axe-core/playwright` | not yet dependency-gated (plan 101 + 146) |
| Canonical visual Linux image | not yet defined |

## Next green slice after 145 config ownership

1. Bun no-Node matrix for firefox/webkit launch
2. Projects: cross-firefox, cross-webkit, mobile-*, accessibility-chromium, visual-chromium-linux
3. Shell + investigations pilot cases
4. Policy `ui.browser-breadth`

Do not claim mobile via viewport-only resize.
