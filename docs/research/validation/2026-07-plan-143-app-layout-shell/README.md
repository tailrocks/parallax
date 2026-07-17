# Plan 143 — App / layout / shell migration

**Recorded:** 2026-07-17

Move router composition, product shell, command palette, theme, route
boundaries, app-status health projection, and quick-navigation ID guessing
behind explicit `app`, `layout`, `features/app-status`, and
`features/quick-navigation` owners. Root route imports the layout facade.
Dashboard sub-nav consumes `loadDashboardNavigation` from `@/features/dashboards`.

## Layout

```text
ui/src/app/create-router.tsx
ui/src/router.tsx                      # TanStack Start adapter
ui/src/layout/
  index.ts
  app-shell.tsx
  command-palette.tsx
  nav-icon.tsx
  route-boundaries.tsx
  theme-switcher.tsx
  tests/**
ui/src/shared/navigation.ts            # nav registry (shared for feature crumbs)
ui/src/features/app-status/**
ui/src/features/quick-navigation/**
ui/src/routes/__root.tsx               # thin root Route + document
```

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/layout src/features/app-status src/features/quick-navigation
```

Browser full-stack/breadth gates close with plans 145/146.
