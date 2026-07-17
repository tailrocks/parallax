import { describe, expect, it } from "vitest"

import { GraphqlBoundaryError } from "@/platform/graphql/error"
import { encodeGraphqlVariables } from "@/platform/graphql/variables"

describe("encodeGraphqlVariables", () => {
  it("sorts object keys and preserves array order", () => {
    const encoded = encodeGraphqlVariables({
      b: 2,
      a: [3, 1, 2],
      c: { z: true, y: null },
    })
    expect(encoded).toBe('{"a":[3,1,2],"b":2,"c":{"y":null,"z":true}}')
  })

  it("omits undefined object properties", () => {
    expect(encodeGraphqlVariables({ a: 1, b: undefined })).toBe('{"a":1}')
  })

  it("rejects undefined root", () => {
    expect(() => encodeGraphqlVariables(undefined)).toThrow(GraphqlBoundaryError)
  })

  it("rejects non-finite numbers", () => {
    expect(() => encodeGraphqlVariables({ n: Number.NaN })).toThrow(GraphqlBoundaryError)
    expect(() => encodeGraphqlVariables({ n: Infinity })).toThrow(GraphqlBoundaryError)
  })

  it("rejects cycles", () => {
    const cyclic: Record<string, unknown> = { a: 1 }
    cyclic["self"] = cyclic
    expect(() => encodeGraphqlVariables(cyclic)).toThrow(GraphqlBoundaryError)
  })
})
