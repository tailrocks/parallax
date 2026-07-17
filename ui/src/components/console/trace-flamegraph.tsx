import { useMemo, useState } from "react"

import type { WaterfallSpan } from "@/components/console/trace-waterfall"
import { serviceColor } from "@/lib/colors"
import { formatDurationNs } from "@/lib/format"
import { packFlameLanes } from "@/lib/trace-tree"
import { cn } from "@/lib/utils"

const LANE_HEIGHT_PX = 30
const LABEL_MIN_WIDTH_PCT = 7

function focusedSpans(
  spans: readonly WaterfallSpan[],
  focusId: string | null
): WaterfallSpan[] {
  if (!focusId) return [...spans]
  const included = new Set([focusId])
  let changed = true
  while (changed) {
    changed = false
    for (const span of spans) {
      if (span.parentSpanId && included.has(span.parentSpanId)) {
        changed = !included.has(span.spanId) || changed
        included.add(span.spanId)
      }
    }
  }
  return spans.filter((span) => included.has(span.spanId))
}

export function TraceFlamegraph({
  spans,
  selectedId,
  onSelect,
}: {
  spans: WaterfallSpan[]
  selectedId: string | null
  onSelect: (spanId: string) => void
}) {
  const [focusId, setFocusId] = useState<string | null>(null)
  const visibleSpans = useMemo(
    () => focusedSpans(spans, focusId),
    [focusId, spans]
  )
  const layout = useMemo(() => packFlameLanes(visibleSpans), [visibleSpans])
  const depthOffsets = useMemo(() => {
    let lanes = 0
    return layout.laneCounts.map((count) => {
      const offset = lanes
      lanes += count
      return offset
    })
  }, [layout.laneCounts])
  const totalLanes = layout.laneCounts.reduce((sum, count) => sum + count, 0)

  if (spans.length === 0) {
    return (
      <div className="py-10 text-center text-sm text-muted-foreground">
        No spans yet — this trace has not emitted any span data.
      </div>
    )
  }

  return (
    <section aria-label="Trace flamegraph">
      <div className="mb-2 flex items-center justify-between gap-3">
        <p className="text-xs text-muted-foreground">
          Click to inspect. Shift-click to focus a subtree.
        </p>
        {focusId ? (
          <button
            type="button"
            className="text-xs text-primary underline-offset-4 hover:underline"
            onClick={() => setFocusId(null)}
          >
            Show whole trace
          </button>
        ) : null}
      </div>
      <div
        className="relative w-full overflow-hidden rounded-md bg-muted/40"
        style={{ height: Math.max(totalLanes, 1) * LANE_HEIGHT_PX }}
      >
        {layout.rows.map(({ span, depth, lane, offsetPct, widthPct }) => {
          const active = span.spanId === selectedId
          const top = ((depthOffsets[depth] ?? 0) + lane) * LANE_HEIGHT_PX
          return (
            <button
              key={span.spanId}
              type="button"
              className={cn(
                "absolute overflow-hidden rounded-sm px-1.5 text-left text-[11px] text-white shadow-sm outline-none",
                "focus-visible:ring-2 focus-visible:ring-ring",
                active && "ring-2 ring-ring"
              )}
              style={{
                backgroundColor: serviceColor(span.service).color,
                height: LANE_HEIGHT_PX - 2,
                left: `${offsetPct}%`,
                top,
                width: `${Math.max(widthPct, 0.2)}%`,
              }}
              title={`${span.name} · ${span.service} · ${formatDurationNs(span.durationNs)}`}
              aria-label={`${span.name}, ${span.service}, ${formatDurationNs(span.durationNs)}`}
              onClick={(event) => {
                onSelect(span.spanId)
                if (event.shiftKey) setFocusId(span.spanId)
              }}
            >
              {widthPct >= LABEL_MIN_WIDTH_PCT ? (
                <span className="block truncate">{span.name}</span>
              ) : null}
            </button>
          )
        })}
      </div>
    </section>
  )
}
