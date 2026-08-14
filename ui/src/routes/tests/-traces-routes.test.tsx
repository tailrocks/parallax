import { describe, expect, it } from "vitest"

import { validateTraceDetailSearch, validateTracesSearch } from "@/features/traces"

describe("traces route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateTracesSearch({})).toEqual({})
    expect(validateTracesSearch({ errors: "1", minMs: "25", live: "1" })).toEqual({
      errors: true,
      minMs: 25,
      live: true,
    })
    expect(validateTraceDetailSearch({ tab: "story", view: "lanes" })).toEqual({
      tab: "story",
      view: "lanes",
      range: undefined,
      from: undefined,
      to: undefined,
      vs: undefined,
      ve: undefined,
      color: undefined,
    })
    expect(validateTracesSearch({ errors: "nope", minMs: "x" }).errors).toBeUndefined()
    expect(validateTracesSearch({ errors: "nope", minMs: "x" }).minMs).toBeUndefined()
  })
})
