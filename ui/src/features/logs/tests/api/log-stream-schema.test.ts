import { describe, expect, it } from "vitest"

import {
  decodeLogStreamRow,
  logStreamBatchDecoder,
} from "@/features/logs/api/log-stream-schema"

describe("logStreamBatchDecoder", () => {
  it("decodes valid rows and drops incomplete ones", () => {
    const result = logStreamBatchDecoder.safeParse([
      {
        tsNanos: "100",
        body: "hello",
        service: "api",
        severityNum: 9,
        severityText: "INFO",
      },
      { body: "missing ts" },
      null,
    ])
    expect(result.success).toBe(true)
    if (!result.success) return
    expect(result.data).toHaveLength(1)
    expect(result.data[0]!.tsNanos).toBe("100")
    expect(result.data[0]!.body).toBe("hello")
    expect(result.data[0]!.service).toBe("api")
  })

  it("rejects non-array frames", () => {
    expect(logStreamBatchDecoder.safeParse({ tsNanos: "1" }).success).toBe(false)
  })

  it("decodeLogStreamRow requires tsNanos and body", () => {
    expect(decodeLogStreamRow({ tsNanos: "1" })).toBeNull()
    expect(decodeLogStreamRow({ body: "x" })).toBeNull()
    expect(decodeLogStreamRow({ tsNanos: "1", body: "x" })?.body).toBe("x")
  })
})
