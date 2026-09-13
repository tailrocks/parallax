import { describe, expect, it } from "vitest"

import { encodeTracesAlertGraduation } from "@/features/traces/model/alert-graduation"

describe("encodeTracesAlertGraduation", () => {
  it("graduates an unfiltered query to p95_latency", () => {
    expect(encodeTracesAlertGraduation({})).toEqual({ signal_type: "p95_latency" })
    expect("service" in encodeTracesAlertGraduation({})).toBe(false)
  })

  it("graduates errors-only to error_rate and carries service", () => {
    expect(encodeTracesAlertGraduation({ errors: true, service: "api" })).toEqual({
      signal_type: "error_rate",
      service: "api",
    })
  })

  it("keeps p95_latency when errors is off, still scoped to service", () => {
    expect(encodeTracesAlertGraduation({ errors: false, service: "checkout" })).toEqual({
      signal_type: "p95_latency",
      service: "checkout",
    })
  })

  it("drops empty service", () => {
    expect(encodeTracesAlertGraduation({ errors: true, service: "" })).toEqual({
      signal_type: "error_rate",
    })
  })
})
