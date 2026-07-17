import { productTest as test, expect } from "../fixtures/test"

/**
 * Plan 148 @bundle browser resource gates.
 * Assert direct entry / navigation never request source maps and that JS/CSS
 * assets load. Runs on fixture-backed contracts server (not production embed).
 */
test.describe("bundle resource traces @bundle", () => {
  test.use({ productDataset: "shell-empty" })

  test("direct shell entry loads without source maps @pw-bundle-shell-entry", async ({ page }) => {
    const mapRequests: string[] = []
    const jsCss: string[] = []
    page.on("request", (request) => {
      const url = request.url()
      if (url.endsWith(".map") || url.includes(".map?")) mapRequests.push(url)
      if (/\.(?:js|css)(?:\?|$)/.test(url)) jsCss.push(url)
    })

    await page.goto("/")
    await expect(page.getByRole("link", { name: /parallax|home|overview/i }).first()).toBeVisible({
      timeout: 15_000,
    })

    expect(mapRequests, `source map requests: ${mapRequests.join(", ")}`).toEqual([])
    // At least one script or stylesheet should have been requested for the shell.
    expect(jsCss.length).toBeGreaterThan(0)
  })

  test("logs navigation does not request source maps @pw-bundle-logs-nav", async ({ page }) => {
    const mapRequests: string[] = []
    page.on("request", (request) => {
      const url = request.url()
      if (url.endsWith(".map") || url.includes(".map?")) mapRequests.push(url)
    })

    await page.goto("/")
    await page.goto("/logs")
    await expect(page.getByRole("heading", { name: /logs/i }).first()).toBeVisible({
      timeout: 15_000,
    })
    expect(mapRequests, `source map requests: ${mapRequests.join(", ")}`).toEqual([])
  })

  test("traces navigation does not request source maps @pw-bundle-traces-nav", async ({ page }) => {
    const mapRequests: string[] = []
    page.on("request", (request) => {
      const url = request.url()
      if (url.endsWith(".map") || url.includes(".map?")) mapRequests.push(url)
    })

    await page.goto("/")
    await page.goto("/traces")
    await expect(page.getByRole("heading", { name: /traces/i }).first()).toBeVisible({
      timeout: 15_000,
    })
    expect(mapRequests, `source map requests: ${mapRequests.join(", ")}`).toEqual([])
  })
})
