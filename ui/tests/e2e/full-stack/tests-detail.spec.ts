import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack tests detail @tests", () => {
  test("case route renders explorer or empty @pw-tests-detail", async ({ page }) => {
    // Real-stack OTLP seed has no test-case rows; still land the route shape.
    await page.goto("/tests/" + "pw-storage-case")
    await expect(page.getByRole("heading", { name: /test/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
