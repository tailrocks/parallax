import { fullStackTest as test, expect } from "../fixtures/test"
import { readFullStackManifest, seedLiveLog, seedLiveSpan } from "../fixtures/full-stack-fixture"

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
      timeout: 20_000,
    })

    const goLive = page.getByRole("button", { name: /Go live/i })
    if (await goLive.isVisible().catch(() => false)) {
      await goLive.click()
    }
    await expect(page.getByRole("button", { name: /^Live$/i }).first()).toBeVisible({
      timeout: 10_000,
    })

    const logsTab = page.getByRole("tab", { name: /^Logs$/i })
    if (await logsTab.isVisible().catch(() => false)) {
      await logsTab.click()
    }

    const body = `pw-live-hub-log-${manifest.dataset_id}-${Date.now()}`
    await seedLiveLog(body)
    await expect(page.getByText(body, { exact: false }).first()).toBeVisible({
      timeout: 45_000,
    })
  })

  test("hub live span appears once @pw-live-runs-span", async ({ page, fullStack }) => {
    const manifest = readFullStackManifest()
    await page.goto(
      `/invocations/${encodeURIComponent(fullStack.invocation_id)}?live=true&tab=traces`
    )
    await expect(page.getByText(fullStack.invocation_id, { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })

    const goLive = page.getByRole("button", { name: /Go live/i })
    if (await goLive.isVisible().catch(() => false)) {
      await goLive.click()
    }
    await expect(page.getByRole("button", { name: /^Live$/i }).first()).toBeVisible({
      timeout: 10_000,
    })

    const tracesTab = page.getByRole("tab", { name: /^Traces$/i })
    if (await tracesTab.isVisible().catch(() => false)) {
      await tracesTab.click()
    }

    const spanName = `pw.live.hub.span.${manifest.dataset_id}.${Date.now()}`
    await seedLiveSpan({ spanName })
    await expect(page.getByText(spanName, { exact: false }).first()).toBeVisible({
      timeout: 45_000,
    })
  })
})
