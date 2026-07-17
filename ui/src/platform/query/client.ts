// Plan 133 — QueryClient factory. One fresh client per server/router request;
// one stable browser client owned by the app router composition (not a free
// module singleton used outside router/provider ownership).

import { QueryClient } from "@tanstack/react-query"

export function createAppQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 15_000,
        gcTime: 5 * 60_000,
        retry: false,
        refetchOnWindowFocus: false,
      },
      mutations: {
        retry: false,
      },
    },
  })
}
