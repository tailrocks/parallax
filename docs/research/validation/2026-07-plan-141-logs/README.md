# Plan 141 — Logs feature migration

**Recorded:** 2026-07-17

Move logs page, reusable log table, search/window helpers, and loader behind
`@/features/logs`. Routes export only `Route`. Invocations consume LogsTable
through the facade.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/logs src/routes/tests/-logs-routes.test.tsx
```

Browser full-stack/breadth gates close with plans 145/146. Plan-152 typed ops
for dynamic log queries can tighten in a follow-up.
