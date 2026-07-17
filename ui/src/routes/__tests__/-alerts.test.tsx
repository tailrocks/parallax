import { describe, expect, it } from "vitest"

import { draftFromTemplate } from "@/lib/alert-rule-form"
import { draftToArgs } from "../alerts.index"

describe("alerts draftToArgs", () => {
  it("serializes a template draft into mutation arguments", () => {
    const draft = draftFromTemplate("high-error-rate", "Checkout errors")
    expect(draft).not.toBeNull()
    const args = draftToArgs(draft!)
    expect(args).toContain('name: "Checkout errors"')
    expect(args).toContain("enabled: true")
    expect(args).toContain('signalType: "error_rate"')
    expect(args).toContain('comparator: "gt"')
    expect(args).toContain("threshold: 0.2")
    expect(args).toContain("windowMinutes: 5")
    expect(args).toContain('severity: "critical"')
    expect(args).toContain("renotifyIntervalMinutes: 30")
    expect(args).not.toContain("thresholdUpper")
    expect(args).not.toContain("metricName")
    expect(args).not.toContain("services")
  })

  it("escapes quotes in names and includes optional fields", () => {
    const draft = draftFromTemplate("slow-p95", 'say "hi"')
    expect(draft).not.toBeNull()
    const args = draftToArgs({
      ...draft!,
      thresholdUpper: 900,
      metricName: "http.server.request.duration",
      metricAggregation: "p95",
      services: ["checkout", "pricing"],
    })
    expect(args).toContain('name: "say \\"hi\\""')
    expect(args).toContain("thresholdUpper: 900")
    expect(args).toContain('metricName: "http.server.request.duration"')
    expect(args).toContain('metricAggregation: "p95"')
    expect(args).toContain('services: ["checkout", "pricing"]')
  })
})
