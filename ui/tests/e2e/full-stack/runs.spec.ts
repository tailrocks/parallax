import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack invocations @runs", () => {
  test("invocations list loads seeded invocation @pw-full-stack-runs", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    await page.goto("/invocations")
    // Product surface title is "CLI Apps" (invocations list).
    await expect(page.getByText("CLI Apps", { exact: true }).first()).toBeVisible({
      timeout: 20_000,
    })
    // Prefer exact invocation id when rendered; otherwise service marker.
    const marker = page
      .getByText(fullStack.invocation_id, { exact: false })
      .or(page.getByText(manifest.service, { exact: false }))
    await expect(marker.first()).toBeVisible({ timeout: 20_000 })
  })
})
