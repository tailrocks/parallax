/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { parseLayout, serializeWidgets } from "@/routes/dashboards.index"
import { EXAMPLES, Route as SqlRoute } from "@/routes/sql"

vi.mock("@/lib/api", () => ({
  gqlString: (value: string) =>
    value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n"),
  graphql: vi.fn().mockResolvedValue({
    sql: { columns: [], rows: [], rowCount: 0 },
  }),
}))

afterEach(cleanup)

describe("final sweep", () => {
  it("keeps SQL examples on real table names", () => {
    const banned = /\botel_spans\b|\botel_logs\b|\botel_metrics_points\b/
    for (const example of EXAMPLES) {
      expect(example.sql).not.toMatch(banned)
    }
  })

  it("preserves dashboard widget unknown fields", () => {
    const widgets = parseLayout(
      '[{"metric":"process.cpu.utilization","agg":"avg","chart":"line","custom":true}]'
    )

    expect(JSON.parse(serializeWidgets(widgets))[0]).toMatchObject({
      metric: "process.cpu.utilization",
      custom: true,
    })
  })

  it("round-trips dashboard label choices without breaking old layouts", () => {
    const widgets = parseLayout(
      '[{"metric":"process.cpu.utilization","agg":"avg","chart":"line"},{"metric":"jvm.memory.used","groupBy":"service_name","filterValue":"checkout"}]'
    )

    expect(widgets[0]).toMatchObject({
      metric: "process.cpu.utilization",
    })
    expect(JSON.parse(serializeWidgets(widgets))[1]).toMatchObject({
      metric: "jvm.memory.used",
      groupBy: "service_name",
      filterValue: "checkout",
    })
  })

  it("renders SQL keyboard hint and examples menu", async () => {
    window.matchMedia = () =>
      ({
        matches: false,
        media: "",
        onchange: null,
        addListener() {},
        removeListener() {},
        addEventListener() {},
        removeEventListener() {},
        dispatchEvent: () => true,
      }) as MediaQueryList

    const rootRoute = createRootRoute({ component: Outlet })
    const component = SqlRoute.options.component!
    const sqlRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/sql",
      component,
    })
    const router = createRouter({
      routeTree: rootRoute.addChildren([sqlRoute]),
      history: createMemoryHistory({ initialEntries: ["/sql"] }),
    })

    render(<RouterProvider router={router} />)
    expect(await screen.findByText("⌘")).toBeTruthy()
    expect(screen.getByText("Enter")).toBeTruthy()
    expect(screen.getByRole("button", { name: /examples/i })).toBeTruthy()
  })
})
