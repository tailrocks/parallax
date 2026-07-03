/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"

import {
  cycleSortParam,
  pageWindow,
  sortRows,
} from "@/components/console/data-table"
import { RelativeTime } from "@/components/console/relative-time"
import { SpanKindChip, spanKindMeta } from "@/components/console/span-kind"
import { StatCard } from "@/components/console/stat-card"
import { resolveRangeSearch } from "@/lib/range"

describe("console kit", () => {
  it("sorts rows nulls-last and cycles sort params", () => {
    const rows = [{ n: 2 }, { n: null }, { n: 1 }, { n: 1 }]
    expect(sortRows(rows, "n:asc", { n: (row) => row.n }).map((row) => row.n)).toEqual([
      1,
      1,
      2,
      null,
    ])
    expect(cycleSortParam(undefined, "n")).toBe("n:desc")
    expect(cycleSortParam("n:desc", "n")).toBe("n:asc")
    expect(cycleSortParam("n:asc", "n")).toBeUndefined()
  })

  it("creates pagination windows", () => {
    expect(pageWindow(1, 1)).toEqual([1])
    expect(pageWindow(4, 7)).toEqual([1, 2, 3, 4, 5, 6, 7])
    expect(pageWindow(10, 20)).toEqual([1, "...", 9, 10, 11, "...", 20])
  })

  it("resolves range search", () => {
    expect(resolveRangeSearch({ range: "1h" }, 1_000_000).key).toBe("1h")
    expect(resolveRangeSearch({ from: "1", to: "2" }).key).toBe("custom")
    expect(resolveRangeSearch({ range: "bad" }).key).toBe("24h")
  })

  it("covers span kind palette and error override", () => {
    for (const kind of ["SERVER", "CLIENT", "INTERNAL", "PRODUCER", "CONSUMER"]) {
      const meta = spanKindMeta(kind)
      expect(meta.icon).toBeDefined()
      expect(meta.bar).toMatch(/^bg-/)
    }
    expect(spanKindMeta("SERVER", "STATUS_CODE_ERROR").bar).toBe("bg-rose-500")
    render(<SpanKindChip kind="SERVER" />)
    expect(screen.getByText("SERVER")).toBeTruthy()
  })

  it("renders StatCard and RelativeTime", () => {
    vi.spyOn(Date, "now").mockReturnValue(1_720_000_000_000)
    render(
      <>
        <StatCard label="Traces" value="42" delta={{ dir: "up", pct: 12 }} />
        <RelativeTime nanos="1719999940000000000" />
      </>,
    )
    expect(screen.getByText("Traces")).toBeTruthy()
    expect(screen.getByText("42")).toBeTruthy()
    expect(screen.getByText("1m ago")).toBeTruthy()
    vi.restoreAllMocks()
  })
})
