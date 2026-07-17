# Plan 128 — TypeScript static safety (strictest passing)

**Status:** DONE (2026-07-17)
**Rescope:** operator unblock 2026-07-17 — when full `skipLibCheck=false`
cannot go green on a latest mutually compatible stable set, adopt the
strictest **passing** product configuration with a **shrink-only** third-party
declaration exception ledger.

## Product contract (required CI)

| Gate | Command | Result |
| --- | --- | --- |
| TypeScript 7 app | `cd ui && bun run typecheck` | exit 0 (`skipLibCheck: true`) |
| Native Oxlint | `cd ui && bun run lint:native` | 0 warnings / 0 errors |
| Type-aware Oxlint | `cd ui && bun run lint:type-aware` | 0 warnings / 0 errors |
| Policy | `cargo xtask policy --only typescript` | pass |
| Tests | `cd ui && bun run test:ci` | 434 passed |

### Compiler options (effective, checked)

`strict`, `noPropertyAccessFromIndexSignature`, `noUncheckedIndexedAccess`,
`exactOptionalPropertyTypes`, `isolatedModules`, `moduleDetection: force`,
`erasableSyntaxOnly`, `verbatimModuleSyntax`, `noImplicitOverride`,
`noImplicitReturns`, unused locals/params, fallthrough, unreachable, unused
labels, unchecked side-effect imports, `allowJs: false`.

## Shrink-only declaration exception ledger

Full-graph probe (`tsc --noEmit --skipLibCheck false`) still fails only in
third-party packages. **Never broaden** this list; remove a row only when
upstream ships a clean declaration graph.

| Owner package | Failure class | Exception |
| --- | --- | --- |
| `@reduxjs/toolkit` (via recharts) | AsyncThunkConfig / TaskAbortError under exact optional | `skipLibCheck` covers |
| `@tabler/icons-react` | imports missing `ReactSVG` from `@types/react` | `skipLibCheck` covers |
| `@tanstack/router-core` | SSR `MakeRouteMatch['__beforeLoadContext']` | `skipLibCheck` covers |
| `@tanstack/router-plugin` / `router-utils` / `start-plugin-core` | missing `@types/babel__*` | `skipLibCheck` covers |
| `@tanstack/devtools-event-bus` | missing `@types/ws` | `skipLibCheck` covers |
| `@xyflow/react` / `@xyflow/system` | exactOptional node position mismatches | `skipLibCheck` covers |

No ambient declaration patches, no application `any` casts to silence
libraries, no handwritten file exclusions from typecheck.

## Escape hatches

| Class | Policy |
| --- | --- |
| Handwritten production `any` | forbidden (none) |
| Production `@ts-ignore` | forbidden (none) |
| Production non-null `!` | removed in this closeout (narrowing / optional props) |
| Generated `as any` | owned by TanStack route tree (`src/routeTree.gen.ts`) only |
| Test-only `!` | allowed in `**/__tests__/**` fixtures |
| External-value `as` casts | no-growth handoff to plans 152 (GraphQL) and 153 (SSE/search/storage/env) |

## Negative

Do not set `skipLibCheck: false` in CI until every ledger row is cleared by
upstream upgrades. A PR that adds a new third-party owner or ambient lie fails
this contract.
