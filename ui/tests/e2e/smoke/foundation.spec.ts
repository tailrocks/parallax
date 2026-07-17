import { test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"

test.describe("foundation shell smoke", () => {
  test("root shell exposes brand and primary navigation @pw-foundation-shell", async ({
    page,
    diagnostics,
  }) => {
    // Touch diagnostics fixture so capture is active for the whole test.
    expect(diagnostics.events).toEqual([])

    const shell = new ShellScreen(page)
    await shell.openRoot()

    await expect(page).toHaveURL(/\/(?:\?.*)?$/)
    await expect(shell.homeLink()).toBeVisible()
    await expect(shell.brandText()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
    await expect(shell.navItem("Traces")).toBeVisible()
    await expect(shell.navItem("Logs")).toBeVisible()
  })
})
