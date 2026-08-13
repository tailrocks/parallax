import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack dashboards @dashboards", () => {
  test("dashboards surface mounts on managed stack @pw-full-stack-dashboards", async ({ page }) => {
    await page.goto("/dashboards")
    await expect(page.getByRole("heading", { name: /dashboard/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByRole("button", { name: "New dashboard" })).toBeVisible()
    // Managed OTLP seed has no saved dashboards — empty copy is the product fact.
    await expect(page.getByText("Create your first dashboard")).toBeVisible()
  })
})
