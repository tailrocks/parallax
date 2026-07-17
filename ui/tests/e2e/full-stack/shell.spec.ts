import { fullStackTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack shell @shell", () => {
  test("shell nav and brand against managed stack @pw-full-stack-shell", async ({ page }) => {
    const manifest = readFullStackManifest()
    expect(manifest.storage).toBe("managed-greptime+turso")
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await expect(shell.brandText()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
    await expect(shell.navItem("Services")).toBeVisible()
    await shell.navItem("Services").click()
    await expect(page).toHaveURL(/\/services/)
  })
})
