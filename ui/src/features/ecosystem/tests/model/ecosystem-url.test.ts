import { describe, expect, it } from "vitest"

import {
  DEFAULT_ECOSYSTEM_URL,
  clampHops,
  decodeEcosystemUrl,
  encodeEcosystemUrl,
  encodeMinTraffic,
  parseFocusMode,
  parseMinTraffic,
} from "@/features/ecosystem/model/ecosystem-url"
import { TRAFFIC_PRESETS } from "@/features/ecosystem/model/ecosystem-topology"

describe("ecosystem URL codec", () => {
  it("defaults when empty", () => {
    expect(decodeEcosystemUrl(new URLSearchParams())).toEqual(
      DEFAULT_ECOSYSTEM_URL
    )
  })

  it("round-trips focus, hops, mode, traffic preset", () => {
    const state = {
      focus: "checkout",
      hops: 2,
      focusMode: "hide" as const,
      minTraffic: TRAFFIC_PRESETS["1%"],
    }
    const encoded = encodeEcosystemUrl(state)
    expect(encoded.get("focus")).toBe("checkout")
    expect(encoded.get("hops")).toBe("2")
    expect(encoded.get("focusMode")).toBe("hide")
    expect(encoded.get("minTraffic")).toBe("1%")
    expect(decodeEcosystemUrl(encoded)).toEqual(state)
  })

  it("omits default hops/mode/traffic from encode", () => {
    const p = encodeEcosystemUrl({
      focus: null,
      hops: 1,
      focusMode: "dim",
      minTraffic: 0,
    })
    expect([...p.keys()]).toEqual([])
  })

  it("parses traffic presets and percent strings", () => {
    expect(parseMinTraffic("all")).toBe(0)
    expect(parseMinTraffic("0.1%")).toBe(TRAFFIC_PRESETS["0.1%"])
    expect(parseMinTraffic("5%")).toBe(TRAFFIC_PRESETS["5%"])
    // bare numbers: ≤1 are fractions; >1 are percents (e.g. 5 → 5%)
    expect(parseMinTraffic("1")).toBeCloseTo(1)
    expect(parseMinTraffic("5")).toBeCloseTo(0.05)
    expect(parseMinTraffic("0.05")).toBeCloseTo(0.05)
    expect(parseMinTraffic("1%")).toBeCloseTo(0.01)
  })

  it("clamps hops and focusMode", () => {
    expect(clampHops(-1)).toBe(0)
    expect(clampHops(9)).toBe(3)
    expect(clampHops(1.9)).toBe(1)
    expect(parseFocusMode("hide")).toBe("hide")
    expect(parseFocusMode("nope")).toBe("dim")
  })

  it("encodeMinTraffic prefers preset labels", () => {
    expect(encodeMinTraffic(0)).toBe("all")
    expect(encodeMinTraffic(0.01)).toBe("1%")
    expect(encodeMinTraffic(0.033)).toBe("3.3%")
  })
})
