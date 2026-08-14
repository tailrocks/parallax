import { METRICS_PILOT_GAUGE, METRICS_PILOT_HISTOGRAM } from "../datasets/catalog"
import { productTest as test, expect } from "../fixtures/test"

test.describe("metrics product pilot", () => {
  test.use({ productDataset: "metrics-pilot" })

  test("lists seeded gauge and histogram @pw-metrics-pilot-catalog", async ({ page, snapshot }) => {
    await page.goto("/metrics")
    await expect(page.getByRole("heading", { name: "Metrics", exact: true })).toBeVisible()
    await page.getByPlaceholder("Search").fill(METRICS_PILOT_GAUGE)
    await expect(page.getByRole("link", { name: METRICS_PILOT_GAUGE })).toBeVisible()
    await page.getByPlaceholder("Search").fill(METRICS_PILOT_HISTOGRAM)
    await expect(page.getByRole("link", { name: METRICS_PILOT_HISTOGRAM })).toBeVisible()

    await page.goto("/metrics/" + encodeURIComponent(METRICS_PILOT_GAUGE))
    await expect(page.getByText(METRICS_PILOT_GAUGE, { exact: false }).first()).toBeVisible()

    const state = await snapshot()
    expect(state.dataset_id).toBe("metrics-pilot")
    expect(state.counts.metrics).toBe(2)
  })
})
