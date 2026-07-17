import { describe, expect, it } from "vitest"

import {
  facetSelectionsFromParam,
  facetSelectionsToParam,
  toggleFacetValue,
} from "@/shared/console/facet-sidebar"

describe("facet selections URL codec", () => {
  it("empty selections encode to undefined", () => {
    expect(facetSelectionsToParam({})).toBeUndefined()
    expect(facetSelectionsToParam({ service: [] })).toBeUndefined()
  })

  it("round-trips multi-facet selections", () => {
    const selections = {
      service: ["checkout", "cart"],
      outcome: ["error"],
    }
    const param = facetSelectionsToParam(selections)
    expect(param).toBeDefined()
    const decoded = facetSelectionsFromParam(param)
    expect(decoded["service"]?.sort()).toEqual(["cart", "checkout"])
    expect(decoded["outcome"]).toEqual(["error"])
  })

  it("round-trips values with commas, colons, and unicode", () => {
    const selections = {
      "error.type": ["Timeout: upstream, retry café"],
    }
    const decoded = facetSelectionsFromParam(facetSelectionsToParam(selections))
    expect(decoded).toEqual(selections)
  })

  it("encodes deterministically (sorted) for stable permalinks", () => {
    const a = facetSelectionsToParam({ b: ["2"], a: ["1"] })
    const b = facetSelectionsToParam({ a: ["1"], b: ["2"] })
    expect(a).toBe(b)
  })

  it("ignores malformed parts and duplicate values", () => {
    expect(facetSelectionsFromParam("nocolon,:novalue,dim:")).toEqual({})
    expect(facetSelectionsFromParam("s:a,s:a")).toEqual({ s: ["a"] })
    expect(facetSelectionsFromParam(undefined)).toEqual({})
  })
})

describe("toggleFacetValue", () => {
  it("adds a value to an empty dimension", () => {
    expect(toggleFacetValue({}, "service", "checkout")).toEqual({
      service: ["checkout"],
    })
  })

  it("removes a selected value and drops the empty dimension", () => {
    expect(toggleFacetValue({ service: ["checkout"] }, "service", "checkout")).toEqual({})
  })

  it("keeps other selections intact (OR within, AND across)", () => {
    const next = toggleFacetValue({ service: ["checkout"], outcome: ["error"] }, "service", "cart")
    expect(next).toEqual({
      service: ["checkout", "cart"],
      outcome: ["error"],
    })
  })

  it("does not mutate the input", () => {
    const input = { service: ["checkout"] }
    toggleFacetValue(input, "service", "cart")
    expect(input).toEqual({ service: ["checkout"] })
  })
})
