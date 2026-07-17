/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Empty } from "@/components/ui/empty"
import { Kbd } from "@/components/ui/kbd"

describe("primitive recipes", () => {
  it("keeps the reference button shape", () => {
    render(<Button>Save</Button>)

    expect(screen.getByRole("button", { name: "Save" }).className).toContain("rounded-full")
  })

  it("keeps badge hue variants", () => {
    render(<Badge variant="rose">error</Badge>)

    expect(screen.getByText("error").closest("[data-slot=badge]")?.className).toContain(
      "shadow-[var(--custom-shadow-rose)]"
    )
  })

  it("keeps card, empty, and kbd squircle recipes", () => {
    render(
      <>
        <Card data-testid="card" />
        <Empty data-testid="empty" />
        <Kbd>Esc</Kbd>
      </>
    )

    expect(screen.getByTestId("card").className).toContain("corner-squircle")
    expect(screen.getByTestId("empty").className).toContain("corner-squircle")
    expect(screen.getByText("Esc").className).toContain("corner-squircle")
  })
})
