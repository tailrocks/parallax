# Plan 133 — TanStack Query sole cache (CLOSED 2026-07-17)

## Evidence

| Gate | Result |
| --- | --- |
| `@tanstack/react-query` | exact `5.101.2` |
| TTL maps in `transport.ts` / `client.ts` | deleted |
| Cache backend | `QueryClient.fetchQuery` only |
| Browser ownership | `getRouter()` creates client; `installBrowserQueryClient`; root `AppQueryProvider` |
| Preload stale | `defaultPreloadStaleTime: 0` |
| Investigations pilot | `features/investigations/queries/{keys,options}.ts` + `ensureQueryData` loaders + mutation invalidation |
| Unit tests | `504` passed |
| Browser contracts | `7` passed |

## Shape

```text
ui/src/platform/query/
  client.ts            # createAppQueryClient
  provider.tsx         # AppQueryProvider
  graphql-query.ts     # browser install + platform graphql keys
ui/src/features/investigations/queries/
  keys.ts
  options.ts
```

Legacy call sites still use `graphqlCached` / `executeCachedGraphqlOperation` names but those functions are pure Query adapters (no TTL/inflight maps). Feature-owned query modules expand as surfaces need typed invalidation beyond the platform graphql keys.
