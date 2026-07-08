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

import { LinkedTraceEdges } from "@/routes/traces.$traceId"
import type { SpanLink, TraceSummary } from "@/lib/api"

function renderWithRouter(component: React.ReactNode) {
  window.scrollTo = () => {}
  const rootRoute = createRootRoute({ component: Outlet })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces/$traceId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, traceRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })
  return render(<RouterProvider router={router} />)
}

describe("LinkedTraceEdges", () => {
  it("renders resolved link targets as causal edge cards", async () => {
    const links: SpanLink[] = [
      {
        traceId: "target-trace",
        spanId: "target-span",
        attributes: '{"messaging.operation":"publish"}',
      },
    ]
    const target: TraceSummary = {
      traceId: "target-trace",
      rootName: "consume work",
      service: "worker",
      startNanos: "20",
      durationNs: "25000000",
      spanCount: 2,
      hasError: true,
    }

    renderWithRouter(
      <LinkedTraceEdges
        links={links}
        linkedTraceById={new Map([[target.traceId, target]])}
      />
    )

    expect(await screen.findByText("worker")).toBeTruthy()
    expect(screen.getByText("consume work")).toBeTruthy()
    expect(screen.getByText("2 spans")).toBeTruthy()
    expect(screen.getByText("error")).toBeTruthy()
    expect(screen.getByRole("link").getAttribute("href")).toBe(
      "/traces/target-trace"
    )
  })
})
