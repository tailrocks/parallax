import { ALERT_RULE_PILOT_NAME } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("alerts product contracts", () => {
  test.use({ productDataset: "alerts-pilot" })

  test("destination rule toggle and delete @pw-alerts-crud", async ({ page }) => {
    await page.goto("/alerts")
    await expect(page.getByRole("heading", { name: "Alerts", exact: true })).toBeVisible()
    await expect(page.getByText(ALERT_RULE_PILOT_NAME)).toBeVisible()

    await page.getByRole("tab", { name: /Destinations/ }).click()
    await page.getByRole("button", { name: "Add destination" }).click()
    await page.getByLabel("Name").fill("Pager hook")
    await page.getByLabel("URL").fill("https://example.test/hooks/pager")
    await page.getByRole("button", { name: "Add destination" }).last().click()
    await expect(page.getByText("Pager hook")).toBeVisible()

    await page.getByRole("tab", { name: /Rules/ }).click()
    await page.getByRole("button", { name: "New rule" }).click()
    await page.getByLabel("Name").fill('Quote\\rule "name"')
    await page.getByRole("button", { name: "Create rule" }).click()
    await expect(page.getByText('Quote\\rule "name"')).toBeVisible()

    const toggle = page.getByRole("switch", { name: 'Enable Quote\\rule "name"' })
    await expect(toggle).toBeChecked()
    await toggle.click()
    await expect(toggle).not.toBeChecked()
    await toggle.click()
    await expect(toggle).toBeChecked()

    const ruleRow = page.getByRole("row").filter({ hasText: 'Quote\\rule "name"' })
    await ruleRow.getByRole("button", { name: "Delete" }).click()
    await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click()
    await expect(page.getByText('Quote\\rule "name"')).toHaveCount(0)

    await page.getByRole("tab", { name: /Destinations/ }).click()
    const destRow = page.getByRole("row").filter({ hasText: "Pager hook" })
    await destRow.getByRole("button", { name: "Delete" }).click()
    await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click()
    await expect(page.getByText("Pager hook")).toHaveCount(0)
  })
})
