import { TRACES_PILOT_TRACE_ID } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("command palette", () => {
  test.use({ productDataset: "traces-pilot" })

  test("paste seeded trace id lands on detail @pw-command-palette-trace", async ({ page }) => {
    await page.goto("/logs")
    await page.keyboard.press("Meta+k")
    const search = page.getByPlaceholder(/search pages/i)
    await expect(search).toBeVisible()
    await search.fill(TRACES_PILOT_TRACE_ID)
    await page.keyboard.press("Enter")
    await expect(page).toHaveURL(new RegExp(`/traces/${TRACES_PILOT_TRACE_ID}`))
  })
})
