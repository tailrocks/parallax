/* @vitest-environment jsdom */

import { act, cleanup, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { useState } from "react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { CommandPalette } from "@/components/console/command-palette"
import { graphql } from "@/lib/api"
import { renderTestRouter } from "@/test/router"

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
  return renderTestRouter(component, {
    componentPaths: [
      "/",
      "/logs",
      "/traces",
      "/traces/$traceId",
      "/runs/$runId",
      "/issues/$fingerprint",
      "/services/$service",
    ],
    initialPath,
    layout: true,
  })
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
})

describe("CommandPalette entity navigation", () => {
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
