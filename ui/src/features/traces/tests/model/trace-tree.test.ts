import { describe, expect, it } from "vitest"

import {
  buildTraceTree,
  computeWindow,
  detectSkew,
  errorPathSpanIds,
  groupByService,
  orderSpans,
  positionPct,
} from "@/features/traces/model/trace-tree"
import type { TraceTreeSpan } from "@/features/traces/model/trace-tree"

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
    const ordered = orderSpans([span("late", "50", "1", "missing"), span("early", "10", "1")])

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

    expect(ordered.map((row) => row.span.spanId)).toEqual(["root", "a", "c", "b"])
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
    const ids = errorPathSpanIds([richSpan("loop", "10", "5", "loop", "api", "STATUS_CODE_ERROR")])
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

    expect(report.suspectPairs).toEqual([{ parentId: "root", childId: "db", driftMs: 200 }])
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

  // Corpus-shaped fixtures (plan 160): mirror the playground corner-case
  // matrix shapes so future rendering fixes regress against the same corpus.
  describe("corner-case corpus shapes", () => {
    it("t-deep: a 14-span linear chain renders full depth without truncation", () => {
      const chain = Array.from({ length: 14 }, (_, i) =>
        span(`s${i}`, String(1_000 + i * 10), String(200 - i * 10), i === 0 ? null : `s${i - 1}`)
      )
      const rows = buildTraceTree(chain)
      expect(rows).toHaveLength(14)
      expect(rows.map((row) => row.depth)).toEqual(Array.from({ length: 14 }, (_, i) => i))
    })

    it("t-wide: 521 spans fanning from one root all render with finite geometry", () => {
      const fan = [
        span("root", "0", "1000000"),
        ...Array.from({ length: 520 }, (_, i) => span(`c${i}`, String(1_000 + i), "500", "root")),
      ]
      const rows = buildTraceTree(fan)
      expect(rows).toHaveLength(521)
      for (const row of rows) {
        expect(Number.isFinite(row.offsetPct)).toBe(true)
        expect(Number.isFinite(row.widthPct)).toBe(true)
        expect(row.widthPct).toBeGreaterThan(0)
      }
    })

    it("t-multiroot: two roots under one trace id both survive ordering", () => {
      const rows = buildTraceTree([
        span("rootB", "2000", "500"),
        span("rootA", "1000", "500"),
        span("childA", "1100", "100", "rootA"),
      ])
      expect(rows.filter((row) => row.depth === 0).map((row) => row.span.spanId)).toEqual([
        "rootA",
        "rootB",
      ])
    })

    it("t-orphan: a child whose parent never arrives renders as a detached root", () => {
      const rows = buildTraceTree([
        span("root", "1000", "500"),
        span("detached", "1200", "100", "never-arrived"),
      ])
      expect(rows.map((row) => row.span.spanId)).toEqual(["root", "detached"])
      expect(rows[1]!.depth).toBe(0)
    })

    it("t-skew: a cross-service child starting 120ms before its CLIENT parent flags skew with non-negative geometry", () => {
      const fixture = [
        richSpan("client", "120000000", "200000000", null, "playground-shapes"),
        richSpan("server", "0", "125000000", "client", "playground-shapes-remote"),
      ]
      for (const row of buildTraceTree(fixture)) {
        expect(row.offsetPct).toBeGreaterThanOrEqual(0)
        expect(row.widthPct).toBeGreaterThanOrEqual(0)
      }
      const report = detectSkew(fixture)
      expect(report.suspectPairs).toEqual([{ parentId: "client", childId: "server", driftMs: 120 }])
    })

    it("t-zero: zero-duration next to a 1µs span never yields NaN layout", () => {
      const rows = buildTraceTree([span("zero", "1000", "0"), span("micro", "1000", "1000")])
      for (const row of rows) {
        expect(Number.isNaN(row.offsetPct)).toBe(false)
        expect(Number.isNaN(row.widthPct)).toBe(false)
        expect(row.widthPct).toBeGreaterThanOrEqual(1.5)
      }
    })
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
