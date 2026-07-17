/** Trace timeline gesture binding (plan 163).
 *
 * Maps pointer/wheel/keyboard gestures onto timeline reducer actions:
 * - drag = marquee → ZOOM_TO_RANGE; travel < 4px stays a span click
 * - Shift+drag or middle-button drag = pan
 * - Ctrl/⌘+wheel = cursor-anchored zoom; Shift+wheel or horizontal wheel =
 *   pan; plain wheel = native vertical scroll (untouched)
 * - `+`/`-` zoom around the viewport center, `0` fits the trace
 *
 * The wheel listener is native with `passive: false` (React's synthetic
 * wheel handlers are passive and cannot preventDefault). Pointer-down
 * captures the viewport so pan deltas convert px→ms against a stable zoom,
 * never a stale closure.
 */

import { useCallback, useEffect, useRef, useState } from "react"

import {
  DRAG_THRESHOLD_PX,
  ZOOM_FACTOR,
  pxDeltaToMs,
  pxToMs,
  type TimelineAction,
  type TimelineViewport,
} from "@/features/traces/model/timeline-viewport"

export interface MarqueeRange {
  /** px offsets inside the timeline area; start may exceed end. */
  startPx: number
  endPx: number
}

export interface UseTimelineInteractionsOptions {
  /** The bar area (span-name sidebar excluded) the gestures bind to. */
  timelineRef: React.RefObject<HTMLElement | null>
  viewport: TimelineViewport
  dispatch: (action: TimelineAction) => void
}

export interface TimelineInteractionHandlers {
  onPointerDown: (event: React.PointerEvent<HTMLElement>) => void
  onPointerMove: (event: React.PointerEvent<HTMLElement>) => void
  onPointerUp: (event: React.PointerEvent<HTMLElement>) => void
  onPointerCancel: (event: React.PointerEvent<HTMLElement>) => void
  onKeyDown: (event: React.KeyboardEvent<HTMLElement>) => void
}

export interface TimelineInteractions {
  /** Active marquee selection, for the overlay rendering. */
  marquee: MarqueeRange | null
  isPanning: boolean
  handlers: TimelineInteractionHandlers
  /** Dispatch helper for row double-click → ZOOM_TO_SPAN. */
  zoomToSpan: (startMs: number, endMs: number) => void
}

interface PointerGesture {
  pointerId: number
  originPx: number
  lastPx: number
  mode: "marquee" | "pan"
  /** Viewport captured at pointer-down: px→ms conversion stays stable. */
  viewport: TimelineViewport
  moved: boolean
}

function timelineOffsetPx(element: HTMLElement, clientX: number): number {
  return clientX - element.getBoundingClientRect().left
}

export function useTimelineInteractions({
  timelineRef,
  viewport,
  dispatch,
}: UseTimelineInteractionsOptions): TimelineInteractions {
  const [marquee, setMarquee] = useState<MarqueeRange | null>(null)
  const [isPanning, setIsPanning] = useState(false)
  const gestureRef = useRef<PointerGesture | null>(null)
  const viewportRef = useRef(viewport)
  viewportRef.current = viewport

  const widthPx = useCallback(() => {
    return timelineRef.current?.getBoundingClientRect().width ?? 0
  }, [timelineRef])

  const onPointerDown = useCallback(
    (event: React.PointerEvent<HTMLElement>) => {
      const element = timelineRef.current
      if (!element) return
      if (event.button !== 0 && event.button !== 1) return
      const pan = event.shiftKey || event.button === 1
      const originPx = timelineOffsetPx(element, event.clientX)
      gestureRef.current = {
        pointerId: event.pointerId,
        originPx,
        lastPx: originPx,
        mode: pan ? "pan" : "marquee",
        viewport: viewportRef.current,
        moved: false,
      }
      if (pan) setIsPanning(true)
      // jsdom lacks pointer capture; real browsers keep the drag alive when
      // the pointer leaves the element.
      if (typeof element.setPointerCapture === "function") {
        element.setPointerCapture(event.pointerId)
      }
    },
    [timelineRef]
  )

  const onPointerMove = useCallback(
    (event: React.PointerEvent<HTMLElement>) => {
      const gesture = gestureRef.current
      const element = timelineRef.current
      if (!gesture || !element || event.pointerId !== gesture.pointerId) return
      const currentPx = timelineOffsetPx(element, event.clientX)
      if (Math.abs(currentPx - gesture.originPx) >= DRAG_THRESHOLD_PX) {
        gesture.moved = true
      }
      if (gesture.mode === "pan") {
        // Dragging content right pans the viewport left.
        const deltaMs = pxDeltaToMs(gesture.lastPx - currentPx, widthPx(), gesture.viewport)
        if (deltaMs !== 0) dispatch({ type: "PAN", deltaMs })
        gesture.lastPx = currentPx
        return
      }
      gesture.lastPx = currentPx
      setMarquee(gesture.moved ? { startPx: gesture.originPx, endPx: currentPx } : null)
    },
    [dispatch, timelineRef, widthPx]
  )

  const finishGesture = useCallback(
    (commit: boolean) => {
      const gesture = gestureRef.current
      gestureRef.current = null
      setIsPanning(false)
      setMarquee(null)
      if (!gesture || !commit) return
      if (gesture.mode === "marquee" && gesture.moved) {
        const width = widthPx()
        dispatch({
          type: "ZOOM_TO_RANGE",
          startMs: pxToMs(gesture.originPx, width, gesture.viewport),
          endMs: pxToMs(gesture.lastPx, width, gesture.viewport),
        })
      }
    },
    [dispatch, widthPx]
  )

  const onPointerUp = useCallback(
    (event: React.PointerEvent<HTMLElement>) => {
      if (gestureRef.current?.pointerId !== event.pointerId) return
      finishGesture(true)
    },
    [finishGesture]
  )

  const onPointerCancel = useCallback(
    (event: React.PointerEvent<HTMLElement>) => {
      if (gestureRef.current?.pointerId !== event.pointerId) return
      finishGesture(false)
    },
    [finishGesture]
  )

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLElement>) => {
      const { startMs, endMs } = viewportRef.current
      const centerMs = (startMs + endMs) / 2
      if (event.key === "+" || event.key === "=") {
        event.preventDefault()
        dispatch({ type: "ZOOM", factor: ZOOM_FACTOR, anchorMs: centerMs })
      } else if (event.key === "-" || event.key === "_") {
        event.preventDefault()
        dispatch({ type: "ZOOM", factor: 1 / ZOOM_FACTOR, anchorMs: centerMs })
      } else if (event.key === "0") {
        event.preventDefault()
        dispatch({ type: "ZOOM_TO_FIT" })
      }
    },
    [dispatch]
  )

  useEffect(() => {
    const element = timelineRef.current
    if (!element) return

    const onWheel = (event: WheelEvent) => {
      const width = element.getBoundingClientRect().width
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault()
        dispatch({
          type: "ZOOM",
          factor: event.deltaY < 0 ? ZOOM_FACTOR : 1 / ZOOM_FACTOR,
          anchorMs: pxToMs(timelineOffsetPx(element, event.clientX), width, viewportRef.current),
        })
        return
      }
      const horizontal = Math.abs(event.deltaX) > Math.abs(event.deltaY)
      if (event.shiftKey || horizontal) {
        event.preventDefault()
        const deltaPx = horizontal ? event.deltaX : event.deltaY
        dispatch({
          type: "PAN",
          deltaMs: pxDeltaToMs(deltaPx, width, viewportRef.current),
        })
      }
      // Plain vertical wheel: native scroll, untouched.
    }

    element.addEventListener("wheel", onWheel, { passive: false })
    return () => element.removeEventListener("wheel", onWheel)
  }, [dispatch, timelineRef])

  const zoomToSpan = useCallback(
    (startMs: number, endMs: number) => {
      dispatch({ type: "ZOOM_TO_SPAN", startMs, endMs })
    },
    [dispatch]
  )

  return {
    marquee,
    isPanning,
    handlers: {
      onPointerDown,
      onPointerMove,
      onPointerUp,
      onPointerCancel,
      onKeyDown,
    },
    zoomToSpan,
  }
}
