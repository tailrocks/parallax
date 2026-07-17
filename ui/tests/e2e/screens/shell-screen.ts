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

  themeButton(label: "System" | "Light" | "Dark") {
    return this.#page.getByRole("button", { name: label })
  }

  notFoundTitle() {
    return this.#page.getByText("Nothing is mounted here")
  }

  apiErrorTitle() {
    return this.#page.getByText("Parallax API did not answer")
  }

  retryRoute() {
    return this.#page.getByRole("button", { name: "Retry route" })
  }

  async openRoot() {
    await this.#page.goto("/")
  }

  async documentThemeClass(): Promise<string> {
    return this.#page.evaluate(() => document.documentElement.className)
  }
}
