import { fullStackTest as test, expect } from "../fixtures/test"

test.describe("full-stack metrics @metrics", () => {
  test("catalog workbench and add-to-dashboard @pw-metrics-workbench", async ({
    page,
    fullStack,
  }) => {
    await page.goto("/metrics")
    await expect(page.getByRole("heading", { name: /metric/i }).first()).toBeVisible({
      timeout: 20_000,
    })
    await expect(page.getByText(fullStack.metric_name, { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })
    await page.goto("/metrics/" + fullStack.metric_name)
    await expect(page.getByText(fullStack.metric_name, { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })
    const add = page.getByRole("button", { name: /add to dashboard/i })
    if (await add.count()) {
      await add.click()
      await expect(page.getByRole("dialog")).toBeVisible()
    }
  })
})
