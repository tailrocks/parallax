import { describe, expect, it } from "vitest"

import { validateLogsSearch } from "@/features/logs"

describe("logs route contracts", () => {
  it("exposes public search and loader for thin route wiring", () => {
    expect(validateLogsSearch({})).toEqual({ live: false })
    expect(
      validateLogsSearch({ live: "1", sev: "17", q: "boom", patterns: "true", service: "checkout" })
    ).toEqual({
      live: true,
      sev: 17,
      q: "boom",
      patterns: true,
      service: "checkout",
    })
  })
})
