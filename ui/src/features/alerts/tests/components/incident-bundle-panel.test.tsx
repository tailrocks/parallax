/* @vitest-environment jsdom */

import { cleanup, render, screen } from "@testing-library/react"
import { afterEach, describe, expect, it } from "vitest"

import { IncidentBundlePanel } from "@/features/alerts/components/incident-bundle-panel"

afterEach(() => {
  cleanup()
})

describe("incident bundle panel", () => {
  it("renders markdown and hash when a bundle exists", () => {
    render(<IncidentBundlePanel markdown="# Alert incident" canonicalHash="abc123" />)
    expect(screen.getByTestId("incident-bundle-panel").textContent).toMatch(/Alert incident/)
    expect(screen.getByText(/bundle abc123/)).toBeTruthy()
  })

  it("renders an empty notice when assembly is missing", () => {
    render(<IncidentBundlePanel markdown={null} canonicalHash={null} />)
    expect(screen.getByText("No evidence bundle on this incident.")).toBeTruthy()
  })
})
