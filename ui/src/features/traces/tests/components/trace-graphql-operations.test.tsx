/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { GraphqlOperationCard } from "@/features/traces/components/trace-graphql-operations"
import type { GraphqlOperation } from "@/features/traces/model/graphql-operations"

afterEach(cleanup)

const operation: GraphqlOperation = {
  operationSpanId: "op",
  operationType: "query",
  operationName: "GetProducts",
  document: "query GetProducts { products { reviews } }",
  durationNs: 1_000_000n,
  fieldErrors: 1,
  roots: [
    {
      path: "products.reviews",
      fieldName: "reviews",
      spanId: "slow-review",
      durationNs: 800_000n,
      selfDurationNs: 800_000n,
      hasError: true,
      callCount: 8,
      children: [],
    },
  ],
}

describe("GraphqlOperationCard", () => {
  it("renders operation badges and selects the backing span", async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()

    render(
      <GraphqlOperationCard operations={[operation]} onSelect={onSelect} />
    )

    expect(screen.getByText("query")).toBeTruthy()
    expect(screen.getByText("GetProducts")).toBeTruthy()
    expect(screen.getByText("1 field error")).toBeTruthy()
    expect(screen.getByText("x8")).toBeTruthy()
    expect(screen.getByText("error")).toBeTruthy()
    expect(screen.getByText("Partial field errors")).toBeTruthy()

    await user.click(screen.getByRole("button", { name: /reviews/i }))
    expect(onSelect).toHaveBeenCalledWith("slow-review")
  })

  it("renders nothing for non-graphql traces", () => {
    const { container } = render(
      <GraphqlOperationCard operations={[]} onSelect={vi.fn()} />
    )

    expect(container.textContent).toBe("")
  })
})
