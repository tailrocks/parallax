/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"

import { TraceGraphqlSection } from "@/features/traces"
import type { GraphqlOperation } from "@/features/traces/model/graphql-operations"

const operation: GraphqlOperation = {
  operationSpanId: "op",
  operationType: "query",
  operationName: "GetProducts",
  document: null,
  durationNs: 1_000_000n,
  fieldErrors: 0,
  roots: [
    {
      path: "products",
      fieldName: "products",
      spanId: "products",
      durationNs: 500_000n,
      selfDurationNs: 500_000n,
      hasError: false,
      callCount: 1,
      children: [],
    },
  ],
}

describe("TraceGraphqlSection", () => {
  it("renders the GraphQL card only when operations exist", () => {
    const empty = render(<TraceGraphqlSection operations={[]} onSelect={vi.fn()} />)
    expect(empty.container.textContent).toBe("")
    empty.unmount()

    render(<TraceGraphqlSection operations={[operation]} onSelect={vi.fn()} />)

    expect(screen.getByText("GraphQL")).toBeTruthy()
    expect(screen.getByText("GetProducts")).toBeTruthy()
  })
})
