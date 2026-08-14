import { describe, expect, it } from "vitest"

import { stepSecondsForRange } from "@/features/overview"

describe("overview route contracts", () => {
  it("exposes public loader helpers for thin route wiring", () => {
    expect(
      stepSecondsForRange({
        key: "1h",
        fromNanos: "0",
        toNanos: "3600000000000",
      })
    ).toBe(60)
    expect(
      stepSecondsForRange({
        key: "custom",
        fromNanos: "0",
        toNanos: "900000000000",
      })
    ).toBe(30)
  })
})
