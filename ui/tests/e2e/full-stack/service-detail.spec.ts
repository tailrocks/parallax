import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack service detail @services", () => {
  test("RED charts and prefiltered links @pw-service-detail", async ({ page, fullStack }) => {
    await page.goto("/services/" + encodeURIComponent(fullStack.service))
    await expect(page.getByRole("heading", { name: fullStack.service }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(/error|latency|throughput|RED/i).first()).toBeVisible()
    const traces = page.getByRole("link", { name: /traces/i }).first()
    if (await traces.count()) {
      await traces.click()
      await expect(page).toHaveURL(/service=|\/traces/)
    }
  })
})
