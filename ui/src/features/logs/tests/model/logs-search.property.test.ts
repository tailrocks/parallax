// Plan-103 bounded property suite: logs URL-search acceptance and
// serialization invariants (docs/research/testing/property-invariants.md).

import fc from "fast-check"
import { describe, expect, it } from "vitest"

import {
  parseSavedViewState,
  serializeLogsSearch,
  validateLogsSearch,
  type LogsSearch,
} from "@/features/logs/model/logs-search"

const text = fc.string({ minLength: 1, maxLength: 24 })

const searchArb: fc.Arbitrary<LogsSearch> = fc.record(
  {
    q: text,
    service: text,
    sev: fc.constantFrom(5, 9, 13, 17),
    where: text,
    range: fc.constantFrom("15m", "1h", "24h", "7d", "custom"),
    from: fc.bigInt({ min: 1n, max: 10n ** 19n }).map(String),
    to: fc.bigInt({ min: 1n, max: 10n ** 19n }).map(String),
    live: fc.constant(true),
    cols: text,
    patterns: fc.constant(true),
    anchor: fc.bigInt({ min: 1n, max: 10n ** 19n }).map(String),
  },
  { requiredKeys: [] }
)

function normalized(search: LogsSearch): LogsSearch {
  // The validator's canonical form: absent optionals stay undefined and
  // `live` collapses to true/false semantics via serialization.
  return validateLogsSearch({ ...search } as Record<string, unknown>)
}

describe("logs search properties", () => {
  it("serialize→parse round-trips every accepted search", () => {
    fc.assert(
      fc.property(searchArb, (search) => {
        const canonical = normalized(search)
        const reparsed = parseSavedViewState(serializeLogsSearch(canonical))
        expect(reparsed).toEqual(canonical)
      }),
      { numRuns: 200 }
    )
  })

  it("never throws and is idempotent on arbitrary junk records", () => {
    const junk = fc.dictionary(
      fc.constantFrom(
        "q",
        "service",
        "sev",
        "where",
        "range",
        "from",
        "to",
        "live",
        "cols",
        "patterns",
        "anchor",
        "unknown"
      ),
      fc.anything()
    )
    fc.assert(
      fc.property(junk, (record) => {
        const once = validateLogsSearch(record)
        const twice = validateLogsSearch({ ...once } as Record<string, unknown>)
        expect(twice).toEqual(once)
      }),
      { numRuns: 200 }
    )
  })
})
