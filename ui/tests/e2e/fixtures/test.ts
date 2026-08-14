import { test as base, expect } from "@playwright/test"

import { foundationDatasetId, type DatasetId } from "./dataset"
import { attachDiagnostics, type DiagnosticSession } from "./diagnostics"
import {
  fullStackSnapshot,
  readFullStackManifest,
  type FullStackRuntimeManifest,
} from "./full-stack-fixture"
import {
  failNextGraphql,
  resetDataset,
  snapshotState,
  type ControlSnapshot,
  type ProductDatasetId,
} from "./product-fixture"

export interface FoundationFixtures {
  datasetId: DatasetId
  diagnostics: DiagnosticSession
}

export interface ProductFixtures {
  /** Dataset applied before the page is opened. Override per-test via test.use. */
  productDataset: ProductDatasetId
  /** Ensures control-plane reset completed for productDataset. */
  seeded: void
  /** Freezes wall-clock labels while leaving application timers running. */
  fixedTime: void
  diagnostics: DiagnosticSession
  /** Substrings that are expected (injected 503, known product DISCREPANCY). */
  allowedDiagnostic: string[]
  snapshot: () => Promise<ControlSnapshot>
  injectGraphqlFailure: () => Promise<void>
}

/**
 * Typed foundation test export: fresh context/page per test (Playwright default),
 * deterministic dataset id, automatic diagnostics capture/cleanup.
 */
export const test = base.extend<FoundationFixtures>({
  datasetId: async ({}, use) => {
    await use(foundationDatasetId("default"))
  },
  diagnostics: async ({ page }, use, testInfo) => {
    const session = attachDiagnostics(page)
    await use(session)
    await session.attach(testInfo)
    const unexpected = session.unexpected()
    session.dispose()
    expect(
      unexpected,
      `unexpected browser diagnostics:\n${unexpected
        .map((event) => `- ${event.kind}: ${event.message}`)
        .join("\n")}`
    ).toEqual([])
  },
})

/**
 * Product-contract fixture: control-plane reset before navigation, diagnostics,
 * and typed postcondition snapshot access. No happy-path page.route stubs.
 */
export const productTest = base.extend<ProductFixtures>({
  productDataset: ["shell-empty", { option: true }],
  fixedTime: [
    async ({ page }, use) => {
      await page.clock.setFixedTime(new Date("2026-07-18T00:00:00Z"))
      await use()
    },
    { auto: true },
  ],
  seeded: [
    async ({ productDataset }, use) => {
      await resetDataset(productDataset)
      await use()
    },
    { auto: true },
  ],
  allowedDiagnostic: [[], { option: true }],
  diagnostics: [
    async ({ page, allowedDiagnostic }, use, testInfo) => {
      const session = attachDiagnostics(page)
      await use(session)
      await session.attach(testInfo)
      const unexpected = session
        .unexpected()
        .filter((event) => !allowedDiagnostic.some((needle) => event.message.includes(needle)))
      session.dispose()
      expect(
        unexpected,
        `unexpected browser diagnostics:\n${unexpected
          .map((event) => `- ${event.kind}: ${event.message}`)
          .join("\n")}`
      ).toEqual([])
    },
    { auto: true },
  ],
  snapshot: async ({}, use) => {
    await use(async () => snapshotState())
  },
  injectGraphqlFailure: async ({}, use) => {
    await use(async () => failNextGraphql())
  },
})

export interface FullStackFixtures {
  fullStack: FullStackRuntimeManifest
  diagnostics: DiagnosticSession
  snapshot: () => ReturnType<typeof fullStackSnapshot>
}

/**
 * Real-stack fixture: reads managed GreptimeDB+Turso runtime manifest produced
 * by `cargo xtask browser-full-stack-serve` after public OTLP seed/readiness.
 */
export const fullStackTest = base.extend<FullStackFixtures>({
  fullStack: async ({}, use) => {
    const manifest = readFullStackManifest()
    expect(manifest.storage).toBe("managed-greptime+turso")
    expect(manifest.issue_fingerprint.length).toBeGreaterThan(0)
    await use(manifest)
  },
  diagnostics: [
    async ({ page }, use, testInfo) => {
      const session = attachDiagnostics(page)
      await use(session)
      await session.attach(testInfo)
      const unexpected = session.unexpected()
      session.dispose()
      // Full-stack against a long-lived QA attach may carry noisy console from
      // unrelated surfaces; only fail on page errors.
      const hard = unexpected.filter((event) => event.kind === "pageerror")
      expect(
        hard,
        `unexpected browser page errors:\n${hard
          .map((event) => `- ${event.kind}: ${event.message}`)
          .join("\n")}`
      ).toEqual([])
    },
    { auto: true },
  ],
  snapshot: async ({}, use) => {
    await use(async () => fullStackSnapshot())
  },
})

export { expect } from "@playwright/test"
