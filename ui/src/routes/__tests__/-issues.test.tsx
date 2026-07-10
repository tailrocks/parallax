/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  defaultParseSearch,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it } from "vitest"

import { customRange, resolvePreset } from "@/lib/range"
import { IssuesContent } from "@/routes/issues.index"
import type { IssuesData } from "@/routes/issues.index"
import { IssueDetailContent } from "@/routes/issues.$fingerprint"

afterEach(cleanup)

const range = resolvePreset("24h", 1_720_000_000_000)
const custom = customRange("1500000000", "4000000000")

function parseHref(href: string) {
  const url = new URL(href, "http://test.local")
  return { search: defaultParseSearch(url.search), url }
}

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
  traceRunId: "run-a",
  releaseVersion: "v1",
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
  const runRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/runs/$runId",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      issuesRoute,
      issueRoute,
      traceRoute,
      runRoute,
    ]),
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
    expect(screen.getByRole("columnheader", { name: "Service" })).toBeTruthy()
    expect(
      screen.getByRole("link", { name: "checkout" }).getAttribute("href")
    ).toBe("/services/checkout?range=24h")
    expect(
      screen.getByRole("link", { name: /trace trace-a/i }).getAttribute("href")
    ).toBe("/traces/trace-a?range=24h")
    const links = screen.getAllByRole("link")
    expect(
      links.some(
        (link) => link.getAttribute("href") === "/issues/panic-a?range=24h"
      )
    ).toBe(true)
  })

  it("preserves custom ranges in rendered drilldown links", async () => {
    renderWithRouter(
      <IssuesContent
        data={issuesFixture}
        search={{}}
        range={custom}
        onSearch={() => {}}
        onIssue={() => {}}
      />
    )

    expect(await screen.findByText("panic")).toBeTruthy()
    const urls = screen
      .getAllByRole("link")
      .map((link) => parseHref(link.getAttribute("href")!))

    for (const pathname of [
      "/services/checkout",
      "/traces/trace-a",
      "/issues/panic-a",
    ]) {
      const match = urls.find((candidate) => candidate.url.pathname === pathname)
      expect(match).toBeTruthy()
      expect(match?.search).toMatchObject({
        range: "custom",
        from: custom.fromNanos,
        to: custom.toNanos,
      })
    }
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
    expect(
      screen.getByText((_, element) => element?.textContent === "release v1")
    ).toBeTruthy()
    expect(screen.getByText("Logs around latest event")).toBeTruthy()
    expect(screen.getByText("parallax issue context panic-a")).toBeTruthy()
    expect(
      screen
        .getAllByRole("link")
        .some((link) => link.getAttribute("href") === "/runs/run-a?range=24h")
    ).toBe(true)
    expect(
      screen
        .getByRole("link", { name: /open trace trace-a/i })
        .getAttribute("href")
    ).toBe("/traces/trace-a?range=24h")
  })
})
