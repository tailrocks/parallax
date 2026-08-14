import { DASHBOARD_PILOT_NAME } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("dashboards product contracts", () => {
  test.use({ productDataset: "dashboards-pilot" })

  test("create widget persist and delete @pw-dashboards-crud", async ({ page, snapshot }) => {
    await page.goto("/dashboards")
    await expect(page.getByRole("heading", { name: "Dashboards", exact: true })).toBeVisible()
    await expect(page.getByRole("link", { name: DASHBOARD_PILOT_NAME }).first()).toBeVisible()

    await page.getByRole("button", { name: "New dashboard" }).click()
    await page.getByPlaceholder("checkout ops").fill("Browser CRUD board")
    await page.getByPlaceholder("Search metrics").fill("http.server.duration")
    await page.getByRole("button", { name: "Add widget" }).click()
    await page.getByRole("button", { name: "Create" }).click()
    await expect(page).toHaveURL(/\/dashboards\/[^/]+/)
    await expect(page.getByRole("heading", { name: "Browser CRUD board" })).toBeVisible()

    await page.goto("/logs")
    await page.goto("/dashboards")
    await page.reload()
    await expect(page.getByRole("link", { name: "Browser CRUD board" }).first()).toBeVisible()

    const card = page.locator("li").filter({ hasText: "Browser CRUD board" })
    await card.getByRole("button", { name: "Delete" }).click()
    await expect(page.getByRole("alertdialog")).toContainText("Browser CRUD board")
    await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click()
    await page.reload()
    await expect(page.getByRole("link", { name: "Browser CRUD board" })).toHaveCount(0)
    await expect(page.getByText(DASHBOARD_PILOT_NAME).first()).toBeVisible()

    const state = await snapshot()
    expect(state.ok).toBe(true)
  })
})
