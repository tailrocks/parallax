import { chromium } from "@playwright/test"

const BASE = "http://127.0.0.1:4001"
const browser = await chromium.launch()
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } })
const errors: string[] = []
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message.slice(0, 200)}`))

// 1. Issues index renders live + links carry service scope.
await page.goto(`${BASE}/issues?range=24h`, { waitUntil: "networkidle" })
await page.getByText("TypeError").first().waitFor({ timeout: 15000 })
const href = await page
  .getByRole("link", { name: /TypeError/ })
  .first()
  .getAttribute("href")
console.log("issues index ok; detail href:", href)
if (!href?.startsWith("/issues/web-shop/370796ba0ea6ec2e")) throw new Error("bad issue href")
await page.screenshot({ path: "/tmp/rum-verify/issues-index.png" })

// 2. Issue detail: backend events bug -> honest error panel, no crash.
await page.goto(`${BASE}/issues/web-shop/370796ba0ea6ec2e?range=24h`, {
  waitUntil: "networkidle",
})
await page.getByText(/did not answer|not found|Retry/i).first().waitFor({ timeout: 15000 })
console.log("issue detail state:", await page.locator("body").innerText().then((t) => t.slice(t.indexOf("Local") + 5, t.indexOf("Local") + 160).replace(/\n/g, " | ")))
await page.screenshot({ path: "/tmp/rum-verify/issue-detail.png" })

// 3. RUM route: vitals + errors + journeys live.
await page.goto(`${BASE}/rum?range=24h`, { waitUntil: "networkidle" })
await page.getByTestId("vital-row-browser_lcp_milliseconds").waitFor({ timeout: 15000 })
await page.getByText("TypeError").first().waitFor({ timeout: 15000 })
await page.getByText("page /checkout").first().waitFor({ timeout: 15000 })
console.log("rum list ok")
await page.screenshot({ path: "/tmp/rum-verify/rum.png" })

// 4. Vital detail + journey trace correlation.
await page.getByTestId("vital-row-browser_lcp_milliseconds").click()
await page.getByText("Trace exemplars").waitFor({ timeout: 15000 })
await page.getByTestId("trace-row-b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2").first().click()
await page.getByText("checkout failed").waitFor({ timeout: 15000 })
console.log("rum detail + correlation ok")
await page.screenshot({ path: "/tmp/rum-verify/rum-detail.png" })

// 5. Palette dead-end regression: metrics + alerts navigate.
await page.keyboard.press("Control+k")
await page.getByPlaceholder(/search pages/i).fill("metrics")
await page.getByRole("dialog").getByText("Metrics", { exact: true }).click()
await page.waitForURL("**/metrics**", { timeout: 10000 })
console.log("palette metrics ok:", page.url())
await page.keyboard.press("Control+k")
await page.getByPlaceholder(/search pages/i).fill("alerts")
await page.getByRole("dialog").getByText("Alerts", { exact: true }).click()
await page.waitForURL("**/alerts**", { timeout: 10000 })
console.log("palette alerts ok:", page.url())

console.log("js pageerrors:", errors.length ? errors : "none")
await browser.close()
if (errors.length) process.exit(1)
