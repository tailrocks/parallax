/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { resolvePreset } from "@/domain/time-range/range"
import { RangePicker } from "@/features/time-range"

describe("RangePicker", () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  function dayButton(label: string): HTMLButtonElement {
    const button = screen
      .getAllByText(label)
      .map((node) => node.closest("button"))
      .find((node): node is HTMLButtonElement => node instanceof HTMLButtonElement)
    if (!button) throw new Error(`missing day button ${label}`)
    return button
  }

  it("emits preset and custom calendar ranges", async () => {
    // Base UI marks the trigger tree inert while the popover is open; skip the
    // pointer-events precheck so preset/calendar clicks stay deterministic.
    const user = userEvent.setup({ pointerEventsCheck: 0 })
    const now = new Date(2026, 0, 15, 12, 0, 0, 0).getTime()
    vi.spyOn(Date, "now").mockReturnValue(now)
    const onChange = vi.fn()
    render(<RangePicker value={resolvePreset("24h", now)} onChange={onChange} />)

    await user.click(screen.getByRole("button", { name: /Last 24h/i }))
    await user.click(await screen.findByRole("button", { name: /Last hour/i }))
    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ key: "1h" }))

    await user.click(dayButton("10"))
    await user.click(dayButton("12"))

    const custom = onChange.mock.calls.at(-1)?.[0]
    if (!custom) throw new Error("missing custom range")
    expect(custom).toEqual(expect.objectContaining({ key: "custom" }))
    expect(BigInt(custom.fromNanos)).toBeLessThan(BigInt(custom.toNanos))
  }, 15_000)
})
