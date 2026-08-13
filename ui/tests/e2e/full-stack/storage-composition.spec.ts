import { fullStackTest as test, expect } from "../fixtures/test"
import {
  graphqlQuery,
  pollIssueStatus,
  readFullStackManifest,
} from "../fixtures/full-stack-fixture"
import { SURFACE_TIMEOUT_MS } from "../support/timeouts"

test.describe("full-stack storage composition @storage", () => {
  test.afterEach(async ({ fullStack }) => {
    const fingerprint = fullStack.issue_fingerprint
    const current = await graphqlQuery<{
      issue: { fingerprint: string; status: string }
    }>(`{ issue(fingerprint: "${fingerprint}") { fingerprint status } }`)
    if (current.issue.status !== "open") {
      await graphqlQuery(
        `mutation { issueSetStatus(fingerprint: "${fingerprint}", status: "open") { fingerprint status } }`
      )
      await pollIssueStatus(fingerprint, "open")
    }
  })

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
    await expect(page.getByRole("button", { name: "Resolve" })).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await page.getByRole("button", { name: "Resolve" }).click()

    const afterUi = await pollIssueStatus(fingerprint, "resolved")
    expect(afterUi.status).toBe("resolved")

    const context = await browser.newContext()
    const fresh = await context.newPage()
    await fresh.goto(`/issues/${fingerprint}`)
    await expect(fresh.getByText("resolved", { exact: false }).first()).toBeVisible({
      timeout: SURFACE_TIMEOUT_MS,
    })
    await expect(fresh.getByRole("button", { name: "Reopen" })).toBeVisible()

    const afterFresh = await graphqlQuery<{
      issue: { fingerprint: string; status: string }
    }>(`{ issue(fingerprint: "${fingerprint}") { fingerprint status } }`)
    expect(afterFresh.issue.status).toBe("resolved")
    await context.close()

    expect(manifest.storage).toBe("managed-greptime+turso")
  })
})
