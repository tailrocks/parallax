import { useEffect, useMemo, useRef } from "react"
import { IconAffiliate } from "@tabler/icons-react"

import { SpanKindChip, spanKindMeta } from "@/components/console/span-kind"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import { buildTraceTree, computeWindow } from "@/lib/trace-tree"
import type { TraceTreeSpan } from "@/lib/trace-tree"
import { formatDurationNs } from "@/lib/format"

export const WHOLE_TRACE_ID = "__whole_trace__"

export interface WaterfallSpan extends TraceTreeSpan {
  service: string
  name: string
  kind: string
  statusCode: string
  statusMessage: string
  durationNs: string
}

export function TraceWaterfall({
  spans,
  selectedId,
  onSelect,
}: {
  spans: WaterfallSpan[]
  selectedId: string | null
  onSelect: (spanId: string | null) => void
}) {
  const rows = useMemo(() => buildTraceTree(spans), [spans])
  const window = useMemo(() => computeWindow(spans), [spans])
  const ids = useMemo(
    () => [WHOLE_TRACE_ID, ...rows.map((row) => row.span.spanId)],
    [rows]
  )
  const containerRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (!selectedId && spans.length > 0) onSelect(WHOLE_TRACE_ID)
  }, [onSelect, selectedId, spans.length])

  const moveSelection = (direction: 1 | -1) => {
    const current = selectedId ? ids.indexOf(selectedId) : 0
    const next = Math.min(Math.max(current + direction, 0), ids.length - 1)
    onSelect(ids[next] ?? WHOLE_TRACE_ID)
  }

  return (
    <div
      ref={containerRef}
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
                ((window.durationNs * BigInt(pct)) / 100n).toString()
              )}
            </span>
          ))}
        </div>
        <div className="text-right tabular-nums">
          {formatDurationNs(window.durationNs.toString())}
        </div>
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
                  {Array.from(new Set(spans.map((span) => span.service))).map(
                    (service) => (
                      <Badge key={service} variant="outline">
                        {service}
                      </Badge>
                    )
                  )}
                </div>
              </div>
            </div>
            <div className="relative h-5">
              <div className="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-primary/70" />
            </div>
            <div className="pr-1 text-right text-[11px] font-medium tabular-nums">
              {formatDurationNs(window.durationNs.toString())}
            </div>
          </button>

          {rows.map(({ span, depth, offsetPct, widthPct }) => {
            const active = span.spanId === selectedId
            const failed = span.statusCode === "STATUS_CODE_ERROR"
            const meta = spanKindMeta(span.kind, span.statusCode)
            return (
              <button
                key={span.spanId}
                type="button"
                onClick={() => onSelect(active ? null : span.spanId)}
                className={cn(
                  "grid w-full cursor-pointer grid-cols-[11rem_minmax(0,1fr)_6.5rem] items-center rounded-md py-1.5 text-left text-sm hover:bg-accent/50",
                  active && "bg-accent/70"
                )}
                data-testid={`trace-row-${span.spanId}`}
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
                        "ring-2 ring-foreground/30 ring-offset-1 ring-offset-background"
                    )}
                    style={{ left: `${offsetPct}%`, width: `${widthPct}%` }}
                  />
                </div>
                <div className="pr-1 text-right text-[11px] font-medium text-muted-foreground tabular-nums">
                  {formatDurationNs(span.durationNs)}
                </div>
              </button>
            )
          })}
        </div>
      </div>
    </div>
  )
}
