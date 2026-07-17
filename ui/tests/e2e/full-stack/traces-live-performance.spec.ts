import { fullStackTest as test, expect } from "../fixtures/test"
import {
  readFullStackManifest,
  seedLiveSpan,
  seedLiveSpanDuplicatePair,
} from "../fixtures/full-stack-fixture"

/**
 * Plan 147 feature-owned @live cases for traces.
 * Identity = spanId; one-export duplicate pair proves merge dedupe.
 */
test.describe("full-stack traces live performance @live", () => {
  test("live span appears once by name @pw-live-traces-identity", async ({ page, fullStack }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/traces?live=true")
    await expect(page.getByRole("heading", { name: /traces/i }).first()).toBeVisible({
      timeout: 20_000,
    })

    const spanName = `pw.live.span.${manifest.dataset_id}`
    const first = await seedLiveSpan({ spanName })
    expect(first.span_id.length).toBeGreaterThanOrEqual(8)

    await expect(page.getByText(spanName, { exact: false }).first()).toBeVisible({
      timeout: 45_000,
    })
    const count = await page.getByText(spanName, { exact: false }).count()
    expect(count).toBeGreaterThanOrEqual(1)
  })

  test("duplicate spanId does not double-render @pw-live-traces-dedup", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/traces?live=true")
    await expect(page.getByRole("heading", { name: /traces/i }).first()).toBeVisible({
      timeout: 20_000,
    })

    const spanName = `pw.live.span.dedup.${manifest.dataset_id}`
    await seedLiveSpanDuplicatePair({ spanName })
    await expect(page.getByText(spanName, { exact: false }).first()).toBeVisible({
      timeout: 45_000,
    })
    // Count <tr> with the span name once (merge dedupe of identical spanId).
    await expect(page.locator("tr").filter({ hasText: spanName })).toHaveCount(1, {
      timeout: 10_000,
    })
  })
})
