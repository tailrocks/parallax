import { describe, expect, it } from "vitest"

import { groupSchemaRows, parseResultRow, parseSchemaRow } from "@/features/sql/model/sql-row"

describe("sql row parse", () => {
  it("maps malformed and non-array rows to empty cells", () => {
    expect(parseResultRow("not-json")).toEqual([])
    expect(parseResultRow("{}")).toEqual([])
    expect(parseResultRow(JSON.stringify(["a", 1]))).toEqual(["a", "1"])
  })

  it("skips falsey schema cells and groups valid rows", () => {
    expect(parseSchemaRow("bad")).toBeNull()
    expect(parseSchemaRow(JSON.stringify([null, "c", "t"]))).toBeNull()
    const grouped = groupSchemaRows([
      JSON.stringify(["t1", "c1", "STRING"]),
      JSON.stringify(["t1", "c2", "INT"]),
      JSON.stringify(["t2", "c1", "BOOL"]),
      "not-json",
    ])
    expect([...grouped.keys()]).toEqual(["t1", "t2"])
    expect(grouped.get("t1")).toEqual([
      { name: "c1", dataType: "STRING" },
      { name: "c2", dataType: "INT" },
    ])
  })
})
