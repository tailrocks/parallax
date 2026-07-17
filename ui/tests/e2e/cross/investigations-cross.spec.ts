import { productTest as test, expect } from "../fixtures/test"
import { InvestigationsScreen } from "../screens/investigations-screen"

const PILOT_NAME = "Checkout latency case"

/** Cross-engine pilot with durable IDs distinct from contracts-chromium. */
test.describe("investigations cross-engine pilot @cross", () => {
  test.use({ productDataset: "investigations-pilot" })

  test("list and open detail on alternate engine @pw-investigations-cross-list-detail", async ({
    page,
  }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()
    await screen.caseLink(PILOT_NAME).click()
    await expect(page.getByRole("heading", { name: PILOT_NAME })).toBeVisible()
  })
})
