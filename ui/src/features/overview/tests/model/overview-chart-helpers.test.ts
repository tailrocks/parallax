import { describe, expect, it } from "vitest"

import type { ResolvedRange } from "@/domain/time-range/range"
import {
  mergeSignalSeries,
  sampleLatencyData,
  sampleSignalData,
} from "@/features/overview/model/overview-chart-helpers"

const hour: ResolvedRange = { key: "1h", fromNanos: "0", toNanos: "3600000000000" }

describe("overview-chart-helpers", () => {
  it("mergeSignalSeries unions timestamps", () => {
    const rows = mergeSignalSeries(
      hour,
      [{ tsNanos: "1", value: 4 }],
      [
        { tsNanos: "1", value: 1 },
        { tsNanos: "2", value: 3 },
      ]
    )
    expect(rows).toHaveLength(2)
    expect(rows[0]).toMatchObject({ tsNanos: "1", spans: 4, errors: 1 })
    expect(rows[1]).toMatchObject({ tsNanos: "2", spans: 0, errors: 3 })
  })

  it("sampleSignalData has 6 points", () => {
    const rows = sampleSignalData(hour)
    expect(rows).toHaveLength(6)
    expect(rows[0]?.tsNanos).toBe("0")
    expect(rows[5]?.tsNanos).toBe("3600000000000")
  })

  it("sampleLatencyData bands non-negative", () => {
    for (const row of sampleLatencyData(hour)) {
      expect(row.p50Band).toBeGreaterThanOrEqual(0)
      expect(row.p95Band).toBeGreaterThanOrEqual(0)
      expect(row.p99Band).toBeGreaterThanOrEqual(0)
      expect(row.p95).toBeGreaterThanOrEqual(row.p50)
      expect(row.p99).toBeGreaterThanOrEqual(row.p95)
    }
  })
})
