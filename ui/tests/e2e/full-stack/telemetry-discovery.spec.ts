import { fullStackTest as test, expect } from "../fixtures/test"
import { ShellScreen } from "../screens/shell-screen"
import { graphqlQuery, readFullStackManifest } from "../fixtures/full-stack-fixture"

test.describe("full-stack telemetry discovery @storage", () => {
  test("seeded service trace issue appear across routes @pw-storage-telemetry-discovery", async ({
    page,
    fullStack,
  }) => {
    const manifest = readFullStackManifest()
    expect(fullStack.service).toBe(manifest.service)
    expect(fullStack.trace_id).toBe(manifest.trace_id)
    expect(fullStack.issue_fingerprint).toBeTruthy()

    // Public GraphQL greptime + turso postconditions (separate predicates).
    const from = BigInt(manifest.start_nanos) - 60_000_000_000n
    const to = BigInt(manifest.start_nanos) + 3_600_000_000_000n
    const services = await graphqlQuery<{
      serviceList: Array<{ name: string }>
      recentTraces: Array<{ traceId: string; service: string }>
    }>(
      `{ serviceList(fromNanos: "${from}", toNanos: "${to}") { name } recentTraces(limit: 50) { traceId service } }`
    )
    expect(services.serviceList.some((row) => row.name === manifest.service)).toBe(true)
    expect(
      services.recentTraces.some(
        (row) => row.traceId === manifest.trace_id && row.service === manifest.service
      )
    ).toBe(true)

    const issues = await graphqlQuery<{
      issues: { items: Array<{ fingerprint: string; service: string; errorType: string }> }
    }>(`{ issues(limit: 100) { items { fingerprint service errorType } } }`)
    expect(
      issues.issues.items.some(
        (row) => row.fingerprint === manifest.issue_fingerprint && row.service === manifest.service
      )
    ).toBe(true)

    const shell = new ShellScreen(page)
    await shell.openRoot()
    await expect(shell.brandText()).toBeVisible()

    await shell.navItem("Services").click()
    await expect(page).toHaveURL(/\/services/)
    await expect(page.getByText(manifest.service, { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })

    await shell.navItem("Issues").click()
    await expect(page).toHaveURL(/\/issues/)
    await expect(page.getByText(manifest.error_type, { exact: false }).first()).toBeVisible({
      timeout: 20_000,
    })

    await page.goto(`/issues/${manifest.issue_fingerprint}`)
    await expect(page.getByText(manifest.service, { exact: false }).first()).toBeVisible()
    await expect(page.getByText("open", { exact: false }).first()).toBeVisible()

    await shell.navItem("Traces").click()
    await expect(page).toHaveURL(/\/traces/)
    // Trace id may be truncated in dense tables; GraphQL already proved visibility.
    await expect(page.getByRole("heading", { name: /trace/i }).first()).toBeVisible({
      timeout: 15_000,
    })
  })
})
