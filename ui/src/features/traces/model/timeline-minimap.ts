import { pxDeltaToMs, pxToMs } from "@/features/traces/model/timeline-viewport"
import type { TimelineAction, TimelineViewport } from "@/features/traces/model/timeline-viewport"

export const MINIMAP_EDGE_HANDLE_PX = 6

export type MinimapHit = "resize-start" | "pan" | "resize-end" | "recenter"

export interface MinimapGesture {
  hit: Exclude<MinimapHit, "recenter">
  originPx: number
  viewport: TimelineViewport
}

/** Classify a minimap pointer against the viewport rectangle. Edge zones win
 * over the interior so a narrow viewport remains resizable. */
export function minimapHitTest(
  pointerPx: number,
  viewportStartPx: number,
  viewportEndPx: number,
  edgeHandlePx = MINIMAP_EDGE_HANDLE_PX
): MinimapHit {
  const start = Math.min(viewportStartPx, viewportEndPx)
  const end = Math.max(viewportStartPx, viewportEndPx)
  if (Math.abs(pointerPx - start) <= edgeHandlePx) return "resize-start"
  if (Math.abs(pointerPx - end) <= edgeHandlePx) return "resize-end"
  if (pointerPx > start && pointerPx < end) return "pan"
  return "recenter"
}

/** Convert a captured minimap drag into the same reducer actions used by the
 * waterfall. The captured viewport prevents drift across repeated moves. */
export function minimapDragAction(
  gesture: MinimapGesture,
  pointerPx: number,
  minimapWidthPx: number,
  traceDurationMs: number
): TimelineAction {
  const full = { startMs: 0, endMs: Math.max(traceDurationMs, 0.1) }
  if (gesture.hit === "pan") {
    return {
      type: "PAN",
      deltaMs: pxDeltaToMs(pointerPx - gesture.originPx, minimapWidthPx, full),
    }
  }
  const pointerMs = pxToMs(pointerPx, minimapWidthPx, full)
  return gesture.hit === "resize-start"
    ? {
        type: "ZOOM_TO_RANGE",
        startMs: pointerMs,
        endMs: gesture.viewport.endMs,
      }
    : {
        type: "ZOOM_TO_RANGE",
        startMs: gesture.viewport.startMs,
        endMs: pointerMs,
      }
}

/** Outside click recenters without changing viewport width. Reducer clamping
 * handles clicks close to trace bounds. */
export function minimapRecenterAction(
  pointerPx: number,
  minimapWidthPx: number,
  traceDurationMs: number,
  viewport: TimelineViewport
): TimelineAction {
  const full = { startMs: 0, endMs: Math.max(traceDurationMs, 0.1) }
  const targetMs = pxToMs(pointerPx, minimapWidthPx, full)
  const centerMs = (viewport.startMs + viewport.endMs) / 2
  return { type: "PAN", deltaMs: targetMs - centerMs }
}
