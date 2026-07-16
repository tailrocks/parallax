/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  EXAMPLES,
  Route as SqlRoute,
  SnippetsMenu,
  SqlResultBody,
  targetForCell,
} from "@/routes/sql"
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

describe("SQL result helpers", () => {
  it("keeps SQL examples on real table names", () => {
    const banned = /\botel_spans\b|\botel_logs\b|\botel_metrics_points\b/
    for (const example of EXAMPLES) {
      expect(example.sql).not.toMatch(banned)
    }
  })

  it("maps supported id columns to route targets", () => {
    expect(targetForCell("trace_id", "trace-a", {})).toEqual({
      to: "/traces/$traceId",
      params: { traceId: "trace-a" },
    })
    expect(targetForCell("span_id", "span-a", { trace_id: "trace-a" })).toEqual(
      {
        to: "/traces/$traceId",
        params: { traceId: "trace-a" },
      }
    )
    expect(targetForCell('"cli.invocation.id"', "run-a", {})).toEqual({
      to: "/invocations/$invocationId",
      params: { invocationId: "run-a" },
    })
    expect(targetForCell("invocation_id", "run-b", {})).toEqual({
      to: "/invocations/$invocationId",
      params: { invocationId: "run-b" },
    })
    expect(targetForCell("span_id", "span-a", {})).toBeNull()
    expect(targetForCell("trace_id", "null", {})).toBeNull()
  })

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
      rows: [
        JSON.stringify([
          "trace-a",
          "run-a",
          "checkout",
          "fp-a",
          "span-a",
          null,
        ]),
      ],
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

describe("SQL route", () => {
  it("renders SQL keyboard hint and examples menu", async () => {
    const rootRoute = createRootRoute({ component: Outlet })
    const component = SqlRoute.options.component!
    const sqlRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/sql",
      component,
    })
    const router = createRouter({
      routeTree: rootRoute.addChildren([sqlRoute]),
      history: createMemoryHistory({ initialEntries: ["/sql"] }),
    })

    render(<RouterProvider router={router} />)
    expect(await screen.findByText("⌘")).toBeTruthy()
    expect(screen.getByText("Enter")).toBeTruthy()
    expect(screen.getByRole("button", { name: /examples/i })).toBeTruthy()
  })
})

describe("SnippetsMenu", () => {
  it("dispatches select, save, and delete actions", async () => {
    const user = userEvent.setup()
    const snippet = {
      id: "snippet-1",
      name: "Errors",
      page: "/sql",
      state: "SELECT * FROM error_events",
      updatedAtNanos: "1",
    }
    const onSelect = vi.fn()
    const onDelete = vi.fn()
    const onSave = vi.fn()
    renderWithRouter(
      <SnippetsMenu
        snippets={[snippet]}
        onSelect={onSelect}
        onDelete={onDelete}
        onSave={onSave}
      />
    )

    await user.click(await screen.findByText("Snippets"))
    await user.click((await screen.findAllByText("Errors"))[0]!)
    expect(onSelect).toHaveBeenCalledWith(snippet)

    await user.click(await screen.findByText("Snippets"))
    await user.click(await screen.findByText("Save current snippet"))
    expect(onSave).toHaveBeenCalled()

    await user.click(await screen.findByText("Snippets"))
    await user.click((await screen.findAllByText("Errors")).at(-1)!)
    expect(onDelete).toHaveBeenCalledWith("snippet-1")
  })
})
