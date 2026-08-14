import type { Page } from "@playwright/test"

/** Linux CI has no Command key; product accepts ctrl or meta. */
export function commandPaletteShortcut(): string {
  return process.platform === "darwin" ? "Meta+k" : "Control+k"
}

export async function openCommandPalette(page: Page): Promise<void> {
  await page.keyboard.press(commandPaletteShortcut())
}
