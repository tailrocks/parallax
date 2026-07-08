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

import { StoryTimeline } from "@/components/console/story-timeline"
import type { StoryBeat } from "@/lib/api"

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
  const logsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "logs",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, traceRoute, logsRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })
  return render(<RouterProvider router={router} />)
}

const beats: StoryBeat[] = [
  {
    tsNanos: "1000000000",
    lane: "api",
    kind: "span.start",
    title: "checkout",
    traceId: "trace-1",
    spanId: "span-root",
    severity: null,
    durationNs: null,
  },
  {
    tsNanos: "2000000000",
    lane: "api",
    kind: "log",
    title: "INFO cache hit",
    traceId: "trace-1",
    spanId: null,
    severity: "INFO",
    durationNs: null,
  },
  {
    tsNanos: "3000000000",
    lane: "db",
    kind: "error",
    title: "SELECT orders error",
    traceId: "trace-1",
    spanId: "span-db",
    severity: "ERROR",
    durationNs: "20000000",
  },
]

describe("StoryTimeline", () => {
  it("renders time ordered lanes with linked error beats", async () => {
    renderWithRouter(<StoryTimeline beats={beats} />)

    const rows = await screen.findAllByTestId("story-row")
    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringContaining("checkout"),
      expect.stringContaining("INFO cache hit"),
      expect.stringContaining("SELECT orders error"),
    ])
    expect(screen.getAllByText("api")).toHaveLength(2)
    expect(screen.getByText("db")).toBeTruthy()
    expect(rows[2]!.className).toContain("border-rose")
    expect(screen.getByText("SELECT orders error").closest("a")?.href).toContain(
      "/traces/trace-1"
    )
    expect(screen.getByText("INFO cache hit").closest("a")?.href).toContain(
      "/logs"
    )
  })
})
