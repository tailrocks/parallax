# Plan 134 — Investigations feature migration

**Recorded:** 2026-07-17

Move investigations list/detail, pin control, state model, and decoded GraphQL
adapters behind `@/features/investigations`. Routes export only `Route`.
Pin consumers (issues, traces, invocations) import the facade only.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2
```

Browser `@investigations` pilots under `ui/tests/e2e` remain; full-stack/breadth
gates close with plans 145/146.
