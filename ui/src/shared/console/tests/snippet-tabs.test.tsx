/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { SNIPPET_TABS, snippetFor, SnippetTabs } from "@/shared/console/snippet-tabs"

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

describe("snippet tabs", () => {
  it("renders rust java js tabs", () => {
    render(<SnippetTabs />)
    expect(screen.getByRole("tab", { name: "Rust" })).toBeTruthy()
    expect(screen.getByRole("tab", { name: "Java" })).toBeTruthy()
    expect(screen.getByRole("tab", { name: "JS" })).toBeTruthy()
    expect(screen.getByText(/init_tracing/)).toBeTruthy()
  })

  it("switching tabs shows the selected snippet", async () => {
    const user = userEvent.setup()
    render(<SnippetTabs />)
    await user.click(screen.getByRole("tab", { name: "Java" }))
    expect(screen.getByText(/OTEL_SERVICE_NAME/)).toBeTruthy()
    await user.click(screen.getByRole("tab", { name: "JS" }))
    expect(screen.getByText(/WebTracerProvider/)).toBeTruthy()
  })

  it("copy writes the visible snippet", async () => {
    const writeText = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined)
    const user = userEvent.setup({ pointerEventsCheck: 0 })
    render(<SnippetTabs />)
    await user.click(screen.getByRole("button", { name: "Copy" }))
    expect(writeText).toHaveBeenCalledWith(snippetFor("rust"))
    expect(writeText.mock.calls[0]?.[0]).toBe(SNIPPET_TABS[0].code)
  })
})
