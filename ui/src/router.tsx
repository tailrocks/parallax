import { createRouter as createTanStackRouter } from "@tanstack/react-router"

import { RouteErrorPanel, RouteNotFoundPanel, RoutePendingPanel } from "@/layout/route-boundaries"
import { createAppQueryClient } from "@/platform/query/client"
import { installBrowserQueryClient } from "@/platform/query/graphql-query"
import { routeTree } from "./routeTree.gen"
import type { AppRouterContext } from "@/router-context"

export type { AppRouterContext } from "@/router-context"

export function getRouter() {
  const queryClient = createAppQueryClient()
  // SPA browser ownership: one stable client for this router lifetime.
  if (typeof window !== "undefined") {
    installBrowserQueryClient(queryClient)
  }

  const router = createTanStackRouter({
    routeTree,
    context: { queryClient } satisfies AppRouterContext,
    scrollRestoration: true,
    defaultPreload: "intent",
    // Query owns loader freshness (plan 133).
    defaultPreloadStaleTime: 0,
    defaultErrorComponent: RouteErrorPanel,
    defaultPendingComponent: RoutePendingPanel,
    defaultNotFoundComponent: RouteNotFoundPanel,
  })

  return router
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>
  }
}
