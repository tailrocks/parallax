import type { Page } from "@playwright/test"

/**
 * Shell screen object — semantic locators only.
 * Exists because foundation smoke reuses brand + primary nav markers.
 */
export class ShellScreen {
  readonly #page: Page

  constructor(page: Page) {
    this.#page = page
  }

  homeLink() {
    return this.#page.getByRole("link", { name: "Parallax home" })
  }

  brandText() {
    return this.#page.getByText("Parallax", { exact: true }).first()
  }

  navItem(label: string) {
    return this.#page.getByRole("link", { name: label })
  }

  async openRoot() {
    await this.#page.goto("/")
  }
}
