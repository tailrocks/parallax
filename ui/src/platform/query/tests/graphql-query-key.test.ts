// Plan 103 — Query-key identity properties (plan 133 sole cache).

import { describe, expect, it } from "vitest"

import { graphqlOperationQueryKey, graphqlRawQueryKey } from "@/platform/query/graphql-query"
import { encodeGraphqlVariables } from "@/platform/graphql/variables"

describe("graphqlOperationQueryKey identity", () => {
  it("is stable for identical operation + variables key", () => {
    const a = graphqlOperationQueryKey("LogsPage", '{"service":"svc"}')
    const b = graphqlOperationQueryKey("LogsPage", '{"service":"svc"}')
    expect(a).toEqual(b)
    expect(a).toEqual(["graphql", "LogsPage", '{"service":"svc"}'])
  })

  it("differs when operation name or variables change", () => {
    const base = graphqlOperationQueryKey("LogsPage", '{"service":"svc"}')
    expect(graphqlOperationQueryKey("TracesPage", '{"service":"svc"}')).not.toEqual(base)
    expect(graphqlOperationQueryKey("LogsPage", '{"service":"other"}')).not.toEqual(base)
  })

  it("canonical variable encoding is order-stable for object keys", () => {
    // encodeGraphqlVariables must produce the same key for equal objects.
    const left = encodeGraphqlVariables({ b: 1, a: 2 })
    const right = encodeGraphqlVariables({ a: 2, b: 1 })
    expect(left).toBe(right)
    expect(graphqlOperationQueryKey("Op", left)).toEqual(graphqlOperationQueryKey("Op", right))
  })
})

describe("graphqlRawQueryKey identity", () => {
  it("keys by exact query string", () => {
    const q = "{ services }"
    expect(graphqlRawQueryKey(q)).toEqual(["graphql-raw", q])
    expect(graphqlRawQueryKey(q)).toEqual(graphqlRawQueryKey(q))
    expect(graphqlRawQueryKey(q)).not.toEqual(graphqlRawQueryKey("{ services  }"))
  })
})
