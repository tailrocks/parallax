/* @vitest-environment jsdom */

import { cleanup, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { renderTestRouter } from "../../src/test/router"

describe("test router harness", () => {
  it("creates isolated memory history for sequential renders", async () => {
    const first = renderTestRouter(<p>first router</p>)
    expect(await screen.findByText("first router")).toBeTruthy()
    first.history.push("/changed")
    expect(first.history.location.pathname).toBe("/changed")
    first.unmount()
    cleanup()

    const second = renderTestRouter(<p>second router</p>)
    expect(await screen.findByText("second router")).toBeTruthy()
    expect(second.history.location.pathname).toBe("/")
  })
})
