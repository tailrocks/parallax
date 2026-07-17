# Plan 140 — Invocations (runs) feature migration

**Recorded:** 2026-07-17

Move invocations list/hub, console invocation tabs/table/journey helpers behind
`@/features/invocations` (post-157 surface for plan 140 runs). Routes export
only `Route`. Logs table consumed via `@/features/logs` facade.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/invocations src/routes/tests/-invocations-routes.test.tsx
```

Browser full-stack/breadth gates close with plans 145/146.
