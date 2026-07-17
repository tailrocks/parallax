import { defineConfig, type PlaywrightTestConfig } from "@playwright/test"

const env = process.env
const isCi = env["CI"] === "true" || env["CI"] === "1"
const browserMode = env["PARALLAX_BROWSER_MODE"] === "foundation" ? "foundation" : "contracts"
const foundationPort = env["PARALLAX_BROWSER_FOUNDATION_PORT"] ?? "4173"
const contractsPort = env["PARALLAX_BROWSER_CONTRACTS_PORT"] ?? "4174"
const port = browserMode === "foundation" ? foundationPort : contractsPort
const baseURL = `http://127.0.0.1:${port}`

/**
 * Plan 132 foundation + plan 144 product contracts.
 *
 * Runtime: lock-local `@playwright/test` forced through Bun
 * (`bunx --bun --no-install`). Browser binaries are provisioned by an explicit
 * install command, never install lifecycle scripts.
 *
 * `PARALLAX_BROWSER_MODE=foundation` keeps the plan 132 stub server.
 * Default is contracts (real GraphQL + injected in-memory adapter).
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
    : [["line"], ["html", { open: "never", outputFolder: "playwright-report" }]],
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
    {
      name: "contracts-chromium",
      testMatch: "**/contracts/**/*.spec.ts",
      use: { browserName: "chromium" },
      // Mutation pilots share one control-plane dataset; serialize workers.
      fullyParallel: false,
    },
  ],
  webServer:
    browserMode === "foundation"
      ? {
          command: "cargo xtask browser-foundation-serve",
          cwd: "..",
          url: `${baseURL}/health`,
          reuseExistingServer: false,
          timeout: 60_000,
          stdout: "pipe",
          stderr: "pipe",
          env: {
            ...env,
            PARALLAX_BROWSER_FOUNDATION_PORT: foundationPort,
          },
        }
      : {
          command: "cargo xtask browser-contracts-serve",
          cwd: "..",
          url: `${baseURL}/health`,
          reuseExistingServer: false,
          timeout: 180_000,
          stdout: "pipe",
          stderr: "pipe",
          env: {
            ...env,
            PARALLAX_BROWSER_CONTRACTS_PORT: contractsPort,
          },
        },
}

if (isCi) {
  config.workers = browserMode === "foundation" ? 2 : 1
  config.globalTimeout = 15 * 60_000
} else if (browserMode === "contracts") {
  config.workers = 1
}

export default defineConfig(config)
