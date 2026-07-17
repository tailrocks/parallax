# Plan 145 — managed Greptime + Turso browser stack (host probe, 2026-07-17)

## Host probe (this machine)

| Check | Result |
| --- | --- |
| Fixed Greptime ports 24000–24003 | **occupied** by existing `greptime` PID (product/dev stack) |
| Parallel second managed supervisor | blocked until ports free (plan fixed-port rule; one worker per host) |
| Plan 132/144 Bun Playwright Chromium | available (prior evidence) |
| `test:browser:full` project | not yet wired (blocked on free ports + lifecycle serve) |

## What landed earlier (deps)

- Plan 132 foundation + plan 144 contracts-chromium + `browser_contracts_serve` (memory+Turso seam)
- Greptime supervisor + `ensure_binary` in `parallax-server`

## Next green slice (when ports free)

1. `cargo xtask browser-full-stack-serve` — unique temp data dir, managed Greptime+Turso, runtime manifest
2. `ui/tests/e2e/full-stack/{telemetry-discovery,storage-composition,live-transport}.spec.ts`
3. `bun run test:browser:full` one worker
4. Path-aware CI job + matrix `@storage` rows

## STOP condition hit

Safe parallel / concurrent managed engines require free 24000–24003; host currently
runs Greptime there. Do not kill foreign PID by port alone. Operator must stop the
dev Greptime (or dedicate a CI runner) before full-stack browser lane can prove green.
