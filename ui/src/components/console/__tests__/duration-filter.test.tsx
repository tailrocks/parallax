import { describe, expect, it } from "vitest"

import { durationSummary } from "@/components/console/duration-filter"

describe("durationSummary", () => {
  it("empty range yields no summary chip", () => {
    expect(durationSummary({})).toBeNull()
  })

  it("min-only renders a lower bound", () => {
    expect(durationSummary({ minMs: 12 })).toBe("≥ 12ms")
    expect(durationSummary({ minMs: 1500 })).toBe("≥ 1.50s")
  })

  it("max-only renders an upper bound", () => {
    expect(durationSummary({ maxMs: 250 })).toBe("≤ 250ms")
  })

  it("min and max render a range", () => {
    expect(durationSummary({ minMs: 10, maxMs: 100 })).toBe("10ms – 100ms")
  })

  it("sub-millisecond values fall through to microseconds", () => {
    expect(durationSummary({ minMs: 0.5 })).toBe("≥ 500µs")
  })
})
