/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import { SnippetsMenu } from "@/features/sql/components/snippets-menu"
import { renderTestRouter } from "@/test/router"

afterEach(cleanup)

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
    renderTestRouter(
      <SnippetsMenu snippets={[snippet]} onSelect={onSelect} onDelete={onDelete} onSave={onSave} />
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
