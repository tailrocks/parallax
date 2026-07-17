import { describe, expect, it } from "vitest"

import { loadOverview, stepSecondsForRange } from "@/features/overview"

describe("overview route contracts", () => {
  it("exposes public loader helpers for thin route wiring", () => {
    expect(typeof loadOverview).toBe("function")
    expect(
      stepSecondsForRange({
        key: "1h",
        fromNanos: "0",
        toNanos: "3600000000000",
      })
    ).toBeGreaterThan(0)
  })
})
