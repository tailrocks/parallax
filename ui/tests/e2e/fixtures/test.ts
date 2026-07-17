import { test as base, expect } from "@playwright/test"

import { foundationDatasetId, type DatasetId } from "./dataset"
import { attachDiagnostics, type DiagnosticSession } from "./diagnostics"

export interface FoundationFixtures {
  datasetId: DatasetId
  diagnostics: DiagnosticSession
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

export { expect } from "@playwright/test"
