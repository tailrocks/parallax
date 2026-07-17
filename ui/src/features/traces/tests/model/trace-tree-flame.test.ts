import { describe, expect, it } from "vitest"

import {
  computeSelfTimes,
  packFlameLanes,
  type TraceTreeSpan,
} from "@/features/traces/model/trace-tree"

function span(
  spanId: string,
  parentSpanId: string | null,
  startMs: number,
  durationMs: number
): TraceTreeSpan {
  return {
    spanId,
    parentSpanId,
    tsNanos: String(BigInt(Math.round(startMs * 1_000_000))),
    durationNs: String(BigInt(Math.round(durationMs * 1_000_000))),
  }
}

const MS = 1_000_000n

describe("computeSelfTimes (plan 163)", () => {
  it("equals the full duration for leaf spans", () => {
    const spans = [span("a", null, 0, 10)]
    expect(computeSelfTimes(spans).get("a")).toBe(10n * MS)
  })

  it("subtracts sequential children", () => {
    const spans = [
      span("root", null, 0, 100),
      span("c1", "root", 10, 20),
      span("c2", "root", 40, 30),
    ]
    expect(computeSelfTimes(spans).get("root")).toBe(50n * MS)
  })

  it("merges overlapping children before subtracting", () => {
    const spans = [
      span("root", null, 0, 100),
      span("c1", "root", 10, 40), // 10..50
      span("c2", "root", 30, 40), // 30..70, overlap 30..50
    ]
    // Union 10..70 = 60ms covered.
    expect(computeSelfTimes(spans).get("root")).toBe(40n * MS)
  })

  it("clips children that spill past the parent window", () => {
    const spans = [
      span("root", null, 0, 50),
      span("c1", "root", 40, 30), // 40..70, clipped to 40..50
    ]
    expect(computeSelfTimes(spans).get("root")).toBe(40n * MS)
  })

  it("never goes negative when children fully cover the parent", () => {
    const spans = [span("root", null, 0, 10), span("c1", "root", 0, 10), span("c2", "root", 0, 10)]
    expect(computeSelfTimes(spans).get("root")).toBe(0n)
  })

  it("only subtracts direct children, not grandchildren", () => {
    const spans = [
      span("root", null, 0, 100),
      span("mid", "root", 10, 40),
      span("leaf", "mid", 15, 10),
    ]
    const selfTimes = computeSelfTimes(spans)
    expect(selfTimes.get("root")).toBe(60n * MS)
    expect(selfTimes.get("mid")).toBe(30n * MS)
    expect(selfTimes.get("leaf")).toBe(10n * MS)
  })
})

describe("packFlameLanes (plan 163)", () => {
  it("shares a lane between non-overlapping siblings", () => {
    const spans = [
      span("root", null, 0, 100),
      span("c1", "root", 0, 40),
      span("c2", "root", 50, 40),
    ]
    const { rows, laneCounts } = packFlameLanes(spans)
    const byId = new Map(rows.map((row) => [row.span.spanId, row]))
    expect(byId.get("c1")?.lane).toBe(0)
    expect(byId.get("c2")?.lane).toBe(0)
    expect(laneCounts).toEqual([1, 1])
  })

  it("stacks overlapping siblings into separate lanes", () => {
    const spans = [
      span("root", null, 0, 100),
      span("c1", "root", 0, 60),
      span("c2", "root", 30, 60),
      span("c3", "root", 65, 20),
    ]
    const { rows, laneCounts } = packFlameLanes(spans)
    const byId = new Map(rows.map((row) => [row.span.spanId, row]))
    expect(byId.get("c1")?.lane).toBe(0)
    expect(byId.get("c2")?.lane).toBe(1)
    // c3 starts after c1 ends, so it reuses lane 0.
    expect(byId.get("c3")?.lane).toBe(0)
    expect(laneCounts).toEqual([1, 2])
  })

  it("packs multiroot traces at depth 0", () => {
    const spans = [
      span("r1", null, 0, 40),
      span("r2", null, 20, 40), // overlaps r1
      span("r3", null, 50, 10), // fits after r1 in lane 0
    ]
    const { rows, laneCounts } = packFlameLanes(spans)
    const byId = new Map(rows.map((row) => [row.span.spanId, row]))
    expect(byId.get("r1")?.lane).toBe(0)
    expect(byId.get("r2")?.lane).toBe(1)
    expect(byId.get("r3")?.lane).toBe(0)
    expect(laneCounts).toEqual([2])
    expect(rows).toHaveLength(3)
  })

  it("positions rows relative to the trace window", () => {
    const spans = [span("root", null, 0, 100), span("c1", "root", 25, 50)]
    const { rows } = packFlameLanes(spans)
    const c1 = rows.find((row) => row.span.spanId === "c1")
    expect(c1?.offsetPct).toBeCloseTo(25, 6)
    expect(c1?.widthPct).toBeCloseTo(50, 6)
  })

  it("handles empty input", () => {
    expect(packFlameLanes([])).toEqual({ rows: [], laneCounts: [] })
  })
})
