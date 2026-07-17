import { describe, expect, it } from "vitest"

import { mergeLiveSpans, type LiveSpanIdentity } from "@/features/traces/model/merge-live-spans"

function span(id: string, start: string): LiveSpanIdentity {
  return { spanId: id, startNanos: start }
}

describe("mergeLiveSpans", () => {
  it("dedupes by spanId and prepends freshest", () => {
    const current = [span("a", "10"), span("b", "5")]
    const incoming = [span("a", "10"), span("c", "20")]
    const result = mergeLiveSpans(current, incoming, 10)
    expect(result.duplicates).toBe(1)
    expect(result.items.map((row) => row.spanId)).toEqual(["c", "a", "b"])
    expect(result.items[1]).toBe(current[0])
  })

  it("enforces capacity drop from the tail", () => {
    const current = [span("a", "3"), span("b", "2")]
    const result = mergeLiveSpans(current, [span("c", "9")], 2)
    expect(result.items).toHaveLength(2)
    expect(result.items[0]!.spanId).toBe("c")
    expect(result.dropped).toBe(1)
  })

  it("does not mutate incoming", () => {
    const incoming = [span("z", "1"), span("y", "2")]
    const snapshot = [...incoming]
    mergeLiveSpans([], incoming, 10)
    expect(incoming).toEqual(snapshot)
  })
})
