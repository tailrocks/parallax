# Plan 137 — Dashboards feature migration

**Recorded:** 2026-07-17

Move dashboard list/detail pages, widget layout model, Plan-152 GraphQL
CRUD, and existing widget-series adapters behind `@/features/dashboards`.
Routes export only `Route`.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 src/features/dashboards
```

Browser full-stack/breadth gates close with plans 145/146.
