import { fullStackTest as test, expect } from "../fixtures/test"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

const TABS = ["Overview", "Traces", "Logs", "Errors", "Sessions & UI", "Jobs & Cycles"] as const

test.describe("full-stack invocation hub @runs", () => {
  test("six hub tabs render seeded invocation @pw-invocation-hub", async ({ page, fullStack }) => {
    await page.goto("/invocations/" + fullStack.invocation_id)
    await expect(page.getByText(fullStack.invocation_id).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    for (const tab of TABS) {
      await page.getByRole("tab", { name: tab }).click()
      await expect(page.getByRole("tab", { name: tab })).toHaveAttribute("aria-selected", "true")
    }
  })
})
