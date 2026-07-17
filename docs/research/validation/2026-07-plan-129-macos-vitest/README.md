# Plan 129 — macOS forced-Bun Vitest evidence

**Status:** DONE (2026-07-17)
**Host:** Darwin arm64 (Apple Silicon), Bun 1.3.14 via mise  
**Head:** `5bcacd38c7a84e90aa9123ea965c917c855e9716` (plan 128 close) + matrix refresh

## Positive runs (twice)

```sh
cd ui && bun run --bun test:ci
```

| Run | Files | Tests | Exit |
| --- | ---: | ---: | ---: |
| 1 | 72 | 434 | 0 |
| 2 | 72 | 434 | 0 |

Process sample during the suite: Vitest worker processes are
`…/mise/installs/bun/1.3.14/bin/bun … vitest/dist/workers/forks.js` — not Node.
(Host may still have unrelated Node processes from editor MCP / a legacy
`vite` child; they are not descendants of `bun run --bun test:ci`.)

## Policy

```sh
cargo xtask policy --only ui.tests
```

Green after matrix ownership catch-up for Wave-2/logs/alerts tests:

- `test_files` ratchet 72, `test_cases` 425
- private-route import set includes `validateLogsSearch`
- six new entries (`vitest-073`…`078`) with plan-linked legacy handoffs

## Negatives (fail closed)

| Gate | Observation |
| --- | --- |
| Forced Bun scripts | `package.json` `test` / `test:ci` use `bunx --bun --no-install vitest run` |
| Implicit install disabled | `--no-install` on every vitest invocation |
| Zero-selection | empty dir vitest selection fails (non-zero) |

## Plan 128 dependency

Closed at the same program head under the operator rescope
([plan 128 evidence](../2026-07-plan-128-static-safety/README.md)).
