# Plan 150 — Overview feature migration

**Recorded:** 2026-07-17

Move overview KPIs, series, RED/latency bands, movers, and recent lists behind
`@/features/overview`. Route `/` exports only `Route`.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/overview src/routes/tests/-overview-routes.test.tsx
```

Browser full-stack/breadth gates close with plans 145/146.
