import { describe, expect, it } from "vitest"

import { validateIssuesSearch } from "@/features/issues"

describe("issues route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateIssuesSearch({})).toEqual({})
    expect(validateIssuesSearch({ status: "open", sort: "EVENTS" })).toEqual({
      status: "open",
      sort: "EVENTS",
    })
    expect(validateIssuesSearch({ status: "nope", sort: "NOPE" })).toEqual({})
  })
})
