import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack sql @sql", () => {
  test("sql surface mounts on managed stack @pw-full-stack-sql", async ({ page }) => {
    await page.goto("/sql")
    await expect(page.getByRole("heading", { name: /sql/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByRole("button", { name: "Run query" })).toBeVisible()
    await expect(page.getByRole("button", { name: "opentelemetry_logs" })).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
