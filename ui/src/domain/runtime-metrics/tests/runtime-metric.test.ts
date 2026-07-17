import { describe, expect, it } from "vitest"

import type { MetricPoint, RuntimeMetric } from "@/domain/runtime-metrics/runtime-metric"

describe("RuntimeMetric domain value", () => {
  it("accepts empty points and null unit", () => {
    const metric: RuntimeMetric = {
      family: "tokio",
      metric: "tokio.runtime.alive_tasks",
      unit: null,
      points: [],
    }
    expect(metric.points).toEqual([])
    expect(metric.unit).toBeNull()
  })

  it("accepts byte and ratio units with ordered points", () => {
    const points: readonly MetricPoint[] = [
      { tsNanos: "1", value: 1024 },
      { tsNanos: "2", value: 2048 },
    ]
    const metric: RuntimeMetric = {
      family: "process",
      metric: "process.memory.usage",
      unit: "bytes",
      points,
    }
    expect(metric.points).toHaveLength(2)
    expect(metric.points[0]?.value).toBe(1024)
    expect(metric.unit).toBe("bytes")
  })
})
