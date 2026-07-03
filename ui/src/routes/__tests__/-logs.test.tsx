/* @vitest-environment jsdom */

import { fireEvent, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { describe, expect, it } from "vitest"

import {
  LogsTable,
  parseLogColumns,
  serializeLogColumns,
  severityVariant,
} from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { formatDateTime } from "@/lib/format"
import type { ResolvedRange } from "@/lib/range"
import { bucketWindow, dragWindow } from "@/routes/logs"

const range: ResolvedRange = {
  key: "7d",
  fromNanos: "1000000000",
  toNanos: "604801000000000",
}

const log: LogDoc = {
  tsNanos: "2000000000",
  service: "checkout",
  severityNum: 17,
  severityText: "ERROR",
  body: "checkout failed",
  traceId: "trace-a",
  spanId: "span-a",
  runId: "run-a",
  scopeName: "seed",
  attributes: '{"error":"boom"}',
  resource: '{"service.name":"checkout"}',
}

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
  const runRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "runs/$runId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, traceRoute, runRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })
  return render(<RouterProvider router={router} />)
}

describe("logs redesign helpers", () => {
  it("computes bucket and drag windows", () => {
    const points = [
      { tsNanos: "1000000000", value: 1 },
      { tsNanos: "31000000000", value: 2 },
      { tsNanos: "61000000000", value: 3 },
    ]

    expect(bucketWindow(points, 1, 30)).toEqual({
      fromNanos: "31000000000",
      toNanos: "61000000000",
    })
    expect(dragWindow(points, 2, 0, 30)).toEqual({
      fromNanos: "1000000000",
      toNanos: "91000000000",
    })
  })

  it("round-trips optional column params", () => {
    expect(parseLogColumns("trace,scope,nope,trace")).toEqual([
      "trace",
      "scope",
    ])
    expect(serializeLogColumns(["service", "scope"])).toBe("service,scope")
  })

  it("maps all severity bands", () => {
    expect(severityVariant(1)).toBe("outline")
    expect(severityVariant(9)).toBe("secondary")
    expect(severityVariant(13)).toBe("amber")
    expect(severityVariant(17)).toBe("rose")
  })
})

describe("LogsTable", () => {
  it("renders date-aware time for multi-day ranges and opens the sheet", async () => {
    renderWithRouter(
      <LogsTable
        logs={[log]}
        range={range}
        columns={["service", "trace", "scope"]}
      />
    )

    expect(await screen.findByText(formatDateTime(log.tsNanos))).toBeTruthy()
    fireEvent.click(screen.getByText("checkout failed"))
    expect(await screen.findByText("Log document")).toBeTruthy()
    expect(screen.getByText("trace trace-a")).toBeTruthy()
  })
})
