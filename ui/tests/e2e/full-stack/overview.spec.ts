import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack overview @overview", () => {
  test("overview renders for seeded stack @pw-full-stack-overview", async ({ page, fullStack }) => {
    await page.goto("/")
    await expect(page.getByRole("heading", { name: /overview/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.service, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
