import { describe, expect, it } from "vitest"

import { validateServicesSearch } from "@/features/services"

describe("services route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateServicesSearch({})).toEqual({})
    expect(validateServicesSearch({ range: "24h", q: "api" })).toEqual({
      range: "24h",
      q: "api",
    })
    expect(validateServicesSearch({ sort: "p95:desc", q: "" })).toEqual({ sort: "p95:desc" })
  })
})
