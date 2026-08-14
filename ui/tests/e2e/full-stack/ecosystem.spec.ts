import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack ecosystem @ecosystem", () => {
  test("ecosystem surface mounts on managed stack @pw-full-stack-ecosystem", async ({
    page,
    fullStack,
  }) => {
    await page.goto("/ecosystem")
    await expect(page.getByRole("heading", { name: /ecosystem/i }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(page.getByText(fullStack.service, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
  })
})
