# Plan 128 TypeScript declaration blocker — refresh 2026-07-17

Validation date: 2026-07-17  
Baseline: Wave 2 closed on `main` (plans 162–168 retired).

## Outcome

Re-validation under the operator unblock directive (2026-07-17) still finds
**no latest mutually compatible stable set** that makes `skipLibCheck=false`
clean under TypeScript 7 with `exactOptionalPropertyTypes`.

Strictest passing product lane remains:

```sh
cd ui && bun run typecheck   # skipLibCheck true, strict flags, exit 0
cd ui && bun run lint        # native + type-aware Oxlint, exit 0
```

## Command

```sh
cd ui && bun ./node_modules/typescript/bin/tsc --noEmit --skipLibCheck false --pretty false
```

## Third-party owners (current failures)

| Owner | Failure class |
| --- | --- |
| `@reduxjs/toolkit` (via recharts) | AsyncThunkConfig / TaskAbortError.code under exact optional properties |
| `@tabler/icons-react` | imports nonexistent `ReactSVG` from `@types/react` |
| `@tanstack/router-core` | SSR `MakeRouteMatch['__beforeLoadContext']` |
| `@tanstack/router-plugin` / `router-utils` / `start-plugin-core` | missing `@types/babel__*` |
| `@tanstack/devtools-event-bus` | missing `@types/ws` |
| `@xyflow/react` / `@xyflow/system` | exactOptionalPropertyTypes node position mismatches |
| `unplugin` (indirect) | optional adapter declaration surface |
| `elkjs` | declaration friction under skipLibCheck=false |

Application source under `skipLibCheck=true` remains clean. Plan 128
rescope: document these as shrink-only third-party declaration exceptions;
do not invent ambient lies or cast-away product types to force green.

## Product decision (executor, 2026-07-17)

Adopt **strictest passing configuration**: keep `skipLibCheck: true` as the
required CI typecheck until upstream packages publish declaration-compatible
releases. Each owner above is a shrink-only exception (remove only when the
package ships a clean graph — never broaden the exception set).
