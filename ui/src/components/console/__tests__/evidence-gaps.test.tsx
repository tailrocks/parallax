/* @vitest-environment jsdom */

import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { EvidenceGapsCard } from "@/components/console/evidence-gaps"
import type { EvidenceGap } from "@/lib/api"

const gaps: EvidenceGap[] = [
  {
    kind: "orphan_span",
    subject: "span-child",
    detail:
      "parent span missing from fetched evidence; this may be a legitimate cross-service root",
  },
]

describe("EvidenceGapsCard", () => {
  it("renders gaps when present", () => {
    render(<EvidenceGapsCard gaps={gaps} />)

    expect(screen.getByTestId("evidence-gaps-card")).toBeTruthy()
    expect(screen.getByText("orphan_span")).toBeTruthy()
    expect(screen.getByText("span-child")).toBeTruthy()
    expect(screen.getByText(/legitimate cross-service root/)).toBeTruthy()
  })

  it("renders nothing for clean evidence", () => {
    const { container } = render(<EvidenceGapsCard gaps={[]} />)

    expect(container.textContent).toBe("")
  })
})
