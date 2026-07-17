import { describe, expect, it } from "vitest"

import {
  goldenAngleColor,
  seriesColor,
  serviceColor,
  severityColor,
  severityToken,
} from "@/lib/colors"

describe("service identity color (plan 162)", () => {
  it("is deterministic: same name, same color, case/space-insensitive", () => {
    expect(serviceColor("checkout")).toEqual(serviceColor("checkout"))
    expect(serviceColor("Checkout").hue).toBe(serviceColor(" checkout ").hue)
  })

  it("distinguishes the playground service set", () => {
    const names = [
      "checkout",
      "inventory",
      "pricing",
      "payment",
      "orders",
      "recommendation",
      "storefront",
      "notifications",
      "web",
      "playground-cli",
    ]
    const hues = names.map((name) => serviceColor(name).hue)
    for (let a = 0; a < hues.length; a += 1) {
      for (let b = a + 1; b < hues.length; b += 1) {
        const distance = Math.min(
          Math.abs(hues[a]! - hues[b]!),
          360 - Math.abs(hues[a]! - hues[b]!)
        )
        expect(distance).toBeGreaterThanOrEqual(2)
      }
    }
  })
})

describe("golden-angle fallback", () => {
  it("keeps 32 series pairwise distinguishable", () => {
    const colors = Array.from({ length: 32 }, (_, i) => goldenAngleColor(i))
    expect(new Set(colors).size).toBe(32)
  })
})

describe("severity ramp", () => {
  it("normalizes aliases onto the six tokens", () => {
    expect(severityToken("WARNING")).toBe("warn")
    expect(severityToken("critical")).toBe("fatal")
    expect(severityToken("Information")).toBe("info")
    expect(severityToken("nonsense")).toBeNull()
  })

  it("resolves ramp tokens to theme variables", () => {
    expect(severityColor("error")).toBe("var(--severity-error)")
  })
})

describe("semantic series detection", () => {
  it("maps severities, status classes, and percentiles", () => {
    expect(seriesColor("ERROR", 3)).toBe("var(--severity-error)")
    expect(seriesColor("5xx", 0)).toBe("var(--chart-error)")
    expect(seriesColor("4xx", 0)).toBe("var(--chart-p95)")
    expect(seriesColor("2xx", 0)).toBe("var(--chart-p50)")
    expect(seriesColor("p95", 0)).toBe("var(--chart-p95)")
    expect(seriesColor("throughput", 0)).toBe("var(--chart-throughput)")
  })

  it("falls back to golden-angle for unknown series", () => {
    expect(seriesColor("custom-series", 7)).toBe(goldenAngleColor(7))
  })
})
