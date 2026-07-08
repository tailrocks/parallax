import { describe, expect, it } from "vitest"

import {
  buildHeatScale,
  percentileBucket,
} from "@/components/console/heat-cell"

function bucket(value: number, values: number[]) {
  return percentileBucket(value, buildHeatScale(values))
}

describe("heat-cell percentile buckets", () => {
  it("handles empty and single-value scales", () => {
    expect(bucket(10, [])).toBe(0)
    expect(bucket(10, [10])).toBe(0)
    expect(bucket(12, [10])).toBe(0)
  })

  it("filters non-finite values", () => {
    expect(bucket(10, [Number.NaN, 5, Infinity, 10])).toBe(4)
  })

  it("keeps ties at the first matching rank", () => {
    expect(bucket(2, [1, 2, 2, 4])).toBe(1)
  })

  it("clamps below min and above max", () => {
    const values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    expect(bucket(0, values)).toBe(0)
    expect(bucket(11, values)).toBe(4)
  })

  it("preserves exact bucket boundaries", () => {
    const values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    expect(bucket(1, values)).toBe(0)
    expect(bucket(3, values)).toBe(1)
    expect(bucket(5, values)).toBe(2)
    expect(bucket(8, values)).toBe(3)
    expect(bucket(10, values)).toBe(4)
  })
})
