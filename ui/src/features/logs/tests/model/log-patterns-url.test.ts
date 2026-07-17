import { describe, expect, it } from "vitest"

import {
  DEFAULT_LOG_PATTERNS_URL,
  decodeLogPatternsUrl,
  encodeLogPatternsUrl,
  encodePatternsFlag,
  mergeLogPatternsParams,
  parsePatternsFlag,
} from "@/features/logs/model/log-patterns-url"

describe("log patterns URL codec", () => {
  it("defaults when empty", () => {
    expect(decodeLogPatternsUrl(new URLSearchParams())).toEqual(DEFAULT_LOG_PATTERNS_URL)
  })

  it("parses patterns flag variants", () => {
    expect(parsePatternsFlag("1")).toBe(true)
    expect(parsePatternsFlag("true")).toBe(true)
    expect(parsePatternsFlag("YES")).toBe(true)
    expect(parsePatternsFlag("0")).toBe(false)
    expect(parsePatternsFlag(null)).toBe(false)
    expect(encodePatternsFlag(true)).toBe("1")
  })

  it("round-trips patterns + template", () => {
    const state = {
      patterns: true,
      patternTemplate: "checkout authorize user=<*>",
    }
    const enc = encodeLogPatternsUrl(state)
    expect(enc.get("patterns")).toBe("1")
    expect(enc.get("pattern")).toBe("checkout authorize user=<*>")
    expect(decodeLogPatternsUrl(enc)).toEqual(state)
  })

  it("omits defaults from encode", () => {
    const enc = encodeLogPatternsUrl({ patterns: false, patternTemplate: null })
    expect([...enc.keys()]).toEqual([])
  })

  it("does not encode template when patterns is off", () => {
    const enc = encodeLogPatternsUrl({
      patterns: false,
      patternTemplate: "should-not-appear",
    })
    expect(enc.get("pattern")).toBeNull()
  })

  it("merges onto existing params without clobbering unrelated keys", () => {
    const base = new URLSearchParams({ service: "checkout", range: "1h" })
    const merged = mergeLogPatternsParams(base, {
      patterns: true,
      patternTemplate: "t <*>",
    })
    expect(merged.get("service")).toBe("checkout")
    expect(merged.get("range")).toBe("1h")
    expect(merged.get("patterns")).toBe("1")
    expect(merged.get("pattern")).toBe("t <*>")
    // turning off clears pattern keys
    const off = mergeLogPatternsParams(merged, {
      patterns: false,
      patternTemplate: null,
    })
    expect(off.get("patterns")).toBeNull()
    expect(off.get("pattern")).toBeNull()
    expect(off.get("service")).toBe("checkout")
  })
})
