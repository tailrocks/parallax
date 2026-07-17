import type { Page } from "@playwright/test"

/** Investigations screen — semantic locators only; reused by pilot contracts. */
export class InvestigationsScreen {
  readonly #page: Page

  constructor(page: Page) {
    this.#page = page
  }

  heading() {
    return this.#page.getByRole("heading", {
      name: "Investigations",
      exact: true,
    })
  }

  emptyState() {
    return this.#page.getByText("No investigations")
  }

  newButton() {
    return this.#page.getByRole("button", { name: /New investigation/i })
  }

  nameInput() {
    return this.#page.getByPlaceholder("Checkout outage")
  }

  createButton() {
    return this.#page.getByRole("button", { name: "Create", exact: true })
  }

  caseLink(name: string) {
    return this.#page.getByRole("link", { name })
  }

  notesField() {
    return this.#page.getByLabel("Markdown")
  }

  saveButton() {
    return this.#page.getByRole("button", { name: "Save", exact: true })
  }

  deleteButton() {
    return this.#page.getByRole("button", { name: "Delete", exact: true })
  }

  confirmDelete() {
    return this.#page
      .getByRole("button", { name: "Delete", exact: true })
      .last()
  }

  pinLabel(label: string) {
    return this.#page.getByText(label, { exact: true })
  }

  async openList() {
    await this.#page.goto("/investigations")
  }

  async openDetail(id: string) {
    await this.#page.goto(`/investigations/${id}`)
  }
}
