/* @vitest-environment jsdom */

import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  defaultParseSearch,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { customRange } from "@/lib/range"
import {
  DashboardCards,
  DashboardCreateDialog,
  dashboardRangeSearch,
  parseLayout,
  serializeWidgets,
} from "@/routes/dashboards.index"
import { EXAMPLES, Route as SqlRoute } from "@/routes/sql"

const apiMock = vi.hoisted(() => ({
  defaultGraphql: vi.fn((query: string) => {
    if (query.includes("metricLabels")) {
      return Promise.resolve({ metricLabels: [] })
    }
    if (query.includes("metricLabelValues")) {
      return Promise.resolve({ metricLabelValues: [] })
    }
    return Promise.resolve({
      sql: { columns: [], rows: [], rowCount: 0, truncated: false },
      savedViews: [],
    })
  }),
  graphql: vi.fn(),
}))

vi.mock("@/lib/api", () => ({
  gqlString: (value: string) =>
    value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n"),
  graphql: apiMock.graphql,
}))

afterEach(cleanup)
afterEach(() => {
  apiMock.graphql.mockReset()
  apiMock.graphql.mockImplementation(apiMock.defaultGraphql)
})

apiMock.graphql.mockImplementation(apiMock.defaultGraphql)

function parseHref(href: string) {
  const url = new URL(href, "http://example.test")
  return { search: defaultParseSearch(url.search), url }
}

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

  it("does not propagate stale dashboard bounds for preset ranges", () => {
    expect(
      dashboardRangeSearch({ range: "24h", from: "1000", to: "2000" })
    ).toEqual({ range: "24h" })
  })

  it("renders dashboard links with custom ranges", async () => {
    const custom = customRange("1500000000", "4000000000")
    const detailSearch = dashboardRangeSearch({
      range: custom.key,
      from: custom.fromNanos,
      to: custom.toNanos,
    })
    expect(detailSearch).toEqual({
      range: "custom",
      from: custom.fromNanos,
      to: custom.toNanos,
    })

    const rootRoute = createRootRoute({ component: Outlet })
    const indexRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/",
      component: () => (
        <DashboardCards
          dashboards={[
            {
              id: "dash-a",
              name: "checkout ops",
              layout: '[{"metric":"process.cpu.utilization"}]',
              updatedAtNanos: "1",
            },
          ]}
          detailSearch={detailSearch}
          onRemove={() => {}}
        />
      ),
    })
    const detailRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/dashboards/$dashboardId",
      component: () => null,
    })
    const router = createRouter({
      routeTree: rootRoute.addChildren([indexRoute, detailRoute]),
      history: createMemoryHistory({ initialEntries: ["/"] }),
    })

    render(<RouterProvider router={router} />)
    const { search, url } = parseHref(
      (await screen.findByRole("link", { name: "checkout ops" })).getAttribute(
        "href"
      )!
    )
    expect(url.pathname).toBe("/dashboards/dash-a")
    expect(search).toMatchObject({
      range: "custom",
      from: custom.fromNanos,
      to: custom.toNanos,
    })
  })

  it("passes custom ranges through dashboard create navigation", async () => {
    const custom = customRange("1500000000", "4000000000")
    const detailSearch = dashboardRangeSearch({
      range: custom.key,
      from: custom.fromNanos,
      to: custom.toNanos,
    })
    const onCreated = vi.fn()
    apiMock.graphql.mockImplementation((query: string) => {
      if (query.includes("dashboardSave")) {
        return Promise.resolve({ dashboardSave: { id: "dash-new" } })
      }
      return apiMock.defaultGraphql(query)
    })

    render(
      <DashboardCreateDialog
        metricNames={["process.cpu.utilization"]}
        detailSearch={detailSearch}
        onCreated={onCreated}
      />
    )

    fireEvent.click(screen.getByRole("button", { name: /new dashboard/i }))
    fireEvent.change(screen.getByPlaceholderText("checkout ops"), {
      target: { value: "checkout ops" },
    })
    fireEvent.change(screen.getByPlaceholderText("Search metrics"), {
      target: { value: "process.cpu.utilization" },
    })
    fireEvent.click(screen.getByRole("button", { name: "Create" }))

    await waitFor(() =>
      expect(onCreated).toHaveBeenCalledWith("dash-new", {
        range: "custom",
        from: custom.fromNanos,
        to: custom.toNanos,
      })
    )
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
