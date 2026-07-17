import { describe, expect, it } from "vitest"

import { mergeLiveLogs, type LiveLogIdentity } from "@/features/logs/model/merge-live-logs"

function makeLog(i: number): LiveLogIdentity {
  return {
    tsNanos: String(1_000_000_000_000n + BigInt(i)),
    body: `body-${i}`,
    service: i % 7 === 0 ? "svc-a" : "svc-b",
    severity: i % 3 === 0 ? "ERROR" : "INFO",
  }
}

describe("mergeLiveLogs performance", () => {
  it("merges 10k current + 1k incoming under 16ms p95 (canonical Bun)", () => {
    const current = Array.from({ length: 10_000 }, (_, i) => makeLog(i))
    // Half new, half duplicates — worst realistic mix.
    const incoming = [
      ...Array.from({ length: 500 }, (_, i) => makeLog(i)),
      ...Array.from({ length: 500 }, (_, i) => makeLog(10_000 + i)),
    ]

    const samples: number[] = []
    for (let run = 0; run < 25; run += 1) {
      const start = performance.now()
      const result = mergeLiveLogs(current, incoming, 10_000)
      samples.push(performance.now() - start)
      expect(result.items.length).toBeLessThanOrEqual(10_000)
      expect(result.duplicates).toBe(500)
    }

    samples.sort((a, b) => a - b)
    const p95 = samples[Math.floor(samples.length * 0.95)]!
    // Cap at 16ms; CI hosts may be noisier — keep deterministic capacity checks always.
    expect(resultCapacityOk(current, incoming)).toBe(true)
    expect(p95).toBeLessThanOrEqual(16)
  })

  it("preserves reference identity for unchanged current items", () => {
    const a = makeLog(1)
    const b = makeLog(2)
    const current = [b, a]
    const result = mergeLiveLogs(current, [makeLog(3)], 10)
    expect(result.items[1]).toBe(b)
    expect(result.items[2]).toBe(a)
  })
})

function resultCapacityOk(
  current: readonly LiveLogIdentity[],
  incoming: readonly LiveLogIdentity[]
): boolean {
  const result = mergeLiveLogs(current, incoming, 10_000)
  return result.items.length <= 10_000 && result.dropped >= 0
}
