import { productTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"

test.describe("shell product contracts", () => {
  test.use({ productDataset: "shell-empty" })

  test("root readiness and primary navigation @pw-shell-root-nav", async ({
    page,
    diagnostics,
  }) => {
    expect(diagnostics.events).toEqual([])
    const shell = new ShellScreen(page)
    await shell.openRoot()

    await expect(page).toHaveURL(/\/(?:\?.*)?$/)
    await expect(shell.homeLink()).toBeVisible()
    await expect(shell.brandText()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
    await expect(shell.navItem("Traces")).toBeVisible()
    await expect(shell.navItem("Logs")).toBeVisible()
    await expect(shell.navItem("Investigations")).toBeVisible()
  })

  test("workspace navigation and deep-link refresh @pw-shell-workspace-deeplink", async ({
    page,
  }) => {
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await shell.navItem("Investigations").click()
    await expect(page).toHaveURL(/\/investigations\/?/)
    await expect(
      page.getByRole("heading", { name: "Investigations", exact: true })
    ).toBeVisible()

    await page.reload()
    await expect(page).toHaveURL(/\/investigations\/?/)
    await expect(
      page.getByRole("heading", { name: "Investigations", exact: true })
    ).toBeVisible()

    await page.goto("/sql")
    await expect(page).toHaveURL(/\/sql\/?/)
    await expect(shell.navItem("SQL")).toBeVisible()
  })

  test("invalid route shows not-found surface @pw-shell-not-found", async ({
    page,
  }) => {
    const shell = new ShellScreen(page)
    await page.goto("/this-route-does-not-exist")
    await expect(shell.notFoundTitle()).toBeVisible()
    await expect(
      page.getByText("Pick a Parallax surface from the navigation.")
    ).toBeVisible()
  })

  test("theme choice persists across reload @pw-shell-theme", async ({
    page,
  }) => {
    const shell = new ShellScreen(page)
    await shell.openRoot()
    await shell.themeButton("Light").click()
    expect(await shell.documentThemeClass()).toContain("light")
    await page.reload()
    expect(await shell.documentThemeClass()).toContain("light")
    await shell.themeButton("Dark").click()
    expect(await shell.documentThemeClass()).toContain("dark")
  })

  test("recoverable API failure surfaces error panel @pw-shell-api-failure", async ({
    page,
    injectGraphqlFailure,
  }) => {
    const shell = new ShellScreen(page)
    await injectGraphqlFailure()
    await page.goto("/")
    await expect(shell.apiErrorTitle()).toBeVisible()
    await expect(page.getByText(/unreachable \(503\)/)).toBeVisible()

    // Full navigation recovers once the one-shot failure is consumed.
    await page.reload()
    await expect(shell.apiErrorTitle()).toHaveCount(0)
    await expect(shell.homeLink()).toBeVisible()
    await expect(shell.navItem("Overview")).toBeVisible()
  })
})
