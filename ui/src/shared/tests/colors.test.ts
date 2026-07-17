import { describe, expect, it } from "vitest"

import {
  goldenAngleColor,
  seriesColor,
  serviceColor,
  severityColor,
  severityToken,
} from "@/shared/colors"

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
    const colors = names.map((name) => serviceColor(name).color)
    // Slots are 15° apart (or a different lightness tier): distinct color
    // strings guarantee a readable difference.
    expect(new Set(colors).size).toBe(names.length)
  })

  it("snaps hues to 15-degree slots so neighbors stay readable", () => {
    expect(serviceColor("checkout").hue % 15).toBe(0)
    expect(serviceColor("pricing").hue % 15).toBe(0)
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
