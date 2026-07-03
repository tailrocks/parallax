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
import { afterEach, describe, expect, it } from "vitest"

import { resolvePreset } from "@/lib/range"
import { IssuesContent } from "@/routes/issues.index"
import type { IssuesData } from "@/routes/issues.index"
import { IssueDetailContent } from "@/routes/issues.$fingerprint"

afterEach(cleanup)

const range = resolvePreset("24h", 1_720_000_000_000)

const issuesFixture: IssuesData = {
  services: ["checkout"],
  issues: {
    total: 1,
    items: [
      {
        fingerprint: "panic-a",
        title: "checkout total overflowed",
        errorType: "panic",
        culprit: "checkout::cart::total",
        service: "checkout",
        status: "open",
        firstSeenNanos: "1719999900000000000",
        lastSeenNanos: "1719999990000000000",
        eventCount: 7,
        lastTraceId: "trace-a",
        tags: '{"route":{"/checkout":7},"env":{"prod":7},"host":{"api-1":1}}',
        trend: [{ tsNanos: "1719999900000000000", count: 7 }],
      },
    ],
  },
}

const detailFixture = {
  issue: {
    ...issuesFixture.issues.items[0]!,
    events: [
      {
        tsNanos: "1719999990000000000",
        service: "checkout",
        message: "checkout total overflowed",
        stacktrace:
          "0: checkout::cart::total\n   at src/cart.rs:99:5\n1: std::panicking::begin_panic\n   at /rustc/library/std/src/panicking.rs:1:1",
        source: "exception",
        traceId: "trace-a",
        spanId: "span-a",
        attributes: "{}",
      },
    ],
  },
  issueTrend: [
    { tsNanos: "1719999900000000000", count: 1 },
    { tsNanos: "1719999990000000000", count: 7 },
  ],
  resource: { "process.runtime.name": "rust" },
  breadcrumbs: [
    {
      tsNanos: "1719999990000000000",
      severityText: "ERROR",
      body: "panicked",
    },
  ],
  traceRunId: null,
}

function renderWithRouter(component: React.ReactNode, path = "/issues") {
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
  const issuesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/issues",
    component: () => component,
  })
  const issueRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/issues/$fingerprint",
    component: () => component,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/traces/$traceId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([issuesRoute, issueRoute, traceRoute]),
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  return render(<RouterProvider router={router} />)
}

describe("Issues route", () => {
  it("renders trend and event cells as detail links", async () => {
    renderWithRouter(
      <IssuesContent
        data={issuesFixture}
        search={{}}
        range={range}
        onSearch={() => {}}
        onIssue={() => {}}
      />
    )

    expect(await screen.findByText("panic")).toBeTruthy()
    const links = screen.getAllByRole("link")
    expect(
      links.some((link) => link.getAttribute("href") === "/issues/panic-a")
    ).toBe(true)
  })

  it("renders parsed stack frames and timestamped breadcrumbs", async () => {
    renderWithRouter(
      <IssueDetailContent
        data={detailFixture}
        range={range}
        onRange={() => {}}
      />,
      "/issues/panic-a"
    )

    expect(await screen.findByText("src/cart.rs:99:5")).toBeTruthy()
    expect(screen.getByText("checkout::cart::total")).toBeTruthy()
    expect(screen.getByText("Logs around latest event")).toBeTruthy()
    expect(screen.getByText("parallax issue context panic-a")).toBeTruthy()
  })
})
