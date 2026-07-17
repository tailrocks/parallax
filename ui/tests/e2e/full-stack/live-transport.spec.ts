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

    await page.goto("/logs?live=true")
    await expect(page.getByRole("heading", { name: "Logs" })).toBeVisible({
      timeout: 20_000,
    })
    const liveToggle = page.getByRole("button", { name: "Live", exact: true })
    await expect(liveToggle).toBeVisible({ timeout: 10_000 })

    const body = `pw-live-once-${manifest.dataset_id}`
    const seeded = await seedLiveLog(body)
    expect(seeded.body).toBe(body)

    const marker = page.getByText(body, { exact: false })
    await expect(marker).toHaveCount(1, { timeout: 45_000 })

    // Disconnect/reconnect via exact Live toggle (not table rows).
    await liveToggle.click()
    const queryToggle = page.getByRole("button", { name: "Query", exact: true })
    await expect(queryToggle).toBeVisible()
    await queryToggle.click()
    await expect(page.getByRole("button", { name: "Live", exact: true })).toBeVisible()

    // Reconnect must not double-render the same record (0 or 1 is fine; never 2+).
    await expect
      .poll(async () => page.getByText(body, { exact: false }).count(), {
        timeout: 10_000,
      })
      .toBeLessThan(2)
  })
})
