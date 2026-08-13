import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack tests detail @tests", () => {
  test("case route renders explorer or empty @pw-tests-detail", async ({ page }) => {
    // Real-stack OTLP seed has no test-case rows; still land the route shape.
    await page.goto("/tests/" + "pw-storage-case")
    await expect(page.getByRole("heading", { name: /test/i }).first()).toBeVisible({
      timeout: 20_000,
    })
  })
})
