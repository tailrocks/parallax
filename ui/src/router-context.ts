// Router context contract, cycle-free: the root route and the router factory
// both import from here (router.tsx pulls routeTree.gen, which pulls the
// root route — a direct type import from @/router would be a cycle).

import type { QueryClient } from "@tanstack/react-query"

export interface AppRouterContext {
  queryClient: QueryClient
}
