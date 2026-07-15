/* @vitest-environment jsdom */

import { act, cleanup, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router"
import { useState } from "react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { CommandPalette } from "@/components/console/command-palette"
import { graphql } from "@/lib/api"

vi.mock("@/lib/api", () => ({
  graphql: vi.fn(),
}))

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

function mockPaletteData() {
  vi.mocked(graphql).mockImplementation(
    async <T,>(query: string): Promise<T> => {
      if (query.includes("{ services }")) {
        return { services: ["checkout", "catalog"] } as T
      }
      if (query.includes("tracesPage")) {
        return {
          tracesPage: {
            items: [
              {
                traceId: "53e97e432cbb9280841b90ca56c4e4c4",
                rootName: "GET /checkout",
                service: "checkout",
                startNanos: "1719999990000000000",
                hasError: false,
              },
            ],
          },
          runs: [
            {
              runId: "run-a",
              command: "cargo test",
              status: "finished",
              startedAtNanos: "1719999980000000000",
              endedAtNanos: "1719999990000000000",
              errorCount: 1,
            },
          ],
        } as T
      }
      throw new Error("unexpected query")
    }
  )
}

function PaletteHarness() {
  const [open, setOpen] = useState(false)
  return <CommandPalette open={open} onOpenChange={setOpen} />
}

function renderWithRouter(component: React.ReactNode, initialPath = "/") {
  const rootRoute = createRootRoute({
    component: () => (
      <>
        {component}
        <Outlet />
      </>
    ),
  })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/",
    component: () => null,
  })
  const logsRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "logs",
    component: () => null,
  })
  const tracesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces",
    component: () => null,
  })
  const traceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "traces/$traceId",
    component: () => null,
  })
  const runRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "runs/$runId",
    component: () => null,
  })
  const issueRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "issues/$fingerprint",
    component: () => null,
  })
  const serviceRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "services/$service",
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      indexRoute,
      logsRoute,
      tracesRoute,
      traceRoute,
      runRoute,
      issueRoute,
      serviceRoute,
    ]),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })

  return {
    router,
    ...render(<RouterProvider router={router} />),
  }
}

describe("CommandPalette", () => {
  it("opens with Cmd/Ctrl+K and closes with Escape", async () => {
    const user = userEvent.setup()
    mockPaletteData()
    renderWithRouter(<PaletteHarness />)

    await act(async () => {})
    await user.keyboard("{Meta>}k{/Meta}")
    const input = await screen.findByPlaceholderText(/search pages/i)
    expect(input).toBeTruthy()

    await user.keyboard("{Escape}")
    await waitFor(() =>
      expect(screen.queryByPlaceholderText(/search pages/i)).toBeNull()
    )
  })

  it("filters pages and navigates on selection", async () => {
    const user = userEvent.setup()
    mockPaletteData()
    const { router } = renderWithRouter(<PaletteHarness />)

    await act(async () => {})
    await user.keyboard("{Control>}k{/Control}")
    const input = await screen.findByPlaceholderText(/search pages/i)
    await user.type(input, "logs")
    await user.click(await screen.findByText("Logs"))

    await waitFor(() => expect(router.state.location.pathname).toBe("/logs"))
  })

  it("shows id jump entries for trace and ambiguous 16-hex ids", async () => {
    const user = userEvent.setup()
    mockPaletteData()
    renderWithRouter(<PaletteHarness />)

    await act(async () => {})
    await user.keyboard("{Meta>}k{/Meta}")
    const input = await screen.findByPlaceholderText(/search pages/i)
    await user.type(input, "53e97e432cbb9280841b90ca56c4e4c4")
    expect(await screen.findByText(/Open trace 53e97e/)).toBeTruthy()

    await user.clear(input)
    await user.type(input, "a7a77b573b7261a1")
    expect(await screen.findByText(/Open run a7a77b/)).toBeTruthy()
    expect(screen.getByText(/Open issue a7a77b/)).toBeTruthy()
    expect(screen.getByText(/Search traces for span a7a77b/)).toBeTruthy()
  })

  it("navigates from logs to a pasted trace id", async () => {
    const user = userEvent.setup()
    mockPaletteData()
    const { router } = renderWithRouter(<PaletteHarness />, "/logs")

    await act(async () => {})
    await user.keyboard("{Meta>}k{/Meta}")
    const input = await screen.findByPlaceholderText(/search pages/i)
    await user.type(input, "53e97e432cbb9280841b90ca56c4e4c4")
    await user.click(await screen.findByText(/Open trace 53e97e/))

    await waitFor(() =>
      expect(router.state.location.pathname).toBe(
        "/traces/53e97e432cbb9280841b90ca56c4e4c4"
      )
    )
  })

  it("keeps pages available when entity fetches fail", async () => {
    vi.mocked(graphql).mockRejectedValue(new Error("offline"))
    renderWithRouter(<CommandPalette open onOpenChange={() => {}} />)

    expect(await screen.findByText("Services unavailable")).toBeTruthy()
    expect(await screen.findByText("Recent unavailable")).toBeTruthy()
    expect(screen.getByText("Overview")).toBeTruthy()
  })
})
