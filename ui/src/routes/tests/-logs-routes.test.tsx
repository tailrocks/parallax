import { describe, expect, it } from "vitest"

import { loadLogs, validateLogsSearch } from "@/features/logs"

describe("logs route contracts", () => {
  it("exposes public search and loader for thin route wiring", () => {
    expect(validateLogsSearch({})).toEqual({ live: false })
    expect(typeof loadLogs).toBe("function")
  })
})
