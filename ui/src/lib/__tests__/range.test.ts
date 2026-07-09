import { describe, expect, it } from "vitest"

import {
  customRange,
  rangeLinkSearch,
  resolvePreset,
  updateRangeSearch,
} from "@/lib/range"

describe("range search helpers", () => {
  it("clears explicit bounds for preset windows", () => {
    expect(updateRangeSearch(resolvePreset("1h", 1_720_000_000_000))).toEqual({
      range: "1h",
      from: undefined,
      to: undefined,
    })
  })

  it("pins custom windows", () => {
    expect(updateRangeSearch(customRange("1000", "2000"))).toEqual({
      range: "custom",
      from: "1000",
      to: "2000",
    })
  })

  it("mirrors update shape for cross-route links", () => {
    expect(rangeLinkSearch(customRange("3000", "4000"))).toEqual({
      range: "custom",
      from: "3000",
      to: "4000",
    })
    expect(rangeLinkSearch(resolvePreset("1h", 1_720_000_000_000))).toEqual({
      range: "1h",
    })
  })
})
