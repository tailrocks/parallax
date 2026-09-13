import { chromium } from "@playwright/test"

const BASE = "http://127.0.0.1:4001"
const browser = await chromium.launch()
const page = await browser.newPage()
page.on("pageerror", (e) => console.log("PAGEERROR:", e.message.slice(0, 300)))
page.on("response", (r) => {
  if (r.url().includes("/graphql") && r.status() !== 200) console.log("GQL non-200:", r.status())
})
await page.goto(`${BASE}/issues/web-shop/370796ba0ea6ec2e?range=24h`, {
  waitUntil: "networkidle",
})
await page.screenshot({ path: "/tmp/rum-verify/debug-issue.png" })
const text = await page.locator("body").innerText()
console.log(text.slice(0, 2000))
await browser.close()
