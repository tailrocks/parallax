import { test as base, expect } from "@playwright/test"

import { foundationDatasetId, type DatasetId } from "./dataset"
import { attachDiagnostics, type DiagnosticSession } from "./diagnostics"
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
  diagnostics: DiagnosticSession
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
  seeded: [
    async ({ productDataset }, use) => {
      await resetDataset(productDataset)
      await use()
    },
    { auto: true },
  ],
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
  snapshot: async ({}, use) => {
    await use(async () => snapshotState())
  },
  injectGraphqlFailure: async ({}, use) => {
    await use(async () => failNextGraphql())
  },
})

export { expect } from "@playwright/test"
