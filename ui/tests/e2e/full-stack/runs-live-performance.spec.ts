import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest, seedLiveLog, seedLiveSpan } from "../fixtures/full-stack-fixture"
import { LIVE_TIMEOUT_MS, SURFACE_TIMEOUT_MS } from "../support/timeouts"

/**
 * Feature-owned @live cases for invocations/runs hub (plan 147).
 * scenario_owner remains features/invocations (runs facade owner).
 */
test.describe("full-stack invocations live performance @live", () => {
  test("hub live log appears under invocation stream @pw-live-runs-log", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    await page.goto(
      `/invocations/${encodeURIComponent(fullStack.invocation_id)}?live=true&tab=logs`
    )
    await expect(page.getByText(fullStack.invocation_id, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })

    const goLive = page.getByRole("button", { name: /Go live/i })
    if ((await goLive.count()) > 0) {
      await goLive.click()
    }
    await expect(page.getByRole("button", { name: /^Live$/i }).first()).toBeVisible({
      timeout: 10_000,
    })

    await expect(page.getByRole("tab", { name: /^Logs$/i })).toBeVisible()
    await page.getByRole("tab", { name: /^Logs$/i }).click()

    const body = `pw-live-hub-log-${manifest.dataset_id}-${Date.now()}`
    await seedLiveLog(body)
    await expect(page.getByText(body, { exact: false }).first()).toBeVisible({
      timeout: LIVE_TIMEOUT_MS,
    })
  })

  test("hub live span appears once @pw-live-runs-span", async ({ page, fullStack }) => {
    const manifest = readFullStackManifest()
    await page.goto(
      `/invocations/${encodeURIComponent(fullStack.invocation_id)}?live=true&tab=traces`
    )
    await expect(page.getByText(fullStack.invocation_id, { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })

    const goLive = page.getByRole("button", { name: /Go live/i })
    if ((await goLive.count()) > 0) {
      await goLive.click()
    }
    await expect(page.getByRole("button", { name: /^Live$/i }).first()).toBeVisible({
      timeout: 10_000,
    })

    await expect(page.getByRole("tab", { name: /^Traces$/i })).toBeVisible()
    await page.getByRole("tab", { name: /^Traces$/i }).click()

    const spanName = `pw.live.hub.span.${manifest.dataset_id}.${Date.now()}`
    await seedLiveSpan({ spanName })
    await expect(page.getByText(spanName, { exact: false }).first()).toBeVisible({
      timeout: LIVE_TIMEOUT_MS,
    })
  })
})
