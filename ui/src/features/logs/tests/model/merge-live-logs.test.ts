import { describe, expect, it } from "vitest"

import { mergeLiveLogs } from "@/features/logs/model/merge-live-logs"

describe("mergeLiveLogs", () => {
  it("prepends ordered fresh items without mutating incoming", () => {
    const current = [{ tsNanos: "20", body: "old" }]
    const incoming = [
      { tsNanos: "10", body: "a" },
      { tsNanos: "30", body: "b" },
    ]
    const copy = [...incoming]
    const result = mergeLiveLogs(current, incoming, 10)
    expect(incoming).toEqual(copy)
    expect(result.items.map((row) => row.body)).toEqual(["b", "a", "old"])
    expect(result.duplicates).toBe(0)
  })

  it("dedupes by identity and respects capacity", () => {
    const current = [{ tsNanos: "20", body: "same" }]
    const incoming = [
      { tsNanos: "20", body: "same" },
      { tsNanos: "40", body: "new" },
      { tsNanos: "50", body: "newer" },
    ]
    const result = mergeLiveLogs(current, incoming, 2)
    expect(result.duplicates).toBe(1)
    expect(result.items).toHaveLength(2)
    expect(result.items[0]?.body).toBe("newer")
    expect(result.dropped).toBe(1)
  })
})
