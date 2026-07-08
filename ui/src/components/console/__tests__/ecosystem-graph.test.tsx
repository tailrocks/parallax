/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { describe, expect, it } from "vitest"

import { EcosystemGraph } from "@/components/console/ecosystem-graph"
import type { ServiceMapEdge, ServiceMapNode } from "@/lib/api"
import { customRange } from "@/lib/range"

function renderWithRouter(component: React.ReactNode) {
  window.scrollTo = () => {}
  const rootRoute = createRootRoute({ component: Outlet })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const serviceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "services/$service",
    component: () => null,
  })
  const tracesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, serviceRoute, tracesRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })
  return render(<RouterProvider router={router} />)
}

const nodes: ServiceMapNode[] = [
  {
    name: "A",
    lastSeenNanos: "100",
    spanCount: "10",
    errorCount: "0",
    p95Ms: 20,
  },
  {
    name: "B",
    lastSeenNanos: "120",
    spanCount: "5",
    errorCount: "1",
    p95Ms: 45,
  },
]

const edges: ServiceMapEdge[] = [
  {
    source: "A",
    target: "B",
    callCount: "2",
    errorCount: "1",
    p50Ms: 25,
    p95Ms: 45,
  },
]

describe("EcosystemGraph", () => {
  it("renders trace-path graph nodes and edge links", async () => {
    renderWithRouter(
      <EcosystemGraph
        nodes={nodes}
        edges={edges}
        range={customRange("0", "200")}
      />
    )

    expect(await screen.findByText("trace-path")).toBeTruthy()
    expect(screen.getByText("A")).toBeTruthy()
    expect(screen.getByText("B")).toBeTruthy()
    expect(screen.getByText("A -> B")).toBeTruthy()
    expect(screen.getByText(/50% errors/)).toBeTruthy()

    const serviceLink = screen.getByText("A").closest("a")
    expect(serviceLink?.href).toContain("/services/A")
    const edgeLink = screen.getByText("A -> B").closest("a")
    expect(edgeLink?.href).toContain("/traces")
    expect(edgeLink?.href).toContain("service=A")
  })
})
