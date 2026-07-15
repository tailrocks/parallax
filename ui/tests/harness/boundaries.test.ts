import { describe, expect, it } from "vitest"

import {
  TEST_NOW_ISO,
  createTestIdFactory,
  testNow,
} from "../../src/test/fixtures"
import { networkEscapeReason } from "../../src/test/network"

describe("test harness boundaries", () => {
  it("reports exact network escape targets", () => {
    expect(networkEscapeReason("https://example.test/private?q=1")).toBe(
      "unexpected test network request: https://example.test/private?q=1"
    )
    expect(networkEscapeReason(new URL("http://127.0.0.1:4000/graphql"))).toBe(
      "unexpected test network request: http://127.0.0.1:4000/graphql"
    )
  })

  it("provides UTC time and deterministic isolated identifiers", () => {
    expect(process.env["TZ"]).toBe("UTC")
    expect(testNow().toISOString()).toBe(TEST_NOW_ISO)
    const first = createTestIdFactory("trace")
    const second = createTestIdFactory("trace")
    expect([first(), first(), second()]).toEqual([
      "trace-001",
      "trace-002",
      "trace-001",
    ])
  })
})
