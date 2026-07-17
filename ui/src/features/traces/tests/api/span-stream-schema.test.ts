import { describe, expect, it } from "vitest"

import {
  decodeSpanStreamRow,
  spanStreamBatchDecoder,
} from "@/features/traces/api/span-stream-schema"

describe("spanStreamBatchDecoder", () => {
  it("decodes spans with spanId and drops others", () => {
    const result = spanStreamBatchDecoder.safeParse([
      {
        spanId: "s1",
        traceId: "t1",
        name: "op",
        tsNanos: "10",
        service: "svc",
      },
      { name: "no-id" },
    ])
    expect(result.success).toBe(true)
    if (!result.success) return
    expect(result.data).toHaveLength(1)
    expect(result.data[0]!.spanId).toBe("s1")
    expect(result.data[0]!.invocationId).toBeNull()
  })

  it("rejects non-array frames", () => {
    expect(spanStreamBatchDecoder.safeParse("s1").success).toBe(false)
  })

  it("decodeSpanStreamRow requires spanId", () => {
    expect(decodeSpanStreamRow({})).toBeNull()
    expect(decodeSpanStreamRow({ spanId: "" })).toBeNull()
    expect(decodeSpanStreamRow({ spanId: "x" })?.spanId).toBe("x")
  })
})
