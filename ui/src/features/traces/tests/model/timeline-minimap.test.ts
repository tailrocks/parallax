import { describe, expect, it } from "vitest"

import {
  minimapDragAction,
  minimapHitTest,
  minimapRecenterAction,
} from "@/features/traces/model/timeline-minimap"

const viewport = { startMs: 200, endMs: 400 }

describe("minimapHitTest", () => {
  it("prioritizes edge resize zones", () => {
    expect(minimapHitTest(102, 100, 300)).toBe("resize-start")
    expect(minimapHitTest(296, 100, 300)).toBe("resize-end")
  })

  it("classifies interior pan and outside recenter", () => {
    expect(minimapHitTest(200, 100, 300)).toBe("pan")
    expect(minimapHitTest(50, 100, 300)).toBe("recenter")
  })

  it("handles reversed viewport coordinates", () => {
    expect(minimapHitTest(100, 300, 100)).toBe("resize-start")
  })
})

describe("minimapDragAction", () => {
  it("pans by full-trace minimap scale", () => {
    expect(minimapDragAction({ hit: "pan", originPx: 100, viewport }, 150, 500, 1_000)).toEqual({
      type: "PAN",
      deltaMs: 100,
    })
  })

  it("resizes either viewport edge", () => {
    expect(
      minimapDragAction({ hit: "resize-start", originPx: 100, viewport }, 50, 500, 1_000)
    ).toEqual({ type: "ZOOM_TO_RANGE", startMs: 100, endMs: 400 })
    expect(
      minimapDragAction({ hit: "resize-end", originPx: 200, viewport }, 350, 500, 1_000)
    ).toEqual({ type: "ZOOM_TO_RANGE", startMs: 200, endMs: 700 })
  })
})

describe("minimapRecenterAction", () => {
  it("moves the captured viewport center to the click", () => {
    expect(minimapRecenterAction(400, 500, 1_000, viewport)).toEqual({
      type: "PAN",
      deltaMs: 500,
    })
  })

  it("degrades safely for a zero-width minimap", () => {
    expect(minimapRecenterAction(400, 0, 1_000, viewport)).toEqual({
      type: "PAN",
      deltaMs: -300,
    })
  })
})
