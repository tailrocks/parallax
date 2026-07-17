import { productTest as test, expect } from "../fixtures/test"
import { InvestigationsScreen } from "../screens/investigations-screen"
import { ShellScreen } from "../screens/shell-screen"

const PILOT_ID = "inv-pilot-001"
const PILOT_NAME = "Checkout latency case"

test.describe("investigations product pilot", () => {
  test.use({ productDataset: "investigations-pilot" })

  test("lists seeded investigation and opens detail @pw-investigations-list-detail", async ({
    page,
    snapshot,
  }) => {
    const screen = new InvestigationsScreen(page)
    await screen.openList()
    await expect(screen.heading()).toBeVisible()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()
    await expect(page.getByText("1 pins")).toBeVisible()

    await screen.caseLink(PILOT_NAME).click()
    await expect(page).toHaveURL(new RegExp(`/investigations/${PILOT_ID}`))
    await expect(page.getByRole("heading", { name: PILOT_NAME })).toBeVisible()
    await expect(screen.pinLabel("Checkout authorize")).toBeVisible()
    await expect(screen.notesField()).toHaveValue("Initial case notes from fixture seed.")

    const state = await snapshot()
    expect(state.investigations.map((row) => row.id)).toEqual([PILOT_ID])
    expect(state.investigations[0]?.name).toBe(PILOT_NAME)
  })

  test("create edit delete pin note and persist @pw-investigations-crud-pilot", async ({
    page,
    snapshot,
  }) => {
    const screen = new InvestigationsScreen(page)
    const shell = new ShellScreen(page)
    await screen.openList()
    await expect(screen.caseLink(PILOT_NAME)).toBeVisible()

    await screen.newButton().click()
    await screen.nameInput().fill("Mutation pilot case")
    await screen.createButton().click()
    await expect(page).toHaveURL(/\/investigations\/[0-9a-fA-F-]+/)
    await expect(page.getByRole("heading", { name: "Mutation pilot case" })).toBeVisible()

    await screen.notesField().fill("Persisted note from browser contract")
    await screen.saveButton().click()
    // Wait for mutation to complete (button stays enabled; assert no error).
    await expect(page.getByText(/error/i)).toHaveCount(0)
    await expect(page.getByRole("heading", { name: "Mutation pilot case" })).toBeVisible()

    // Typed postcondition before relying on list cache.
    const afterSave = await snapshot()
    expect(afterSave.investigations.some((row) => row.name === "Mutation pilot case")).toBe(true)
    const created = afterSave.investigations.find((row) => row.name === "Mutation pilot case")
    expect(created?.state).toContain("Persisted note from browser contract")

    // Cross-route navigation then hard reload so list bypasses client TTL cache.
    await shell.navItem("Traces").click()
    await expect(page).toHaveURL(/\/traces/)
    await page.goto("/investigations")
    await page.reload()
    await expect(screen.caseLink("Mutation pilot case")).toBeVisible()
    await screen.caseLink("Mutation pilot case").click()
    await expect(screen.notesField()).toHaveValue("Persisted note from browser contract")

    await screen.deleteButton().click()
    await page.getByRole("alertdialog").getByRole("button", { name: "Delete" }).click()
    await expect(page).toHaveURL(/\/investigations\/?/)
    await page.reload()
    await expect(screen.caseLink("Mutation pilot case")).toHaveCount(0)

    const afterDelete = await snapshot()
    expect(afterDelete.investigations.every((row) => row.name !== "Mutation pilot case")).toBe(true)
    expect(afterDelete.investigations.some((row) => row.id === PILOT_ID)).toBe(true)
  })
})
