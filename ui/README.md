# Parallax UI

TanStack Start SPA (React + TypeScript + shadcn/ui on Base UI) for the Parallax
local-first observability console. The app talks **only** to the canonical
GraphQL API served by `parallax serve` (default `:4000`).

## Develop

**Bun only** (runtime + package manager). Never `npm`, `pnpm`, `yarn`, or
Node-as-runtime — see [../AGENTS.md](../AGENTS.md).

```bash
bun ci
bun run dev          # http://127.0.0.1:3000
```

Vite proxies `/graphql` to `http://127.0.0.1:4000`. Start the backend first:

```bash
cargo run -p parallax-cli -- serve
```

## Gates

```bash
bun run typecheck
bun run lint
bun run test
bun run test:ci      # CI-strict (fails if no tests match)
bun run build
```

## Adding shadcn components

```bash
bunx --bun --no-install shadcn add <component>
```

Full UI conventions: [root AGENTS.md](../AGENTS.md).
