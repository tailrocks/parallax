import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack trace detail @traces", () => {
  test("deep-link tree flame and critical path @pw-trace-detail", async ({ page, fullStack }) => {
    await page.goto("/traces/" + fullStack.trace_id + "?view=tree")
    await expect(page.getByText("pw.storage.root").first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.getByRole("radio", { name: /Flame view/i }).click()
    await expect(page.getByRole("radio", { name: /Flame view/i })).toBeChecked()
    await page.getByRole("button", { name: "Critical path" }).click()
    await page.getByText("pw.storage.root").first().click()
    await expect(page.getByText(/span|service|duration/i).first()).toBeVisible()
  })
})
