/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { defaultParseSearch } from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { customRange, resolvePreset } from "@/domain/time-range/range"
import { IssuesContent, type IssuesData } from "@/features/issues"
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
})
