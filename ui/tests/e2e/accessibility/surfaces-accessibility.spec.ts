import { productTest as test, expect } from "../fixtures/test"
import { assertNoAxeViolations } from "../fixtures/accessibility-fixture"

test.describe("logs a11y @a11y", () => {
  test.use({ productDataset: "logs-pilot" })
  test("logs list axe @pw-a11y-logs", async ({ page }) => {
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: "Logs", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)
  })
})

test.describe("traces a11y @a11y", () => {
  test.use({ productDataset: "traces-pilot" })
  test("traces list axe @pw-a11y-traces", async ({ page }) => {
    await page.goto("/traces")
    await expect(page.getByRole("heading", { name: "Traces", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)
  })
})

test.describe("issues a11y @a11y", () => {
  test.use({ productDataset: "traces-pilot" })
  test("issues list axe @pw-a11y-issues", async ({ page }) => {
    await page.goto("/issues")
    await expect(page.getByRole("heading", { name: "Issues", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)
  })
})

test.describe("services a11y @a11y", () => {
  test.use({ productDataset: "traces-pilot" })
  test("services list axe @pw-a11y-services", async ({ page }) => {
    await page.goto("/services")
    await expect(page.getByRole("heading", { name: "Services", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)
  })
})

test.describe("dashboards a11y @a11y", () => {
  test.use({ productDataset: "dashboards-pilot" })
  test("dashboards list axe @pw-a11y-dashboards", async ({ page }) => {
    await page.goto("/dashboards")
    await expect(page.getByRole("heading", { name: "Dashboards", exact: true })).toBeVisible()
    await assertNoAxeViolations(page)
  })
})
