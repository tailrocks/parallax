import { describe, expect, it } from "vitest"

import {
  BAR_CLAMP_MAX_PCT,
  BAR_CLAMP_MIN_PCT,
  DEFAULT_MAX_WINDOW_MS,
  MIN_VISIBLE_MS,
  barLabelVisibility,
  barRect,
  clampViewport,
  initialTimelineState,
  msToPct,
  pxDeltaToMs,
  pxToMs,
  timelineReducer,
  type TimelineState,
} from "@/lib/timeline-viewport"

function stateWith(
  viewport: { startMs: number; endMs: number },
  traceDurationMs: number
): TimelineState {
  return {
    viewport,
    traceDurationMs,
    search: "",
    collapsed: new Set<string>(),
  }
}

describe("initialTimelineState", () => {
  it("fits short traces entirely", () => {
    expect(initialTimelineState(250).viewport).toEqual({
      startMs: 0,
      endMs: 250,
    })
  })

  it("opens traces longer than 10s zoomed to the first 10s", () => {
    expect(initialTimelineState(60_000).viewport).toEqual({
      startMs: 0,
      endMs: DEFAULT_MAX_WINDOW_MS,
    })
  })

  it("gives zero-duration traces the minimum visible window", () => {
    const state = initialTimelineState(0)
    expect(state.viewport.endMs - state.viewport.startMs).toBe(MIN_VISIBLE_MS)
  })
})

describe("clampViewport", () => {
  it("shifts a window that runs past the trace end back inside", () => {
    expect(clampViewport({ startMs: 80, endMs: 130 }, 100)).toEqual({
      startMs: 50,
      endMs: 100,
    })
  })

  it("shifts a negative start to zero without resizing", () => {
    expect(clampViewport({ startMs: -20, endMs: 30 }, 100)).toEqual({
      startMs: 0,
      endMs: 50,
    })
  })

  it("caps the window at the trace duration", () => {
    expect(clampViewport({ startMs: -50, endMs: 500 }, 100)).toEqual({
      startMs: 0,
      endMs: 100,
    })
  })

  it("enforces the minimum visible window", () => {
    const clamped = clampViewport({ startMs: 10, endMs: 10.0001 }, 100)
    expect(clamped.endMs - clamped.startMs).toBeCloseTo(MIN_VISIBLE_MS, 10)
  })
})

describe("ZOOM", () => {
  it("keeps the ms under the cursor fixed (anchor invariance)", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 1_000)
    const anchorMs = 25
    const next = timelineReducer(state, { type: "ZOOM", factor: 2, anchorMs })
    const before =
      (anchorMs - state.viewport.startMs) /
      (state.viewport.endMs - state.viewport.startMs)
    const after =
      (anchorMs - next.viewport.startMs) /
      (next.viewport.endMs - next.viewport.startMs)
    expect(after).toBeCloseTo(before, 10)
    expect(next.viewport.endMs - next.viewport.startMs).toBeCloseTo(50, 10)
  })

  it("stops zooming in at the minimum window", () => {
    const state = stateWith({ startMs: 10, endMs: 10 + MIN_VISIBLE_MS }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM",
      factor: 10,
      anchorMs: 10,
    })
    expect(next.viewport.endMs - next.viewport.startMs).toBeCloseTo(
      MIN_VISIBLE_MS,
      10
    )
  })

  it("stops zooming out at the trace bounds", () => {
    const state = stateWith({ startMs: 40, endMs: 60 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM",
      factor: 0.01,
      anchorMs: 50,
    })
    expect(next.viewport).toEqual({ startMs: 0, endMs: 100 })
  })

  it("ignores non-positive or non-finite factors", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    expect(
      timelineReducer(state, { type: "ZOOM", factor: 0, anchorMs: 50 })
    ).toBe(state)
    expect(
      timelineReducer(state, {
        type: "ZOOM",
        factor: Number.NaN,
        anchorMs: 50,
      })
    ).toBe(state)
  })
})

describe("PAN", () => {
  it("shifts the window preserving its width", () => {
    const state = stateWith({ startMs: 10, endMs: 30 }, 100)
    const next = timelineReducer(state, { type: "PAN", deltaMs: 15 })
    expect(next.viewport).toEqual({ startMs: 25, endMs: 45 })
  })

  it("clamps at the trace start", () => {
    const state = stateWith({ startMs: 10, endMs: 30 }, 100)
    const next = timelineReducer(state, { type: "PAN", deltaMs: -50 })
    expect(next.viewport).toEqual({ startMs: 0, endMs: 20 })
  })

  it("clamps at the trace end", () => {
    const state = stateWith({ startMs: 70, endMs: 90 }, 100)
    const next = timelineReducer(state, { type: "PAN", deltaMs: 50 })
    expect(next.viewport).toEqual({ startMs: 80, endMs: 100 })
  })
})

describe("ZOOM_TO_SPAN", () => {
  it("pads the span by 10% on each side", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM_TO_SPAN",
      startMs: 40,
      endMs: 60,
    })
    expect(next.viewport.startMs).toBeCloseTo(38, 10)
    expect(next.viewport.endMs).toBeCloseTo(62, 10)
  })

  it("shifts padding that would cross the trace start (width preserved)", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM_TO_SPAN",
      startMs: 0,
      endMs: 50,
    })
    expect(next.viewport.startMs).toBe(0)
    expect(next.viewport.endMs).toBeCloseTo(60, 10)
  })

  it("gives zero-duration spans the minimum window", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM_TO_SPAN",
      startMs: 50,
      endMs: 50,
    })
    expect(next.viewport.endMs - next.viewport.startMs).toBeCloseTo(
      MIN_VISIBLE_MS,
      10
    )
  })
})

describe("ZOOM_TO_RANGE", () => {
  it("adopts the marquee range regardless of drag direction", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM_TO_RANGE",
      startMs: 60,
      endMs: 20,
    })
    expect(next.viewport).toEqual({ startMs: 20, endMs: 60 })
  })

  it("expands a sub-minimum marquee around its center", () => {
    const state = stateWith({ startMs: 0, endMs: 100 }, 100)
    const next = timelineReducer(state, {
      type: "ZOOM_TO_RANGE",
      startMs: 50,
      endMs: 50.01,
    })
    const { startMs, endMs } = next.viewport
    expect(endMs - startMs).toBeCloseTo(MIN_VISIBLE_MS, 10)
    expect((startMs + endMs) / 2).toBeCloseTo(50.005, 10)
  })
})

describe("ZOOM_TO_FIT and secondary actions", () => {
  it("fits the whole trace", () => {
    const state = stateWith({ startMs: 40, endMs: 45 }, 1_000)
    expect(timelineReducer(state, { type: "ZOOM_TO_FIT" }).viewport).toEqual({
      startMs: 0,
      endMs: 1_000,
    })
  })

  it("sets search text", () => {
    const state = initialTimelineState(100)
    expect(
      timelineReducer(state, { type: "SET_SEARCH", search: "db" }).search
    ).toBe("db")
  })

  it("toggles collapse on and off without mutating prior state", () => {
    const state = initialTimelineState(100)
    const on = timelineReducer(state, {
      type: "TOGGLE_COLLAPSE",
      spanId: "a",
    })
    expect(on.collapsed.has("a")).toBe(true)
    expect(state.collapsed.has("a")).toBe(false)
    const off = timelineReducer(on, { type: "TOGGLE_COLLAPSE", spanId: "a" })
    expect(off.collapsed.has("a")).toBe(false)
  })
})

describe("px↔ms conversion", () => {
  const viewport = { startMs: 100, endMs: 300 }

  it("maps container offsets onto the visible window", () => {
    expect(pxToMs(0, 1_000, viewport)).toBe(100)
    expect(pxToMs(500, 1_000, viewport)).toBe(200)
    expect(pxToMs(1_000, 1_000, viewport)).toBe(300)
  })

  it("converts px deltas at the current zoom", () => {
    expect(pxDeltaToMs(250, 1_000, viewport)).toBe(50)
    expect(pxDeltaToMs(-100, 1_000, viewport)).toBe(-20)
  })

  it("degrades safely at zero width", () => {
    expect(pxToMs(50, 0, viewport)).toBe(100)
    expect(pxDeltaToMs(50, 0, viewport)).toBe(0)
  })

  it("maps ms onto viewport percent", () => {
    expect(msToPct(100, viewport)).toBe(0)
    expect(msToPct(200, viewport)).toBe(50)
    expect(msToPct(350, viewport)).toBe(125)
  })
})

describe("barRect", () => {
  const viewport = { startMs: 100, endMs: 200 }

  it("skips spans fully outside the viewport", () => {
    expect(barRect(0, 50, viewport)).toBeNull()
    expect(barRect(250, 50, viewport)).toBeNull()
  })

  it("positions an in-view span", () => {
    expect(barRect(125, 50, viewport)).toEqual({ leftPct: 25, widthPct: 50 })
  })

  it("clamps a full-trace span to the [-50, 150] envelope", () => {
    const rect = barRect(0, 10_000, viewport)
    expect(rect).toEqual({
      leftPct: BAR_CLAMP_MIN_PCT,
      widthPct: BAR_CLAMP_MAX_PCT - BAR_CLAMP_MIN_PCT,
    })
  })

  it("keeps zero-duration spans as zero-width rects for CSS min-width", () => {
    expect(barRect(150, 0, viewport)).toEqual({ leftPct: 50, widthPct: 0 })
  })
})

describe("barLabelVisibility", () => {
  it("gates the name label at 56px and the duration label at 140px", () => {
    expect(barLabelVisibility(55)).toEqual({ name: false, duration: false })
    expect(barLabelVisibility(56)).toEqual({ name: true, duration: false })
    expect(barLabelVisibility(140)).toEqual({ name: true, duration: true })
  })
})
