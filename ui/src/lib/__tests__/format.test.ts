import { describe, expect, it, vi } from "vitest"

import {
  formatBytes,
  formatCount,
  formatDelta,
  formatDurationNs,
  formatLogBodyPreview,
  formatRelative,
  formatTimeShort,
  formatTimeInRange,
  stripAnsi,
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
    const label = formatTimeInRange("2000000000", {
      fromNanos: "1000000000",
      toNanos: "172801000000000",
      timeZone: "UTC",
    })
    expect(label).toBe("Jan 1, 00:00:02")
  })

  it("formats short time labels", () => {
    expect(
      formatTimeShort("2000000000", {
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
        timeZone: "UTC",
      })
    ).toBe("00:02")
    expect(
      formatTimeInRange("2000000000", {
        fromNanos: "1000000000",
        toNanos: "3000000000",
        timeZone: "UTC",
      })
    ).toBe("00:00:02")
  })

  it("returns consistent output across repeated formatter calls", () => {
    const options = {
      minute: "2-digit" as const,
      second: "2-digit" as const,
      hour12: false,
      timeZone: "UTC",
    }
    const first = formatTimeShort("2000000000", options)
    const second = formatTimeShort("2000000000", options)
    expect(first).toBe(second)
    expect(first).toBe("00:02")
  })
})

describe("log body preview (plan 160, corpus l-bodies)", () => {
  it("D-008: strips ANSI escapes instead of rendering raw bytes", () => {
    expect(stripAnsi("\u001b[31merror\u001b[0m with \u001b[1mANSI\u001b[0m")).toBe(
      "error with ANSI"
    )
  })

  it("D-009: caps oversized bodies with an explicit size hint", () => {
    const body = `oversized body: ${"x".repeat(40_000)}`
    const preview = formatLogBodyPreview(body)
    expect(preview.length).toBeLessThan(600)
    expect(preview).toContain("\u2026")
    expect(preview).toContain("chars")
  })

  it("leaves short clean bodies untouched", () => {
    expect(formatLogBodyPreview("hello")).toBe("hello")
  })
})
