import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack traces @traces", () => {
  test("traces surface loads for seeded stack @pw-full-stack-traces", async ({ page }) => {
    await page.goto("/traces")
    await expect(page.getByRole("heading", { name: /trace/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText("pw.storage.root").first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
