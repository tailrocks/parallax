/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"

import {
  WhereClauseChips,
  WhereClauseEditor,
} from "@/components/console/where-clause-editor"
import type { WhereFilter } from "@/lib/where-clause"

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

describe("WhereClauseEditor", () => {
  it("shows the serialized applied filters", () => {
    const filters: WhereFilter[] = [
      { key: "service", op: "=", value: "checkout" },
    ]
    render(<WhereClauseEditor filters={filters} onApply={() => {}} />)
    expect(screen.getByLabelText("Where clause")).toHaveProperty(
      "value",
      "service = checkout"
    )
  })

  it("opens key autocomplete while typing and accepts with Enter", async () => {
    const user = userEvent.setup()
    render(
      <WhereClauseEditor
        filters={[]}
        onApply={() => {}}
        keySuggestions={["service", "severity"]}
      />
    )
    const input = screen.getByLabelText("Where clause")
    await user.type(input, "se")
    expect(
      screen.getByRole("listbox", { name: "Autocomplete suggestions" })
    ).toBeDefined()
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual([
      "service",
      "severity",
    ])
    await user.keyboard("{Enter}")
    expect(input).toHaveProperty("value", "service ")
  })

  it("suggests operators after a key", async () => {
    const user = userEvent.setup()
    render(
      <WhereClauseEditor
        filters={[]}
        onApply={() => {}}
        keySuggestions={["service"]}
      />
    )
    await user.type(screen.getByLabelText("Where clause"), "service !")
    expect(
      screen.getAllByRole("option").map((option) => option.textContent)
    ).toEqual(["!="])
  })

  it("suggests top values for the active key", async () => {
    const user = userEvent.setup()
    render(
      <WhereClauseEditor
        filters={[]}
        onApply={() => {}}
        keySuggestions={["service"]}
        valueSuggestionsFor={(key) =>
          key === "service" ? ["checkout", "cart"] : []
        }
      />
    )
    await user.type(screen.getByLabelText("Where clause"), "service = c")
    expect(
      screen.getAllByRole("option").map((option) => option.textContent)
    ).toEqual(["checkout", "cart"])
  })

  it("applies a valid clause with meta+Enter", async () => {
    const user = userEvent.setup()
    const onApply = vi.fn()
    render(<WhereClauseEditor filters={[]} onApply={onApply} />)
    const input = screen.getByLabelText("Where clause")
    await user.click(input)
    await user.paste('service = "checkout"')
    await user.keyboard("{Meta>}{Enter}{/Meta}")
    expect(onApply).toHaveBeenCalledWith([
      { key: "service", op: "=", value: "checkout" },
    ])
  })

  it("shows a positioned parse error and never applies invalid input", async () => {
    const user = userEvent.setup()
    const onApply = vi.fn()
    render(<WhereClauseEditor filters={[]} onApply={onApply} />)
    const input = screen.getByLabelText("Where clause")
    await user.click(input)
    await user.paste('service = "checkout')
    expect(screen.getByText(/unterminated string \(at 10\)/)).toBeDefined()
    await user.keyboard("{Meta>}{Enter}{/Meta}")
    expect(onApply).not.toHaveBeenCalled()
  })
})

describe("WhereClauseChips", () => {
  it("renders one removable chip per filter", async () => {
    const user = userEvent.setup()
    const onRemove = vi.fn()
    render(
      <WhereClauseChips
        filters={[
          { key: "service", op: "=", value: "checkout" },
          { key: "body", op: "CONTAINS", value: "timeout" },
        ]}
        onRemove={onRemove}
      />
    )
    await user.click(screen.getByLabelText("Remove filter body"))
    expect(onRemove).toHaveBeenCalledWith(1)
  })

  it("renders nothing when empty", () => {
    const { container } = render(
      <WhereClauseChips filters={[]} onRemove={() => {}} />
    )
    expect(container.innerHTML).toBe("")
  })
})
