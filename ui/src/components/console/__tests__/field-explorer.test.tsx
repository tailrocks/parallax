/* @vitest-environment jsdom */

import { screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { FieldExplorer } from "@/components/console/field-explorer"
import { graphql } from "@/lib/api"
import type { ResolvedRange } from "@/lib/range"
import { renderTestRouter } from "@/test/router"

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal()
  return {
    ...(actual as object),
    graphql: vi.fn(),
  }
})

const range: ResolvedRange = {
  key: "1h",
  fromNanos: "1000",
  toNanos: "2000",
}

afterEach(() => {
  vi.mocked(graphql).mockReset()
})

function mockFieldExplorerQueries() {
  vi.mocked(graphql).mockImplementation(
    async <T,>(query: string): Promise<T> => {
      if (query.includes("fieldKeys")) {
        return {
          fieldKeys: [
            {
              key: "resource.service.name",
              namespace: "service",
              source: "RESOURCE",
              rowCount: "3",
              nonNullCount: "3",
              coverage: 1,
              isIdentifier: false,
            },
            {
              key: "http.request.method",
              namespace: "http",
              source: "SPAN",
              rowCount: "3",
              nonNullCount: "2",
              coverage: 2 / 3,
              isIdentifier: false,
            },
          ],
        } as T
      }
      if (query.includes("fieldStats")) {
        return {
          fieldStats: {
            key: "resource.service.name",
            namespace: "service",
            source: "RESOURCE",
            rowCount: "3",
            nonNullCount: "3",
            distinctCount: "1",
            coverage: 1,
            capped: false,
            isIdentifier: false,
            topValues: [{ value: "checkout", count: "3" }],
          },
        } as T
      }
      throw new Error(`unexpected query ${query}`)
    }
  )
}

describe("FieldExplorer", () => {
  it("loads field keys, stats, service filters, and SQL pivots", async () => {
    const user = userEvent.setup()
    const onApplyService = vi.fn()
    mockFieldExplorerQueries()

    renderTestRouter(
      <FieldExplorer range={range} onApplyService={onApplyService} />,
      { targetPaths: ["/sql"] }
    )

    await user.click(await screen.findByRole("button", { name: /fields/i }))

    expect(await screen.findByText("resource.service.name")).toBeTruthy()
    expect(await screen.findByText("checkout")).toBeTruthy()
    expect(screen.getByText("Coverage")).toBeTruthy()
    expect(
      screen.getByRole("button", { name: /open sql for checkout/i })
    ).toBeTruthy()
    expect(
      screen.getByRole("button", { name: /open exclusion sql for checkout/i })
    ).toBeTruthy()

    await user.click(screen.getByRole("button", { name: /include/i }))
    expect(onApplyService).toHaveBeenCalledWith("checkout")
    expect(vi.mocked(graphql).mock.calls[0]?.[0]).toContain("fieldKeys")
    expect(vi.mocked(graphql).mock.calls[1]?.[0]).toContain("fieldStats")
  })
})
