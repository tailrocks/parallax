import { describe, expect, it, vi } from "vitest"

import {
  formatCount,
  formatDelta,
  formatDurationNs,
  formatRelative,
  formatTimeInRange,
} from "@/lib/format"

describe("formatters", () => {
  it("formats durations across units", () => {
    expect(formatDurationNs(950_000)).toBe("950µs")
    expect(formatDurationNs(12_300_000)).toBe("12ms")
    expect(formatDurationNs(1_240_000_000)).toBe("1.24s")
    expect(formatDurationNs(134_000_000_000)).toBe("2m 14s")
  })

  it("formats counts and deltas", () => {
    expect(formatCount(12_340)).toBe("12k")
    expect(formatCount(1_200_000)).toBe("1.2M")
    expect(formatDelta(120, 100)).toEqual({ dir: "up", pct: 20 })
  })

  it("formats relative ns strings with precision", () => {
    vi.spyOn(Date, "now").mockReturnValue(1_720_000_000_000)
    expect(formatRelative("1719999999999999999")).toBe("0s ago")
    vi.restoreAllMocks()
  })

  it("uses date labels for multi-day ranges", () => {
    const label = formatTimeInRange("1719999999999999999", {
      fromNanos: "1719400000000000000",
      toNanos: "1720000000000000000",
    })
    expect(label).toMatch(/\d/)
  })
})
