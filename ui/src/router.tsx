import { createRouter as createTanStackRouter } from "@tanstack/react-router"

import {
  RouteErrorPanel,
  RouteNotFoundPanel,
  RoutePendingPanel,
} from "@/layout/route-boundaries"
import { routeTree } from "./routeTree.gen"

export function getRouter() {
  const router = createTanStackRouter({
    routeTree,
    scrollRestoration: true,
    defaultPreload: "intent",
    defaultPreloadStaleTime: 15_000,
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
