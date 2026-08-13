import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack issues @issues", () => {
  test("seeded issue open on issues list and detail @pw-full-stack-issues", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    await page.goto("/issues")
    await expect(page.getByText(manifest.error_type, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.goto(`/issues/${fullStack.issue_fingerprint}`)
    await expect(page.getByText(manifest.service, { exact: false }).first()).toBeVisible({
      timeout: 15_000,
    })
  })
})
