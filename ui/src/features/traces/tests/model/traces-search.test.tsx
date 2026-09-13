/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import type { TraceSummary } from "@/features/traces/model/wire"
import { customRange, resolvePreset, updateRangeSearch } from "@/domain/time-range/range"
import {
  TraceTable,
  traceDetailSearch,
  paramToTraceSort,
  parseTracesViewState,
  patchTracesSearch,
  serializeTracesSearch,
  traceSortToParam,
  validateTracesSearch,
} from "@/features/traces"

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
      patchTracesSearch({ page: 4, sort: "DURATION_DESC", service: "api" }, { q: "checkout" })
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

  it("builds trace detail links for preset and custom ranges", () => {
    expect(traceDetailSearch(resolvePreset("24h", 1_720_000_000_000))).toEqual({
      range: "24h",
    })

    const custom = customRange("1500000000", "4000000000")
    expect(traceDetailSearch(custom)).toEqual({
      range: "custom",
      from: custom.fromNanos,
      to: custom.toNanos,
    })
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

describe("traces saved views", () => {
  it("round-trips search state through serialize and parse", () => {
    const search = {
      q: "checkout",
      service: "api",
      errors: true,
      minMs: 25,
      sort: "DURATION_DESC" as const,
      range: "1h",
    }
    const state = serializeTracesSearch(search)
    expect(state.startsWith("?")).toBe(true)
    expect(parseTracesViewState(state)).toEqual(search)
  })

  it("skips pagination and live mode", () => {
    expect(serializeTracesSearch({ live: true, page: 3, q: "x" })).toBe("?q=x")
    expect(parseTracesViewState("?sort=NOPE&page=bad")).toEqual({})
  })
})

describe("TraceTable keyboard nav", () => {
  afterEach(cleanup)

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
  ]

  it("opens the row selected with j + Enter", () => {
    const onOpen = vi.fn()
    render(
      <TraceTable
        rows={rows}
        durationValues={rows.map((row) => Number(row.durationNs))}
        range={range}
        sort={undefined}
        onSort={vi.fn()}
        onOpen={onOpen}
      />
    )
    fireEvent.keyDown(window, { key: "j" })
    fireEvent.keyDown(window, { key: "j" })
    fireEvent.keyDown(window, { key: "Enter" })
    expect(onOpen).toHaveBeenCalledWith("trace-b")
  })
})
