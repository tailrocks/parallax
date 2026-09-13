import { describe, expect, it } from "vitest"

import { dominantDbQueries, normalizeDbQuery } from "@/features/traces/model/dominant-db"

describe("dominantDbQueries", () => {
  it("groups normalized SQL and ranks by total duration", () => {
    const ranked = dominantDbQueries([
      {
        spanId: "a",
        service: "checkout",
        durationNs: "10",
        attributes: JSON.stringify({ "db.query.text": "SELECT * FROM orders WHERE id = 1" }),
      },
      {
        spanId: "b",
        service: "checkout",
        durationNs: "20",
        attributes: JSON.stringify({ "db.query.text": "SELECT * FROM orders WHERE id = 99" }),
      },
      {
        spanId: "c",
        service: "inventory",
        durationNs: "100",
        attributes: JSON.stringify({ "db.query.text": "SELECT * FROM inventory" }),
      },
      {
        spanId: "http",
        service: "checkout",
        durationNs: "50",
        attributes: "{}",
      },
    ])
    expect(ranked).toHaveLength(2)
    expect(ranked[0]?.example).toBe("SELECT * FROM inventory")
    expect(ranked[0]?.count).toBe(1)
    expect(ranked[1]?.count).toBe(2)
    expect(ranked[1]?.exampleSpanId).toBe("b")
    expect(normalizeDbQuery("SELECT * FROM orders WHERE id = 1")).toContain("<n>")
  })
})
