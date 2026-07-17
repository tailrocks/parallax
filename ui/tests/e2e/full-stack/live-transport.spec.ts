import { fullStackTest as test, expect } from "../fixtures/test"
import { seedLiveLog, readFullStackManifest } from "../fixtures/full-stack-fixture"

/**
 * One-event @storage infrastructure smoke for live transport.
 * Plan 147 owns burst/capacity/identity/perf @live cases — do not expand here.
 */
test.describe("full-stack live transport @storage", () => {
  test("one post-open log appears once through live path @pw-storage-live-transport", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: /log/i }).first()).toBeVisible({
      timeout: 20_000,
    })

    // Prefer an explicit Live toggle when the surface exposes one.
    const liveToggle = page.getByRole("button", { name: /live/i }).first()
    if (await liveToggle.isVisible().catch(() => false)) {
      await liveToggle.click()
    }

    const body = `pw-live-once-${manifest.dataset_id}`
    const seeded = await seedLiveLog(body)
    expect(seeded.body).toBe(body)

    // Bound eventual visibility via UI text (SSE or poll refresh).
    const marker = page.getByText(body, { exact: false })
    await expect(marker).toHaveCount(1, { timeout: 30_000 })

    // Hide/show cycle should not duplicate the same record.
    await page.goto("/")
    await page.goto("/logs")
    if (await liveToggle.isVisible().catch(() => false)) {
      // After navigation the toggle is a new locator.
      const again = page.getByRole("button", { name: /live/i }).first()
      if (await again.isVisible().catch(() => false)) {
        await again.click()
      }
    }
    await expect(page.getByText(body, { exact: false })).toHaveCount(1, { timeout: 30_000 })
  })
})
