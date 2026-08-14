import { fullStackTest as test, expect } from "../fixtures/test"
import { graphqlQuery } from "../fixtures/full-stack-fixture"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

/** Greptime native table base — same map as `native_metric_table_base`. */
function catalogMetricName(otlpName: string): string {
  return otlpName.replace(/[^A-Za-z0-9_]/g, "_")
}

test.describe("full-stack metrics @metrics", () => {
  test("catalog workbench and add-to-dashboard @pw-metrics-workbench", async ({
    page,
    fullStack,
  }) => {
    const canonical = catalogMetricName(fullStack.metric_name)
    let catalogName = canonical
    await expect
      .poll(
        async () => {
          const data = await graphqlQuery<{ metricNames: string[] }>(`{ metricNames }`)
          const found = data.metricNames.find(
            (name) => name === canonical || name === fullStack.metric_name
          )
          if (found) catalogName = found
          return found ?? ""
        },
        { timeout: SURFACE_TIMEOUT_MS }
      )
      .not.toEqual("")
    await page.goto("/metrics")
    await expect(page.getByRole("heading", { name: /metric/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.getByPlaceholder("Search").fill(catalogName)
    await expect(page.getByRole("link", { name: catalogName }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.goto("/metrics/" + catalogName)
    await expect(page.getByText(catalogName, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    const add = page.getByRole("button", { name: /add to dashboard/i })
    await expect(add).toBeVisible({ timeout: SURFACE_TIMEOUT_MS })
    await add.click()
    await expect(page.getByRole("dialog")).toBeVisible()
  })
})
