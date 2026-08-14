import { describe, expect, it } from "vitest"

import {
  patchServicesSearch,
  validateServicesSearch,
} from "@/features/services/model/services-search"

describe("services-search", () => {
  it("keeps known sort and query", () => {
    expect(validateServicesSearch({ range: "24h", q: "api", sort: "p95:desc" })).toEqual({
      range: "24h",
      q: "api",
      sort: "p95:desc",
    })
  })

  it("drops unknown sort", () => {
    expect(validateServicesSearch({ sort: "nope" })).toEqual({})
  })

  it("patchServicesSearch removes empty", () => {
    expect(patchServicesSearch({ q: "api", sort: "name:asc" }, { q: "" })).toEqual({
      sort: "name:asc",
    })
  })

  it("rejects empty q", () => {
    expect(validateServicesSearch({ q: "" })).toEqual({})
  })
})
