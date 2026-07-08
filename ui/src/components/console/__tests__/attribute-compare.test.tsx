/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { AttributeComparePanel } from "@/components/console/attribute-compare"
import type { AttributeCompareRow } from "@/lib/api"

const rows: AttributeCompareRow[] = [
  {
    key: "service.version",
    value: "2.0.0",
    selectedCount: "9",
    selectedTotal: "10",
    baselineCount: "1",
    baselineTotal: "20",
    score: 0.85,
  },
]

describe("AttributeComparePanel", () => {
  it("renders ranked BubbleUp rows with paired percentages", () => {
    render(<AttributeComparePanel rows={rows} />)

    expect(screen.getByText("#1")).toBeTruthy()
    expect(screen.getByText("service.version")).toBeTruthy()
    expect(screen.getByText("2.0.0")).toBeTruthy()
    expect(screen.getByText("90%")).toBeTruthy()
    expect(screen.getByText("5.0%")).toBeTruthy()
    expect(screen.getByText("85%")).toBeTruthy()
  })

  it("renders an empty state", () => {
    render(<AttributeComparePanel rows={[]} />)

    expect(
      screen.getByText("No overrepresented span attributes in this window.")
    ).toBeTruthy()
  })
})
