import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack tests detail @tests", () => {
  test("case route renders explorer or empty @pw-tests-detail", async ({ page }) => {
    // Real-stack OTLP seed has no test-case rows; still land the route shape.
    // Key must be versioned (`tc1:` + 64 hex) or testCase() rejects parse
    // and the loader throws RouteErrorPanel instead of EmptyState.
    await page.goto("/tests/tc1:" + "0".repeat(64))
    await expect(page.getByText("Test case not found")).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
