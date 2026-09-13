import { describe, expect, it } from "vitest"

import { encodeLogsAlertGraduation } from "@/features/logs/model/alert-graduation"

describe("encodeLogsAlertGraduation", () => {
  it("graduates to log_count without a service scope", () => {
    expect(encodeLogsAlertGraduation({})).toEqual({ signal_type: "log_count" })
    expect("service" in encodeLogsAlertGraduation({})).toBe(false)
  })

  it("carries the filtered service", () => {
    expect(encodeLogsAlertGraduation({ service: "checkout" })).toEqual({
      signal_type: "log_count",
      service: "checkout",
    })
  })

  it("drops empty service", () => {
    expect(encodeLogsAlertGraduation({ service: "" })).toEqual({ signal_type: "log_count" })
    expect(encodeLogsAlertGraduation({ service: undefined })).toEqual({ signal_type: "log_count" })
  })
})
