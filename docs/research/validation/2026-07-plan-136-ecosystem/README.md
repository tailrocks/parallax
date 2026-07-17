# Plan 136 — Ecosystem feature migration

**Recorded:** 2026-07-17

Move service-map GraphQL, topology/layout/url model, React Flow graph, and page
controls behind `@/features/ecosystem`. Route exports only `Route`. Worker layout
imports the feature layout model.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 src/features/ecosystem
```

Browser `@ecosystem` full-stack/breadth gates close with plans 145/146.
