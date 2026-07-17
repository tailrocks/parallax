import { productTest as test, expect } from "../fixtures/test"
import { InvestigationsScreen } from "../screens/investigations-screen"

const PILOT_NAME = "Checkout latency case"

test.describe("investigations mobile pilot @mobile", () => {
  test.use({ productDataset: "investigations-pilot" })

  test("list open detail and dialog on mobile @pw-investigations-mobile", async ({ page }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()
    await screen.caseLink(PILOT_NAME).click()
    await expect(page.getByRole("heading", { name: PILOT_NAME })).toBeVisible()

    await page.goto("/investigations")
    await screen.newButton().click()
    await expect(screen.nameInput()).toBeVisible()
    // Touch-friendly target: name input must be interactable.
    await screen.nameInput().tap()
    await screen.nameInput().fill("Mobile pilot case")
    await expect(screen.nameInput()).toHaveValue("Mobile pilot case")

    const overflow = await page.evaluate(() => document.documentElement.scrollWidth)
    const client = await page.evaluate(() => document.documentElement.clientWidth)
    expect(overflow).toBeLessThanOrEqual(client + 1)
  })
})
