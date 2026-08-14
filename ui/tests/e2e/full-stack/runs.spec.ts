import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack invocations @runs", () => {
  test("invocations list loads seeded invocation @pw-full-stack-runs", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    await page.goto("/invocations")
    // Product surface title is "CLI Apps" (invocations list).
    await expect(page.getByText("CLI Apps", { exact: true }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.invocation_id, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(manifest.service, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
