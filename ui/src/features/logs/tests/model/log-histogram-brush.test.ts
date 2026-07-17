import { describe, expect, it } from "vitest"

import {
  DEFAULT_HISTOGRAM_BUCKETS,
  buildUniformBuckets,
  pxToTime,
  snapBrushToBuckets,
  timeToPx,
  type HistogramBucket,
} from "@/features/logs/model/log-histogram-brush"

const buckets: HistogramBucket[] = [
  { start: 0, end: 10, count: 1 },
  { start: 10, end: 20, count: 2 },
  { start: 20, end: 30, count: 3 },
  { start: 30, end: 40, count: 4 },
]

describe("snapBrushToBuckets", () => {
  it("snaps a drag range to inclusive bucket edges", () => {
    expect(snapBrushToBuckets({ start: 12, end: 28 }, buckets)).toEqual({
      start: 10,
      end: 30,
    })
  })

  it("normalizes inverted brushes", () => {
    expect(snapBrushToBuckets({ start: 28, end: 12 }, buckets)).toEqual({
      start: 10,
      end: 30,
    })
  })

  it("point brush selects the containing bucket", () => {
    expect(snapBrushToBuckets({ start: 15, end: 15 }, buckets)).toEqual({
      start: 10,
      end: 20,
    })
  })

  it("returns null for empty buckets or miss", () => {
    expect(snapBrushToBuckets({ start: 1, end: 2 }, [])).toBeNull()
    expect(snapBrushToBuckets({ start: 100, end: 110 }, buckets)).toBeNull()
  })
})

describe("pxToTime / timeToPx", () => {
  it("round-trips at the domain ends", () => {
    expect(pxToTime(0, 100, 1_000, 2_000)).toBe(1_000)
    expect(pxToTime(100, 100, 1_000, 2_000)).toBe(2_000)
    expect(timeToPx(1_000, 100, 1_000, 2_000)).toBe(0)
    expect(timeToPx(2_000, 100, 1_000, 2_000)).toBe(100)
  })

  it("clamps out-of-range pixels", () => {
    expect(pxToTime(-10, 100, 0, 100)).toBe(0)
    expect(pxToTime(150, 100, 0, 100)).toBe(100)
  })
})

describe("buildUniformBuckets", () => {
  it("builds ~targetCount buckets covering the window", () => {
    const b = buildUniformBuckets(0, 150, DEFAULT_HISTOGRAM_BUCKETS)
    expect(b).toHaveLength(DEFAULT_HISTOGRAM_BUCKETS)
    expect(b[0]?.start).toBe(0)
    expect(b[b.length - 1]?.end).toBe(150)
    // contiguous
    for (let i = 1; i < b.length; i++) {
      expect(b[i]?.start).toBe(b[i - 1]?.end)
    }
  })

  it("returns empty for non-positive windows", () => {
    expect(buildUniformBuckets(10, 10, 10)).toEqual([])
    expect(buildUniformBuckets(20, 10, 10)).toEqual([])
  })
})
