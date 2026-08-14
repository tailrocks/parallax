/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { GroupingCard } from "@/features/issues/components/grouping-card"

afterEach(() => {
  cleanup()
})

describe("grouping card", () => {
  it("renders type, template, and frame", () => {
    render(
      <GroupingCard
        algorithmVersion="fp-v1"
        errorType="TypeError"
        messageTemplate="connection to <host> refused"
        anchorFrame="pool::acquire"
        operation={null}
      />
    )
    expect(screen.getByTestId("grouping-card").textContent).toMatch(/Grouped by/)
    expect(screen.getByText(/TypeError/)).toBeTruthy()
    expect(screen.getByText(/pool::acquire/)).toBeTruthy()
    expect(screen.getByText(/fp-v1/)).toBeTruthy()
  })
})
