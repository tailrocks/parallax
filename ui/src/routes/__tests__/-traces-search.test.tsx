/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"

import type { TraceSummary } from "@/lib/api"
import { resolvePreset, updateRangeSearch } from "@/lib/range"
import {
  TraceTable,
  paramToTraceSort,
  patchTracesSearch,
  traceSortToParam,
  validateTracesSearch,
} from "@/routes/traces.index"

const range = {
  key: "1h",
  fromNanos: "1000000000",
  toNanos: "4000000000",
}

describe("traces search params", () => {
  it("tolerates garbage and keeps valid params", () => {
    expect(
      validateTracesSearch({
        page: "bad",
        sort: "NOPE",
        errors: "1",
        minMs: "25",
        live: "1",
      })
    ).toEqual({ errors: true, minMs: 25, live: true })
  })

  it("resets page when filters change", () => {
    expect(
      patchTracesSearch(
        { page: 4, sort: "DURATION_DESC", service: "api" },
        { q: "checkout" }
      )
    ).toEqual({ sort: "DURATION_DESC", service: "api", q: "checkout" })
  })

  it("clears pinned bounds when a preset range is picked", () => {
    expect(
      patchTracesSearch(
        { range: "custom", from: "1000", to: "2000", page: 3 },
        updateRangeSearch(resolvePreset("1h", 1_720_000_000_000))
      )
    ).toEqual({ range: "1h" })
  })

  it("round-trips sort params", () => {
    expect(traceSortToParam("DURATION_ASC")).toBe("duration:asc")
    expect(paramToTraceSort("duration:desc")).toBe("DURATION_DESC")
    expect(paramToTraceSort("spans:asc")).toBeUndefined()
  })
})

describe("TraceTable", () => {
  it("renders a compact trace table from a mocked payload", () => {
    const rows: TraceSummary[] = [
      {
        traceId: "trace-a",
        rootName: "GET /checkout",
        service: "api",
        startNanos: "2000000000",
        durationNs: "10000000",
        spanCount: 3,
        hasError: false,
      },
      {
        traceId: "trace-b",
        rootName: "POST /pay",
        service: "payments",
        startNanos: "3000000000",
        durationNs: "90000000",
        spanCount: 7,
        hasError: true,
      },
      {
        traceId: "trace-c",
        rootName: "worker.flush",
        service: "worker",
        startNanos: "3500000000",
        durationNs: "20000000",
        spanCount: 1,
        hasError: false,
      },
    ]

    render(
      <TraceTable
        rows={rows}
        durationValues={rows.map((row) => Number(row.durationNs))}
        range={range}
        sort="duration:desc"
        onSort={vi.fn()}
        onOpen={vi.fn()}
      />
    )

    expect(screen.getByText("GET /checkout")).toBeTruthy()
    expect(screen.getByText("payments")).toBeTruthy()
    expect(screen.getByText("errors")).toBeTruthy()
  })
})
