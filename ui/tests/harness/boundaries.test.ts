import { describe, expect, it } from "vitest"

import { TEST_NOW_ISO, createTestIdFactory, testNow } from "../../src/test/fixtures"
import { networkEscapeReason } from "../../src/test/network"
import { registerTestReset, resetRegisteredTestState } from "../../src/test/resets"
import { installTimerTracker, pendingTimerMessage } from "../../src/test/timers"

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
    expect([first(), first(), second()]).toEqual(["trace-001", "trace-002", "trace-001"])
  })

  it("runs registered state resets without retaining removed owners", async () => {
    const calls: string[] = []
    const unregister = registerTestReset(() => {
      calls.push("active")
    })
    await resetRegisteredTestState()
    unregister()
    await resetRegisteredTestState()
    expect(calls).toEqual(["active"])
  })

  it("reports pending timer classes exactly", () => {
    expect(pendingTimerMessage({ intervals: 0, timeouts: 0 })).toBeNull()
    expect(pendingTimerMessage({ intervals: 2, timeouts: 1 })).toBe(
      "test leaked timers: 1 timeout(s), 2 interval(s)"
    )
  })

  it("tracks and clears pending timer handles exactly", () => {
    const tracker = installTimerTracker()
    try {
      const schedule = Reflect.get(globalThis, "setTimeout") as (
        callback: () => void,
        delay: number
      ) => ReturnType<typeof globalThis.setTimeout>
      const cancel = Reflect.get(globalThis, "clearTimeout") as (
        handle: ReturnType<typeof globalThis.setTimeout>
      ) => void
      const handle = schedule(() => {}, 60_000)
      expect(tracker.pending()).toEqual({ intervals: 0, timeouts: 1 })
      cancel(handle)
      expect(tracker.pending()).toEqual({ intervals: 0, timeouts: 0 })
    } finally {
      tracker.restore()
    }
  })
})
