import { describe, expect, it } from "vitest"

import {
  DEFAULT_LOG_DENSITY,
  decodePinnedColumns,
  encodeLogWrap,
  encodePinnedColumns,
  logDensityClass,
  parseLogDensity,
  parseLogWrap,
  pinColumn,
  togglePinnedColumn,
  unpinColumn,
} from "@/lib/log-table-prefs"

describe("pinned columns URL codec", () => {
  it("decodes CSV and drops empties/dupes", () => {
    expect(decodePinnedColumns("http.route, service.name, http.route")).toEqual([
      "http.route",
      "service.name",
    ])
    expect(decodePinnedColumns("  ,  ")).toEqual([])
    expect(decodePinnedColumns(null)).toEqual([])
  })

  it("round-trips encode/decode", () => {
    const keys = ["http.route", "db.system"]
    expect(decodePinnedColumns(encodePinnedColumns(keys))).toEqual(keys)
  })

  it("pin/unpin/toggle", () => {
    expect(pinColumn(["a"], "b")).toEqual(["a", "b"])
    expect(pinColumn(["a"], "a")).toEqual(["a"])
    expect(unpinColumn(["a", "b"], "a")).toEqual(["b"])
    expect(togglePinnedColumn(["a"], "b")).toEqual(["a", "b"])
    expect(togglePinnedColumn(["a", "b"], "a")).toEqual(["b"])
  })
})

describe("density and wrap", () => {
  it("parses density with default", () => {
    expect(parseLogDensity("compact")).toBe("compact")
    expect(parseLogDensity("comfortable")).toBe("comfortable")
    expect(parseLogDensity("nope")).toBe(DEFAULT_LOG_DENSITY)
    expect(parseLogDensity(null)).toBe(DEFAULT_LOG_DENSITY)
  })

  it("maps density to class names", () => {
    expect(logDensityClass("compact")).toBe("log-rows-compact")
    expect(logDensityClass("comfortable")).toBe("log-rows-comfortable")
  })

  it("parses and encodes wrap", () => {
    expect(parseLogWrap("1")).toBe(true)
    expect(parseLogWrap("true")).toBe(true)
    expect(parseLogWrap("0")).toBe(false)
    expect(parseLogWrap(null)).toBe(false)
    expect(encodeLogWrap(true)).toBe("1")
    expect(encodeLogWrap(false)).toBe("0")
  })
})
