/* @vitest-environment jsdom */

import { cleanup, fireEvent, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import { IssueDetailContent, IssuesContent, type IssuesData } from "@/features/issues"
import type * as IssuesApi from "@/features/issues/api/issues-api"
import { loadIssueCorrelation } from "@/features/issues/api/issues-api"
import type { IssueDetailData } from "@/features/issues/model/issue-detail"
import { patchIssuesSearch, validateIssuesSearch } from "@/features/issues/model/issues-search"
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

function renderWithRouter(component: React.ReactNode, path = "/issues") {
  return renderTestRouter(component, {
    componentPaths: ["/issues", "/issues/$service/$fingerprint"],
    initialPath: path,
    targetPaths: ["/traces/$traceId", "/invocations/$invocationId"],
  })
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
        tags: "{}",
        trend: [{ tsNanos: "1719999900000000000", count: 7 }],
      },
    ],
  },
}

const detailFixture: IssueDetailData = {
  issue: {
    ...issuesFixture.issues.items[0]!,
    environmentCounts: [
      { environment: "production", count: 5 },
      { environment: "staging", count: 2 },
    ],
    groupingExplanation: null,
    events: [
      {
        tsNanos: "1719999990000000000",
        service: "checkout",
        message: "checkout total overflowed",
        stacktrace: null,
        source: "exception",
        traceId: "",
        spanId: "span-a",
        environment: "production",
        serviceVersion: null,
        mappedFrames: [],
        attributes: "{}",
      },
    ],
  },
  issueTrend: [{ tsNanos: "1719999990000000000", count: 7 }],
}

describe("Issues environment filter", () => {
  it("keeps environment in search state", () => {
    expect(validateIssuesSearch({ environment: "production" })).toEqual({
      environment: "production",
    })
    expect(validateIssuesSearch({ environment: 7 })).toEqual({})
    expect(patchIssuesSearch({}, { environment: "staging" })).toEqual({
      environment: "staging",
    })
    expect(patchIssuesSearch({ environment: "staging" }, { environment: undefined })).toEqual({})
  })

  it("renders the environment control and reports changes", async () => {
    const onSearch = vi.fn()
    renderWithRouter(
      <IssuesContent
        data={issuesFixture}
        search={{ environment: "production" }}
        range={range}
        onSearch={onSearch}
        onIssue={() => {}}
      />
    )

    const input = (await screen.findByPlaceholderText("Environment")) as HTMLInputElement
    expect(input.value).toBe("production")
    fireEvent.change(input, { target: { value: "staging" } })
    expect(onSearch).toHaveBeenCalledWith({ environment: "staging" })
    fireEvent.click(screen.getByRole("button", { name: "Clear" }))
    expect(onSearch).toHaveBeenCalledWith({
      q: undefined,
      service: undefined,
      status: undefined,
      environment: undefined,
    })
  })

  it("renders the per-environment rollup and occurrence environments", async () => {
    vi.mocked(loadIssueCorrelation).mockResolvedValue({ status: "trace-unavailable" })
    renderWithRouter(
      <IssueDetailContent data={detailFixture} range={range} onRange={() => {}} />,
      "/issues/checkout/panic-a"
    )

    expect(await screen.findByText("Environments")).toBeTruthy()
    expect(screen.getAllByText("production")).toHaveLength(2)
    expect(screen.getByText("staging")).toBeTruthy()
    expect(screen.getByText("x5")).toBeTruthy()
    expect(screen.getByText("x2")).toBeTruthy()
  })
})
