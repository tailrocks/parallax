import { productTest as test, expect } from "../fixtures/test"
import { assertNoHorizontalOverflow } from "../support/overflow"

test.describe("logs mobile @mobile", () => {
  test.use({ productDataset: "logs-pilot" })
  test("logs list no horizontal overflow @pw-mobile-logs", async ({ page }) => {
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: "Logs", exact: true })).toBeVisible({
      timeout: 15_000,
    })
    await assertNoHorizontalOverflow(page)
  })
})

test.describe("traces mobile @mobile", () => {
  test.use({ productDataset: "traces-pilot" })
  test("traces list no horizontal overflow @pw-mobile-traces", async ({ page }) => {
    await page.goto("/traces")
    await expect(page.getByRole("heading", { name: "Traces", exact: true })).toBeVisible({
      timeout: 15_000,
    })
    await assertNoHorizontalOverflow(page)
  })
})

test.describe("issues mobile @mobile", () => {
  test.use({ productDataset: "traces-pilot" })
  test("issues list no horizontal overflow @pw-mobile-issues", async ({ page }) => {
    await page.goto("/issues")
    await expect(page.getByRole("heading", { name: "Issues", exact: true })).toBeVisible({
      timeout: 15_000,
    })
    await assertNoHorizontalOverflow(page)
  })
})

test.describe("services mobile @mobile", () => {
  test.use({ productDataset: "traces-pilot" })
  test("services list no horizontal overflow @pw-mobile-services", async ({ page }) => {
    await page.goto("/services")
    await expect(page.getByRole("heading", { name: "Services", exact: true })).toBeVisible({
      timeout: 15_000,
    })
    await assertNoHorizontalOverflow(page)
  })
})

test.describe("dashboards mobile @mobile", () => {
  test.use({ productDataset: "dashboards-pilot" })
  test("dashboards list no horizontal overflow @pw-mobile-dashboards", async ({ page }) => {
    await page.goto("/dashboards")
    await expect(page.getByRole("heading", { name: "Dashboards", exact: true })).toBeVisible({
      timeout: 15_000,
    })
    await assertNoHorizontalOverflow(page)
  })
})
