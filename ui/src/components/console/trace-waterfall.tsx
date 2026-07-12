import type { CSSProperties, RefCallback } from "react"
import { useCallback, useEffect, useMemo, useRef } from "react"
import { IconAffiliate } from "@tabler/icons-react"
import { useVirtualizer } from "@tanstack/react-virtual"

import { SpanKindChip, spanKindMeta } from "@/components/console/span-kind"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import {
  buildTraceTree,
  computeWindow,
  errorPathSpanIds,
  groupByService,
} from "@/lib/trace-tree"
import type { OrderedTraceSpan, TraceTreeSpan } from "@/lib/trace-tree"
import { formatDurationNs } from "@/lib/format"

export const WHOLE_TRACE_ID = "__whole_trace__"
export type TraceViewMode = "tree" | "errors" | "lanes"
const MINIMAP_MAX_BARS = 2_000

export interface WaterfallSpan extends TraceTreeSpan {
  service: string
  name: string
  kind: string
  statusCode: string
  statusMessage: string
  durationNs: string
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
}: {
  spans: WaterfallSpan[]
  selectedId: string | null
  onSelect: (spanId: string | null) => void
  highlightIds?: ReadonlySet<string> | undefined
  mode?: TraceViewMode
}) {
  const allRows = useMemo(() => buildTraceTree(spans), [spans])
  const traceWindow = useMemo(() => computeWindow(spans), [spans])
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
    const { span, depth, offsetPct, widthPct } = row
    const active = span.spanId === selectedId
    const highlighted = highlightIds?.has(span.spanId) ?? false
    const failed = span.statusCode === "STATUS_CODE_ERROR"
    const meta = spanKindMeta(span.kind, span.statusCode)
    return (
      <button
        key={span.spanId}
        ref={setRowRef(span.spanId)}
        type="button"
        onClick={() => onSelect(active ? null : span.spanId)}
        className={cn(
          "grid w-full cursor-pointer grid-cols-[11rem_minmax(0,1fr)_6.5rem] items-center rounded-md border-l-2 border-transparent py-1.5 text-left text-sm hover:bg-accent/50",
          active && "bg-accent/70",
          highlighted && "border-primary bg-primary/5",
          style && "absolute top-0 left-0"
        )}
        data-testid={`trace-row-${span.spanId}`}
        style={style}
      >
        <div
          className="flex min-w-0 items-start gap-2 pr-3"
          style={{ paddingLeft: (depth + 1) * 14 + 4 }}
        >
          <SpanKindChip kind={span.kind} statusCode={span.statusCode} />
          <div className="min-w-0">
            <span className="block break-words">{span.name}</span>
            <div className="mt-1 flex flex-wrap items-center gap-1">
              <Badge variant="outline">{span.service}</Badge>
              {failed ? <Badge variant="rose">error</Badge> : null}
            </div>
          </div>
        </div>
        <div className="relative h-5">
          <div
            className={cn(
              "absolute top-1/2 h-2 -translate-y-1/2 rounded-full",
              meta.bar,
              active &&
                "ring-2 ring-foreground/30 ring-offset-1 ring-offset-background",
              highlighted &&
                "ring-2 ring-primary/60 ring-offset-1 ring-offset-background"
            )}
            style={{ left: `${offsetPct}%`, width: `${widthPct}%` }}
          />
        </div>
        <div className="pr-1 text-right text-[11px] font-medium text-muted-foreground tabular-nums">
          {formatDurationNs(span.durationNs)}
        </div>
      </button>
    )
  }

  const renderLaneHeader = (item: LaneItem, style?: CSSProperties) => (
    <div
      key={item.key}
      className={cn(
        "sticky top-0 z-20 grid w-full grid-cols-[11rem_minmax(0,1fr)_6.5rem] items-center rounded-md border border-border/70 bg-background/95 px-2 py-1 text-xs text-muted-foreground backdrop-blur",
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
        }
      }}
      className="outline-none"
    >
      <div className="grid grid-cols-[11rem_minmax(0,1fr)_6.5rem] items-center pb-1 text-[11px] text-muted-foreground">
        <div />
        <div className="grid grid-cols-4">
          {[0, 25, 50, 75].map((pct) => (
            <span key={pct} className="tabular-nums">
              +
              {formatDurationNs(
                ((traceWindow.durationNs * BigInt(pct)) / 100n).toString()
              )}
            </span>
          ))}
        </div>
        <div className="text-right tabular-nums">
          {formatDurationNs(traceWindow.durationNs.toString())}
        </div>
      </div>

      <div
        className="relative mb-2 ml-[11rem] h-6 rounded-md border border-border/70 bg-muted/30"
        aria-label="Trace minimap"
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
      </div>

      <div className="relative">
        <div className="pointer-events-none absolute inset-y-0 right-[6.5rem] left-[11rem] z-0 grid grid-cols-4">
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
              "grid w-full cursor-pointer grid-cols-[11rem_minmax(0,1fr)_6.5rem] items-center rounded-md py-1.5 text-left text-sm hover:bg-accent/50",
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
