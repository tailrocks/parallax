/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { LinkedTraceEdges, TraceCompareResult } from "@/features/traces"
import type { SpanLink, TraceDiff, TraceSummary } from "@/lib/api"
import { renderTestRouter } from "@/test/router"

describe("LinkedTraceEdges", () => {
  it("renders resolved link targets as causal edge cards", async () => {
    const links: SpanLink[] = [
      {
        traceId: "target-trace",
        spanId: "target-span",
        attributes: '{"messaging.operation":"publish"}',
      },
    ]
    const target: TraceSummary = {
      traceId: "target-trace",
      rootName: "consume work",
      service: "worker",
      startNanos: "20",
      durationNs: "25000000",
      spanCount: 2,
      hasError: true,
    }

    renderTestRouter(
      <LinkedTraceEdges
        links={links}
        linkedTraceById={new Map([[target.traceId, target]])}
        rangeSearch={{ range: "24h" }}
      />,
      { targetPaths: ["/traces/$traceId"] }
    )

    expect(await screen.findByText("worker")).toBeTruthy()
    expect(screen.getByText("consume work")).toBeTruthy()
    expect(screen.getByText("2 spans")).toBeTruthy()
    expect(screen.getByText("error")).toBeTruthy()
    expect(screen.getByRole("link").getAttribute("href")).toBe(
      "/traces/target-trace?range=24h"
    )
  })
})

describe("TraceCompareResult", () => {
  it("renders added removed and changed sections", () => {
    const diff: TraceDiff = {
      added: [
        {
          spanId: "retry",
          service: "api",
          name: "POST /checkout/retry",
          kind: "CLIENT",
          statusCode: "STATUS_CODE_UNSET",
          durationNs: "15000000",
          depth: 1,
          matchKey: "api|client|1|retry|0",
        },
      ],
      removed: [
        {
          spanId: "cache",
          service: "api",
          name: "GET /cache",
          kind: "CLIENT",
          statusCode: "STATUS_CODE_UNSET",
          durationNs: "5000000",
          depth: 1,
          matchKey: "api|client|1|cache|0",
        },
      ],
      changed: [
        {
          before: {
            spanId: "db-a",
            service: "db",
            name: "SELECT orders",
            kind: "CLIENT",
            statusCode: "STATUS_CODE_UNSET",
            durationNs: "10000000",
            depth: 1,
            matchKey: "db|client|1|select|0",
          },
          after: {
            spanId: "db-b",
            service: "db",
            name: "SELECT orders",
            kind: "CLIENT",
            statusCode: "STATUS_CODE_ERROR",
            durationNs: "25000000",
            depth: 1,
            matchKey: "db|client|1|select|0",
          },
          durationDeltaNs: "15000000",
          durationDeltaPct: 150,
          statusChanged: true,
        },
      ],
    }

    render(<TraceCompareResult diff={diff} />)

    expect(screen.getByText("Added")).toBeTruthy()
    expect(screen.getByText("Removed")).toBeTruthy()
    expect(screen.getByText("Changed")).toBeTruthy()
    expect(screen.getByText("POST /checkout/retry")).toBeTruthy()
    expect(screen.getByText("GET /cache")).toBeTruthy()
    expect(screen.getByText("SELECT orders")).toBeTruthy()
    expect(screen.getByText("+15ms")).toBeTruthy()
    expect(screen.getByText("ERROR")).toBeTruthy()
  })
})
