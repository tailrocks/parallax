import AxeBuilder from "@axe-core/playwright"
import type { Page } from "@playwright/test"
import { expect } from "@playwright/test"

/**
 * Plan 146 accessibility helper. Runs axe only after the caller reaches a
 * stable page state. Exceptions must be exact rule + selector + owner.
 */
export async function assertNoAxeViolations(
  page: Page,
  options?: {
    include?: string[]
    exclude?: string[]
    disableRules?: string[]
  }
): Promise<void> {
  let builder = new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
  if (options?.include?.length) {
    for (const selector of options.include) {
      builder = builder.include(selector)
    }
  }
  if (options?.exclude?.length) {
    for (const selector of options.exclude) {
      builder = builder.exclude(selector)
    }
  }
  if (options?.disableRules?.length) {
    builder = builder.disableRules(options.disableRules)
  }
  const results = await builder.analyze()
  const serious = results.violations.filter(
    (v) => v.impact === "critical" || v.impact === "serious"
  )
  expect(
    serious,
    serious
      .map(
        (v) =>
          `${v.id} (${v.impact}): ${v.help} — nodes: ${v.nodes
            .map((n) => n.target.join(" "))
            .join("; ")}`
      )
      .join("\n")
  ).toEqual([])
}
