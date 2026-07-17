import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack sql @sql", () => {
  test("sql surface mounts on managed stack @pw-full-stack-sql", async ({ page }) => {
    await page.goto("/sql")
    await expect(page.getByRole("heading", { name: /sql/i }).first()).toBeVisible({
      timeout: 20_000,
    })
  })
})
