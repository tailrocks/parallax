import { describe, expect, it } from "vitest"

import type { ResolvedRange } from "@/domain/time-range/range"
import {
  exemplarMarkers,
  latestErrorRate,
  latencyBands,
  stepSecondsForRange,
  toLineData,
  type MetricExemplar,
  type SpanRed,
} from "@/features/services/model/service-detail"

const hour: ResolvedRange = { key: "1h", fromNanos: "0", toNanos: "3600000000000" }
const halfHour: ResolvedRange = { key: "custom", fromNanos: "0", toNanos: "1800000000000" }
const fifteen: ResolvedRange = { key: "custom", fromNanos: "0", toNanos: "900000000000" }

const red: SpanRed = {
  rate: [{ tsNanos: "1", value: 10 }],
  errorRate: [{ tsNanos: "1", value: 0.25 }],
  p50: [{ tsNanos: "10", value: 10 }],
  p95: [{ tsNanos: "10", value: 20 }],
  p99: [{ tsNanos: "10", value: 30 }],
}

const exemplar: MetricExemplar = {
  tsNanos: "1800000000000",
  service: "checkout",
  name: "http.server.duration",
  value: 15,
  traceId: "aa",
  spanId: "bb",
  invocationId: null,
  attributes: "",
}

describe("service-detail model", () => {
  it("stepSeconds clamps below 30 min", () => {
    expect(stepSecondsForRange(fifteen)).toBe(30)
    expect(stepSecondsForRange(halfHour)).toBe(30)
  })

  it("stepSeconds is 60 for one hour", () => {
    expect(stepSecondsForRange(hour)).toBe(60)
  })

  it("latestErrorRate reads the last point", () => {
    expect(latestErrorRate(red)).toBe(0.25)
    expect(latestErrorRate({ ...red, errorRate: [] })).toBe(0)
  })

  it("latencyBands stack non-negative deltas", () => {
    expect(latencyBands(red)).toEqual([
      {
        tsNanos: "10",
        label: expect.any(String),
        p50: 10,
        p95: 20,
        p99: 30,
        p50Band: 10,
        p95Band: 10,
        p99Band: 10,
      },
    ])
  })

  it("toLineData sorts and maps values", () => {
    const rows = toLineData(
      {
        a: [
          { tsNanos: "2", value: 2 },
          { tsNanos: "1", value: 1 },
        ],
      },
      (_key, value) => value * 2
    )
    expect(rows.map((row) => row.tsNanos)).toEqual(["1", "2"])
    expect(rows[0]?.["a"]).toBe(2)
  })

  it("exemplarMarkers empty on zero span", () => {
    const empty: ResolvedRange = { key: "custom", fromNanos: "5", toNanos: "5" }
    expect(exemplarMarkers([exemplar], [], empty)).toEqual([])
  })

  it("exemplarMarkers clamp x/y", () => {
    const markers = exemplarMarkers([exemplar], [{ tsNanos: "10", p99: 30 }], hour)
    expect(markers).toHaveLength(1)
    expect(markers[0]!.x).toBeGreaterThanOrEqual(5)
    expect(markers[0]!.x).toBeLessThanOrEqual(95)
    expect(markers[0]!.y).toBeGreaterThanOrEqual(12)
    expect(markers[0]!.y).toBeLessThanOrEqual(86)
  })
})
