import { productTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"

test.describe("shell mobile/touch @mobile", () => {
  test.use({ productDataset: "shell-empty" })

  test("device settings active and nav without horizontal overflow @pw-shell-mobile", async ({
    page,
  }, testInfo) => {
    // Prove Playwright device descriptor is live (not a viewport-only claim).
    const projectUse = testInfo.project.use
    expect(projectUse.isMobile, "mobile project must set isMobile").toBe(true)
    expect(projectUse.hasTouch, "mobile project must set hasTouch").toBe(true)
    expect(projectUse.viewport?.width ?? 0).toBeLessThanOrEqual(430)

    const shell = new ShellScreen(page)
    await shell.openRoot()
    // Desktop brand text lives in a collapsible sidebar; on mobile the home
    // control and page heading remain the stable landmarks.
    await expect(
      shell
        .homeLink()
        .or(page.getByRole("heading", { name: /overview/i }))
        .first()
    ).toBeVisible({
      timeout: 15_000,
    })

    const isMobile = await page.evaluate(() => {
      return {
        touch: "ontouchstart" in window || navigator.maxTouchPoints > 0,
        width: window.innerWidth,
        ua: navigator.userAgent,
      }
    })
    expect(isMobile.width).toBeLessThanOrEqual(430)
    expect(isMobile.touch || /Mobile|Android|iPhone/i.test(isMobile.ua)).toBe(true)

    // Mobile nav may live behind a menu control — use direct deep-link + tap.
    await page.goto("/investigations")
    await expect(page.getByRole("heading", { name: "Investigations", exact: true })).toBeVisible({
      timeout: 15_000,
    })

    const overflow = await page.evaluate(() => {
      const doc = document.documentElement
      return {
        scrollWidth: doc.scrollWidth,
        clientWidth: doc.clientWidth,
      }
    })
    expect(overflow.scrollWidth).toBeLessThanOrEqual(overflow.clientWidth + 1)
  })
})
