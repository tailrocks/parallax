import { productTest as test, expect } from "../fixtures/test"
import { InvestigationsScreen } from "../screens/investigations-screen"

const PILOT_NAME = "Checkout latency case"

test.describe("investigations visual pilot @visual", () => {
  test.use({ productDataset: "investigations-pilot" })

  test("investigations list snapshot @pw-investigations-visual", async ({ page }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()
    await page.emulateMedia({ reducedMotion: "reduce", colorScheme: "dark" })
    await expect(page).toHaveScreenshot("investigations-list-dark.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixels: 120,
    })
  })
})
