import { useMemo } from "react"
import { Link } from "@tanstack/react-router"
import { IconActivity, IconArrowUpRight } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { HeatCell, buildHeatScale } from "@/shared/console/heat-cell"
import { RelativeTime } from "@/shared/console/relative-time"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { LiveSpan, TraceSummary } from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import { rangeLinkSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

/** Fold live finished spans into per-trace prepend rows (newest first). */
export function mergeLiveTraces(
  traces: TraceSummary[],
  liveSpans: LiveSpan[]
): TraceSummary[] {
  const known = new Set(traces.map((trace) => trace.traceId))
  const prepend = new Map<string, TraceSummary>()
  for (const span of liveSpans) {
    if (known.has(span.traceId)) continue
    const existing = prepend.get(span.traceId)
    if (existing) {
      existing.spanCount += 1
      existing.hasError ||= span.statusCode === "STATUS_CODE_ERROR"
      continue
    }
    prepend.set(span.traceId, {
      traceId: span.traceId,
      rootName: span.name,
      service: span.service,
      startNanos: span.tsNanos,
      durationNs: span.durationNs,
      spanCount: 1,
      hasError: span.statusCode === "STATUS_CODE_ERROR",
    })
  }
  return [...[...prepend.values()].reverse(), ...traces]
}

export function InvocationTracesTab({
  traces,
  liveSpans,
  live,
  range,
}: {
  traces: TraceSummary[]
  liveSpans: LiveSpan[]
  live: boolean
  range: ResolvedRange
}) {
  const rows = useMemo(
    () => mergeLiveTraces(traces, live ? liveSpans : []),
    [traces, liveSpans, live]
  )
  const scale = useMemo(
    () => buildHeatScale(rows.map((trace) => Number(trace.durationNs))),
    [rows]
  )
  if (rows.length === 0) {
    return (
      <EmptyState
        icon={IconActivity}
        title="No traces"
        description="Nothing yet — this invocation has not produced any traces."
      />
    )
  }
  return (
    <div className="overflow-hidden rounded-lg border bg-card">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Root</TableHead>
            <TableHead className="w-24 text-right">Spans</TableHead>
            <TableHead className="w-24 text-right">Errors</TableHead>
            <TableHead className="w-32 text-right">Duration</TableHead>
            <TableHead className="w-32 text-right">Start</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((trace) => (
            <TableRow
              key={trace.traceId}
              className={cn(
                trace.hasError && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
              )}
            >
              <TableCell>
                <Link
                  to="/traces/$traceId"
                  params={{ traceId: trace.traceId }}
                  search={rangeLinkSearch(range)}
                  className="inline-flex items-center gap-1 font-medium hover:underline"
                >
                  <span className="max-w-96 truncate">
                    {trace.rootName || trace.traceId}
                  </span>
                  <IconArrowUpRight className="size-3.5" />
                </Link>
                <div className="text-xs text-muted-foreground">
                  {trace.service}
                </div>
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {formatCount(trace.spanCount)}
              </TableCell>
              <TableCell className="text-right">
                {trace.hasError ? <Badge variant="rose">error</Badge> : "-"}
              </TableCell>
              <TableCell className="text-right">
                <HeatCell value={Number(trace.durationNs)} scale={scale}>
                  {formatDurationNs(trace.durationNs)}
                </HeatCell>
              </TableCell>
              <TableCell className="text-right text-muted-foreground">
                <RelativeTime nanos={trace.startNanos} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
