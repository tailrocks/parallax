/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { SqlResultBody } from "@/features/sql/components/sql-result-body"
import { renderTestRouter } from "@/test/router"

afterEach(cleanup)

function renderWithRouter(component: React.ReactNode) {
  return renderTestRouter(component, {
    targetPaths: [
      "/traces/$traceId",
      "/invocations/$invocationId",
      "/issues/$fingerprint",
      "/services/$service",
    ],
  })
}

describe("SqlResultBody", () => {
  it("renders truncation notice and linkified cells", async () => {
    const result = {
      columns: [
        "trace_id",
        '"cli.invocation.id"',
        "service_name",
        "fingerprint",
        "span_id",
        "empty",
      ],
      rows: [JSON.stringify(["trace-a", "run-a", "checkout", "fp-a", "span-a", null])],
      rowCount: 1,
      truncated: true,
    }
    const { container } = renderWithRouter(<SqlResultBody result={result} />)

    expect(await screen.findByText(/Result capped at 2,000 rows/)).toBeTruthy()
    const links = Array.from(container.querySelectorAll("a")).map((link) =>
      link.getAttribute("href")
    )
    expect(links).toEqual([
      "/traces/trace-a",
      "/invocations/run-a",
      "/services/checkout",
      "/issues/fp-a",
      "/traces/trace-a",
    ])
    expect(screen.getByText("null").closest("a")).toBeNull()
  })
})
