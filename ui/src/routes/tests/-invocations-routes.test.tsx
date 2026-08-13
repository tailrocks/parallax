import { describe, expect, it } from "vitest"

import { validateHubSearch, validateInvocationsSearch } from "@/features/invocations"

describe("invocations route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateInvocationsSearch({})).toEqual({})
    expect(validateHubSearch({})).toEqual({})
    expect(
      validateInvocationsSearch({ mode: "daemon", status: "running", live: true, q: "pay" })
    ).toEqual({
      mode: "daemon",
      status: "running",
      live: true,
      q: "pay",
    })
    expect(validateHubSearch({ tab: "traces", live: "true" })).toEqual({
      tab: "traces",
      live: true,
    })
  })
})
