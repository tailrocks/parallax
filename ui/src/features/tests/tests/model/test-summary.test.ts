import { describe, expect, it } from "vitest"

import {
  flakyLabel,
  rollupLabel,
  suiteLabel,
  type TestFlakyState,
  type TestRollup,
} from "@/features/tests/model/test-summary"

describe("test-summary", () => {
  it("suiteLabel empty and joined", () => {
    expect(suiteLabel([])).toBe("—")
    expect(suiteLabel(["a", "b"])).toBe("a / b")
  })

  it("rollupLabel every variant", () => {
    const cases: Array<[TestRollup, string]> = [
      ["PASSED", "passed"],
      ["FLAKY_PASS", "flaky pass"],
      ["FAILED", "failed"],
      ["BROKEN", "broken"],
      ["SKIPPED", "skipped"],
      ["UNKNOWN", "unknown"],
    ]
    for (const [rollup, label] of cases) {
      expect(rollupLabel(rollup)).toBe(label)
    }
  })

  it("flakyLabel every variant", () => {
    const cases: Array<[TestFlakyState, string]> = [
      ["HEALTHY", "healthy"],
      ["FLAKY", "flaky"],
      ["FIXED", "fixed"],
      ["BROKEN", "broken"],
    ]
    for (const [state, label] of cases) {
      expect(flakyLabel(state)).toBe(label)
    }
  })
})
