import { describe, expect, it } from "vitest"

import {
  parseWhereClause,
  quoteWhereValue,
  serializeWhereClause,
  type WhereFilter,
} from "@/shared/where-clause"

describe("where-clause reserved words", () => {
  it("reserved words in value position keep original spelling", () => {
    expect(parseWhereClause("a = nOt")).toEqual({
      ok: true,
      filters: [{ key: "a", op: "=", value: "nOt" }],
    })
    expect(parseWhereClause("a = AND")).toEqual({
      ok: true,
      filters: [{ key: "a", op: "=", value: "AND" }],
    })
    expect(parseWhereClause("body CONTAINS contains")).toEqual({
      ok: true,
      filters: [{ key: "body", op: "CONTAINS", value: "contains" }],
    })
  })

  it("reserved words in key position stay keys", () => {
    expect(parseWhereClause("nOt = 1")).toEqual({
      ok: true,
      filters: [{ key: "nOt", op: "=", value: "1" }],
    })
    expect(parseWhereClause("AND != 2")).toEqual({
      ok: true,
      filters: [{ key: "AND", op: "!=", value: "2" }],
    })
  })

  it("reserved-word values quoted (case-insensitive)", () => {
    expect(quoteWhereValue("nOt")).toBe('"nOt"')
    expect(quoteWhereValue("AND")).toBe('"AND"')
    expect(quoteWhereValue("contains")).toBe('"contains"')
  })

  it("serialize→parse reserved-word keys and values", () => {
    const cases: WhereFilter[][] = [
      [{ key: "a", op: "=", value: "nOt" }],
      [{ key: "nOt", op: "=", value: "and" }],
      [{ key: "contains", op: "CONTAINS", value: "NOT" }],
      [{ key: "AND", op: "!=", value: "CONTAINS" }],
    ]
    for (const filters of cases) {
      expect(parseWhereClause(serializeWhereClause(filters))).toEqual({
        ok: true,
        filters,
      })
    }
  })
})
