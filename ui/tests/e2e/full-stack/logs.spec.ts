import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack logs @logs", () => {
  test("logs surface loads for seeded service @pw-full-stack-logs", async ({ page, fullStack }) => {
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.service, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.log_body, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
