import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack overview @overview", () => {
  test("overview renders for seeded stack @pw-full-stack-overview", async ({ page, fullStack }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)
    await page.goto("/")
    await expect(page.getByRole("heading", { name: /overview/i }).first()).toBeVisible({
      timeout: 20_000,
    })
  })
})
