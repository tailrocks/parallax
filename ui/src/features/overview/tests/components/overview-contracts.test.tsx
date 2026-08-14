/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { graphqlCached } from "@/platform/graphql/transport"
import type { ResolvedRange } from "@/domain/time-range/range"
import { OverviewContent, latencyBands, loadOverview } from "@/features/overview"
import type { OverviewData } from "@/features/overview"
import { renderTestRouter } from "@/test/router"

vi.mock("@/platform/graphql/transport", () => {
  return {
    graphql: vi.fn(),
    graphqlCached: vi.fn(),
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
  servicesNow: [{ name: "checkout", spanCount: "120", errorCount: "6", p95Ms: 90 }],
  servicesPrev: [{ name: "checkout", spanCount: "100", errorCount: "1", p95Ms: 80 }],
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
  return renderTestRouter(component, {
    targetPaths: ["/issues/$fingerprint", "/traces/$traceId", "/services/$service"],
  })
}

describe("Overview route", () => {
  it("loads the one-document overview query", async () => {
    vi.mocked(graphqlCached).mockResolvedValueOnce(fixture)

    await expect(loadOverview(range)).resolves.toBe(fixture)
    expect(vi.mocked(graphqlCached).mock.calls[0]?.[0]).toContain("overview")
    expect(vi.mocked(graphqlCached).mock.calls[0]?.[0]).toContain("signalCountSeries")
    expect(vi.mocked(graphqlCached).mock.calls[0]?.[0]).toContain("servicesNow")
    expect(vi.mocked(graphqlCached).mock.calls[0]?.[0]).toContain("tracesPage")
  })

  it("renders KPIs and linked recent lists", async () => {
    const rendered = renderWithRouter(
      <OverviewContent data={fixture} range={range} onRangeChange={vi.fn()} />
    )

    expect((await screen.findAllByText("Spans")).length).toBeGreaterThan(0)
    expect(screen.getByText("Logs")).toBeTruthy()
    expect(screen.getByText("Error rate")).toBeTruthy()
    expect(screen.getByText("p95 latency")).toBeTruthy()
    expect(screen.getByText("What changed")).toBeTruthy()
    expect(screen.getByText("checkout error rate 1.0% -> 5.0%")).toBeTruthy()

    const hrefs = screen.getAllByRole("link").map((link) => link.getAttribute("href"))
    expect(hrefs).toContain("/traces?range=24h")
    expect(hrefs).toContain("/logs?range=24h")
    expect(hrefs).toContain("/issues?status=open&range=24h")
    expect(hrefs).toContain("/traces?sort=DURATION_DESC&range=24h")
    expect(hrefs).toContain("/services/checkout?range=24h")

    const invertedDelta = rendered.container.querySelector("[data-slot='badge'][class*='emerald']")
    expect(invertedDelta).toBeTruthy()
    expect(
      (await screen.findByRole("link", { name: /checkout timeout/i })).getAttribute("href")
    ).toBe("/issues/issue-a?range=24h")
    expect(
      (
        await screen.findByRole("link", {
          name: new RegExp("post /checkout", "i"),
        })
      ).getAttribute("href")
    ).toBe("/traces/trace-a?range=24h")
  })

  it("renders onboarding when there is no telemetry", async () => {
    renderWithRouter(<OverviewContent data={zeroFixture()} range={range} onRangeChange={vi.fn()} />)

    expect(await screen.findByText("Send your first telemetry")).toBeTruthy()
    expect(screen.getByTestId("instrument-snippet-tabs").textContent).toMatch(/init_tracing/)
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
