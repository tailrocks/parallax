import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack tests surface @tests", () => {
  test("tests explorer loads product chrome @pw-full-stack-tests", async ({ page }) => {
    await page.goto("/tests")
    await expect(page.getByRole("heading", { name: "Tests" })).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(/variant-scoped test results/i)).toBeVisible({
      timeout: 10_000,
    })
    await expect(page.getByPlaceholder("Search name / suite")).toBeVisible()
  })
})
