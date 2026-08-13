import { fullStackTest as test, expect } from "../fixtures/test"

const TABS = ["Overview", "Traces", "Logs", "Errors", "Sessions & UI", "Jobs & Cycles"] as const

test.describe("full-stack invocation hub @runs", () => {
  test("six hub tabs render seeded invocation @pw-invocation-hub", async ({ page, fullStack }) => {
    await page.goto("/invocations/" + fullStack.invocation_id)
    await expect(page.getByText(fullStack.invocation_id).first()).toBeVisible({
      timeout: 20_000,
    })
    for (const tab of TABS) {
      await page.getByRole("tab", { name: tab }).click()
      await expect(page.getByRole("tab", { name: tab })).toHaveAttribute("data-state", /active|on/)
    }
  })
})
