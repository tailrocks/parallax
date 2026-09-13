/* @vitest-environment jsdom */

import { act, cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  SHORTCUT_LIST,
  SHORTCUTS,
  isTypingTarget,
  matchesShortcut,
  useFilterFocusShortcut,
  useFocusRequest,
  useShortcut,
} from "@/shared/keyboard"

afterEach(cleanup)

function fireKey(key: string, init?: KeyboardEventInit, target?: HTMLElement) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init })
  ;(target ?? window.document.body).dispatchEvent(event)
  return event
}

describe("shortcut registry", () => {
  it("covers the five canonical shortcuts with renderable keys", () => {
    expect(SHORTCUT_LIST.map((shortcut) => shortcut.id)).toEqual([
      "palette",
      "focus-search",
      "focus-filter",
      "show-shortcuts",
      "clear-close",
    ])
    for (const shortcut of SHORTCUT_LIST) {
      expect(shortcut.label.length).toBeGreaterThan(0)
      expect(shortcut.keys.length).toBeGreaterThan(0)
    }
  })

  it("matches mod, shift, and bare keys exactly", () => {
    const modK = new KeyboardEvent("keydown", { key: "k", metaKey: true })
    expect(matchesShortcut(SHORTCUTS.palette.spec, modK)).toBe(true)
    const ctrlK = new KeyboardEvent("keydown", { key: "K", ctrlKey: true })
    expect(matchesShortcut(SHORTCUTS.palette.spec, ctrlK)).toBe(true)
    const bareK = new KeyboardEvent("keydown", { key: "k" })
    expect(matchesShortcut(SHORTCUTS.palette.spec, bareK)).toBe(false)

    const question = new KeyboardEvent("keydown", { key: "?", shiftKey: true })
    expect(matchesShortcut(SHORTCUTS["show-shortcuts"].spec, question)).toBe(true)
    const bareQuestion = new KeyboardEvent("keydown", { key: "?" })
    expect(matchesShortcut(SHORTCUTS["show-shortcuts"].spec, bareQuestion)).toBe(true)
    const slash = new KeyboardEvent("keydown", { key: "/" })
    expect(matchesShortcut(SHORTCUTS["focus-search"].spec, slash)).toBe(true)
    expect(matchesShortcut(SHORTCUTS["show-shortcuts"].spec, slash)).toBe(false)

    const shiftF = new KeyboardEvent("keydown", { key: "F", shiftKey: true })
    const altF = new KeyboardEvent("keydown", { key: "f", altKey: true })
    expect(matchesShortcut(SHORTCUTS["focus-filter"].spec, altF)).toBe(false)
    expect(matchesShortcut({ key: "f", shift: true }, shiftF)).toBe(true)
    expect(
      matchesShortcut({ key: "f", shift: true }, new KeyboardEvent("keydown", { key: "f" }))
    ).toBe(false)
  })

  it("detects typing targets", () => {
    render(<input aria-label="probe" />)
    const input = screen.getByLabelText("probe")
    expect(isTypingTarget(new KeyboardEvent("keydown", { key: "f" }))).toBe(false)
    const targeted = new KeyboardEvent("keydown", { key: "f", bubbles: true })
    let seen: boolean | null = null
    input.addEventListener("keydown", (event) => {
      seen = isTypingTarget(event as KeyboardEvent)
    })
    input.dispatchEvent(targeted)
    expect(seen).toBe(true)
  })
})

describe("useShortcut", () => {
  it("fires plain-letter shortcuts outside inputs, skips inside", () => {
    const handler = vi.fn()
    function Probe() {
      useShortcut("focus-filter", handler)
      return <input aria-label="probe" />
    }
    render(<Probe />)
    fireKey("f")
    expect(handler).toHaveBeenCalledTimes(1)
    fireKey("f", {}, screen.getByLabelText("probe"))
    expect(handler).toHaveBeenCalledTimes(1)
  })

  it("fires mod shortcuts even while typing", () => {
    const handler = vi.fn()
    function Probe() {
      useShortcut("palette", handler)
      return <input aria-label="probe" />
    }
    render(<Probe />)
    fireKey("k", { metaKey: true }, screen.getByLabelText("probe"))
    expect(handler).toHaveBeenCalledTimes(1)
  })

  it("honors enabled=false", () => {
    const handler = vi.fn()
    function Probe() {
      useShortcut("focus-filter", handler, { enabled: false })
      return null
    }
    render(<Probe />)
    fireKey("f")
    expect(handler).not.toHaveBeenCalled()
  })
})

describe("filter focus helpers", () => {
  it("requests focus through the registry shortcut", () => {
    function Probe() {
      const [focusKey, requestFocus] = useFocusRequest()
      useFilterFocusShortcut(requestFocus)
      return <output aria-label="focus-key">{focusKey}</output>
    }
    render(<Probe />)
    expect(screen.getByLabelText("focus-key").textContent).toBe("0")
    act(() => {
      fireKey("F")
    })
    expect(screen.getByLabelText("focus-key").textContent).toBe("1")
  })
})
