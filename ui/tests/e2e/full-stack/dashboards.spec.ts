import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack dashboards @dashboards", () => {
  test("dashboards surface mounts on managed stack @pw-full-stack-dashboards", async ({ page }) => {
    await page.goto("/dashboards")
    await expect(page.getByRole("heading", { name: /dashboard/i }).first()).toBeVisible({
      timeout: 20_000,
    })
  })
})
