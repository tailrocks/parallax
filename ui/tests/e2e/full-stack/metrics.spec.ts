import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack metrics @metrics", () => {
  test("catalog workbench and add-to-dashboard @pw-metrics-workbench", async ({
    page,
    fullStack,
  }) => {
    await page.goto("/metrics")
    await expect(page.getByRole("heading", { name: /metric/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.metric_name, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.goto("/metrics/" + fullStack.metric_name)
    await expect(page.getByText(fullStack.metric_name, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    const add = page.getByRole("button", { name: /add to dashboard/i })
    await expect(add).toBeVisible({ timeout: SURFACE_TIMEOUT_MS })
    await add.click()
    await expect(page.getByRole("dialog")).toBeVisible()
  })
})
