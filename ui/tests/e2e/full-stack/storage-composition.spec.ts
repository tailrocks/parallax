import { fullStackTest as test, expect } from "../fixtures/test"
import {
  graphqlQuery,
  pollIssueStatus,
  readFullStackManifest,
} from "../fixtures/full-stack-fixture"

test.describe("full-stack storage composition @storage", () => {
  test("issue status mutation persists in Turso across contexts @pw-storage-composition", async ({
    page,
    browser,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    const fingerprint = fullStack.issue_fingerprint
    expect(fingerprint).toBeTruthy()

    const before = await graphqlQuery<{
      issue: { fingerprint: string; status: string }
    }>(`{ issue(fingerprint: "${fingerprint}") { fingerprint status } }`)
    expect(before.issue.status).toBe("open")

    await page.goto(`/issues/${fingerprint}`)
    await expect(page.getByRole("button", { name: "Resolve" })).toBeVisible({ timeout: 20_000 })
    await page.getByRole("button", { name: "Resolve" }).click()

    // Typed public GraphQL postcondition (Turso-backed metadata).
    const afterUi = await pollIssueStatus(fingerprint, "resolved")
    expect(afterUi.status).toBe("resolved")

    // Fresh BrowserContext — no client TTL cache carryover.
    const context = await browser.newContext()
    const fresh = await context.newPage()
    await fresh.goto(`/issues/${fingerprint}`)
    await expect(fresh.getByText("resolved", { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })
    await expect(fresh.getByRole("button", { name: "Reopen" })).toBeVisible()

    const afterFresh = await graphqlQuery<{
      issue: { fingerprint: string; status: string }
    }>(`{ issue(fingerprint: "${fingerprint}") { fingerprint status } }`)
    expect(afterFresh.issue.status).toBe("resolved")

    // Restore open so repeated local runs stay deterministic.
    await fresh.getByRole("button", { name: "Reopen" }).click()
    await pollIssueStatus(fingerprint, "open")
    await context.close()

    expect(manifest.storage).toBe("managed-greptime+turso")
  })
})
