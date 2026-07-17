// Plan-103 bounded property suite: where-clause grammar round-trip and
// total-parse safety (docs/research/testing/property-invariants.md).

import fc from "fast-check"
import { describe, expect, it } from "vitest"

import {
  WHERE_OPS,
  parseWhereClause,
  serializeWhereClause,
  type WhereFilter,
} from "@/shared/where-clause"

const identArb = fc
  .tuple(
    fc.constantFrom(..."abcdefghijklmnopqrstuvwxyz_".split("")),
    fc.stringMatching(/^[A-Za-z0-9_.\-/]{0,14}$/)
  )
  .map(([head, tail]) => head + tail)

const filterArb: fc.Arbitrary<WhereFilter> = fc.record({
  key: identArb,
  op: fc.constantFrom(...WHERE_OPS),
  value: fc.string({ maxLength: 20 }),
})

describe("where-clause properties", () => {
  it("serialize→parse round-trips arbitrary filter lists", () => {
    fc.assert(
      fc.property(fc.array(filterArb, { maxLength: 6 }), (filters) => {
        const parsed = parseWhereClause(serializeWhereClause(filters))
        expect(parsed).toEqual({ ok: true, filters })
      }),
      { numRuns: 300 }
    )
  })

  it("parses arbitrary input totally — a typed result, never a throw", () => {
    fc.assert(
      fc.property(fc.string({ maxLength: 60 }), (input) => {
        const parsed = parseWhereClause(input)
        const wellFormed = parsed.ok
          ? true
          : parsed.error.start >= 0 && parsed.error.end >= parsed.error.start
        expect(wellFormed).toBe(true)
      }),
      { numRuns: 300 }
    )
  })
})
