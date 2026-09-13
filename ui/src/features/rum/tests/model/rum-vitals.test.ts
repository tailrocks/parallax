import { describe, expect, it } from "vitest"

import {
  formatVitalValue,
  matchVital,
  normalizeVitalValue,
  rateVital,
} from "@/features/rum/model/rum-vitals"

describe("matchVital", () => {
  it("matches bare vital names", () => {
    expect(matchVital("lcp")).toBe("LCP")
    expect(matchVital("inp")).toBe("INP")
    expect(matchVital("cls")).toBe("CLS")
    expect(matchVital("ttfb")).toBe("TTFB")
  })

  it("matches browser-prefixed and spelled-out names", () => {
    expect(matchVital("browser.lcp")).toBe("LCP")
    expect(matchVital("browser_largest_contentful_paint")).toBe("LCP")
    expect(matchVital("web_vitals.inp.p75")).toBe("INP")
    expect(matchVital("cumulative-layout-shift")).toBe("CLS")
  })

  it("rejects non-vital metrics", () => {
    expect(matchVital("http.server.duration")).toBeNull()
    expect(matchVital("process.cpu.time")).toBeNull()
    expect(matchVital("")).toBeNull()
  })
})

describe("rateVital", () => {
  it("rates LCP against web.dev thresholds", () => {
    expect(rateVital("LCP", 2000)).toBe("good")
    expect(rateVital("LCP", 3000)).toBe("needs-improvement")
    expect(rateVital("LCP", 5000)).toBe("poor")
  })

  it("rates CLS as a unitless score", () => {
    expect(rateVital("CLS", 0.05)).toBe("good")
    expect(rateVital("CLS", 0.2)).toBe("needs-improvement")
    expect(rateVital("CLS", 0.3)).toBe("poor")
  })
})

describe("normalizeVitalValue", () => {
  it("scales seconds to milliseconds", () => {
    expect(normalizeVitalValue(2.5, "s")).toBe(2500)
    expect(normalizeVitalValue(2500, "ms")).toBe(2500)
    expect(normalizeVitalValue(2500, null)).toBe(2500)
  })
})

describe("formatVitalValue", () => {
  it("formats time vitals and CLS scores", () => {
    expect(formatVitalValue("LCP", 2500)).toBe("2.50s")
    expect(formatVitalValue("INP", 150)).toBe("150ms")
    expect(formatVitalValue("CLS", 0.1234)).toBe("0.123")
  })
})
