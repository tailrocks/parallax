import { describe, expect, it } from "vitest"

import { buildGraphqlOperations } from "@/lib/graphql-trace"
import type { GraphqlTraceSpan } from "@/lib/graphql-trace"

function span(
  spanId: string,
  parentSpanId: string | null,
  attributes: Record<string, unknown>,
  overrides: Partial<GraphqlTraceSpan> = {}
): GraphqlTraceSpan {
  return {
    spanId,
    parentSpanId,
    tsNanos: "0",
    durationNs: "100",
    name: spanId,
    statusCode: "STATUS_CODE_UNSET",
    attributes: JSON.stringify(attributes),
    ...overrides,
  }
}

describe("graphql trace builder", () => {
  it("builds a nested field tree with self durations", () => {
    const operations = buildGraphqlOperations([
      span(
        "op",
        null,
        {
          "graphql.operation.type": "query",
          "graphql.operation.name": "GetProducts",
          "graphql.document": "query GetProducts { products { reviews } }",
        },
        { durationNs: "1000" }
      ),
      span(
        "products",
        "op",
        {
          "graphql.field.name": "products",
          "graphql.field.path": "products",
        },
        { durationNs: "500", tsNanos: "10" }
      ),
      span(
        "reviews",
        "products",
        {
          "graphql.field.name": "reviews",
          "graphql.field.path": "products.reviews",
        },
        { durationNs: "200", tsNanos: "20" }
      ),
    ])

    expect(operations).toHaveLength(1)
    expect(operations[0]).toMatchObject({
      operationSpanId: "op",
      operationType: "query",
      operationName: "GetProducts",
      document: "query GetProducts { products { reviews } }",
      fieldErrors: 0,
    })
    const root = operations[0]!.roots[0]!
    expect(root.path).toBe("products")
    expect(root.selfDurationNs).toBe(300n)
    expect(root.children[0]).toMatchObject({
      path: "products.reviews",
      fieldName: "reviews",
      selfDurationNs: 200n,
    })
  })

  it("merges repeated sibling paths into one N+1 node", () => {
    const fields = Array.from({ length: 8 }, (_, index) =>
      span(
        `review-${index}`,
        "op",
        {
          "graphql.field.name": "reviews",
          "graphql.field.path": "products.reviews",
        },
        {
          durationNs: String((index + 1) * 10),
          tsNanos: String(index),
        }
      )
    )

    const operations = buildGraphqlOperations([
      span("op", null, { "graphql.operation.type": "query" }),
      ...fields,
    ])

    expect(operations[0]!.roots).toHaveLength(1)
    expect(operations[0]!.roots[0]).toMatchObject({
      path: "products.reviews",
      callCount: 8,
      spanId: "review-7",
      durationNs: 360n,
    })
  })

  it("counts partial field errors under a successful operation", () => {
    const operations = buildGraphqlOperations([
      span("op", null, { "graphql.operation.type": "query" }),
      span(
        "price",
        "op",
        {
          "graphql.field.name": "price",
          "graphql.field.path": "product.price",
        },
        { statusCode: "STATUS_CODE_ERROR" }
      ),
    ])

    expect(operations[0]!.fieldErrors).toBe(1)
    expect(operations[0]!.roots[0]).toMatchObject({
      hasError: true,
      fieldName: "price",
    })
  })

  it("ignores spans without graphql operation attributes", () => {
    const operations = buildGraphqlOperations([
      span("http", null, { "http.route": "/graphql" }),
      span("resolver", "http", { "graphql.field.name": "products" }),
    ])

    expect(operations).toEqual([])
  })

  it("sorts roots by start time then path", () => {
    const operations = buildGraphqlOperations([
      span("op", null, { "graphql.operation.type": "query" }),
      span(
        "b",
        "op",
        {
          "graphql.field.name": "b",
          "graphql.field.path": "b",
        },
        { tsNanos: "20" }
      ),
      span(
        "c",
        "op",
        {
          "graphql.field.name": "c",
          "graphql.field.path": "c",
        },
        { tsNanos: "10" }
      ),
      span(
        "a",
        "op",
        {
          "graphql.field.name": "a",
          "graphql.field.path": "a",
        },
        { tsNanos: "10" }
      ),
    ])

    expect(operations[0]!.roots.map((node) => node.path)).toEqual([
      "a",
      "c",
      "b",
    ])
  })
})
