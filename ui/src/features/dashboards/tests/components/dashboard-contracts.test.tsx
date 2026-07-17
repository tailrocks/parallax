/* @vitest-environment jsdom */

import { cleanup, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { defaultParseSearch } from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { customRange } from "@/domain/time-range/range"
import {
  DashboardCards,
  DashboardCreateDialog,
  dashboardRangeSearch,
  loadWidgetSeries,
  parseLayout,
  serializeWidgets,
} from "@/features/dashboards"
import { renderTestRouter } from "@/test/router"

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

vi.mock("@/platform/graphql/transport", () => ({
  gqlString: (value: string) =>
    value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n"),
  graphql: apiMock.graphql,
  graphqlCached: apiMock.graphql,
}))

vi.mock("@/features/dashboards/api/dashboard-api", () => ({
  loadDashboardsList: vi.fn(),
  loadDashboardDetail: vi.fn(),
  saveDashboard: vi.fn(async () => ({
    id: "dash-new",
    name: "checkout ops",
    layout: "[]",
    updatedAtNanos: "1",
  })),
  deleteDashboard: vi.fn(async () => undefined),
}))

afterEach(() => {
  cleanup()
  apiMock.graphql.mockReset()
  apiMock.graphql.mockImplementation(apiMock.defaultGraphql)
})

apiMock.graphql.mockImplementation(apiMock.defaultGraphql)

function parseHref(href: string) {
  const url = new URL(href, "http://example.test")
  return { search: defaultParseSearch(url.search), url }
}

describe("dashboard contracts", () => {
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
    expect(dashboardRangeSearch({ range: "24h", from: "1000", to: "2000" })).toEqual({
      range: "24h",
    })
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

    renderTestRouter(
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
      />,
      { targetPaths: ["/dashboards/$dashboardId"] }
    )
    const { search, url } = parseHref(
      (await screen.findByRole("link", { name: "checkout ops" })).getAttribute("href")!
    )
    expect(url.pathname).toBe("/dashboards/dash-a")
    expect(search).toMatchObject({
      range: "custom",
      from: custom.fromNanos,
      to: custom.toNanos,
    })
  })

  it("loads N widget series with one aliased GraphQL document", async () => {
    const fetch = vi.fn(async (_query: string) => ({
      series_0: [{ groupValue: null, points: [{ tsNanos: "1", value: 1 }] }],
      series_1: [{ groupValue: null, points: [{ tsNanos: "1", value: 2 }] }],
      series_2: [{ groupValue: null, points: [{ tsNanos: "1", value: 3 }] }],
    }))
    const widgets = [
      { metric: "a", agg: "avg", chart: "line", title: "A" },
      { metric: "b", agg: "sum", chart: "area", title: "B" },
      { metric: "c", agg: "max", chart: "bar", title: "C" },
    ]
    const series = await loadWidgetSeries(
      widgets,
      { fromNanos: "10", toNanos: "20" },
      fetch as never
    )
    expect(fetch).toHaveBeenCalledTimes(1)
    const doc = String(fetch.mock.calls[0]?.[0] ?? "")
    expect(doc).toContain("series_0:")
    expect(doc).toContain("query DashboardWidgetSeries")
    expect(doc).not.toContain('name: "a"')
    expect(series).toHaveLength(3)
    expect(series[1]?.[0]?.points[0]?.value).toBe(2)
  })

  it("passes custom ranges through dashboard create navigation", async () => {
    const user = userEvent.setup()
    const custom = customRange("1500000000", "4000000000")
    const detailSearch = dashboardRangeSearch({
      range: custom.key,
      from: custom.fromNanos,
      to: custom.toNanos,
    })
    const onCreated = vi.fn()

    render(
      <DashboardCreateDialog
        metricNames={["process.cpu.utilization"]}
        detailSearch={detailSearch}
        initialWidget={{
          metric: "process.cpu.utilization",
          agg: "avg",
          chart: "line",
          title: "cpu",
        }}
        onCreated={onCreated}
      />
    )

    await user.type(screen.getByPlaceholderText("checkout ops"), "checkout ops")
    await user.click(screen.getByRole("button", { name: "Create" }))

    await waitFor(() =>
      expect(onCreated).toHaveBeenCalledWith("dash-new", {
        range: "custom",
        from: custom.fromNanos,
        to: custom.toNanos,
      })
    )
  })
})
