import { defineConfig, devices, type PlaywrightTestConfig } from "@playwright/test"

const env = process.env
const isCi = env["CI"] === "true" || env["CI"] === "1"
const browserMode =
  env["PARALLAX_BROWSER_MODE"] === "foundation"
    ? "foundation"
    : env["PARALLAX_BROWSER_MODE"] === "full-stack"
      ? "full-stack"
      : env["PARALLAX_BROWSER_MODE"] === "breadth"
        ? "breadth"
        : "contracts"
const foundationPort = env["PARALLAX_BROWSER_FOUNDATION_PORT"] ?? "4173"
const contractsPort = env["PARALLAX_BROWSER_CONTRACTS_PORT"] ?? "4174"
const fullStackReadyPort = env["PARALLAX_BROWSER_FULL_STACK_READY_PORT"] ?? "4176"
const fullStackPublicPort = env["PARALLAX_BROWSER_FULL_STACK_PORT"] ?? "4175"
const fullStackApi = env["PARALLAX_FULL_STACK_BASE_URL"] ?? "http://127.0.0.1:4000"
const fullStackPublic =
  env["PARALLAX_BROWSER_FULL_STACK_PUBLIC_URL"] ?? `http://127.0.0.1:${fullStackPublicPort}`
const port =
  browserMode === "foundation"
    ? foundationPort
    : browserMode === "full-stack"
      ? fullStackPublicPort
      : contractsPort
const baseURL = browserMode === "full-stack" ? fullStackPublic : `http://127.0.0.1:${port}`
const readyURL =
  browserMode === "full-stack"
    ? `http://127.0.0.1:${fullStackReadyPort}/health`
    : `${baseURL}/health`

/**
 * Plan 132 foundation + plan 144 product contracts + plan 145 full-stack +
 * plan 146 cross/mobile/a11y/visual breadth.
 *
 * Runtime: lock-local `@playwright/test` forced through Bun
 * (`bunx --bun --no-install`). Browser binaries are provisioned by an explicit
 * install command, never install lifecycle scripts.
 *
 * `PARALLAX_BROWSER_MODE=foundation` keeps the plan 132 stub server.
 * `PARALLAX_BROWSER_MODE=full-stack` uses managed GreptimeDB + Turso (or attach).
 * `PARALLAX_BROWSER_MODE=breadth` reuses the fixture-backed contracts server.
 * Default is contracts (real GraphQL + injected in-memory adapter).
 */
const config: PlaywrightTestConfig = {
  testDir: "./tests/e2e",
  fullyParallel: true,
  forbidOnly: isCi,
  retries: 0,
  timeout: browserMode === "full-stack" ? 60_000 : 30_000,
  expect: { timeout: browserMode === "full-stack" ? 15_000 : 5_000 },
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
    navigationTimeout: browserMode === "full-stack" ? 30_000 : 15_000,
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
    {
      name: "full-stack-chromium",
      testMatch: "**/full-stack/**/*.spec.ts",
      use: { browserName: "chromium" },
      // One worker owns one managed Greptime fixed-port stack (or attach).
      fullyParallel: false,
      workers: 1,
    },
    // Plan 146 breadth — fixture-backed contracts server, selected pilots only.
    {
      name: "cross-firefox",
      testMatch: "**/cross/**/*.spec.ts",
      use: { browserName: "firefox" },
      fullyParallel: false,
    },
    {
      name: "cross-webkit",
      testMatch: "**/cross/**/*.spec.ts",
      use: { browserName: "webkit" },
      fullyParallel: false,
    },
    {
      name: "mobile-chromium",
      testMatch: "**/mobile/**/*.spec.ts",
      use: {
        browserName: "chromium",
        ...devices["Pixel 7"],
      },
      fullyParallel: false,
    },
    {
      name: "mobile-webkit",
      testMatch: "**/mobile/**/*.spec.ts",
      use: {
        browserName: "webkit",
        ...devices["iPhone 14"],
      },
      fullyParallel: false,
    },
    {
      name: "accessibility-chromium",
      testMatch: "**/accessibility/**/*.spec.ts",
      use: { browserName: "chromium" },
      fullyParallel: false,
    },
    {
      name: "visual-chromium-linux",
      testMatch: "**/visual/**/*.spec.ts",
      // Platform-neutral snapshot names so host-authored goldens can be
      // revalidated on digest-pinned Linux CI (plan 146). Font AA may still
      // differ; thresholds live in the visual specs/manifest.
      snapshotPathTemplate: "{testDir}/{testFilePath}-snapshots/{arg}{ext}",
      use: {
        browserName: "chromium",
        viewport: { width: 1440, height: 900 },
        deviceScaleFactor: 1,
      },
      fullyParallel: false,
    },
  ],
  webServer:
    browserMode === "foundation"
      ? {
          command: "cargo xtask browser-foundation-serve",
          cwd: "..",
          url: readyURL,
          reuseExistingServer: false,
          timeout: 60_000,
          stdout: "pipe",
          stderr: "pipe",
          env: {
            ...env,
            PARALLAX_BROWSER_FOUNDATION_PORT: foundationPort,
          },
        }
      : browserMode === "full-stack"
        ? {
            command: "cargo xtask browser-full-stack-serve",
            cwd: "..",
            url: readyURL,
            reuseExistingServer: false,
            timeout: 180_000,
            stdout: "pipe",
            stderr: "pipe",
            env: {
              ...env,
              PARALLAX_BROWSER_FULL_STACK_READY_PORT: fullStackReadyPort,
              PARALLAX_BROWSER_FULL_STACK_PORT: fullStackPublicPort,
              // Host QA: attach. CI: set PARALLAX_FULL_STACK_MODE=managed.
              PARALLAX_FULL_STACK_MODE:
                env["PARALLAX_FULL_STACK_MODE"] ?? (isCi ? "managed" : "attach"),
              PARALLAX_FULL_STACK_BASE_URL: fullStackApi,
              PARALLAX_FULL_STACK_OTLP_HTTP:
                env["PARALLAX_FULL_STACK_OTLP_HTTP"] ?? "http://127.0.0.1:4318",
            },
          }
        : {
            // contracts + breadth share the fixture-backed contracts server.
            command: "cargo xtask browser-contracts-serve",
            cwd: "..",
            url: readyURL,
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
} else if (
  browserMode === "contracts" ||
  browserMode === "full-stack" ||
  browserMode === "breadth"
) {
  config.workers = 1
}

export default defineConfig(config)
