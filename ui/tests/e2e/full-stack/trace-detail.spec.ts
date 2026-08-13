import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack trace detail @traces", () => {
  test("deep-link tree flame and critical path @pw-trace-detail", async ({ page, fullStack }) => {
    await page.goto("/traces/" + fullStack.trace_id + "?view=tree")
    await expect(page.getByText("pw.storage.root").first()).toBeVisible({ timeout: 20_000 })
    await page.getByRole("radio", { name: "Flame" }).click()
    await expect(page.getByRole("radio", { name: "Flame" })).toBeChecked()
    await page.getByRole("button", { name: "Critical path" }).click()
    await page.getByText("pw.storage.root").first().click()
    await expect(page.getByText(/span|service|duration/i).first()).toBeVisible()
  })
})
