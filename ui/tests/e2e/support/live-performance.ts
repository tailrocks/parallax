/**
 * Shared measurement helpers for plan 147 @live full-stack specs.
 * Collects typed observations only — no product assertions, no matrix ownership.
 */

import { expect } from "@playwright/test"
import type { Page } from "@playwright/test"

export interface LiveBurstObservation {
  readonly seededCount: number
  readonly visibleExactCounts: ReadonlyArray<number>
  readonly maxVisibleCount: number
}

export async function countExactText(page: Page, text: string): Promise<number> {
  return page.getByText(text, { exact: false }).count()
}

export async function waitForExactTextCount(
  page: Page,
  text: string,
  expected: number,
  timeoutMs = 45_000
): Promise<number> {
  let last = 0
  try {
    await expect
      .poll(
        async () => {
          last = await countExactText(page, text)
          return last
        },
        { timeout: timeoutMs, intervals: [200] }
      )
      .toBe(expected)
  } catch {
    // Bounded predicate did not converge; caller asserts on the last count.
  }
  return last
}

export function summarizeBurst(
  seededCount: number,
  visibleExactCounts: ReadonlyArray<number>
): LiveBurstObservation {
  return {
    seededCount,
    visibleExactCounts,
    maxVisibleCount: visibleExactCounts.reduce((max, n) => Math.max(max, n), 0),
  }
}

/** Best-effort wait for a Live control on product pages. */
export async function waitForLiveToggle(page: Page, timeoutMs = 10_000) {
  const toggle = page.getByRole("button", { name: /Live/i }).first()
  await toggle.waitFor({ state: "visible", timeout: timeoutMs })
  return toggle
}
