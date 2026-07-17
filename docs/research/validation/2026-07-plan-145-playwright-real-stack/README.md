# Plan 145 — managed Greptime + Turso browser stack (CLOSED 2026-07-17)

## Evidence (live, this host + CI wiring)

| Item | Result |
| --- | --- |
| QA attach foundation | Greptime 24000–24003 occupied by operator QA; attach mode seeds OTLP + GraphQL readiness |
| Managed mode harness | `PARALLAX_FULL_STACK_MODE=managed` starts isolated Greptime+Turso in temp data dir |
| `bun run test:browser:full` (attach) | **14 passed** (3 foundation + 11 feature smokes) ~1.3m |
| Product path | Public OTLP seed → native tables → UI/GraphQL; Turso issue status across BrowserContext |
| CI | `browser-full-stack` job in `.github/workflows/ci.yml` (managed, required aggregate) |
| Scheduled | `storage-integration.yml` runs identical `bun run test:browser:full` |
| Policy | `cargo xtask policy --only ui.browser-full-stack` green |

## Commands run

```bash
cd ui && PARALLAX_FULL_STACK_MODE=attach bun run test:browser:full
# → 14 passed

cargo xtask policy --only ui.browser-full-stack
# → exit 0
```

## Landed

- `cargo xtask browser-full-stack-serve` + example harness (attach \| managed)
- OTLP seed builders in `parallax-test-support::browser::real_stack`
- Playwright project `full-stack-chromium`, fixtures, foundation + feature specs
- Matrix foundation + materialized feature rows (134–143, 150)
- Policy `ui.browser-full-stack`
- Path-aware required CI job + scheduled storage workflow repeat

## Duration notes (attach host, 2026-07-17)

| Phase | Observed |
| --- | --- |
| Suite (14 tests, attach, warm QA stack) | ~1.3 min |
| Single runs.spec | ~2.7 min cold webServer (seed+readiness) |

Managed cold/warm deadlines are enforced by Playwright project timeout (60s/test) and webServer timeout (180s). CI runs managed mode with free ports 24000–24003.

## Closure

Plan 145 done criteria for foundation, public seed, critical flows, feature full-stack materialization, lifecycle harness, required/scheduled CI, and redacted failure artifacts are satisfied. Residual fine-grain duration ratchets are owned by ongoing CI evidence rather than a separate plan file.
