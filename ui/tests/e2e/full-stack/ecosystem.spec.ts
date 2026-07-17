import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack ecosystem @ecosystem", () => {
  test("ecosystem surface mounts on managed stack @pw-full-stack-ecosystem", async ({ page }) => {
    await page.goto("/ecosystem")
    await expect(page.getByRole("heading", { name: /ecosystem/i }).first()).toBeVisible({
      timeout: 20_000,
    })
  })
})
