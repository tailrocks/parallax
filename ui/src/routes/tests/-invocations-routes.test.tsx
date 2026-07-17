import { describe, expect, it } from "vitest"

import {
  loadInvocationHub,
  loadInvocations,
  validateHubSearch,
  validateInvocationsSearch,
} from "@/features/invocations"

describe("invocations route contracts", () => {
  it("exposes public search and loaders for thin route wiring", () => {
    expect(validateInvocationsSearch({})).toEqual({})
    expect(validateHubSearch({})).toEqual({})
    expect(typeof loadInvocations).toBe("function")
    expect(typeof loadInvocationHub).toBe("function")
  })
})
