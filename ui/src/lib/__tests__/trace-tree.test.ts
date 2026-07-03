import { describe, expect, it } from "vitest"

import {
  buildTraceTree,
  computeWindow,
  orderSpans,
  positionPct,
} from "@/lib/trace-tree"
import type { TraceTreeSpan } from "@/lib/trace-tree"

function span(
  spanId: string,
  tsNanos: string,
  durationNs: string,
  parentSpanId: string | null = null
): TraceTreeSpan {
  return { spanId, parentSpanId, tsNanos, durationNs }
}

describe("trace tree", () => {
  it("orders a parent chain with increasing depth", () => {
    const ordered = orderSpans([
      span("leaf", "30", "10", "child"),
      span("root", "10", "50"),
      span("child", "20", "20", "root"),
    ])

    expect(ordered.map((row) => [row.span.spanId, row.depth])).toEqual([
      ["root", 0],
      ["child", 1],
      ["leaf", 2],
    ])
  })

  it("treats missing parents as orphan roots", () => {
    const ordered = orderSpans([
      span("late", "50", "1", "missing"),
      span("early", "10", "1"),
    ])

    expect(ordered.map((row) => row.span.spanId)).toEqual(["early", "late"])
    expect(ordered.map((row) => row.depth)).toEqual([0, 0])
  })

  it("sorts siblings by start time then span id", () => {
    const ordered = orderSpans([
      span("root", "0", "100"),
      span("b", "20", "1", "root"),
      span("c", "10", "1", "root"),
      span("a", "10", "1", "root"),
    ])

    expect(ordered.map((row) => row.span.spanId)).toEqual([
      "root",
      "a",
      "c",
      "b",
    ])
  })

  it("keeps zero-duration traces drawable", () => {
    const window = computeWindow([span("only", "100", "0")])

    expect(window).toEqual({ startNs: 100n, durationNs: 1n })
    expect(buildTraceTree([span("only", "100", "0")])[0]).toMatchObject({
      offsetPct: 0,
      widthPct: 1.5,
    })
  })

  it("clamps positions inside the remaining track", () => {
    const window = { startNs: 100n, durationNs: 100n }

    expect(positionPct(50n, 200n, window)).toEqual({
      offsetPct: 0,
      widthPct: 100,
    })
    expect(positionPct(190n, 50n, window)).toEqual({
      offsetPct: 90,
      widthPct: 10,
    })
  })
})
