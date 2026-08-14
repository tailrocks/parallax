import { TRACES_PILOT_TRACE_ID } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"
import { openCommandPalette } from "../support/keyboard"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("command palette", () => {
  test.use({ productDataset: "traces-pilot" })

  test("paste seeded trace id lands on detail @pw-command-palette-trace", async ({ page }) => {
    await page.goto("/logs")
    await openCommandPalette(page)
    const search = page.getByPlaceholder(/search pages/i)
    await expect(search).toBeVisible({ timeout: SURFACE_TIMEOUT_MS })
    await search.fill(TRACES_PILOT_TRACE_ID)
    await page.keyboard.press("Enter")
    await expect(page).toHaveURL(new RegExp(`/traces/${TRACES_PILOT_TRACE_ID}`))
  })
})
