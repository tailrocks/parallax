/** Trace timeline viewport model (plan 163).
 *
 * Every viewport mutation flows through `timelineReducer` — gestures, the
 * minimap, keyboard shortcuts, and future features (span diffing, log-overlay
 * lanes) dispatch actions; nothing mutates zoom/scroll state directly. All
 * times are milliseconds relative to the trace window start (0 = trace
 * start), so the reducer stays independent of absolute nanosecond timestamps.
 */

export interface TimelineViewport {
  /** Visible window start, ms relative to trace start. */
  startMs: number
  /** Visible window end, ms relative to trace start. Always > startMs. */
  endMs: number
}

export interface TimelineState {
  viewport: TimelineViewport
  /** Full trace duration in ms; the immovable outer bound. */
  traceDurationMs: number
  search: string
  collapsed: ReadonlySet<string>
}

/** Smallest visible window: zooming stops at 0.1ms. */
export const MIN_VISIBLE_MS = 0.1
/** Pointer travel under this many px stays a click, not a drag. */
export const DRAG_THRESHOLD_PX = 4
/** Traces longer than this open zoomed to their first 10s. */
export const DEFAULT_MAX_WINDOW_MS = 10_000
/** Wheel/keyboard zoom step. */
export const ZOOM_FACTOR = 1.15
/** Padding added on each side of a span when zooming to it. */
export const ZOOM_TO_SPAN_PADDING = 0.1
/** Bar edges clamp to this range (% of viewport) when deeply zoomed so a
 * full-trace span never creates a gigapixel element. */
export const BAR_CLAMP_MIN_PCT = -50
export const BAR_CLAMP_MAX_PCT = 150
/** In-bar span-name label renders only when the bar is at least this wide. */
export const BAR_NAME_LABEL_MIN_PX = 56
/** In-bar duration label needs this much width. */
export const BAR_DURATION_LABEL_MIN_PX = 140

export type TimelineAction =
  | { type: "ZOOM"; factor: number; anchorMs: number }
  | { type: "PAN"; deltaMs: number }
  | { type: "ZOOM_TO_SPAN"; startMs: number; endMs: number }
  | { type: "ZOOM_TO_RANGE"; startMs: number; endMs: number }
  | { type: "ZOOM_TO_FIT" }
  | { type: "SET_SEARCH"; search: string }
  | { type: "TOGGLE_COLLAPSE"; spanId: string }

/** Effective outer bound: degenerate traces still get a visible window. */
function boundMs(traceDurationMs: number): number {
  return Math.max(traceDurationMs, MIN_VISIBLE_MS)
}

/** Clamp a desired window onto the trace: width within
 * [MIN_VISIBLE_MS, trace duration], then shifted (never resized) back into
 * [0, trace duration]. */
export function clampViewport(
  desired: TimelineViewport,
  traceDurationMs: number
): TimelineViewport {
  const bound = boundMs(traceDurationMs)
  const width = Math.min(Math.max(desired.endMs - desired.startMs, MIN_VISIBLE_MS), bound)
  let startMs = desired.startMs
  if (startMs < 0) startMs = 0
  if (startMs + width > bound) startMs = bound - width
  return { startMs, endMs: startMs + width }
}

export function initialTimelineState(traceDurationMs: number): TimelineState {
  const bound = boundMs(traceDurationMs)
  return {
    viewport: { startMs: 0, endMs: Math.min(bound, DEFAULT_MAX_WINDOW_MS) },
    traceDurationMs: bound,
    search: "",
    collapsed: new Set<string>(),
  }
}

function zoom(state: TimelineState, factor: number, anchorMs: number): TimelineState {
  if (!Number.isFinite(factor) || factor <= 0) return state
  const { startMs, endMs } = state.viewport
  const width = endMs - startMs
  const nextWidth = width / factor
  // Anchor invariance: the ms under the cursor keeps its viewport fraction.
  const anchorRatio = (anchorMs - startMs) / width
  const viewport = clampViewport(
    {
      startMs: anchorMs - anchorRatio * nextWidth,
      endMs: anchorMs + (1 - anchorRatio) * nextWidth,
    },
    state.traceDurationMs
  )
  return { ...state, viewport }
}

export function timelineReducer(state: TimelineState, action: TimelineAction): TimelineState {
  switch (action.type) {
    case "ZOOM":
      return zoom(state, action.factor, action.anchorMs)
    case "PAN": {
      const { startMs, endMs } = state.viewport
      const viewport = clampViewport(
        { startMs: startMs + action.deltaMs, endMs: endMs + action.deltaMs },
        state.traceDurationMs
      )
      return { ...state, viewport }
    }
    case "ZOOM_TO_SPAN": {
      const spanStart = Math.min(action.startMs, action.endMs)
      const spanEnd = Math.max(action.startMs, action.endMs)
      const pad = Math.max((spanEnd - spanStart) * ZOOM_TO_SPAN_PADDING, MIN_VISIBLE_MS / 2)
      const viewport = clampViewport(
        { startMs: spanStart - pad, endMs: spanEnd + pad },
        state.traceDurationMs
      )
      return { ...state, viewport }
    }
    case "ZOOM_TO_RANGE": {
      const startMs = Math.min(action.startMs, action.endMs)
      const endMs = Math.max(action.startMs, action.endMs)
      // A sub-minimum marquee zooms to the minimum window around its center.
      if (endMs - startMs < MIN_VISIBLE_MS) {
        const center = (startMs + endMs) / 2
        const viewport = clampViewport(
          {
            startMs: center - MIN_VISIBLE_MS / 2,
            endMs: center + MIN_VISIBLE_MS / 2,
          },
          state.traceDurationMs
        )
        return { ...state, viewport }
      }
      const viewport = clampViewport({ startMs, endMs }, state.traceDurationMs)
      return { ...state, viewport }
    }
    case "ZOOM_TO_FIT":
      return {
        ...state,
        viewport: { startMs: 0, endMs: boundMs(state.traceDurationMs) },
      }
    case "SET_SEARCH":
      return { ...state, search: action.search }
    case "TOGGLE_COLLAPSE": {
      const collapsed = new Set(state.collapsed)
      if (collapsed.has(action.spanId)) {
        collapsed.delete(action.spanId)
      } else {
        collapsed.add(action.spanId)
      }
      return { ...state, collapsed }
    }
  }
}

/** Convert a horizontal px offset inside the timeline area (sidebar already
 * subtracted) into an absolute trace-relative ms. */
export function pxToMs(
  offsetPx: number,
  timelineWidthPx: number,
  viewport: TimelineViewport
): number {
  if (timelineWidthPx <= 0) return viewport.startMs
  const width = viewport.endMs - viewport.startMs
  return viewport.startMs + (offsetPx / timelineWidthPx) * width
}

/** Convert a px delta into an ms delta at the current zoom. */
export function pxDeltaToMs(
  deltaPx: number,
  timelineWidthPx: number,
  viewport: TimelineViewport
): number {
  if (timelineWidthPx <= 0) return 0
  return (deltaPx / timelineWidthPx) * (viewport.endMs - viewport.startMs)
}

/** Absolute ms → viewport percent (unclamped; callers clamp per use). */
export function msToPct(ms: number, viewport: TimelineViewport): number {
  const width = viewport.endMs - viewport.startMs
  if (width <= 0) return 0
  return ((ms - viewport.startMs) / width) * 100
}

export interface BarRect {
  leftPct: number
  widthPct: number
}

/** Viewport-relative bar geometry. Returns null when the span is fully
 * outside the viewport (the row renders without a bar); edges clamp to
 * [-50%, 150%] so a full-trace span stays a sane element when deeply
 * zoomed. Consumers apply the `max(2px, N%)` minimum hit width in CSS. */
export function barRect(
  spanStartMs: number,
  spanDurationMs: number,
  viewport: TimelineViewport
): BarRect | null {
  const spanEndMs = spanStartMs + Math.max(spanDurationMs, 0)
  if (spanEndMs < viewport.startMs || spanStartMs > viewport.endMs) return null
  const left = Math.max(msToPct(spanStartMs, viewport), BAR_CLAMP_MIN_PCT)
  const right = Math.min(msToPct(spanEndMs, viewport), BAR_CLAMP_MAX_PCT)
  return { leftPct: left, widthPct: Math.max(right - left, 0) }
}

export interface BarLabelVisibility {
  name: boolean
  duration: boolean
}

/** Label gating by rendered bar width. */
export function barLabelVisibility(barWidthPx: number): BarLabelVisibility {
  return {
    name: barWidthPx >= BAR_NAME_LABEL_MIN_PX,
    duration: barWidthPx >= BAR_DURATION_LABEL_MIN_PX,
  }
}
