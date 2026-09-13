import { describe, expect, it } from "vitest"

import {
  parseSavedViewState,
  serializeLogsSearch,
  validateLogsSearch,
} from "@/features/logs/model/logs-search"

describe("logs trace filter", () => {
  it("keeps a trace id through validate and serialize", () => {
    const search = validateLogsSearch({ trace: "abc123", q: "boom" })
    expect(search.trace).toBe("abc123")
    const state = serializeLogsSearch(search)
    expect(state).toContain("trace=abc123")
    expect(parseSavedViewState(state)).toMatchObject({ trace: "abc123", q: "boom" })
  })

  it("drops empty trace values", () => {
    expect(validateLogsSearch({ trace: "" }).trace).toBeUndefined()
    expect(validateLogsSearch({ trace: 42 }).trace).toBeUndefined()
    expect(serializeLogsSearch({})).toBe("")
  })
})
