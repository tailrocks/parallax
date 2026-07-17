import { describe, expect, it } from "vitest"

import { createBoundedFrameBuffer } from "@/platform/sse/bounded-frame-buffer"

describe("createBoundedFrameBuffer", () => {
  it("drops oldest on overflow and reports diagnostics", () => {
    const buf = createBoundedFrameBuffer<number>({ maxBufferedItems: 3 })
    buf.push([1, 2, 3, 4, 5])
    expect(buf.size).toBe(3)
    expect(buf.diagnostics.dropped).toBe(2)
    expect(buf.flush()).toEqual([3, 4, 5])
    expect(buf.size).toBe(0)
  })
})
