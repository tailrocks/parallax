import { describe, expect, it } from "vitest"

import type { StoryBeat } from "@/domain/story/story-beat"

describe("StoryBeat domain value", () => {
  it("accepts full and sparse beat shapes", () => {
    const full: StoryBeat = {
      tsNanos: "1000000000",
      lane: "api",
      kind: "error",
      title: "boom",
      traceId: "trace-1",
      spanId: "span-1",
      severity: "ERROR",
      durationNs: "20",
    }
    const sparse: StoryBeat = {
      tsNanos: "0",
      lane: "worker",
      kind: "log",
      title: "hello",
      traceId: "trace-2",
      spanId: null,
      severity: null,
      durationNs: null,
    }
    expect(full.spanId).toBe("span-1")
    expect(sparse.severity).toBeNull()
    expect(sparse.durationNs).toBeNull()
  })

  it("keeps nanosecond timestamps as strings", () => {
    const beat: StoryBeat = {
      tsNanos: "9223372036854775807",
      lane: "db",
      kind: "span.start",
      title: "query",
      traceId: "t",
      spanId: "s",
      severity: null,
      durationNs: null,
    }
    expect(typeof beat.tsNanos).toBe("string")
    expect(BigInt(beat.tsNanos)).toBe(9223372036854775807n)
  })
})
