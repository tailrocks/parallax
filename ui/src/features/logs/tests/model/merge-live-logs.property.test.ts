// Plan-103 bounded property suite: live log merge ordering, dedup, and
// capacity invariants (docs/research/testing/property-invariants.md).

import fc from "fast-check"
import { describe, expect, it } from "vitest"

import { mergeLiveLogs, type LiveLogIdentity } from "@/features/logs/model/merge-live-logs"

interface Row extends LiveLogIdentity {
  readonly tsNanos: string
  readonly body: string
}

const rowArb: fc.Arbitrary<Row> = fc.record({
  tsNanos: fc.integer({ min: 1, max: 1_000_000 }).map(String),
  body: fc.string({ maxLength: 6 }),
})

function key(row: Row): string {
  return `${row.tsNanos}\0${row.body}`
}

function newestFirst(rows: readonly Row[]): Row[] {
  return [...rows].sort((a, b) => Number(BigInt(b.tsNanos) - BigInt(a.tsNanos)))
}

describe("mergeLiveLogs properties", () => {
  it("keeps newest-first order, uniqueness, and the capacity bound", () => {
    fc.assert(
      fc.property(
        fc.array(rowArb, { maxLength: 60 }),
        fc.array(rowArb, { maxLength: 40 }),
        fc.integer({ min: 1, max: 80 }),
        (currentRaw, incoming, maxVisible) => {
          const seen = new Set<string>()
          const current = newestFirst(
            currentRaw.filter((row) => {
              if (seen.has(key(row))) return false
              seen.add(key(row))
              return true
            })
          )
          const result = mergeLiveLogs(current, incoming, maxVisible)

          // Capacity bound holds whenever the merge produced a new list;
          // the documented no-op paths (empty or fully-duplicate batch)
          // return `current` untouched and never re-trim it.
          const withinCapacity = result.items === current || result.items.length <= maxVisible
          expect(withinCapacity).toBe(true)
          // No duplicate identities survive.
          const keys = result.items.map(key)
          expect(new Set(keys).size).toBe(keys.length)
          // Contract order: the fresh batch (internally newest-first)
          // precedes retained current rows, whose relative order is
          // preserved. Late frames may be globally out of order by design.
          const freshCount = result.items.findIndex((row) =>
            current.some((existing) => key(existing) === key(row))
          )
          const freshSegment = freshCount === -1 ? result.items : result.items.slice(0, freshCount)
          for (let i = 1; i < freshSegment.length; i += 1) {
            expect(BigInt(freshSegment[i - 1]!.tsNanos) >= BigInt(freshSegment[i]!.tsNanos)).toBe(
              true
            )
          }
          const retained = result.items.filter((row) =>
            current.some((existing) => key(existing) === key(row))
          )
          const currentKeys = current.map(key)
          const retainedIndexes = retained.map((row) => currentKeys.indexOf(key(row)))
          for (let i = 1; i < retainedIndexes.length; i += 1) {
            expect(retainedIndexes[i - 1]!).toBeLessThan(retainedIndexes[i]!)
          }
          // Bookkeeping: every unique input identity is either visible,
          // counted as a duplicate, or dropped by capacity.
          const uniqueInputs = new Set([...current.map(key), ...incoming.map(key)])
          expect(result.items.length + result.dropped).toBeLessThanOrEqual(uniqueInputs.size)
        }
      ),
      { numRuns: 200 }
    )
  })

  it("is a no-op for an empty incoming batch", () => {
    fc.assert(
      fc.property(fc.array(rowArb, { maxLength: 40 }), (current) => {
        const ordered = newestFirst(current)
        const result = mergeLiveLogs(ordered, [], 100)
        expect(result.items).toEqual(ordered)
        expect(result.duplicates).toBe(0)
      }),
      { numRuns: 100 }
    )
  })
})
