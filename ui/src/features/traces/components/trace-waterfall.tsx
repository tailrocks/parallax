import type { CSSProperties, RefCallback } from "react"
import {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react"
import { IconAffiliate } from "@tabler/icons-react"
import { useVirtualizer } from "@tanstack/react-virtual"

import { SpanKindChip } from "@/features/traces/components/trace-span-kind"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import {
  buildTraceTree,
  computeSelfTimes,
  computeWindow,
  errorPathSpanIds,
  groupByService,
} from "@/features/traces/model/trace-tree"
import { colorForSpan } from "@/lib/color-by"
import type { ColorByStrategy } from "@/lib/color-by"
import {
  barLabelVisibility,
  barRect,
  initialTimelineState,
  timelineReducer,
} from "@/lib/timeline-viewport"
import type { TimelineViewport } from "@/lib/timeline-viewport"
import { useTimelineInteractions } from "@/hooks/use-timeline-interactions"
import type {
  OrderedTraceSpan,
  TraceTreeSpan,
} from "@/features/traces/model/trace-tree"
import { formatDurationNs } from "@/lib/format"

export const WHOLE_TRACE_ID = "__whole_trace__"
export type TraceViewMode = "tree" | "errors" | "lanes"
const MINIMAP_MAX_BARS = 2_000
/** Deepest visually indented level; deeper rows stop indenting so the fixed
 * name column keeps room for the span name (corpus id t-deep). */
const INDENT_DEPTH_CAP = 10

export interface WaterfallSpan extends TraceTreeSpan {
  service: string
  name: string
  kind: string
  statusCode: string
  statusMessage: string
  durationNs: string
  /** Parsed span attributes for attribute color-by (optional). */
  attributeMap?: Record<string, string>
}

type WaterfallRow = OrderedTraceSpan<WaterfallSpan>
type LaneItem = {
  type: "lane"
  key: string
  service: string
  spanCount: number
  durationNs: string
}
type SpanItem = { type: "span"; row: WaterfallRow }
type WaterfallItem = LaneItem | SpanItem

function laneDurationNs(rows: readonly WaterfallRow[]): string {
  if (rows.length === 0) return "0"
  return computeWindow(rows.map((row) => row.span)).durationNs.toString()
}

function visualItemsForMode(
  mode: TraceViewMode,
  rows: readonly WaterfallRow[]
): WaterfallItem[] {
  if (mode !== "lanes") {
    return rows.map((row) => ({ type: "span", row }))
  }

  return groupByService(rows).flatMap((group, index) => [
    {
      type: "lane" as const,
      key: `${index}-${group.service}-${group.spans[0]?.span.spanId ?? "empty"}`,
      service: group.service,
      spanCount: group.spans.length,
      durationNs: laneDurationNs(group.spans),
    },
    ...group.spans.map((row) => ({ type: "span" as const, row })),
  ])
}

function sampledMinimapRows(rows: readonly WaterfallRow[]): WaterfallRow[] {
  if (rows.length <= MINIMAP_MAX_BARS) return [...rows]
  const step = Math.ceil(rows.length / MINIMAP_MAX_BARS)
  return rows.filter((_, index) => index % step === 0)
}

export function TraceWaterfall({
  spans,
  selectedId,
  onSelect,
  highlightIds,
  mode = "tree",
  colorBy = { kind: "service" },
  initialViewport,
  onViewportChange,
}: {
  spans: WaterfallSpan[]
  selectedId: string | null
  onSelect: (spanId: string | null) => void
  highlightIds?: ReadonlySet<string> | undefined
  mode?: TraceViewMode
  colorBy?: ColorByStrategy
  /** Restore a viewport (e.g. from URL search params). */
  initialViewport?: TimelineViewport | undefined
  onViewportChange?: ((viewport: TimelineViewport | null) => void) | undefined
}) {
  const allRows = useMemo(() => buildTraceTree(spans), [spans])
  const traceWindow = useMemo(() => computeWindow(spans), [spans])
  const traceDurationMs = Number(traceWindow.durationNs) / 1_000_000
  const [timeline, dispatch] = useReducer(
    timelineReducer,
    traceDurationMs,
    (durationMs) => {
      const state = initialTimelineState(durationMs)
      return initialViewport ? { ...state, viewport: initialViewport } : state
    }
  )
  const viewport = timeline.viewport
  const isFit =
    viewport.startMs <= 0 && viewport.endMs >= timeline.traceDurationMs
  const selfTimes = useMemo(() => computeSelfTimes(spans), [spans])
  const reportViewport = useRef(onViewportChange)
  reportViewport.current = onViewportChange
  useEffect(() => {
    reportViewport.current?.(isFit ? null : viewport)
  }, [viewport, isFit])
  const barAreaRef = useRef<HTMLDivElement | null>(null)
  const [barAreaWidth, setBarAreaWidth] = useState(800)
  useEffect(() => {
    const node = barAreaRef.current
    if (!node) return
    const observer = new ResizeObserver((entries) => {
      const width = entries[0]?.contentRect.width
      if (width) setBarAreaWidth(width)
    })
    observer.observe(node)
    return () => observer.disconnect()
  }, [])
  const interactions = useTimelineInteractions({
    timelineRef: barAreaRef,
    viewport,
    dispatch,
  })
  const spanStartMs = useCallback(
    (span: WaterfallSpan) =>
      Number(BigInt(span.tsNanos) - traceWindow.startNs) / 1_000_000,
    [traceWindow.startNs]
  )
  // Minimap = second controller over the same viewport state: drag the
  // rectangle interior to pan, its edges to resize, click elsewhere to
  // recenter. Percentages are relative to the FULL trace.
  const minimapRef = useRef<HTMLDivElement | null>(null)
  const surfaceDown = useRef<{ x: number; y: number } | null>(null)
  const minimapGesture = useRef<{
    mode: "pan" | "resize-start" | "resize-end"
    originMs: number
    viewport: TimelineViewport
  } | null>(null)
  const minimapViewportPct = {
    left: (viewport.startMs / timeline.traceDurationMs) * 100,
    width:
      ((viewport.endMs - viewport.startMs) / timeline.traceDurationMs) * 100,
  }
  const minimapMsAt = (clientX: number): number => {
    const node = minimapRef.current
    if (!node) return 0
    const rect = node.getBoundingClientRect()
    const ratio = Math.min(Math.max((clientX - rect.left) / rect.width, 0), 1)
    return ratio * timeline.traceDurationMs
  }
  const onMinimapPointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    event.preventDefault()
    // jsdom has no pointer capture; feature-detect like the gesture hook.
    if (typeof event.currentTarget.setPointerCapture === "function") {
      event.currentTarget.setPointerCapture(event.pointerId)
    }
    const ms = minimapMsAt(event.clientX)
    const edgeMs = timeline.traceDurationMs * 0.015
    const { startMs, endMs } = viewport
    if (Math.abs(ms - startMs) <= edgeMs) {
      minimapGesture.current = { mode: "resize-start", originMs: ms, viewport }
    } else if (Math.abs(ms - endMs) <= edgeMs) {
      minimapGesture.current = { mode: "resize-end", originMs: ms, viewport }
    } else if (ms > startMs && ms < endMs) {
      minimapGesture.current = { mode: "pan", originMs: ms, viewport }
    } else {
      // Click outside the rectangle: recenter the current width there.
      const width = endMs - startMs
      dispatch({
        type: "ZOOM_TO_RANGE",
        startMs: ms - width / 2,
        endMs: ms + width / 2,
      })
      minimapGesture.current = {
        mode: "pan",
        originMs: ms,
        viewport: {
          startMs: ms - width / 2,
          endMs: ms + width / 2,
        },
      }
    }
  }
  const onMinimapPointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    const gesture = minimapGesture.current
    if (!gesture) return
    const ms = minimapMsAt(event.clientX)
    if (gesture.mode === "pan") {
      dispatch({
        type: "ZOOM_TO_RANGE",
        startMs: gesture.viewport.startMs + (ms - gesture.originMs),
        endMs: gesture.viewport.endMs + (ms - gesture.originMs),
      })
    } else if (gesture.mode === "resize-start") {
      dispatch({
        type: "ZOOM_TO_RANGE",
        startMs: ms,
        endMs: gesture.viewport.endMs,
      })
    } else {
      dispatch({
        type: "ZOOM_TO_RANGE",
        startMs: gesture.viewport.startMs,
        endMs: ms,
      })
    }
  }
  const onMinimapPointerUp = (event: React.PointerEvent<HTMLDivElement>) => {
    minimapGesture.current = null
    if (
      typeof event.currentTarget.hasPointerCapture === "function" &&
      event.currentTarget.hasPointerCapture(event.pointerId)
    ) {
      event.currentTarget.releasePointerCapture(event.pointerId)
    }
  }
  const errorPathIds = useMemo(() => errorPathSpanIds(spans), [spans])
  const showNoErrors = mode === "errors" && errorPathIds.size === 0
  const rows = useMemo(
    () =>
      mode === "errors" && errorPathIds.size > 0
        ? allRows.filter((row) => errorPathIds.has(row.span.spanId))
        : allRows,
    [allRows, errorPathIds, mode]
  )
  const visualItems = useMemo(
    () => visualItemsForMode(mode, rows),
    [mode, rows]
  )
  const minimapRows = useMemo(() => sampledMinimapRows(rows), [rows])
  const ids = useMemo(
    () => [WHOLE_TRACE_ID, ...rows.map((row) => row.span.spanId)],
    [rows]
  )
  const services = useMemo(
    () => Array.from(new Set(spans.map((span) => span.service))),
    [spans]
  )
  const spanIds = useMemo(
    () => new Set(spans.map((span) => span.spanId)),
    [spans]
  )
  const shouldVirtualize = visualItems.length > 300
  const itemIndexBySpanId = useMemo(() => {
    const index = new Map<string, number>()
    visualItems.forEach((item, itemIndex) => {
      if (item.type === "span") index.set(item.row.span.spanId, itemIndex)
    })
    return index
  }, [visualItems])
  const rowsRef = useRef<HTMLDivElement | null>(null)
  const rowRefs = useRef(new Map<string, HTMLButtonElement>())
  const rowVirtualizer = useVirtualizer({
    count: visualItems.length,
    getScrollElement: () => rowsRef.current,
    estimateSize: (index) => (visualItems[index]?.type === "lane" ? 34 : 54),
    overscan: 12,
  })
  const virtualItems = rowVirtualizer.getVirtualItems()

  const setRowRef =
    (spanId: string): RefCallback<HTMLButtonElement> =>
    (node) => {
      if (node) rowRefs.current.set(spanId, node)
      else rowRefs.current.delete(spanId)
    }

  const scrollToSpan = useCallback(
    (spanId: string) => {
      const itemIndex = itemIndexBySpanId.get(spanId)
      if (itemIndex === undefined) return
      if (shouldVirtualize) {
        rowVirtualizer.scrollToIndex(itemIndex, { align: "auto" })
      }
      globalThis.setTimeout(() => {
        rowRefs.current.get(spanId)?.scrollIntoView({ block: "nearest" })
      }, 0)
    },
    [itemIndexBySpanId, rowVirtualizer, shouldVirtualize]
  )

  useEffect(() => {
    if (!selectedId && spans.length > 0) onSelect(WHOLE_TRACE_ID)
  }, [onSelect, selectedId, spans.length])

  const moveSelection = (direction: 1 | -1) => {
    const current = selectedId ? ids.indexOf(selectedId) : 0
    const next = Math.min(Math.max(current + direction, 0), ids.length - 1)
    if (shouldVirtualize && next > 0) {
      scrollToSpan(ids[next] ?? WHOLE_TRACE_ID)
    }
    onSelect(ids[next] ?? WHOLE_TRACE_ID)
  }

  useEffect(() => {
    if (!shouldVirtualize || !selectedId || selectedId === WHOLE_TRACE_ID) {
      return
    }
    scrollToSpan(selectedId)
  }, [scrollToSpan, selectedId, shouldVirtualize])

  const renderSpanRow = (row: WaterfallRow, style?: CSSProperties) => {
    const { span, depth } = row
    const active = span.spanId === selectedId
    const highlighted = highlightIds?.has(span.spanId) ?? false
    const failed = span.statusCode === "STATUS_CODE_ERROR"
    const detached =
      Boolean(span.parentSpanId) && !spanIds.has(span.parentSpanId ?? "")
    const startMs = spanStartMs(span)
    const durationMs = Number(span.durationNs) / 1_000_000
    const rect = barRect(startMs, durationMs, viewport)
    const barPx = rect ? (rect.widthPct / 100) * barAreaWidth : 0
    const labels = barLabelVisibility(barPx)
    const barColor = failed
      ? "var(--chart-error)"
      : colorForSpan(colorBy, {
          service: span.service,
          kind: span.kind,
          statusCode: span.statusCode,
          attributes: span.attributeMap ?? {},
        })
    const selfNs = selfTimes.get(span.spanId)
    return (
      <button
        key={span.spanId}
        ref={setRowRef(span.spanId)}
        type="button"
        onClick={() => onSelect(active ? null : span.spanId)}
        onDoubleClick={() =>
          interactions.zoomToSpan(startMs, startMs + durationMs)
        }
        className={cn(
          "grid w-full cursor-pointer grid-cols-[16rem_minmax(0,1fr)_6.5rem] items-center rounded-md border-l-2 border-transparent py-1.5 text-left text-sm hover:bg-accent/50",
          active && "bg-accent/70",
          highlighted && "border-primary bg-primary/5",
          style && "absolute top-0 left-0"
        )}
        data-testid={`trace-row-${span.spanId}`}
        style={style}
      >
        <div
          className="flex min-w-0 items-start gap-2 pr-3"
          // Indentation caps so very deep chains keep a readable name column;
          // depth beyond the cap reads from the row order, as in any tree UI.
          style={{ paddingLeft: Math.min(depth, INDENT_DEPTH_CAP) * 14 + 8 }}
        >
          <SpanKindChip compact kind={span.kind} statusCode={span.statusCode} />
          <div className="min-w-0">
            <span className="block truncate" title={span.name}>
              {span.name}
            </span>
            <div className="mt-1 flex flex-wrap items-center gap-1">
              <Badge variant="outline">{span.service}</Badge>
              {failed ? <Badge variant="rose">error</Badge> : null}
              {detached ? (
                <Badge
                  variant="amber"
                  title="Parent span never arrived; shown at the top level"
                >
                  detached
                </Badge>
              ) : null}
            </div>
          </div>
        </div>
        <div className="relative h-5 overflow-hidden">
          {rect ? (
            <div
              className={cn(
                "absolute top-1/2 flex h-3 -translate-y-1/2 items-center overflow-hidden rounded-full px-1",
                active &&
                  "ring-2 ring-foreground/30 ring-offset-1 ring-offset-background",
                highlighted &&
                  "ring-2 ring-primary/60 ring-offset-1 ring-offset-background"
              )}
              style={{
                left: `${rect.leftPct}%`,
                // 2px minimum hit target, computed here — jsdom's CSS parser
                // rejects `max()` expressions in width.
                width: `${Math.max(rect.widthPct, (2 / Math.max(barAreaWidth, 1)) * 100)}%`,
                backgroundColor: barColor,
              }}
            >
              {labels.name ? (
                <span className="truncate text-[10px] leading-none text-white/95 mix-blend-luminosity">
                  {span.name}
                  {labels.duration
                    ? ` · ${formatDurationNs(span.durationNs)}`
                    : ""}
                </span>
              ) : null}
            </div>
          ) : null}
        </div>
        <div
          className="pr-1 text-right text-[11px] font-medium text-muted-foreground tabular-nums"
          title={
            selfNs !== undefined
              ? `self ${formatDurationNs(selfNs.toString())}`
              : undefined
          }
        >
          {formatDurationNs(span.durationNs)}
        </div>
      </button>
    )
  }

  const renderLaneHeader = (item: LaneItem, style?: CSSProperties) => (
    <div
      key={item.key}
      className={cn(
        "sticky top-0 z-20 grid w-full grid-cols-[16rem_minmax(0,1fr)_6.5rem] items-center rounded-md border border-border/70 bg-background/95 px-2 py-1 text-xs text-muted-foreground backdrop-blur",
        style && "absolute top-0 left-0"
      )}
      data-testid="trace-lane-header"
      style={style}
    >
      <div className="truncate font-medium text-foreground">{item.service}</div>
      <div className="tabular-nums">
        {item.spanCount.toLocaleString()} span
        {item.spanCount === 1 ? "" : "s"}
      </div>
      <div className="text-right tabular-nums">
        {formatDurationNs(item.durationNs)}
      </div>
    </div>
  )

  const renderVisualItem = (item: WaterfallItem, style?: CSSProperties) =>
    item.type === "lane"
      ? renderLaneHeader(item, style)
      : renderSpanRow(item.row, style)

  return (
    <div
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === "ArrowDown" || event.key === "j") {
          event.preventDefault()
          moveSelection(1)
        } else if (event.key === "ArrowUp" || event.key === "k") {
          event.preventDefault()
          moveSelection(-1)
        } else {
          interactions.handlers.onKeyDown(event)
        }
      }}
      className="outline-none"
    >
      <div className="grid grid-cols-[16rem_minmax(0,1fr)_6.5rem] items-center pb-1 text-[11px] text-muted-foreground">
        <div className="pl-1">
          {isFit ? null : (
            <button
              type="button"
              className="rounded border border-border/70 px-1.5 py-0.5 hover:bg-accent/50"
              onClick={() => dispatch({ type: "ZOOM_TO_FIT" })}
            >
              Reset zoom (0)
            </button>
          )}
        </div>
        <div className="grid grid-cols-4">
          {[0, 25, 50, 75].map((pct) => (
            <span key={pct} className="tabular-nums">
              +
              {formatDurationNs(
                Math.round(
                  (viewport.startMs +
                    ((viewport.endMs - viewport.startMs) * pct) / 100) *
                    1_000_000
                )
              )}
            </span>
          ))}
        </div>
        <div className="text-right tabular-nums">
          +{formatDurationNs(Math.round(viewport.endMs * 1_000_000))}
        </div>
      </div>

      <div
        ref={minimapRef}
        className="relative mb-2 ml-[16rem] h-6 cursor-crosshair rounded-md border border-border/70 bg-muted/30"
        aria-label="Trace minimap"
        onPointerDown={onMinimapPointerDown}
        onPointerMove={onMinimapPointerMove}
        onPointerUp={onMinimapPointerUp}
      >
        {minimapRows.map((row) => {
          const failed = row.span.statusCode === "STATUS_CODE_ERROR"
          return (
            <button
              key={`minimap-${row.span.spanId}`}
              type="button"
              tabIndex={-1}
              aria-label={`Select ${row.span.name}`}
              data-testid="trace-minimap-bar"
              onClick={() => {
                onSelect(row.span.spanId)
                scrollToSpan(row.span.spanId)
              }}
              className={cn(
                "absolute top-1 bottom-1 min-w-0.5 rounded-full",
                failed ? "bg-rose-500" : "bg-primary/50"
              )}
              style={{
                left: `${row.offsetPct}%`,
                width: `${row.widthPct}%`,
              }}
            />
          )
        })}
        {/* Dim outside-viewport regions; the rectangle is the controller. */}
        <div
          className="pointer-events-none absolute inset-y-0 left-0 bg-background/60"
          style={{ width: `${minimapViewportPct.left}%` }}
        />
        <div
          className="pointer-events-none absolute inset-y-0 right-0 bg-background/60"
          style={{
            width: `${Math.max(0, 100 - minimapViewportPct.left - minimapViewportPct.width)}%`,
          }}
        />
        <div
          data-testid="trace-minimap-viewport"
          className="absolute inset-y-0 rounded-sm border border-primary/70 bg-primary/10"
          style={{
            left: `${minimapViewportPct.left}%`,
            width: `${minimapViewportPct.width}%`,
          }}
        >
          <span className="absolute inset-y-0 left-0 w-1 cursor-ew-resize" />
          <span className="absolute inset-y-0 right-0 w-1 cursor-ew-resize" />
        </div>
      </div>

      <div className="relative">
        {/* Gesture surface over the bar column: marquee-zoom, pan, wheel. */}
        <div
          ref={barAreaRef}
          data-testid="trace-gesture-surface"
          className={cn(
            "absolute inset-y-0 right-[6.5rem] left-[16rem] z-20",
            interactions.isPanning ? "cursor-grabbing" : "cursor-crosshair"
          )}
          onPointerDown={(event) => {
            surfaceDown.current = { x: event.clientX, y: event.clientY }
            interactions.handlers.onPointerDown(event)
          }}
          onPointerMove={interactions.handlers.onPointerMove}
          onPointerUp={(event) => {
            interactions.handlers.onPointerUp(event)
            const down = surfaceDown.current
            surfaceDown.current = null
            if (!down) return
            const travel = Math.hypot(
              event.clientX - down.x,
              event.clientY - down.y
            )
            if (travel >= 4) return
            // A tap is a span click: forward it to the row underneath the
            // gesture surface.
            const surface = event.currentTarget
            surface.style.pointerEvents = "none"
            const target = document.elementFromPoint(
              event.clientX,
              event.clientY
            )
            surface.style.pointerEvents = ""
            target
              ?.closest<HTMLButtonElement>('[data-testid^="trace-row-"]')
              ?.click()
          }}
          onPointerCancel={interactions.handlers.onPointerCancel}
        >
          {interactions.marquee ? (
            <div
              data-testid="trace-marquee"
              className="absolute inset-y-0 bg-primary/15 ring-1 ring-primary/50"
              style={{
                left: Math.min(
                  interactions.marquee.startPx,
                  interactions.marquee.endPx
                ),
                width: Math.abs(
                  interactions.marquee.endPx - interactions.marquee.startPx
                ),
              }}
            />
          ) : null}
        </div>
        <div className="pointer-events-none absolute inset-y-0 right-[6.5rem] left-[16rem] z-0 grid grid-cols-4">
          {[0, 1, 2, 3].map((line) => (
            <span key={line} className="border-l border-border/50" />
          ))}
        </div>

        <div className="relative z-10 flex flex-col gap-0.5">
          <button
            type="button"
            onClick={() =>
              onSelect(selectedId === WHOLE_TRACE_ID ? null : WHOLE_TRACE_ID)
            }
            className={cn(
              "grid w-full cursor-pointer grid-cols-[16rem_minmax(0,1fr)_6.5rem] items-center rounded-md py-1.5 text-left text-sm hover:bg-accent/50",
              selectedId === WHOLE_TRACE_ID && "bg-accent/70"
            )}
          >
            <div className="flex min-w-0 items-start gap-2 pr-3 pl-1">
              <span className="flex size-5 shrink-0 items-center justify-center rounded-md bg-primary/15 text-primary">
                <IconAffiliate className="size-3" />
              </span>
              <div className="min-w-0">
                <span className="font-medium break-words">Whole trace</span>
                <div className="mt-1 flex flex-wrap gap-1">
                  {services.map((service) => (
                    <Badge key={service} variant="outline">
                      {service}
                    </Badge>
                  ))}
                </div>
              </div>
            </div>
            <div className="relative h-5">
              <div className="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-primary/70" />
            </div>
            <div className="pr-1 text-right text-[11px] font-medium tabular-nums">
              {formatDurationNs(traceWindow.durationNs.toString())}
            </div>
          </button>

          {showNoErrors ? (
            <p className="rounded-md border border-border/70 bg-muted/30 px-3 py-2 text-sm text-muted-foreground">
              No errored spans. Showing full trace.
            </p>
          ) : null}

          {shouldVirtualize ? (
            <div
              ref={rowsRef}
              className="max-h-[min(70vh,720px)] overflow-auto"
            >
              <div
                className="relative w-full"
                style={{ height: rowVirtualizer.getTotalSize() }}
              >
                {virtualItems.map((virtualItem) => {
                  const item = visualItems[virtualItem.index]
                  return item
                    ? renderVisualItem(item, {
                        height: virtualItem.size,
                        transform: `translateY(${virtualItem.start}px)`,
                      })
                    : null
                })}
              </div>
            </div>
          ) : (
            visualItems.map((item) => renderVisualItem(item))
          )}
        </div>
      </div>
    </div>
  )
}
