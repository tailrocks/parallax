import { defineConfig, type PlaywrightTestConfig } from "@playwright/test"

const env = process.env
const isCi = env["CI"] === "true" || env["CI"] === "1"
const foundationPort = env["PARALLAX_BROWSER_FOUNDATION_PORT"] ?? "4173"
const baseURL = `http://127.0.0.1:${foundationPort}`

/**
 * Plan 132 Bun-only Playwright foundation.
 *
 * Runtime: lock-local `@playwright/test` forced through Bun
 * (`bunx --bun --no-install`). Browser binaries are provisioned by an explicit
 * install command, never install lifecycle scripts.
 */
const config: PlaywrightTestConfig = {
  testDir: "./tests/e2e",
  fullyParallel: true,
  forbidOnly: isCi,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 5_000 },
  reporter: isCi
    ? [
        ["line"],
        ["blob", { outputDir: "blob-report" }],
        ["junit", { outputFile: "test-results/junit.xml" }],
      ]
    : [
        ["line"],
        ["html", { open: "never", outputFolder: "playwright-report" }],
      ],
  use: {
    baseURL,
    locale: "en-US",
    timezoneId: "UTC",
    colorScheme: "dark",
    launchOptions: {
      args: ["--force-prefers-reduced-motion"],
    },
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    contextOptions: {
      reducedMotion: "reduce",
    },
  },
  projects: [
    {
      name: "foundation-chromium",
      testMatch: "**/smoke/**/*.spec.ts",
      use: { browserName: "chromium" },
    },
  ],
  webServer: {
    command: "cargo xtask browser-foundation-serve",
    cwd: "..",
    url: `${baseURL}/health`,
    reuseExistingServer: false,
    timeout: 60_000,
    stdout: "pipe",
    stderr: "pipe",
  },
}

if (isCi) {
  config.workers = 2
  config.globalTimeout = 10 * 60_000
}

export default defineConfig(config)
