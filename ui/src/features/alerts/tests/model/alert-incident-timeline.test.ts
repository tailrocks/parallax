import { describe, expect, it } from "vitest"

import {
  buildIncidentTimeline,
  severityMixSegments,
} from "@/features/alerts/model/alert-incident-timeline"

describe("buildIncidentTimeline", () => {
  it("orders open → renotify → resolved with deliveries and checks", () => {
    const events = buildIncidentTimeline(
      {
        firstTriggeredAtNanos: 1000,
        lastTriggeredAtNanos: 3000,
        resolvedAtNanos: 5000,
        status: "resolved",
        lastValue: 0.4,
      },
      [
        {
          eventType: "triggered",
          deliveredAtNanos: 1100,
          attemptCount: 1,
        },
        {
          eventType: "resolved",
          deliveredAtNanos: 5100,
          attemptCount: 1,
        },
      ],
      [
        { atNanos: 2000, status: "breach", value: 0.5 },
        { atNanos: 4000, status: "healthy", value: 0.01 },
      ]
    )
    expect(events.map((e) => e.kind)).toEqual([
      "triggered",
      "delivery_ok",
      "check_breach",
      "check_healthy",
      "resolved",
      "delivery_ok",
    ])
    // resolved status: no renotify row from lastTriggered (only while open)
    expect(events.every((e) => e.kind !== "renotify")).toBe(true)
  })

  it("adds renotify when open and last > first", () => {
    const events = buildIncidentTimeline({
      firstTriggeredAtNanos: 100,
      lastTriggeredAtNanos: 200,
      status: "open",
      lastValue: 0.9,
    })
    expect(events.map((e) => e.kind)).toEqual(["triggered", "renotify"])
  })

  it("marks failed deliveries", () => {
    const events = buildIncidentTimeline(
      {
        firstTriggeredAtNanos: 1,
        lastTriggeredAtNanos: 1,
        status: "open",
      },
      [
        {
          eventType: "triggered",
          deliveredAtNanos: 2,
          attemptCount: 5,
          error: "timeout",
        },
      ]
    )
    expect(events.some((e) => e.kind === "delivery_fail")).toBe(true)
    expect(events.find((e) => e.kind === "delivery_fail")?.detail).toBe("timeout")
  })
})

describe("severityMixSegments", () => {
  it("sums to 100 with largest remainder", () => {
    const segs = severityMixSegments([
      { severity: "INFO", count: 1 },
      { severity: "ERROR", count: 2 },
    ])
    expect(segs.reduce((s, x) => s + x.pct, 0)).toBe(100)
    expect(segs.find((s) => s.severity === "ERROR")?.pct).toBe(67)
    expect(segs.find((s) => s.severity === "INFO")?.pct).toBe(33)
  })

  it("returns empty for zero total", () => {
    expect(severityMixSegments([{ severity: "INFO", count: 0 }])).toEqual([])
  })
})
