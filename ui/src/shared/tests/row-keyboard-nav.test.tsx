/* @vitest-environment jsdom */

import { cleanup, fireEvent, render } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { rowKeyboardAttrs, useRowKeyboardNav } from "@/lib/row-keyboard-nav"

afterEach(cleanup)

function Harness({
  scope = "test",
  count,
  onOpen,
  enabled,
}: {
  scope?: string
  count: number
  onOpen: (index: number) => void
  enabled?: boolean
}) {
  const active = useRowKeyboardNav({ scope, count, onOpen, enabled })
  return (
    <div>
      {Array.from({ length: count }, (_, index) => (
        <div
          key={index}
          {...rowKeyboardAttrs(scope, index)}
          data-active={active === index ? "true" : undefined}
        >
          row-{index}
        </div>
      ))}
      <input data-testid="editor" />
    </div>
  )
}

function activeIndex(container: HTMLElement): number {
  const rows = Array.from(container.querySelectorAll("[data-kb-index]"))
  return rows.findIndex((row) => row.getAttribute("data-active") === "true")
}

describe("useRowKeyboardNav", () => {
  it("moves with j/k and opens with Enter", () => {
    const onOpen = vi.fn()
    const { container } = render(<Harness count={3} onOpen={onOpen} />)
    expect(activeIndex(container)).toBe(-1)
    fireEvent.keyDown(window, { key: "j" })
    expect(activeIndex(container)).toBe(0)
    fireEvent.keyDown(window, { key: "j" })
    expect(activeIndex(container)).toBe(1)
    fireEvent.keyDown(window, { key: "ArrowDown" })
    expect(activeIndex(container)).toBe(2)
    fireEvent.keyDown(window, { key: "ArrowDown" })
    expect(activeIndex(container)).toBe(2)
    fireEvent.keyDown(window, { key: "k" })
    expect(activeIndex(container)).toBe(1)
    fireEvent.keyDown(window, { key: "Enter" })
    expect(onOpen).toHaveBeenCalledWith(1)
  })

  it("ignores keystrokes from inputs and modified shortcuts", () => {
    const onOpen = vi.fn()
    const { container, getByTestId } = render(<Harness count={3} onOpen={onOpen} />)
    fireEvent.keyDown(getByTestId("editor"), { key: "j" })
    expect(activeIndex(container)).toBe(-1)
    fireEvent.keyDown(window, { key: "j", metaKey: true })
    expect(activeIndex(container)).toBe(-1)
  })

  it("clears with Escape and stays inert when disabled", () => {
    const onOpen = vi.fn()
    const { container, rerender } = render(<Harness count={3} onOpen={onOpen} />)
    fireEvent.keyDown(window, { key: "j" })
    expect(activeIndex(container)).toBe(0)
    fireEvent.keyDown(window, { key: "Escape" })
    expect(activeIndex(container)).toBe(-1)
    rerender(<Harness count={3} onOpen={onOpen} enabled={false} />)
    fireEvent.keyDown(window, { key: "j" })
    expect(activeIndex(container)).toBe(-1)
  })

  it("clamps when rows shrink", () => {
    const onOpen = vi.fn()
    const { container, rerender } = render(<Harness count={3} onOpen={onOpen} />)
    fireEvent.keyDown(window, { key: "j" })
    fireEvent.keyDown(window, { key: "j" })
    fireEvent.keyDown(window, { key: "j" })
    expect(activeIndex(container)).toBe(2)
    rerender(<Harness count={1} onOpen={onOpen} />)
    expect(activeIndex(container)).toBe(0)
  })
})
