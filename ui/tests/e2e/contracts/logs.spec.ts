import { LOGS_PILOT_BODY } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("logs product pilot", () => {
  test.use({ productDataset: "logs-pilot" })

  test("lists seeded log body across services @pw-logs-pilot-body", async ({ page, snapshot }) => {
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: "Logs", exact: true })).toBeVisible()
    await expect(page.getByText(LOGS_PILOT_BODY)).toBeVisible()
    await expect(page.getByText("checkout", { exact: true }).first()).toBeVisible()
    await expect(page.getByText("billing", { exact: true }).first()).toBeVisible()

    const state = await snapshot()
    expect(state.counts.logs).toBe(6)
  })
})
