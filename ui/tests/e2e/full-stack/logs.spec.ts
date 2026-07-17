import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack logs @logs", () => {
  test("logs surface loads for seeded service @pw-full-stack-logs", async ({ page, fullStack }) => {
    const manifest = readFullStackManifest()
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: 20_000,
    })
    // Seed body or service may appear; at minimum the surface is live.
    const marker = page
      .getByText(fullStack.service, { exact: false })
      .or(page.getByText(manifest.log_body, { exact: false }))
    await expect(marker.first()).toBeVisible({ timeout: 20_000 })
  })
})
