# Plan 151 — UI architecture closure (lib residual claimed, 2026-07-17)

## Done this slice

- Claimed handwritten `ui/src/lib/*` (except shadcn `utils.ts`) into owners:
  - `shared/`: `format`, `colors`, `color-by`, `where-clause` (+ tests)
  - `domain/range.ts`: short reexport of `domain/time-range/range`
  - `features/logs`: log brush/pattern/prefs + `model/wire` + `api/gql`
  - `features/traces`: timeline models + `model/wire` + timeline interactions hook
  - `features/invocations`: invocation model/facets + `model/wire` + `api/gql`
  - `features/runtime-metrics`: metric aggregation
  - `features/overview`: chart helpers
  - `features/alerts`: new feature facade for rule form / incident timeline / gql
  - `features/issues` / `services`: thin wire types
- Deleted `lib/api.ts`, `lib/range.ts`, `lib/use-visible.ts` reexports
- GraphQL transport consumers use `@/platform/graphql/transport` (routes/layout
  hold temporary layer exceptions)
- Matrix handoffs cleared for moved unit tests under final `tests/` topology
- `printWidth` 80 → 100 (path renames no longer force multi-line import growth)

## Residual (blocks full 151 retirement)

- Browser full-stack (145) and breadth (146) still open
- Fat metrics/alerts routes still import platform GraphQL via layer exceptions
- Plans 133 / 147 / 148 remain blocked on 151 full closure + 145/146

## Verify

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.ratchets
cargo xtask policy --only ui.tests
cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci
```
