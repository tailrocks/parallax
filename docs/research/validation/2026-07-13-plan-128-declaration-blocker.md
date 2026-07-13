# Plan 128 TypeScript 7 declaration blocker

Validation date: 2026-07-13  
Candidate baseline: `b75cac9`

## Outcome

Plan 128 reached its mandatory STOP condition: no latest mutually compatible
stable dependency set currently produces a clean complete declaration graph
under TypeScript 7.0.2 with `skipLibCheck=false`. The application compiler,
native Oxlint, and type-aware Oxlint lanes remain green with `skipLibCheck=true`.
Parallax did not add an ambient declaration, package patch, suppression,
exclusion, `any`, assertion, or second compiler/linter.

## Reproduction

From `ui/`:

```sh
bun ./node_modules/typescript/bin/tsc --noEmit --skipLibCheck false --pretty false
```

After updating the mutually related direct TanStack, Vite, Vitest, Tailwind,
Base UI, shadcn, and virtualizer packages and installing published Babel/ws
declaration packages, the remaining owners are:

| Owner | Exact observed failure |
| --- | --- |
| `@reduxjs/toolkit@2.12.0`, through `recharts@3.9.2` | generic async-thunk configurations fail `AsyncThunkConfig`/`PreventCircular` constraints; `TaskAbortError.code` violates `SerializedError` under exact optional properties |
| `@tabler/icons-react@3.44.0` | its declaration imports nonexistent `ReactSVG` from current `@types/react@19.2.17` |
| `@tanstack/router-core@1.171.14`, exact dependency of current `@tanstack/react-router@1.170.17` | SSR declarations index `MakeRouteMatch['__beforeLoadContext']`, which TypeScript 7 rejects |
| `unplugin@3.0.0`, through the current TanStack router plugin | its public declaration eagerly imports optional Farm, Rspack, Bun, esbuild, Rollup, unloader, and webpack adapters that are not product dependencies |

Installing all optional unplugin adapters was tested in the lock graph and
rejected. It added 267 irrelevant packages, two blocked lifecycle scripts, and
new Farm/Bun/Vite declaration conflicts. Those packages were removed through
Bun. Pinning `@tabler/icons-react@3.43.0` and adding
`@reduxjs/toolkit@2.11.2` directly also failed: Tabler retained the invalid
`ReactSVG` import, while Recharts retained its own nested 2.12.0 Toolkit.

The TypeScript documentation confirms that `skipLibCheck` covers every `.d.ts`
file and can hide conflicts; it is not being mislabeled as a clean graph. The
upstream declarations are visible in the current
[Tabler repository](https://github.com/tabler/tabler-icons),
[Redux Toolkit repository](https://github.com/reduxjs/redux-toolkit),
[TanStack Router SSR source](https://github.com/TanStack/router/blob/main/packages/router-core/src/ssr/types.ts),
and [unplugin repository](https://github.com/unjs/unplugin).

## Unblock condition

Resume Plan 128 when latest stable releases of the owning packages form a clean
TypeScript 7 declaration graph without installing unused optional adapters, or
when the operator authorizes a different dependency replacement plan. Re-run
the exact command above first. The plan may not resume by weakening strict or
exact-optional settings, keeping `skipLibCheck=true` as a claimed completion,
patching declarations locally, or replacing errors with suppressions/casts.

No research prompt changed: this is execution evidence for an existing static
safety STOP condition, not a change in research direction or product intent.
