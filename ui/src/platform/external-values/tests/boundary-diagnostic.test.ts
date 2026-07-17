import { describe, expect, it } from "vitest"

import { formatBoundaryDiagnostic } from "@/platform/external-values/boundary-diagnostic"
import { boundaryError } from "@/platform/external-values/boundary-error"

describe("formatBoundaryDiagnostic", () => {
  it("renders only stable fields and caps length", () => {
    const error = boundaryError("test.boundary", "invalid-json", '{"secret":"payload"}', {
      length: 12,
    })
    const rendered = formatBoundaryDiagnostic(error, 80)
    expect(rendered.length).toBeLessThanOrEqual(80)
    expect(rendered).toContain("code=invalid-json")
    expect(rendered).not.toContain("secret")
    expect(rendered).not.toContain("payload")
  })
})
