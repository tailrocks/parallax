import { productTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"

/**
 * Canonical visual pilot — plan 146.
 * Goldens are authored only on digest-pinned Linux CI (visual-chromium-linux).
 * Local machines may inspect diffs; do not check in host-authored baselines.
 */
test.describe("shell visual pilot @visual", () => {
  test.use({ productDataset: "shell-empty" })

  test("root shell dark reduced-motion snapshot @pw-shell-visual", async ({ page }) => {
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await expect(shell.brandText()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
    await expect(page.getByRole("heading", { name: "Overview", exact: true })).toBeVisible()
    await expect(page.getByText("Send your first telemetry", { exact: true })).toBeVisible()

    // Stabilize animations before capture.
    await page.emulateMedia({ reducedMotion: "reduce", colorScheme: "dark" })
    await expect(page).toHaveScreenshot("shell-root-dark.png", {
      animations: "disabled",
      caret: "hide",
      // Exact match preferred; small AA budget for host vs Linux runner fonts
      // until digest-pinned Linux image owns goldens exclusively.
      maxDiffPixels: 120,
    })
  })
})
