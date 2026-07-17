/* @vitest-environment jsdom */

import { act, cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { usePageVisible } from "@/platform/visibility/use-page-visible"

function Harness() {
  const visible = usePageVisible()
  return <output data-testid="visible">{visible ? "yes" : "no"}</output>
}

afterEach(() => {
  cleanup()
  Object.defineProperty(document, "hidden", {
    configurable: true,
    get: () => false,
  })
})

describe("usePageVisible", () => {
  it("tracks document.visibilitychange", () => {
    let hidden = false
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => hidden,
    })

    render(<Harness />)
    expect(screen.getByTestId("visible").textContent).toBe("yes")

    act(() => {
      hidden = true
      document.dispatchEvent(new Event("visibilitychange"))
    })
    expect(screen.getByTestId("visible").textContent).toBe("no")

    act(() => {
      hidden = false
      document.dispatchEvent(new Event("visibilitychange"))
    })
    expect(screen.getByTestId("visible").textContent).toBe("yes")
  })
})
