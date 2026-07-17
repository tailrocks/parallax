/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { afterEach, describe, expect, it, vi } from "vitest"

import { SqlPage } from "@/features/sql"

vi.mock("@/features/sql/api/sql-api", () => ({
  loadSqlSchema: vi.fn(async () => new Map()),
  loadSqlSnippets: vi.fn(async () => []),
  runSql: vi.fn(),
  saveSqlSnippet: vi.fn(),
  deleteSqlSnippet: vi.fn(),
}))

afterEach(cleanup)

describe("SQL route composition", () => {
  it("renders SQL keyboard hint and examples menu", async () => {
    const rootRoute = createRootRoute({ component: Outlet })
    const sqlRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: "/sql",
      component: () => <SqlPage />,
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
