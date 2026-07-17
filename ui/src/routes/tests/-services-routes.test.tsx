import { describe, expect, it } from "vitest"

import {
  loadServiceDetail,
  loadServices,
  validateServicesSearch,
} from "@/features/services"

describe("services route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateServicesSearch({})).toEqual({})
    expect(validateServicesSearch({ range: "24h", q: "api" })).toEqual({
      range: "24h",
      q: "api",
    })
    expect(typeof loadServices).toBe("function")
    expect(typeof loadServiceDetail).toBe("function")
  })
})
