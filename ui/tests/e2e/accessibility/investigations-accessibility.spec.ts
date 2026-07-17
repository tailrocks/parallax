import { productTest as test, expect } from "../fixtures/test"
import { assertNoAxeViolations } from "../fixtures/accessibility-fixture"
import { InvestigationsScreen } from "../screens/investigations-screen"

const PILOT_NAME = "Checkout latency case"

test.describe("investigations accessibility pilot @a11y", () => {
  test.use({ productDataset: "investigations-pilot" })

  test("list detail and dialog axe keyboard focus @pw-investigations-a11y-detail", async ({
    page,
  }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()
    await assertNoAxeViolations(page)

    await screen.caseLink(PILOT_NAME).click()
    await expect(page.getByRole("heading", { name: PILOT_NAME })).toBeVisible()
    await assertNoAxeViolations(page)

    // Create dialog: focus trap/restore via Escape.
    await page.goto("/investigations")
    await screen.newButton().click()
    await expect(screen.nameInput()).toBeVisible()
    await assertNoAxeViolations(page)
    await page.keyboard.press("Escape")
    await expect(screen.nameInput()).toHaveCount(0)
    await expect(screen.heading()).toBeVisible()
  })
})
