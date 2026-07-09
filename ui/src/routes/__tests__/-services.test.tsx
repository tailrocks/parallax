/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
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
import {
  ServicesIndexContent,
  serviceErrorRate,
  serviceHref,
} from "@/routes/services"
import type { ServicesData } from "@/routes/services"
import { ServiceDetailContent } from "@/routes/services.$service"
import type { ServiceDetailData } from "@/routes/services.$service"

afterEach(cleanup)

const range = resolvePreset("24h", 1_720_000_000_000)

const servicesFixture: ServicesData = {
  serviceList: [
    {
      name: "api gateway",
      lastSeenNanos: "1719999990000000000",
      spanCount: "20",
      errorCount: "2",
      p95Ms: 120,
    },
    {
      name: "checkout/core",
      lastSeenNanos: "1719999980000000000",
      spanCount: "10",
      errorCount: "0",
      p95Ms: null,
    },
  ],
  serviceCatalog: [
    {
      name: "api gateway",
      serviceVersion: "v1",
      serviceNamespace: "edge",
      deploymentEnvironment: "prod",
      telemetrySdkLanguage: "rust",
      telemetrySdkName: "opentelemetry",
      telemetrySdkVersion: "0.32.1",
      lastSeenNanos: "1719999990000000000",
      instanceCount: "2",
    },
    {
      name: "checkout/core",
      serviceVersion: null,
      serviceNamespace: null,
      deploymentEnvironment: null,
      telemetrySdkLanguage: null,
      telemetrySdkName: null,
      telemetrySdkVersion: null,
      lastSeenNanos: "1719999980000000000",
      instanceCount: "0",
    },
  ],
}

const detailFixture: ServiceDetailData = {
  red: {
    rate: [{ tsNanos: "1719999980000000000", value: 10 }],
    errorRate: [{ tsNanos: "1719999980000000000", value: 0.1 }],
    p50: [{ tsNanos: "1719999980000000000", value: 20 }],
    p95: [{ tsNanos: "1719999980000000000", value: 90 }],
    p99: [{ tsNanos: "1719999980000000000", value: 140 }],
  },
  overview: {
    cpu: [],
    memory: [],
    requestRate: [],
    errorRate: [],
    latencyP50: [],
    latencyP95: [],
    latencyP99: [],
  },
  releases: [
    {
      version: "v1",
      firstSeenNanos: "1719999900000000000",
      lastSeenNanos: "1719999980000000000",
      spanCount: "10",
    },
  ],
  serviceCatalog: servicesFixture.serviceCatalog,
  httpDurationExemplars: [],
  rpcDurationExemplars: [],
  runtimeSnapshot: [],
  tracesPage: {
    items: [
      {
        traceId: "trace-a",
        rootName: "POST /checkout",
        service: "api gateway",
        startNanos: "1719999980000000000",
        durationNs: "90000000",
        spanCount: 4,
        hasError: true,
      },
    ],
  },
}

function renderWithRouter(component: React.ReactNode, path = "/services") {
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
  const servicesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/services",
    component: () => component,
  })
  const serviceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/services/$service",
    component: () => component,
  })
  const tracesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/traces/$traceId",
    component: () => null,
  })
  const tracesIndexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/traces",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      servicesRoute,
      serviceRoute,
      tracesRoute,
      tracesIndexRoute,
    ]),
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  return render(<RouterProvider router={router} />)
}

describe("Services route", () => {
  it("derives error rates and encoded detail hrefs", () => {
    expect(serviceErrorRate(servicesFixture.serviceList[0]!)).toBe(0.1)
    expect(serviceHref("checkout/core")).toBe("/services/checkout%2Fcore")
  })

  it("renders index rows with detail links", async () => {
    renderWithRouter(
      <ServicesIndexContent
        data={servicesFixture}
        search={{}}
        range={range}
        onSearch={() => {}}
      />
    )

    expect(await screen.findByText("api gateway")).toBeTruthy()
    expect(screen.getByText("checkout/core")).toBeTruthy()
    expect(screen.getByText("v1")).toBeTruthy()
    expect(screen.getByText("rust")).toBeTruthy()
    expect(screen.getByText("prod")).toBeTruthy()
    expect(
      screen.getByRole("link", { name: /api gateway/i }).getAttribute("href")
    ).toBe("/services/api%20gateway?range=24h")

    const spansHref = screen
      .getByRole("link", { name: "20" })
      .getAttribute("href")
    const spansUrl = new URL(spansHref!, "http://example.test")
    expect(spansUrl.pathname).toBe("/traces")
    expect(spansUrl.searchParams.get("service")).toBe("api gateway")
    expect(spansUrl.searchParams.get("range")).toBe("24h")
    expect(spansUrl.searchParams.has("from")).toBe(false)
    expect(spansUrl.searchParams.has("to")).toBe(false)
  })

  it("renders detail stats and hides infra band without CPU/memory", async () => {
    renderWithRouter(
      <ServiceDetailContent
        service="api gateway"
        data={detailFixture}
        range={range}
        onRange={() => {}}
      />,
      "/services/api%20gateway"
    )

    expect((await screen.findAllByText("Requests")).length).toBeGreaterThan(0)
    expect(screen.getByText("Releases")).toBeTruthy()
    expect(screen.getAllByText("v1").length).toBeGreaterThan(1)
    expect(screen.getByText("Identity")).toBeTruthy()
    expect(screen.getByText("edge")).toBeTruthy()
    expect(screen.getByText("opentelemetry 0.32.1")).toBeTruthy()
    expect(screen.getByText("Error rate")).toBeTruthy()
    expect(screen.getByText("p95 latency")).toBeTruthy()
    expect(screen.queryByText("Runtime metrics")).toBeNull()
    expect(
      screen.getByText(
        "No trace exemplar attached; showing traces near this timestamp"
      )
    ).toBeTruthy()
    const tracesUrl = new URL(
      screen.getByRole("link", { name: /traces/i }).getAttribute("href")!,
      "http://example.test"
    )
    expect(tracesUrl.pathname).toBe("/traces")
    expect(tracesUrl.searchParams.get("service")).toBe("api gateway")
    expect(tracesUrl.searchParams.get("range")).toBe("24h")

    const logsUrl = new URL(
      screen.getByRole("link", { name: /logs/i }).getAttribute("href")!,
      "http://example.test"
    )
    expect(logsUrl.pathname).toBe("/logs")
    expect(logsUrl.searchParams.get("service")).toBe("api gateway")
    expect(logsUrl.searchParams.get("range")).toBe("24h")

    const issuesUrl = new URL(
      screen.getByRole("link", { name: /issues/i }).getAttribute("href")!,
      "http://example.test"
    )
    expect(issuesUrl.pathname).toBe("/issues")
    expect(issuesUrl.searchParams.get("service")).toBe("api gateway")
    expect(issuesUrl.searchParams.get("range")).toBe("24h")
    expect(
      screen
        .getByRole("link", { name: /post \/checkout/i })
        .getAttribute("href")
    ).toBe("/traces/trace-a?range=24h")
  })

  it("opens trace exemplar popovers from latency markers", async () => {
    renderWithRouter(
      <ServiceDetailContent
        service="api gateway"
        data={{
          ...detailFixture,
          httpDurationExemplars: [
            {
              tsNanos: "1719999980000000000",
              service: "api gateway",
              name: "http.server.request.duration",
              value: 120,
              traceId: "trace-exemplar",
              spanId: "span-exemplar",
              runId: "run-a",
              attributes: "{}",
            },
          ],
        }}
        range={range}
        onRange={() => {}}
      />,
      "/services/api%20gateway"
    )

    fireEvent.click(
      await screen.findByRole("button", {
        name: /trace exemplar trace-exemplar/i,
      })
    )

    expect(await screen.findByText("Trace exemplar")).toBeTruthy()
    expect(screen.getByText("span-exemplar")).toBeTruthy()
    expect(
      screen.getByRole("link", { name: /open trace/i }).getAttribute("href")
    ).toBe("/traces/trace-exemplar?range=24h")
  })
})
