/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { defaultParseSearch } from "@tanstack/react-router"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { customRange, resolvePreset } from "@/domain/time-range/range"
import { IssueDetailContent, IssuesContent, type IssuesData } from "@/features/issues"
import type * as IssuesApi from "@/features/issues/api/issues-api"
import { loadIssueCorrelation } from "@/features/issues/api/issues-api"
import { renderTestRouter } from "@/test/router"

vi.mock("@/features/issues/api/issues-api", async (importOriginal) => {
  const actual = await importOriginal<typeof IssuesApi>()
  return {
    ...actual,
    loadIssueCorrelation: vi.fn(),
  }
})

afterEach(() => {
  cleanup()
  vi.mocked(loadIssueCorrelation).mockReset()
})

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
    groupingExplanation: null,
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
      {
        tsNanos: "1719999980000000000",
        service: "checkout",
        message: "checkout overflowed while charging",
        stacktrace:
          "0: checkout::cart::charge\n   at src/charge.rs:42:7\n1: std::panic::panic_any\n   at /rustc/library/std/src/panic.rs:2:2",
        source: "exception",
        traceId: "trace-b",
        spanId: "span-b",
        attributes: '{"order":{"id":',
      },
      {
        tsNanos: "1719999970000000000",
        service: "checkout",
        message: "checkout overflowed without a trace",
        stacktrace: "0: checkout::cart::untraced\n   at src/untraced.rs:7:1",
        source: "exception",
        traceId: "",
        spanId: "span-c",
        attributes: "{}",
      },
    ],
  },
  issueTrend: [
    { tsNanos: "1719999900000000000", count: 1 },
    { tsNanos: "1719999990000000000", count: 7 },
  ],
}

type DeferredCorrelation = {
  promise: Promise<Awaited<ReturnType<typeof loadIssueCorrelation>>>
  resolve: (result: Awaited<ReturnType<typeof loadIssueCorrelation>>) => void
  reject: (error: unknown) => void
}

const pendingCorrelations = new Map<string, DeferredCorrelation>()

function deferCorrelation(
  traceId: string,
  result: Awaited<ReturnType<typeof loadIssueCorrelation>> = correlation(traceId)
) {
  deferPendingCorrelation(traceId)
  pendingCorrelations.get(traceId)!.resolve(result)
}

function deferPendingCorrelation(traceId: string) {
  let resolve!: DeferredCorrelation["resolve"]
  let reject!: DeferredCorrelation["reject"]
  const promise = new Promise<Awaited<ReturnType<typeof loadIssueCorrelation>>>(
    (promiseResolve, promiseReject) => {
      resolve = promiseResolve
      reject = promiseReject
    }
  )
  pendingCorrelations.set(traceId, { promise, resolve, reject })
}

function correlation(traceId: string): Awaited<ReturnType<typeof loadIssueCorrelation>> {
  return {
    status: "ready",
    correlation: {
      invocationId: traceId === "trace-a" ? "invocation-a" : "invocation-b",
      resource: {},
      releaseVersion: traceId === "trace-a" ? "release-a" : "release-b",
      logs: [
        {
          tsNanos: "1719999995000000000",
          severityText: traceId === "trace-a" ? "WARN" : "ERROR",
          body: traceId === "trace-a" ? "latest log body" : "second log body",
        },
      ],
    },
  }
}

beforeEach(() => {
  pendingCorrelations.clear()
  vi.mocked(loadIssueCorrelation).mockImplementation((traceId) => {
    const deferred = pendingCorrelations.get(traceId)
    if (!deferred) throw new Error(`missing correlation for ${traceId}`)
    return deferred.promise
  })
})

function renderWithRouter(component: React.ReactNode, path = "/issues") {
  return renderTestRouter(component, {
    componentPaths: ["/issues", "/issues/$service/$fingerprint"],
    initialPath: path,
    targetPaths: ["/traces/$traceId", "/invocations/$invocationId"],
  })
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
    expect(screen.getByRole("link", { name: "checkout" }).getAttribute("href")).toBe(
      "/services/checkout?range=24h"
    )
    expect(screen.getByRole("link", { name: /trace trace-a/i }).getAttribute("href")).toBe(
      "/traces/trace-a?range=24h"
    )
    const links = screen.getAllByRole("link")
    expect(
      links.some((link) => link.getAttribute("href") === "/issues/checkout/panic-a?range=24h")
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
    const urls = screen.getAllByRole("link").map((link) => parseHref(link.getAttribute("href")!))

    for (const pathname of ["/services/checkout", "/traces/trace-a", "/issues/checkout/panic-a"]) {
      const match = urls.find((candidate) => candidate.url.pathname === pathname)
      expect(match).toBeTruthy()
      expect(match?.search).toMatchObject({
        range: "custom",
        from: custom.fromNanos,
        to: custom.toNanos,
      })
    }
  })

  it("renders parsed stack frames and issue context", async () => {
    deferCorrelation("trace-a", { status: "trace-unavailable" })
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )

    expect(await screen.findByText("src/cart.rs:99:5")).toBeTruthy()
    expect(screen.getByText("checkout::cart::total")).toBeTruthy()
    expect(screen.getByText("parallax issue context panic-a")).toBeTruthy()
    expect(
      screen
        .getAllByRole("link", { name: /open trace trace-a/i })
        .some((link) => link.getAttribute("href") === "/traces/trace-a?range=24h")
    ).toBe(true)
  })

  it("selects the latest occurrence and renders ready trace correlation", async () => {
    deferCorrelation("trace-a")
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )

    expect(
      await screen.findByRole("link", { name: "invocation" }).then((link) => link)
    ).toBeTruthy()
    expect(screen.getByRole("link", { name: "invocation" }).getAttribute("href")).toBe(
      "/invocations/invocation-a?range=24h"
    )
    expect(
      screen
        .getAllByRole("link", { name: /open trace trace-a/i })
        .some((link) => link.getAttribute("href") === "/traces/trace-a?range=24h")
    ).toBe(true)
    expect(screen.getByText("release-a")).toBeTruthy()
    expect(screen.getByText("WARN")).toBeTruthy()
    expect(screen.getByText("latest log body")).toBeTruthy()
    expect(vi.mocked(loadIssueCorrelation).mock.calls).toEqual([["trace-a"]])
  })

  it("updates selected occurrence data and correlation", async () => {
    const user = userEvent.setup()
    deferCorrelation("trace-a")
    deferCorrelation("trace-b")
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )
    await screen.findByRole("link", { name: "invocation" })

    await user.click(screen.getByRole("button", { name: /checkout overflowed while charging/ }))

    expect(await screen.findByText("src/charge.rs:42:7")).toBeTruthy()
    expect(screen.getByText("Attributes could not be parsed.")).toBeTruthy()
    expect(screen.getByText('{"order":{"id":')).toBeTruthy()
    expect(
      await screen.findByRole("link", { name: "invocation" }).then((link) => link)
    ).toBeTruthy()
    expect(screen.getByRole("link", { name: "invocation" }).getAttribute("href")).toBe(
      "/invocations/invocation-b?range=24h"
    )
    expect(screen.getByText("release-b")).toBeTruthy()
    expect(screen.getByText("ERROR")).toBeTruthy()
    expect(screen.getByText("second log body")).toBeTruthy()
    expect(vi.mocked(loadIssueCorrelation).mock.calls).toEqual([["trace-a"], ["trace-b"]])

    const selected = screen.getByRole("button", {
      name: /checkout overflowed while charging/,
    })
    expect(selected.getAttribute("aria-current")).toBe("true")
    expect(
      screen.getByRole("button", { name: /checkout total overflowed/ }).getAttribute("aria-current")
    ).toBeNull()
  })

  it("renders no-trace correlation for an event without a trace", async () => {
    const user = userEvent.setup()
    deferCorrelation("trace-a")
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )
    await screen.findByRole("link", { name: "invocation" })

    await user.click(screen.getByRole("button", { name: /checkout overflowed without a trace/ }))

    expect(await screen.findByText("No trace linked to this event.")).toBeTruthy()
    expect(vi.mocked(loadIssueCorrelation).mock.calls).toEqual([["trace-a"]])
  })

  it("renders retry for unavailable and failed trace correlation", async () => {
    const user = userEvent.setup()
    deferCorrelation("trace-a", { status: "trace-unavailable" })
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )

    expect(await screen.findByText("Trace is unavailable.")).toBeTruthy()
    expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy()

    deferPendingCorrelation("trace-a")
    await user.click(screen.getByRole("button", { name: "Retry" }))
    pendingCorrelations.get("trace-a")!.reject(new Error("correlation failed"))

    expect(await screen.findByText("correlation failed")).toBeTruthy()
    expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy()
    expect(vi.mocked(loadIssueCorrelation).mock.calls).toEqual([["trace-a"], ["trace-a"]])
  })
})
