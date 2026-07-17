import { describe, expect, it } from "vitest"

import {
  ALERT_RULE_TEMPLATES,
  draftFromTemplate,
  validateAlertRuleDraft,
  type AlertRuleDraft,
} from "@/features/alerts/model/alert-rule-form"

function base(overrides: Partial<AlertRuleDraft> = {}): AlertRuleDraft {
  return {
    name: "High errors",
    enabled: true,
    signalType: "error_rate",
    comparator: "gt",
    threshold: 0.2,
    windowMinutes: 5,
    minimumSampleCount: 10,
    consecutiveBreachesRequired: 2,
    consecutiveHealthyRequired: 2,
    severity: "critical",
    renotifyIntervalMinutes: 30,
    ...overrides,
  }
}

describe("validateAlertRuleDraft", () => {
  it("accepts a well-formed high-error-rate draft", () => {
    expect(validateAlertRuleDraft(base()).ok).toBe(true)
  })

  it("requires name and positive window", () => {
    const r = validateAlertRuleDraft(base({ name: "  ", windowMinutes: 0 }))
    expect(r.ok).toBe(false)
    expect(r.errors.some((e) => e.includes("name"))).toBe(true)
    expect(r.errors.some((e) => e.includes("windowMinutes"))).toBe(true)
  })

  it("requires thresholdUpper for between and enforces order", () => {
    const missing = validateAlertRuleDraft(base({ comparator: "between", threshold: 10 }))
    expect(missing.ok).toBe(false)
    expect(missing.errors.some((e) => e.includes("thresholdUpper"))).toBe(true)

    const inverted = validateAlertRuleDraft(
      base({
        comparator: "between",
        threshold: 20,
        thresholdUpper: 10,
      })
    )
    expect(inverted.ok).toBe(false)

    const ok = validateAlertRuleDraft(
      base({
        comparator: "between",
        threshold: 10,
        thresholdUpper: 20,
        signalType: "p95_latency",
      })
    )
    expect(ok.ok).toBe(true)
  })

  it("requires metricName for metric signal", () => {
    const r = validateAlertRuleDraft(base({ signalType: "metric", metricName: "" }))
    expect(r.ok).toBe(false)
    expect(r.errors.some((e) => e.includes("metricName"))).toBe(true)
  })

  it("flags error_rate thresholds outside [0,1]", () => {
    expect(validateAlertRuleDraft(base({ threshold: 1.5 })).ok).toBe(false)
    expect(validateAlertRuleDraft(base({ threshold: 0.5 })).ok).toBe(true)
  })
})

describe("templates", () => {
  it("exposes the five plan-167 presets", () => {
    expect(ALERT_RULE_TEMPLATES.map((t) => t.id)).toEqual([
      "high-error-rate",
      "slow-p95",
      "slow-p99",
      "throughput-drop",
      "log-error-burst",
    ])
  })

  it("builds a named draft from a template", () => {
    const d = draftFromTemplate("high-error-rate", "checkout errors")
    expect(d?.name).toBe("checkout errors")
    expect(d?.signalType).toBe("error_rate")
    expect(d?.threshold).toBe(0.2)
    expect(d?.enabled).toBe(true)
    expect(validateAlertRuleDraft(d!)).toEqual({ ok: true, errors: [] })
  })

  it("returns null for unknown template ids", () => {
    expect(draftFromTemplate("nope", "x")).toBeNull()
  })
})
