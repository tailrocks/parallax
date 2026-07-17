import { fullStackTest as test, expect } from "../fixtures/test"
import {
  readFullStackManifest,
  seedLiveLog,
  seedLiveLogBurst,
  seedLiveLogDuplicatePair,
} from "../fixtures/full-stack-fixture"
import { summarizeBurst, waitForExactTextCount } from "../support/live-performance"

/**
 * Plan 147 feature-owned @live cases for logs.
 * Distinct from plan 145 @storage one-event smoke (live-transport.spec.ts).
 */
test.describe("full-stack logs live performance @live", () => {
  test("burst of five live logs each appear once @pw-live-logs-burst", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/logs?live=true")
    await expect(page.getByRole("heading", { name: "Logs" })).toBeVisible({
      timeout: 20_000,
    })
    await expect(page.getByRole("button", { name: "Live", exact: true })).toBeVisible({
      timeout: 10_000,
    })

    const prefix = `pw-live-burst-${manifest.dataset_id}`
    const seeded = await seedLiveLogBurst(5, prefix)
    expect(seeded.count).toBe(5)
    expect(seeded.bodies).toHaveLength(5)

    const counts: number[] = []
    for (const body of seeded.bodies) {
      const n = await waitForExactTextCount(page, body, 1)
      counts.push(n)
      expect(n, `body ${body} must appear exactly once`).toBe(1)
    }
    const observation = summarizeBurst(seeded.count, counts)
    expect(observation.maxVisibleCount).toBe(1)
  })

  test("duplicate identity does not double-render @pw-live-logs-dedup", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/logs?live=true")
    await expect(page.getByRole("heading", { name: "Logs" })).toBeVisible({
      timeout: 20_000,
    })

    const body = `pw-live-dup-${manifest.dataset_id}`
    // Two identical rows in one OTLP export — merge identity must keep one.
    await seedLiveLogDuplicatePair(body)
    await expect(page.getByText(body, { exact: false }).first()).toBeVisible({
      timeout: 45_000,
    })
    // Log rows use role=button; count <tr> with the body marker.
    await expect(page.locator("tr").filter({ hasText: body })).toHaveCount(1, {
      timeout: 10_000,
    })
  })

  test("filter generation change drops old stream delivery @pw-live-logs-filter-reset", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)

    await page.goto("/logs?live=true")
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: 30_000,
    })
    await expect(page.getByRole("button", { name: "Live", exact: true })).toBeVisible({
      timeout: 10_000,
    })

    const before = `pw-live-pre-filter-${manifest.dataset_id}-${Date.now()}`
    await seedLiveLog(before)
    await expect(page.locator("tr").filter({ hasText: before })).toHaveCount(1, {
      timeout: 45_000,
    })

    // Wrong service filter → stream regenerates; prior live rows cleared.
    await page.goto(`/logs?live=true&service=no-such-service-${manifest.dataset_id}`)
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: 30_000,
    })
    await expect(page.locator("tr").filter({ hasText: before })).toHaveCount(0, {
      timeout: 10_000,
    })

    // Correct service again: new seed appears once.
    await page.goto(`/logs?live=true&service=${encodeURIComponent(manifest.service)}`)
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: 30_000,
    })
    const after = `pw-live-post-filter-${manifest.dataset_id}-${Date.now()}`
    await seedLiveLog(after)
    await expect(page.locator("tr").filter({ hasText: after })).toHaveCount(1, {
      timeout: 45_000,
    })
  })
})
