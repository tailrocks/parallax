import { describe, expect, it } from "vitest"

import { traceDetailQuery } from "@/features/traces/api/trace-detail-query"

describe("traceDetailQuery", () => {
  it("selects Trace.dominantDbQueries from GraphQL, not a client re-rank", () => {
    const query = traceDetailQuery("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    expect(query).toContain('trace(traceId: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")')
    expect(query).toContain("dominantDbQueries")
    expect(query).toContain("exampleSpanId")
    expect(query).toContain("totalNs")
    expect(query).toContain("normalized")
  })
})
