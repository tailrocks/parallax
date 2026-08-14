import { describe, expect, it } from "vitest"

import {
  INVOCATION_OUTCOME,
  INVOCATION_STATUS,
  SPAN_STATUS,
  TEST_FLAKY,
  TEST_RESULT,
  TEST_ROLLUP,
  errorCountTone,
  type DomainTone,
} from "@/shared/colors"

const TONE_SLOTS: Array<keyof DomainTone> = ["color", "badge", "bar", "chip", "icon"]

function expectExhaustive<K extends string>(record: Record<K, DomainTone>, keys: readonly K[]) {
  expect(Object.keys(record).sort()).toEqual([...keys].sort())
  for (const key of keys) {
    for (const slot of TONE_SLOTS) {
      expect(record[key][slot].length).toBeGreaterThan(0)
    }
  }
}

describe("domain color records (plan 172)", () => {
  it("covers every span status tone slot", () => {
    expectExhaustive(SPAN_STATUS, ["ok", "error", "unset"])
    expect(SPAN_STATUS.error.color).toBe("var(--severity-error)")
  })

  it("covers every invocation status and outcome tone slot", () => {
    expectExhaustive(INVOCATION_STATUS, ["running", "finished", "failed", "stale"])
    expectExhaustive(INVOCATION_OUTCOME, ["success", "skip", "cancellation", "error"])
    expect(INVOCATION_STATUS.failed.color).toBe("var(--severity-error)")
  })

  it("covers every test rollup, result, and flaky tone slot", () => {
    expectExhaustive(TEST_ROLLUP, [
      "PASSED",
      "FLAKY_PASS",
      "FAILED",
      "BROKEN",
      "SKIPPED",
      "UNKNOWN",
    ])
    expectExhaustive(TEST_RESULT, ["PASSED", "FAILED", "BROKEN", "SKIPPED", "UNKNOWN"])
    expectExhaustive(TEST_FLAKY, ["HEALTHY", "FLAKY", "FIXED", "BROKEN"])
    expect(errorCountTone(0)).toBe("text-muted-foreground/40")
    expect(errorCountTone(2)).toBe(SPAN_STATUS.error.icon)
  })
})
