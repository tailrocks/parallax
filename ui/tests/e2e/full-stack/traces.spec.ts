import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack traces @traces", () => {
  test("traces surface loads for seeded stack @pw-full-stack-traces", async ({ page }) => {
    const manifest = readFullStackManifest()
    await page.goto("/traces")
    await expect(page.getByRole("heading", { name: /trace/i }).first()).toBeVisible({
      timeout: 20_000,
    })
    expect(manifest.trace_id.length).toBeGreaterThan(0)
  })
})
