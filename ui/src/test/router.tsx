import { render } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import type { ReactNode } from "react"

export function renderTestRouter(
  component: ReactNode,
  options: Readonly<{
    initialPath?: string
    targetPaths?: readonly string[]
  }> = {}
) {
  const rootRoute = createRootRoute({ component: Outlet })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const targetRoutes = (options.targetPaths ?? []).map((path) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path,
      component: () => null,
    })
  )
  const history = createMemoryHistory({
    initialEntries: [options.initialPath ?? "/"],
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, ...targetRoutes]),
    history,
  })
  return { ...render(<RouterProvider router={router} />), history, router }
}
