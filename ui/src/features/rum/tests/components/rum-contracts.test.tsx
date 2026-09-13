/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import {
  RumContent,
  type RumData,
  type RumSessionDetailData,
  type RumSessionRow,
  type RumTraceData,
  type RumVitalData,
} from "@/features/rum"
import { renderTestRouter } from "@/test/router"

afterEach(() => {
  cleanup()
})

const range = resolvePreset("24h", 1_720_000_000_000)

const rumFixture: RumData = {
  services: ["web-shop"],
  service: "web-shop",
  vitals: [
    {
      name: "browser.lcp",
      vital: "LCP",
      kind: "histogram",
      unit: "ms",
      services: ["web-shop"],
      lastDatapointNanos: "1719999990000000000",
      pointCount: "42",
      p75: 3000,
      rating: "needs-improvement",
    },
  ],
  issues: [
    {
      fingerprint: "fp-a",
      title: "TypeError: null checkout",
      errorType: "TypeError",
      culprit: "cart.ts:12",
      service: "web-shop",
      status: "open",
      lastSeenNanos: "1719999990000000000",
      eventCount: 7,
      lastTraceId: "trace-a",
    },
  ],
  issueTotal: 1,
  errorTraces: [
    {
      traceId: "trace-a",
      rootName: "GET /checkout",
      service: "web-shop",
      startNanos: "1719999990000000000",
      durationNs: "12000000",
      spanCount: 9,
      hasError: true,
    },
  ],
  errorTraceTotal: "1",
  journeys: [
    {
      traceId: "trace-j",
      rootName: "page /cart",
      service: "web-shop",
      startNanos: "1719999980000000000",
      durationNs: "8000000",
      spanCount: 5,
      hasError: false,
    },
  ],
  journeyTotal: "1",
}

const vitalFixture: RumVitalData = {
  row: rumFixture.vitals[0]!,
  trend: [{ tsNanos: "1719999990000000000", value: 3000 }],
  exemplars: [
    {
      tsNanos: "1719999990000000000",
      service: "web-shop",
      name: "browser.lcp",
      value: 3100,
      traceId: "trace-a",
      spanId: "span-a",
      attributes: "{}",
    },
  ],
}

const traceFixture: RumTraceData = {
  traceId: "trace-a",
  linked: [
    {
      traceId: "trace-b",
      rootName: "POST /api/checkout",
      service: "checkout-api",
      startNanos: "1719999990000000000",
      durationNs: "5000000",
      spanCount: 4,
      hasError: false,
    },
  ],
  logs: [
    {
      tsNanos: "1719999990000000000",
      severityText: "ERROR",
      body: "checkout failed",
      service: "web-shop",
      spanId: "span-a",
    },
  ],
}

const sessionRowFixture: RumSessionRow = {
  sessionId: "sess-1",
  service: "web-shop",
  startNanos: "1719999900000000000",
  endNanos: "1719999990000000000",
  spanCount: 4,
  traceCount: 2,
  viewCount: 2,
  vitalCount: 1,
  errorCount: 1,
  hasError: true,
}

const sessionFixture: RumSessionDetailData = {
  session: sessionRowFixture,
  views: [
    {
      tsNanos: "1719999900000000000",
      screen: "home",
      path: "/",
      traceId: "trace-v",
      spanId: "span-v",
    },
    {
      tsNanos: "1719999950000000000",
      screen: "checkout",
      path: "/checkout",
      traceId: "trace-v2",
      spanId: "span-v2",
    },
  ],
  vitals: [
    {
      tsNanos: "1719999920000000000",
      name: "LCP",
      value: 1200,
      rating: "good",
      traceId: "trace-v",
      spanId: "span-lcp",
    },
  ],
  errors: [
    {
      tsNanos: "1719999990000000000",
      name: "web.error.handled",
      errorType: "TypeError",
      message: "boom",
      traceId: "trace-e",
      spanId: "span-e",
    },
  ],
}

function renderRum(
  ui: React.ReactNode,
  path = "/rum?service=web-shop&vital=browser.lcp&traceId=trace-a"
) {
  return renderTestRouter(ui, {
    componentPaths: ["/rum"],
    initialPath: path,
    targetPaths: ["/traces/$traceId", "/issues/$service/$fingerprint", "/metrics/$metricName"],
  })
}

describe("RUM route", () => {
  it("renders vitals with ratings and exemplar trace links", async () => {
    const onSearch = vi.fn()
    renderRum(
      <RumContent
        data={rumFixture}
        vital={vitalFixture}
        trace={traceFixture}
        sessions={[sessionRowFixture]}
        session={null}
        search={{ service: "web-shop", vital: "browser.lcp", traceId: "trace-a" }}
        range={range}
        onSearch={onSearch}
      />
    )

    expect(await screen.findByTestId("vital-row-browser.lcp")).toBeTruthy()
    expect(screen.getByText("Needs improvement")).toBeTruthy()
    const exemplarLinks = screen.getAllByTestId("trace-link-trace-a")
    expect(exemplarLinks.some((link) => link.getAttribute("href") === "/traces/trace-a")).toBe(true)
  })

  it("links errors to service-scoped issue detail and correlates the trace", async () => {
    const onSearch = vi.fn()
    renderRum(
      <RumContent
        data={rumFixture}
        vital={vitalFixture}
        trace={traceFixture}
        sessions={[sessionRowFixture]}
        session={null}
        search={{ service: "web-shop", vital: "browser.lcp", traceId: "trace-a" }}
        range={range}
        onSearch={onSearch}
      />
    )

    expect(await screen.findByText("TypeError")).toBeTruthy()
    expect(screen.getByRole("link", { name: "TypeError" }).getAttribute("href")).toContain(
      "/issues/web-shop/fp-a"
    )
    expect(screen.getByText("checkout failed")).toBeTruthy()
    expect(screen.getByRole("link", { name: /checkout-api/ }).getAttribute("href")).toContain(
      "/traces/trace-b"
    )
  })

  it("selects a journey trace on row click", async () => {
    const user = userEvent.setup()
    const onSearch = vi.fn()
    renderRum(
      <RumContent
        data={rumFixture}
        vital={null}
        trace={null}
        sessions={[sessionRowFixture]}
        session={null}
        search={{ service: "web-shop" }}
        range={range}
        onSearch={onSearch}
      />
    )

    await user.click(await screen.findByTestId("trace-row-trace-j"))
    expect(onSearch).toHaveBeenCalledWith({ traceId: "trace-j" })
  })

  it("toggles a session on row click and renders its timeline", async () => {
    const user = userEvent.setup()
    const onSearch = vi.fn()
    renderRum(
      <RumContent
        data={rumFixture}
        vital={null}
        trace={null}
        sessions={[sessionRowFixture]}
        session={sessionFixture}
        search={{ service: "web-shop", sessionId: "sess-1" }}
        range={range}
        onSearch={onSearch}
      />
    )

    await user.click(await screen.findByTestId("session-row-sess-1"))
    expect(onSearch).toHaveBeenCalledWith({ sessionId: undefined })
    expect(screen.getByText(/Session timeline/)).toBeTruthy()
    expect(screen.getByText("checkout")).toBeTruthy()
    // Vitals table + session timeline both render LCP.
    expect(screen.getAllByText("LCP")).toHaveLength(2)
    expect(screen.getByText("boom")).toBeTruthy()
    expect(
      screen.getByTestId("trace-link-trace-e").getAttribute("href")
    ).toContain("/traces/trace-e")
  })

  it("explains an empty backend honestly", async () => {
    const onSearch = vi.fn()
    const empty: RumData = {
      services: [],
      service: null,
      vitals: [],
      issues: [],
      issueTotal: 0,
      errorTraces: [],
      errorTraceTotal: "0",
      journeys: [],
      journeyTotal: "0",
    }
    renderRum(
      <RumContent
        data={empty}
        vital={null}
        trace={null}
        sessions={[]}
        session={null}
        search={{}}
        range={range}
        onSearch={onSearch}
      />,
      "/rum"
    )

    expect(await screen.findByText("No web vitals yet")).toBeTruthy()
    expect(screen.getByText("No sessions")).toBeTruthy()
    expect(screen.getByText("No issues")).toBeTruthy()
    expect(screen.getByText("No journeys")).toBeTruthy()
  })
})
