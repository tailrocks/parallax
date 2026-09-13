import { readFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { describe, expect, it } from "vitest"

const here = dirname(fileURLToPath(import.meta.url))

describe("rum journeys query", () => {
  it("loads tracesPage, not a browser rumSessions API", () => {
    const query = readFileSync(
      join(here, "../../api/rum-journeys.graphql"),
      "utf8"
    )
    expect(query).toContain("tracesPage(")
    expect(query).not.toMatch(/\bsessions\s*\(/)
    expect(query).not.toContain("rumSessions")
  })
})
