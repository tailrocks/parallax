import { productTest as test, expect } from "../fixtures/test"

test.describe("overview empty onboarding", () => {
  test.use({
    productDataset: "shell-empty",
    permissions: ["clipboard-read", "clipboard-write"],
  })

  test("overview empty state shows copyable setup snippets @pw-overview-empty-snippets", async ({
    page,
  }) => {
    await page.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          writeText: async () => undefined,
        },
      })
    })
    await page.goto("/")
    await expect(page.getByText("Send your first telemetry", { exact: true })).toBeVisible()
    const tabs = page.getByTestId("instrument-snippet-tabs")
    await expect(tabs.getByRole("tab", { name: "Rust" })).toBeVisible()
    await expect(tabs.getByRole("tab", { name: "Java" })).toBeVisible()
    await expect(tabs.getByRole("tab", { name: "JS" })).toBeVisible()
    await expect(tabs.getByText(/init_tracing/)).toBeVisible()

    await tabs.getByRole("tab", { name: "Java" }).click()
    await expect(tabs.getByText(/OTEL_SERVICE_NAME/)).toBeVisible()
    await tabs.getByRole("button", { name: "Copy" }).click()
    await expect(tabs.getByRole("button", { name: "Copied" })).toBeVisible()
  })
})
