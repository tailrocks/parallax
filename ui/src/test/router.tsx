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
    componentPaths?: readonly string[]
    initialPath?: string
    layout?: boolean
    targetPaths?: readonly string[]
  }> = {}
) {
  const rootRoute = createRootRoute({
    component: options.layout
      ? () => (
          <>
            {component}
            <Outlet />
          </>
        )
      : Outlet,
  })
  const componentRoutes = (options.componentPaths ?? ["/"]).map((path) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path,
      component: options.layout ? () => null : () => component,
    })
  )
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
    routeTree: rootRoute.addChildren([...componentRoutes, ...targetRoutes]),
    history,
  })
  return { ...render(<RouterProvider router={router} />), history, router }
}
