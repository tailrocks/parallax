import { describe, expect, it, vi } from "vitest"

import {
  formatBytes,
  formatCount,
  formatDelta,
  formatDurationNs,
  formatRelative,
  formatTimeShort,
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

  it("formats bytes as IEC units", () => {
    expect(formatBytes(0)).toBe("0 B")
    expect(formatBytes(512)).toBe("512 B")
    expect(formatBytes(3_355_443)).toBe("3.2 MiB")
    expect(formatBytes(5 * 1024 ** 4)).toBe("5.0 TiB")
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

  it("formats short time labels", () => {
    expect(
      formatTimeShort("1719999999999999999", {
        minute: "2-digit",
        second: "2-digit",
      })
    ).toMatch(/\d/)
  })
})
