import { describe, expect, it } from "vitest"

import { validateEcosystemSearch } from "@/features/ecosystem/model/ecosystem-search"

describe("ecosystem focus search", () => {
  it("preserves valid focus controls", () => {
    expect(
      validateEcosystemSearch({
        focus: "checkout",
        hops: "2",
        focusMode: "hide",
        minTraffic: "1%",
        range: "custom",
        from: "10",
        to: "20",
      })
    ).toEqual({
      focus: "checkout",
      hops: 2,
      focusMode: "hide",
      minTraffic: "1%",
      range: "custom",
      from: "10",
      to: "20",
    })
  })

  it("canonicalizes defaults and rejects invalid values", () => {
    expect(
      validateEcosystemSearch({
        focus: "",
        hops: 9,
        focusMode: "remove",
        minTraffic: "100%",
      })
    ).toEqual({
      focus: undefined,
      hops: undefined,
      focusMode: undefined,
      minTraffic: undefined,
      range: undefined,
      from: undefined,
      to: undefined,
    })
  })
})
