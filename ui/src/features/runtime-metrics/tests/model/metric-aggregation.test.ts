import { describe, expect, it } from "vitest"

import {
  clampCounterDelta,
  coerceAggregation,
  decodeMetricQuerySpec,
  defaultAggregation,
  encodeGraduationParams,
  encodeMetricQuerySpec,
  inferMetricKind,
  isLegalAggregation,
  legalAggregations,
  type MetricQuerySpec,
} from "@/features/runtime-metrics/model/metric-aggregation"

describe("legalAggregations", () => {
  it("types sum/gauge/histogram aggregations", () => {
    expect(legalAggregations("sum")).toEqual(["rate", "increase", "sum"])
    expect(legalAggregations("gauge")).toEqual(["avg", "min", "max", "last"])
    expect(legalAggregations("histogram")).toEqual(["p50", "p95", "p99", "avg"])
    expect(legalAggregations("unknown")).toEqual([])
  })

  it("rejects illegal combinations", () => {
    expect(isLegalAggregation("sum", "p95")).toBe(false)
    expect(isLegalAggregation("gauge", "rate")).toBe(false)
    expect(isLegalAggregation("histogram", "sum")).toBe(false)
    expect(isLegalAggregation("histogram", "p99")).toBe(true)
  })

  it("defaults to the first legal aggregation", () => {
    expect(defaultAggregation("sum")).toBe("rate")
    expect(defaultAggregation("gauge")).toBe("avg")
    expect(defaultAggregation("histogram")).toBe("p50")
    expect(defaultAggregation("unknown")).toBeNull()
  })
})

describe("coerceAggregation", () => {
  it("keeps legal values and replaces stale ones", () => {
    expect(coerceAggregation("sum", "increase")).toBe("increase")
    expect(coerceAggregation("sum", "p99")).toBe("rate")
    expect(coerceAggregation("gauge", null)).toBe("avg")
    expect(coerceAggregation("unknown", "avg")).toBeNull()
  })
})

describe("inferMetricKind", () => {
  it("maps common type and name shapes", () => {
    expect(inferMetricKind("Sum")).toBe("sum")
    expect(inferMetricKind("http.server.request.count_total")).toBe("sum")
    expect(inferMetricKind("Gauge")).toBe("gauge")
    expect(inferMetricKind("Histogram")).toBe("histogram")
    expect(inferMetricKind("http_server_duration_bucket")).toBe("histogram")
    expect(inferMetricKind("Summary")).toBe("summary")
    expect(inferMetricKind("nope")).toBe("unknown")
  })
})

describe("URL encode/decode round-trip", () => {
  const spec: MetricQuerySpec = {
    name: "http.server.duration",
    kind: "histogram",
    aggregation: "p95",
    where: 'service = "checkout"',
    groupBy: "http.route",
    stepSeconds: 60,
  }

  it("round-trips through URLSearchParams", () => {
    const encoded = encodeMetricQuerySpec(spec)
    const decoded = decodeMetricQuerySpec(encoded)
    expect(decoded).toEqual(spec)
  })

  it("coerces illegal agg from a stale URL", () => {
    const params = new URLSearchParams({
      q: "requests",
      type: "sum",
      agg: "p99",
    })
    expect(decodeMetricQuerySpec(params)).toEqual({
      name: "requests",
      kind: "sum",
      aggregation: "rate",
      where: undefined,
      groupBy: undefined,
      stepSeconds: undefined,
    })
  })

  it("returns null without a metric name", () => {
    expect(decodeMetricQuerySpec(new URLSearchParams())).toBeNull()
  })
})

describe("graduation params", () => {
  const spec: MetricQuerySpec = {
    name: "error_ratio",
    kind: "gauge",
    aggregation: "avg",
  }

  it("alert graduation carries signal_type=metric", () => {
    const p = encodeGraduationParams(spec, "alert")
    expect(p.get("signal_type")).toBe("metric")
    expect(p.get("metric_name")).toBe("error_ratio")
    expect(p.get("metric_aggregation")).toBe("avg")
    expect(p.get("q")).toBe("error_ratio")
  })

  it("dashboard graduation sets widget=metric", () => {
    const p = encodeGraduationParams(spec, "dashboard")
    expect(p.get("widget")).toBe("metric")
    expect(p.get("signal_type")).toBeNull()
  })
})

describe("clampCounterDelta", () => {
  it("clamps resets to zero (m-shapes counter reset)", () => {
    expect(clampCounterDelta(5, 100)).toBe(0)
    expect(clampCounterDelta(150, 100)).toBe(50)
    expect(clampCounterDelta(100, 100)).toBe(0)
  })
})
