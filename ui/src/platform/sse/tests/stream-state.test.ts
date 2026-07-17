import { describe, expect, it } from "vitest"

import { reduceStreamStatus, type LiveStreamStatus } from "@/platform/sse/stream-state"

describe("reduceStreamStatus", () => {
  it("starts connecting then opens", () => {
    let status: LiveStreamStatus = "idle"
    status = reduceStreamStatus(status, { type: "start" })
    expect(status).toBe("connecting")
    status = reduceStreamStatus(status, { type: "opened" })
    expect(status).toBe("open")
  })

  it("marks reconnecting after open transport error", () => {
    let status: LiveStreamStatus = "open"
    status = reduceStreamStatus(status, { type: "transport-error" })
    expect(status).toBe("reconnecting")
    status = reduceStreamStatus(status, { type: "opened" })
    expect(status).toBe("open")
  })

  it("marks error when connect fails before open", () => {
    let status: LiveStreamStatus = "connecting"
    status = reduceStreamStatus(status, { type: "transport-error" })
    expect(status).toBe("error")
  })

  it("stops to idle", () => {
    expect(reduceStreamStatus("open", { type: "stop" })).toBe("idle")
    expect(reduceStreamStatus("reconnecting", { type: "stop" })).toBe("idle")
  })
})
