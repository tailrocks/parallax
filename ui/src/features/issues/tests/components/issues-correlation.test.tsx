/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import { IssueDetailContent, type IssuesData } from "@/features/issues"
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
    targetPaths: ["/traces/$traceId", "/invocations/$invocationId", "/logs"],
  })
}

describe("Issue correlation failures", () => {
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
