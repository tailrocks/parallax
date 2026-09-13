/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { ErrorState, SectionError } from "@/shared/console/error-state"

afterEach(cleanup)

describe("error states", () => {
  it("announces ErrorState and retries", async () => {
    const user = userEvent.setup()
    const onRetry = vi.fn()
    render(<ErrorState title="Logs did not load" message="boom" onRetry={onRetry} />)
    expect(screen.getByRole("alert")).toBeTruthy()
    expect(screen.getByText("Logs did not load")).toBeTruthy()
    await user.click(screen.getByRole("button", { name: "Retry" }))
    expect(onRetry).toHaveBeenCalledTimes(1)
  })

  it("omits retry when no handler is given", () => {
    render(<ErrorState title="Facets unavailable" />)
    expect(screen.queryByRole("button", { name: "Retry" })).toBeNull()
  })

  it("renders SectionError as one announced row", async () => {
    const user = userEvent.setup()
    const onRetry = vi.fn()
    const { container } = render(<SectionError message="older load failed" onRetry={onRetry} />)
    expect(container.querySelector('[role="alert"]')?.textContent).toContain("older load failed")
    await user.click(screen.getByRole("button", { name: "Retry" }))
    expect(onRetry).toHaveBeenCalledTimes(1)
  })
})
