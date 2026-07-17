import { productTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"

/** Cross-engine pilot with durable IDs distinct from contracts-chromium. */
test.describe("shell cross-engine pilot @cross", () => {
  test.use({ productDataset: "shell-empty" })

  test("root readiness on alternate engine @pw-shell-cross-root-nav", async ({ page }) => {
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await expect(shell.brandText()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
    await expect(shell.navItem("Investigations")).toBeVisible()
  })
})
