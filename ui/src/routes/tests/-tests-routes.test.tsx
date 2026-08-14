import { describe, expect, it } from "vitest"

import { validateTestsSearch } from "@/features/tests"

describe("tests route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateTestsSearch({})).toEqual({})
    expect(validateTestsSearch({ status: "FLAKY_PASS", sort: "NAME" })).toEqual({
      status: "FLAKY_PASS",
      sort: "NAME",
    })
    expect(validateTestsSearch({ status: "nope", sort: "NOPE" })).toEqual({})
  })
})
