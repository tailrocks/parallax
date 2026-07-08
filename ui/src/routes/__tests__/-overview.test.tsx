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

import { graphql } from "@/lib/api"
import type { ResolvedRange } from "@/lib/range"
import { OverviewContent, latencyBands, loadOverview } from "@/routes/index"
import type { OverviewData } from "@/routes/index"

vi.mock("@/lib/api", () => {
  return {
    graphql: vi.fn(),
  }
})

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

const range: ResolvedRange = {
  key: "24h",
  fromNanos: "1000000000",
  toNanos: "86401000000000",
}

const fixture: OverviewData = {
  overview: {
    spanCount: "120",
    traceCount: "12",
    logCount: "48",
    metricPointCount: "0",
    errorCount: "6",
    errorRate: 0.25,
    activeServices: 4,
  },
  previousOverview: {
    spanCount: "96",
    traceCount: "10",
    logCount: "40",
    metricPointCount: "0",
    errorCount: "8",
    errorRate: 0.5,
    activeServices: 3,
  },
  spansSeries: [
    { tsNanos: "1000000000", value: 20 },
    { tsNanos: "2000000000", value: 40 },
  ],
  errorsSeries: [
    { tsNanos: "1000000000", value: 1 },
    { tsNanos: "2000000000", value: 3 },
  ],
  red: {
    rate: [{ tsNanos: "1000000000", value: 2 }],
    errorRate: [{ tsNanos: "1000000000", value: 0.25 }],
    p50: [{ tsNanos: "1000000000", value: 25 }],
    p95: [{ tsNanos: "1000000000", value: 90 }],
    p99: [{ tsNanos: "1000000000", value: 140 }],
  },
  previousRed: {
    rate: [{ tsNanos: "1000000000", value: 2 }],
    errorRate: [{ tsNanos: "1000000000", value: 0.5 }],
    p50: [{ tsNanos: "1000000000", value: 40 }],
    p95: [{ tsNanos: "1000000000", value: 120 }],
    p99: [{ tsNanos: "1000000000", value: 160 }],
  },
  issues: {
    items: [
      {
        fingerprint: "issue-a",
        title: "checkout timeout",
        service: "checkout",
        lastSeenNanos: "2000000000",
        eventCount: 7,
        status: "open",
      },
    ],
  },
  tracesPage: {
    items: [
      {
        traceId: "trace-a",
        rootName: "POST /checkout",
        service: "api",
        startNanos: "1000000000",
        durationNs: "90000000",
        spanCount: 5,
        hasError: true,
      },
    ],
  },
}

function zeroFixture(): OverviewData {
  return {
    ...fixture,
    overview: {
      ...fixture.overview,
      spanCount: "0",
      traceCount: "0",
      logCount: "0",
      errorCount: "0",
      errorRate: 0,
      activeServices: 0,
    },
    issues: { items: [] },
    tracesPage: { items: [] },
  }
}

function renderWithRouter(component: React.ReactNode) {
  window.scrollTo = () => {}
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
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => component,
  })
  const issueRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "issues/$fingerprint",
    component: () => null,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces/$traceId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, issueRoute, traceRoute]),
    history: createMemoryHistory({ initialEntries: ["/"] }),
  })

  return render(<RouterProvider router={router} />)
}

describe("Overview route", () => {
  it("loads the one-document overview query", async () => {
    vi.mocked(graphql).mockResolvedValueOnce(fixture)

    await expect(loadOverview(range)).resolves.toBe(fixture)
    expect(vi.mocked(graphql).mock.calls[0]?.[0]).toContain("overview")
    expect(vi.mocked(graphql).mock.calls[0]?.[0]).toContain("signalCountSeries")
    expect(vi.mocked(graphql).mock.calls[0]?.[0]).toContain("tracesPage")
  })

  it("renders KPIs and linked recent lists", async () => {
    const rendered = renderWithRouter(
      <OverviewContent data={fixture} range={range} onRangeChange={vi.fn()} />
    )

    expect((await screen.findAllByText("Spans")).length).toBeGreaterThan(0)
    expect(screen.getByText("Logs")).toBeTruthy()
    expect(screen.getByText("Error rate")).toBeTruthy()
    expect(screen.getByText("p95 latency")).toBeTruthy()

    const hrefs = screen
      .getAllByRole("link")
      .map((link) => link.getAttribute("href"))
    expect(hrefs).toContain("/traces?range=24h")
    expect(hrefs).toContain("/logs?range=24h")
    expect(hrefs).toContain("/issues?status=open&range=24h")
    expect(hrefs).toContain("/traces?sort=DURATION_DESC&range=24h")

    const invertedDelta = rendered.container.querySelector(
      "[data-slot='badge'][class*='emerald']"
    )
    expect(invertedDelta).toBeTruthy()
    expect(
      (
        await screen.findByRole("link", { name: /checkout timeout/i })
      ).getAttribute("href")
    ).toBe("/issues/issue-a")
    expect(
      (
        await screen.findByRole("link", {
          name: new RegExp("post /checkout", "i"),
        })
      ).getAttribute("href")
    ).toBe("/traces/trace-a")
  })

  it("renders onboarding when there is no telemetry", async () => {
    renderWithRouter(
      <OverviewContent
        data={zeroFixture()}
        range={range}
        onRangeChange={vi.fn()}
      />
    )

    expect(await screen.findByText("Send your first telemetry")).toBeTruthy()
    expect(screen.getByText("http://127.0.0.1:4317")).toBeTruthy()
    expect(screen.queryByText("Recent issues")).toBeNull()
  })

  it("guards negative stacked latency bands", () => {
    expect(
      latencyBands({
        rate: [],
        errorRate: [],
        p50: [{ tsNanos: "1", value: 50 }],
        p95: [{ tsNanos: "1", value: 40 }],
        p99: [{ tsNanos: "1", value: 35 }],
      })[0]
    ).toMatchObject({ p50Band: 50, p95Band: 0, p99Band: 0 })
  })
})
