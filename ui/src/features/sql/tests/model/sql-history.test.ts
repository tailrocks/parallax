import { describe, expect, it } from "vitest"

import {
  parseHistoryWire,
  pushHistoryEntry,
  SQL_HISTORY_CAP,
} from "@/features/sql/model/sql-history"

describe("sql history", () => {
  it("returns empty for absent, malformed, and non-array wire", () => {
    expect(parseHistoryWire(null)).toEqual([])
    expect(parseHistoryWire("not-json")).toEqual([])
    expect(parseHistoryWire("{}")).toEqual([])
  })

  it("preserves mixed-array members without filtering", () => {
    const mixed = parseHistoryWire(JSON.stringify(["a", 1, null]))
    expect(mixed).toEqual(["a", 1, null])
  })

  it("dedupes, orders most-recent-first, and caps at 20", () => {
    const base = Array.from({ length: 20 }, (_, i) => `q${i}`)
    const next = pushHistoryEntry(base, "fresh")
    expect(next[0]).toBe("fresh")
    expect(next).toHaveLength(SQL_HISTORY_CAP)
    expect(next).not.toContain("q19")
    expect(pushHistoryEntry(["a", "b"], "b")).toEqual(["b", "a"])
  })
})
