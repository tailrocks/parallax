import { describe, expect, it } from "vitest"

import { mergeLiveSpans, type LiveSpanIdentity } from "@/features/traces/model/merge-live-spans"

function span(id: string, start: string): LiveSpanIdentity {
  return { spanId: id, startNanos: start }
}

function makeSpan(i: number): LiveSpanIdentity {
  return {
    spanId: `span-${i}`,
    startNanos: String(1_000_000_000_000n + BigInt(i)),
  }
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

  it("merges 10k current + 1k incoming under 16ms p95 (canonical Bun)", () => {
    const current = Array.from({ length: 10_000 }, (_, i) => makeSpan(i))
    const incoming = [
      ...Array.from({ length: 500 }, (_, i) => makeSpan(i)),
      ...Array.from({ length: 500 }, (_, i) => makeSpan(10_000 + i)),
    ]

    const samples: number[] = []
    for (let run = 0; run < 25; run += 1) {
      const start = performance.now()
      const result = mergeLiveSpans(current, incoming, 10_000)
      samples.push(performance.now() - start)
      expect(result.items.length).toBeLessThanOrEqual(10_000)
      expect(result.duplicates).toBe(500)
    }

    samples.sort((a, b) => a - b)
    const p95 = samples[Math.floor(samples.length * 0.95)]!
    expect(p95).toBeLessThanOrEqual(16)
  })
})
