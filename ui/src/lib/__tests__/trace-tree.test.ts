import { describe, expect, it } from "vitest"

import {
  buildTraceTree,
  computeWindow,
  detectSkew,
  errorPathSpanIds,
  groupByService,
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

function richSpan(
  spanId: string,
  tsNanos: string,
  durationNs: string,
  parentSpanId: string | null = null,
  service = "api",
  statusCode = "STATUS_CODE_UNSET"
): TraceTreeSpan & { service: string; statusCode: string } {
  return { spanId, parentSpanId, tsNanos, durationNs, service, statusCode }
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

  it("keeps errored spans with their ancestor chain", () => {
    const ids = errorPathSpanIds([
      richSpan("root", "0", "100"),
      richSpan("child", "10", "30", "root"),
      richSpan("leaf", "20", "5", "child", "api", "STATUS_CODE_ERROR"),
      richSpan("sibling", "40", "5", "root"),
    ])

    expect(Array.from(ids)).toEqual(["leaf", "child", "root"])
  })

  it("terminates when an error span is its own parent", () => {
    const ids = errorPathSpanIds([
      richSpan("loop", "10", "5", "loop", "api", "STATUS_CODE_ERROR"),
    ])
    expect(Array.from(ids)).toEqual(["loop"])
  })

  it("terminates a two-node parent cycle containing an error span", () => {
    const ids = errorPathSpanIds([
      richSpan("a", "10", "5", "b", "api", "STATUS_CODE_ERROR"),
      richSpan("b", "20", "5", "a", "api", "STATUS_CODE_UNSET"),
    ])
    expect(new Set(ids)).toEqual(new Set(["a", "b"]))
  })

  it("groups ordered rows into contiguous service lanes", () => {
    const rows = buildTraceTree([
      richSpan("root", "0", "100", null, "api"),
      richSpan("db", "10", "20", "root", "db"),
      richSpan("api-child", "40", "10", "root", "api"),
    ])

    expect(
      groupByService(rows).map((group) => [
        group.service,
        group.spans.map((row) => row.span.spanId),
      ])
    ).toEqual([
      ["api", ["root"]],
      ["db", ["db"]],
      ["api", ["api-child"]],
    ])
  })

  it("detects cross-service clock skew and ignores same-service drift", () => {
    const report = detectSkew([
      richSpan("root", "1000000000", "100000000", null, "api"),
      richSpan("db", "800000000", "10000000", "root", "db"),
      richSpan("local", "700000000", "10000000", "root", "api"),
    ])

    expect(report.suspectPairs).toEqual([
      { parentId: "root", childId: "db", driftMs: 200 },
    ])
    expect(report.maxDriftMs).toBe(200)
  })

  it("detects a backdated rootless span in the same trace", () => {
    const report = detectSkew([
      richSpan("skewed", "1000000000", "10000000", null, "api"),
      richSpan("root", "3601000000000", "100000000", null, "api"),
    ])

    expect(report.suspectPairs).toEqual([
      { parentId: "root", childId: "skewed", driftMs: 3_599_990 },
    ])
    expect(report.maxDriftMs).toBe(3_599_990)
  })

  it("detects a backdated same-service child when drift is extreme", () => {
    const report = detectSkew([
      richSpan("root", "3601000000000", "100000000", null, "api"),
      richSpan("skewed", "1000000000", "10000000", "root", "api"),
    ])

    expect(report.suspectPairs).toEqual([
      { parentId: "root", childId: "skewed", driftMs: 3_600_000 },
    ])
    expect(report.maxDriftMs).toBe(3_600_000)
  })
})
