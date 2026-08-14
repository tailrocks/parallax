import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack services @services", () => {
  test("seeded service visible on services list @pw-full-stack-services", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    await page.goto("/services")
    await expect(page.getByText(fullStack.service, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    expect(manifest.service).toBe(fullStack.service)
  })
})
