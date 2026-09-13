import { describe, expect, it } from "vitest"

import {
  formatCount,
  formatRate,
  mapPipelineStatus,
  parseCount,
  queuePressure,
  totalAccepted,
  totalDropped,
} from "@/features/pipeline/model/pipeline-status"

describe("pipeline-status model", () => {
  it("parses exact counts and clamps garbage to zero", () => {
    expect(parseCount("42")).toBe(42n)
    expect(parseCount("9007199254740993")).toBe(9007199254740993n)
    expect(parseCount("nope")).toBe(0n)
    expect(parseCount("-5")).toBe(0n)
  })

  it("formats counts with grouping, keeping huge values exact", () => {
    expect(formatCount("1234567")).toBe("1,234,567")
    expect(formatCount("9007199254740993")).toBe("9007199254740993")
  })

  it("formats rates as percentages", () => {
    expect(formatRate(1)).toBe("100%")
    expect(formatRate(0.1)).toBe("10%")
    expect(formatRate(Number.NaN)).toBe("—")
  })

  it("derives queue pressure from depth ratios", () => {
    const queue = { signal: "traces", depth: 0, capacity: 256, highWater: 0, accepted: "0" }
    expect(queuePressure(queue)).toBe("ok")
    expect(queuePressure({ ...queue, depth: 192 })).toBe("filling")
    expect(queuePressure({ ...queue, depth: 256 })).toBe("full")
    expect(queuePressure({ ...queue, capacity: 0 })).toBe("ok")
  })

  it("totals drops and accepted across rows", () => {
    const drops = [
      { signal: "traces", reason: "ingress_reject", count: "3", detail: "" },
      { signal: null, reason: "live_tail_lag", count: "7", detail: "" },
    ]
    const queues = [
      { signal: "traces", depth: 0, capacity: 8, highWater: 0, accepted: "9" },
      { signal: "logs", depth: 0, capacity: 8, highWater: 0, accepted: "11" },
    ]
    expect(totalDropped(drops)).toBe(10n)
    expect(totalAccepted(queues)).toBe(20n)
  })

  it("maps the GraphQL payload through unchanged", () => {
    const status = mapPipelineStatus({
      samplingPolicy: [
        {
          signal: "traces",
          service: null,
          rule: "head",
          rate: 1,
          enforcedBy: "server-ingest",
          description: "keep-all",
        },
      ],
      ingestDrops: [],
      ingestQueues: [],
    })
    expect(status.policies).toHaveLength(1)
    expect(status.policies[0]?.signal).toBe("traces")
    expect(status.drops).toHaveLength(0)
  })
})
