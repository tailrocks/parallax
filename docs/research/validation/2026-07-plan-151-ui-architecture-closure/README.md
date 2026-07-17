# Plan 151 — UI architecture closure (partial, 2026-07-17)

## Done this slice

- Mechanical residual move: `components/console/**` → `shared/console/**`,
  `hooks/**` → `shared/hooks/**` (imports, ownership, matrix updated).
- Architecture / tests / ratchets policies green after the move.
- `ui/AGENTS.md` final placement table recorded for the live tree.

## Still residual (blocks full retirement)

- Handwritten `ui/src/lib/*` helpers beyond `utils.ts` (format, api TTL cache,
  where-clause, metrics/alerts helpers, etc.) need feature/platform/domain
  owners before zero-generic-bucket criteria hold.
- Browser full-stack (145) and breadth (146) still open — delegated feature
  `@*` rows remain reserved.
- Plans 133 / 147 / 148 remain blocked on 151 full closure + 145/146.

## Verify

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/shared/console/tests src/shared/hooks/tests
```
