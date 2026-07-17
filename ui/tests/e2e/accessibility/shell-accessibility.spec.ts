import { productTest as test, expect } from "../fixtures/test"
import { assertNoAxeViolations } from "../fixtures/accessibility-fixture"
import { ShellScreen } from "../screens/shell-screen"

test.describe("shell accessibility @a11y", () => {
  test.use({ productDataset: "shell-empty" })

  test("root shell passes axe and keyboard focus @pw-shell-a11y", async ({ page }) => {
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await expect(shell.brandText()).toBeVisible()
    await assertNoAxeViolations(page)

    // Tab reaches primary navigation (semantic links are keyboard reachable).
    await page.keyboard.press("Tab")
    const focused = page.locator(":focus")
    await expect(focused).toBeVisible()
    const tag = await focused.evaluate((el) => el.tagName.toLowerCase())
    expect(["a", "button", "input", "select", "textarea"]).toContain(tag)
  })

  test("investigations list keyboard and axe @pw-investigations-a11y", async ({ page }) => {
    await page.goto("/investigations")
    await expect(page.getByRole("heading", { name: "Investigations", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)

    // Escape does not leave a stuck dialog when none is open.
    await page.keyboard.press("Escape")
    await expect(page.getByRole("heading", { name: "Investigations", exact: true })).toBeVisible()
  })
})
