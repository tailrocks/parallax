import { fullStackTest as test, expect } from "../fixtures/test"
import { InvestigationsScreen } from "../screens/investigations-screen"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack investigations @investigations", () => {
  test("investigations surface mounts on managed stack @pw-full-stack-investigations", async ({
    page,
  }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.heading()).toBeVisible({ timeout: SURFACE_TIMEOUT_MS })
    // Empty or populated is fine; surface must be reachable without intercepts.
    await expect(screen.newButton()).toBeVisible()
  })
})
