import { productTest as test, expect } from "../fixtures/test"

test.describe("sql product contracts", () => {
  test.use({ productDataset: "sql-pilot" })

  test("run query snippet save and delete @pw-sql-crud", async ({ page }) => {
    await page.goto("/sql")
    await expect(page.getByRole("heading", { name: "SQL", exact: true })).toBeVisible()

    const editor = page.locator('textarea[name="sql-statement"]')
    await editor.fill("SELECT count(*)")
    await page.getByRole("button", { name: "Run query" }).click()
    await expect(page.getByText("Query result")).toBeVisible()
    await expect(page.getByText(/1 rows/)).toBeVisible()

    await editor.fill("SELECT nope FROM missing_table")
    await page.getByRole("button", { name: "Run query" }).click()
    await expect(page.locator("p.text-destructive")).toBeVisible()
    await expect(page.getByText("No logs yet")).toHaveCount(0)

    await page.getByRole("button", { name: "Snippets" }).click()
    await page.getByRole("menuitem", { name: "Save current snippet" }).click()
    await page.getByPlaceholder("Snippet name").fill("quote's count")
    await page.getByRole("button", { name: "Save" }).click()
    await page.reload()
    await page.getByRole("button", { name: "Snippets" }).click()
    await expect(page.getByRole("menuitem", { name: "quote's count" }).first()).toBeVisible()
    await page.getByRole("menuitem", { name: "quote's count" }).nth(1).click()
    await page.getByRole("button", { name: "Snippets" }).click()
    await expect(page.getByRole("menuitem", { name: "quote's count" })).toHaveCount(0)
  })
})
