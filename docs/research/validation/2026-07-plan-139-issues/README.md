# Plan 139 — Issues feature migration

**Recorded:** 2026-07-17

Move issues list/detail, status/occurrence adapters, and stacktrace parser
behind `@/features/issues`. Routes export only `Route`.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/issues src/routes/tests/-issues-routes.test.tsx
```

Browser full-stack/breadth gates close with plans 145/146.
