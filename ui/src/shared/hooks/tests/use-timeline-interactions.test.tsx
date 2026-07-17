/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from "@testing-library/react"
import { useRef } from "react"
import { afterEach, describe, expect, it, vi } from "vitest"

import { useTimelineInteractions } from "@/shared/hooks/use-timeline-interactions"
import {
  ZOOM_FACTOR,
  type TimelineAction,
  type TimelineViewport,
} from "@/lib/timeline-viewport"

const VIEWPORT: TimelineViewport = { startMs: 0, endMs: 1_000 }

function Harness({
  dispatch,
  viewport = VIEWPORT,
}: {
  dispatch: (action: TimelineAction) => void
  viewport?: TimelineViewport
}) {
  const timelineRef = useRef<HTMLDivElement | null>(null)
  const { handlers, marquee } = useTimelineInteractions({
    timelineRef,
    viewport,
    dispatch,
  })
  return (
    <div data-testid="timeline" ref={timelineRef} {...handlers}>
      <output data-testid="marquee">
        {marquee ? `${marquee.startPx}:${marquee.endPx}` : "none"}
      </output>
    </div>
  )
}

/** 1000px-wide timeline at x=0, so px == ms with the 1s viewport. */
function mountTimeline(dispatch: (action: TimelineAction) => void) {
  render(<Harness dispatch={dispatch} />)
  const element = screen.getByTestId("timeline")
  element.getBoundingClientRect = () =>
    ({
      left: 0,
      width: 1_000,
      top: 0,
      height: 100,
      right: 1_000,
      bottom: 100,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    }) as DOMRect
  return element
}

afterEach(cleanup)

describe("useTimelineInteractions (plan 163)", () => {
  it("keeps pointer travel under 4px a click, not a zoom", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.pointerDown(element, { pointerId: 1, button: 0, clientX: 100 })
    fireEvent.pointerMove(element, { pointerId: 1, clientX: 102 })
    fireEvent.pointerUp(element, { pointerId: 1, clientX: 102 })
    expect(dispatch).not.toHaveBeenCalled()
    expect(screen.getByTestId("marquee").textContent).toBe("none")
  })

  it("commits a marquee drag as ZOOM_TO_RANGE with px→ms payload", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.pointerDown(element, { pointerId: 1, button: 0, clientX: 100 })
    fireEvent.pointerMove(element, { pointerId: 1, clientX: 300 })
    expect(screen.getByTestId("marquee").textContent).toBe("100:300")
    fireEvent.pointerUp(element, { pointerId: 1, clientX: 300 })
    expect(dispatch).toHaveBeenCalledWith({
      type: "ZOOM_TO_RANGE",
      startMs: 100,
      endMs: 300,
    })
    expect(screen.getByTestId("marquee").textContent).toBe("none")
  })

  it("shift-drag pans with relative deltas (content-drag direction)", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.pointerDown(element, {
      pointerId: 1,
      button: 0,
      shiftKey: true,
      clientX: 500,
    })
    fireEvent.pointerMove(element, { pointerId: 1, clientX: 450 })
    fireEvent.pointerMove(element, { pointerId: 1, clientX: 400 })
    fireEvent.pointerUp(element, { pointerId: 1, clientX: 400 })
    const pans = dispatch.mock.calls
      .map(([action]) => action as TimelineAction)
      .filter((action) => action.type === "PAN")
    expect(pans).toEqual([
      { type: "PAN", deltaMs: 50 },
      { type: "PAN", deltaMs: 50 },
    ])
    // A pan never commits a marquee zoom.
    expect(
      dispatch.mock.calls.some(
        ([action]) => (action as TimelineAction).type === "ZOOM_TO_RANGE"
      )
    ).toBe(false)
  })

  it("ctrl-wheel zooms anchored at the cursor", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.wheel(element, { ctrlKey: true, deltaY: -100, clientX: 250 })
    expect(dispatch).toHaveBeenCalledWith({
      type: "ZOOM",
      factor: ZOOM_FACTOR,
      anchorMs: 250,
    })
    fireEvent.wheel(element, { ctrlKey: true, deltaY: 100, clientX: 250 })
    expect(dispatch).toHaveBeenCalledWith({
      type: "ZOOM",
      factor: 1 / ZOOM_FACTOR,
      anchorMs: 250,
    })
  })

  it("shift-wheel and horizontal wheel pan; plain wheel stays native", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.wheel(element, { shiftKey: true, deltaY: 100 })
    expect(dispatch).toHaveBeenCalledWith({ type: "PAN", deltaMs: 100 })
    dispatch.mockClear()
    fireEvent.wheel(element, { deltaX: 80, deltaY: 10 })
    expect(dispatch).toHaveBeenCalledWith({ type: "PAN", deltaMs: 80 })
    dispatch.mockClear()
    fireEvent.wheel(element, { deltaY: 120 })
    expect(dispatch).not.toHaveBeenCalled()
  })

  it("keyboard: +/- zoom around the viewport center, 0 fits", () => {
    const dispatch = vi.fn()
    const element = mountTimeline(dispatch)
    fireEvent.keyDown(element, { key: "+" })
    expect(dispatch).toHaveBeenCalledWith({
      type: "ZOOM",
      factor: ZOOM_FACTOR,
      anchorMs: 500,
    })
    fireEvent.keyDown(element, { key: "-" })
    expect(dispatch).toHaveBeenCalledWith({
      type: "ZOOM",
      factor: 1 / ZOOM_FACTOR,
      anchorMs: 500,
    })
    fireEvent.keyDown(element, { key: "0" })
    expect(dispatch).toHaveBeenCalledWith({ type: "ZOOM_TO_FIT" })
  })
})
