import { LOGS_PILOT_BODY } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("logs saved views", () => {
  test.use({ productDataset: "logs-pilot" })

  test("filter save restore and delete view @pw-logs-views-crud", async ({ page }) => {
    await page.goto("/logs")
    await expect(page.getByText(LOGS_PILOT_BODY)).toBeVisible()
    await expect(page.getByText("billing declined")).toBeVisible()

    const where = page.getByLabel("Where clause")
    await where.fill('service = "checkout"')
    await where.press("Enter")
    await expect(page).toHaveURL(/where=/)
    await expect(page.getByText(LOGS_PILOT_BODY)).toBeVisible()
    await expect(page.getByText("billing declined")).toHaveCount(0)

    await page.getByRole("button", { name: "Views" }).click()
    await page.getByRole("menuitem", { name: "Save current view" }).click()
    await page.getByPlaceholder("View name").fill("pay's checkout")
    await page.getByRole("button", { name: "Save" }).click()

    await page.goto("/logs")
    await expect(page.getByText("billing declined")).toBeVisible()
    await page.getByRole("button", { name: "Views" }).click()
    await page.getByRole("menuitem", { name: "pay's checkout" }).first().click()
    await expect(page).toHaveURL(/where=/)
    await expect(page.getByText(LOGS_PILOT_BODY)).toBeVisible()
    await expect(page.getByText("billing declined")).toHaveCount(0)

    await page.getByRole("button", { name: "Views" }).click()
    await page.getByRole("menuitem", { name: "pay's checkout" }).nth(1).click()
    await page.getByRole("button", { name: "Views" }).click()
    await expect(page.getByRole("menuitem", { name: "pay's checkout" })).toHaveCount(0)
  })
})
