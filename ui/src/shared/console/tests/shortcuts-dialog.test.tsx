/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it } from "vitest"

import { ShortcutsDialog, useShortcutsDialog } from "@/shared/console/shortcuts-dialog"
import { SHORTCUT_LIST } from "@/shared/keyboard"

afterEach(cleanup)

describe("ShortcutsDialog", () => {
  it("lists every registry shortcut", () => {
    render(<ShortcutsDialog open onOpenChange={() => {}} />)
    expect(screen.getByText("Keyboard shortcuts")).toBeTruthy()
    for (const shortcut of SHORTCUT_LIST) {
      expect(screen.getByText(shortcut.label)).toBeTruthy()
    }
  })

  it("opens on ? through the registry", async () => {
    const user = userEvent.setup()
    function Probe() {
      const { open, setOpen } = useShortcutsDialog()
      return (
        <>
          <output aria-label="open">{String(open)}</output>
          <ShortcutsDialog open={open} onOpenChange={setOpen} />
        </>
      )
    }
    render(<Probe />)
    expect(screen.getByLabelText("open").textContent).toBe("false")
    await user.keyboard("?")
    expect(screen.getByLabelText("open").textContent).toBe("true")
  })

  it("ignores ? while typing", async () => {
    const user = userEvent.setup()
    function Probe() {
      const { open, setOpen } = useShortcutsDialog()
      return (
        <>
          <input aria-label="probe" />
          <output aria-label="open">{String(open)}</output>
          <ShortcutsDialog open={open} onOpenChange={setOpen} />
        </>
      )
    }
    render(<Probe />)
    await user.click(screen.getByLabelText("probe"))
    await user.keyboard("?")
    expect(screen.getByLabelText("open").textContent).toBe("false")
  })
})
