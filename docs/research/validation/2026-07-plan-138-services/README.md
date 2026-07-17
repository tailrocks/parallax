# Plan 138 — Services feature migration

**Recorded:** 2026-07-17

Move services list/detail pages, search, catalog merge, RED transforms, and
decoded GraphQL adapters behind `@/features/services`. Routes export only
`Route`.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/services src/routes/tests/-services-routes.test.tsx
```

Browser full-stack/breadth gates close with plans 145/146.
