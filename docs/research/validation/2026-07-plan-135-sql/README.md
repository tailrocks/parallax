# Plan 135 — SQL feature migration

**Recorded:** 2026-07-17

Move the SQL workspace (editor, schema browser, results, history, snippets)
behind `@/features/sql`. Route exports only `Route`. History uses the platform
storage adapter; elapsed time uses the platform monotonic clock. GraphQL
operations are Plan-152 named documents with Zod result schemas.

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 src/features/sql src/routes/tests
```

Browser `@sql` full-stack/breadth gates close with plans 145/146.
Reserved matrix row `pw-reserved-sql-contracts` remains until those land.
