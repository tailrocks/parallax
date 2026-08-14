import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack trace detail @traces", () => {
  test("deep-link tree flame and critical path @pw-trace-detail", async ({ page, fullStack }) => {
    await page.goto("/traces/" + fullStack.trace_id + "?view=tree")
    await expect(page.getByText("pw.storage.root").first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    const flame = page.getByRole("button", { name: "Flame view" })
    await expect(flame).toBeVisible({ timeout: SURFACE_TIMEOUT_MS })
    await flame.click()
    await expect(flame).toHaveAttribute("aria-pressed", "true")
    await page.getByRole("button", { name: "Critical path" }).click()
    await page.getByText("pw.storage.root").first().click()
    await expect(page.getByText(/span|service|duration/i).first()).toBeVisible()
  })
})
