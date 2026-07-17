import { describe, expect, it } from "vitest"

import {
  draftFromTemplate,
  alertDestinationSaveMutation,
  alertRuleSaveMutation,
  parseStringArray,
  ruleConditionLabel,
} from "@/features/alerts"

describe("alertRuleSaveMutation", () => {
  it("serializes a template draft into an input-object mutation", () => {
    const draft = draftFromTemplate("high-error-rate", "Checkout errors")
    expect(draft).not.toBeNull()
    const mutation = alertRuleSaveMutation(draft!)
    expect(mutation).toContain("alertRuleSave(input: {")
    expect(mutation).toContain('name: "Checkout errors"')
    expect(mutation).toContain("enabled: true")
    expect(mutation).toContain('signalType: "error_rate"')
    expect(mutation).toContain('comparator: "gt"')
    expect(mutation).toContain("threshold: 0.2")
    expect(mutation).toContain("windowMinutes: 5")
    expect(mutation).toContain('severity: "critical"')
    expect(mutation).toContain("renotifyIntervalMinutes: 30")
    expect(mutation).not.toContain("thresholdUpper")
    expect(mutation).not.toContain("metricName")
    expect(mutation).not.toContain("services")
    expect(mutation).not.toContain("id:")
  })

  it("escapes quotes and includes optional fields", () => {
    const draft = draftFromTemplate("slow-p95", 'say "hi"')
    expect(draft).not.toBeNull()
    const mutation = alertRuleSaveMutation(
      {
        ...draft!,
        thresholdUpper: 900,
        metricName: "http.server.request.duration",
        metricAggregation: "p95",
        services: ["checkout", "pricing"],
      },
      { id: "alr_1", destinationIds: ["dst_1", "dst_2"] }
    )
    expect(mutation).toContain('id: "alr_1"')
    expect(mutation).toContain('name: "say \\"hi\\""')
    expect(mutation).toContain("thresholdUpper: 900")
    expect(mutation).toContain('metricName: "http.server.request.duration"')
    expect(mutation).toContain('metricAggregation: "p95"')
    expect(mutation).toContain('services: ["checkout", "pricing"]')
    expect(mutation).toContain('destinationIds: ["dst_1", "dst_2"]')
  })

  it("floors minimumSampleCount at 1 for the API contract", () => {
    const draft = draftFromTemplate("throughput-drop", "Traffic")
    expect(draft).not.toBeNull()
    const mutation = alertRuleSaveMutation({
      ...draft!,
      minimumSampleCount: 0,
    })
    expect(mutation).toContain("minimumSampleCount: 1")
  })
})

describe("alertDestinationSaveMutation", () => {
  it("wraps the url into escaped config JSON", () => {
    const mutation = alertDestinationSaveMutation(
      "Ops hook",
      "webhook",
      "http://127.0.0.1:9099/hook"
    )
    expect(mutation).toContain('name: "Ops hook"')
    expect(mutation).toContain('kind: "webhook"')
    expect(mutation).toContain('config: "{\\"url\\":\\"http://127.0.0.1:9099/hook\\"}"')
    expect(mutation).not.toContain("id:")
  })
})

describe("alerts display helpers", () => {
  it("parses opaque JSON string arrays defensively", () => {
    expect(parseStringArray('["a", "b"]')).toEqual(["a", "b"])
    expect(parseStringArray("[]")).toEqual([])
    expect(parseStringArray("not json")).toEqual([])
    expect(parseStringArray('{"a": 1}')).toEqual([])
    expect(parseStringArray('["a", 2]')).toEqual(["a"])
  })

  it("labels rule conditions incl. ranges", () => {
    expect(
      ruleConditionLabel({
        signalType: "error_rate",
        comparator: "gt",
        threshold: 0.2,
        thresholdUpper: null,
        windowMinutes: 5,
      })
    ).toBe("error_rate > 0.2 over 5m")
    expect(
      ruleConditionLabel({
        signalType: "throughput",
        comparator: "not_between",
        threshold: 1,
        thresholdUpper: 10,
        windowMinutes: 15,
      })
    ).toBe("throughput outside 1–10 over 15m")
  })
})
