import { describe, expect, it } from "vitest"

import {
  graphqlWireFixture,
  sseWireFixture,
} from "../../src/test/wire-fixtures"

describe("external wire fixtures", () => {
  it("returns fresh unknown GraphQL characterization values", () => {
    const first: unknown = graphqlWireFixture("data")
    const second: unknown = graphqlWireFixture("data")
    expect(first).toEqual({ data: { services: ["checkout"] } })
    expect(second).toEqual(first)
    expect(second).not.toBe(first)
    expect(graphqlWireFixture("empty")).toEqual({ data: { services: [] } })
    expect(graphqlWireFixture("error")).toEqual({
      data: null,
      errors: [{ extensions: { code: "INTERNAL" }, message: "query failed" }],
    })
    expect(graphqlWireFixture("malformed")).toEqual({
      data: { services: "not-an-array" },
    })
  })

  it("returns fresh unknown SSE data, malformed, reconnect, and completion values", () => {
    const data: unknown = sseWireFixture("data")
    expect(data).toEqual({
      data: '{"items":[{"traceId":"trace-1"}]}',
      event: "message",
    })
    expect(sseWireFixture("malformed")).toEqual({
      data: "{not-json",
      event: "message",
    })
    expect(sseWireFixture("reconnect")).toEqual({ data: "", event: "error" })
    expect(sseWireFixture("complete")).toEqual({
      data: "",
      event: "complete",
    })
    expect(sseWireFixture("data")).not.toBe(data)
  })
})
