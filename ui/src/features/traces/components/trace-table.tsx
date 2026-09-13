import { useMemo } from "react"
import { Badge } from "@/components/ui/badge"
import { ServiceDot } from "@/shared/console/service-dot"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { SortableHead } from "@/shared/console/data-table"
import { HeatCell, buildHeatScale } from "@/shared/console/heat-cell"
import { RelativeTime } from "@/shared/console/relative-time"
import { formatDurationNs, formatTimeInRange } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"
import type { TraceSummary } from "@/features/traces/model/wire"
import { cn } from "@/lib/utils"
import { rowKeyboardAttrs, useRowKeyboardNav } from "@/lib/row-keyboard-nav"

export function TraceTable({
  rows,
  durationValues,
  range,
  sort,
  onSort,
  onOpen,
}: {
  rows: TraceSummary[]
  durationValues: number[]
  range: ResolvedRange
  sort: string | undefined
  onSort: (next: string | undefined) => void
  onOpen: (traceId: string) => void
}) {
  const durationScale = useMemo(() => buildHeatScale(durationValues), [durationValues])
  const activeRow = useRowKeyboardNav({
    scope: "traces",
    count: rows.length,
    onOpen: (index) => {
      const trace = rows[index]
      if (trace) onOpen(trace.traceId)
    },
  })
  return (
    <Table density="compact" className="table-fixed">
      <TableHeader>
        <TableRow>
          <TableHead>Trace</TableHead>
          <TableHead className="w-28 text-right">
            <SortableHead sort={sort ?? ""} sortKey="spans" onSort={onSort}>
              Spans
            </SortableHead>
          </TableHead>
          <TableHead className="w-32 text-right">
            <SortableHead sort={sort ?? ""} sortKey="duration" onSort={onSort}>
              Duration
            </SortableHead>
          </TableHead>
          <TableHead className="w-32 text-right">
            <SortableHead sort={sort ?? ""} sortKey="when" onSort={onSort}>
              When
            </SortableHead>
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((trace, index) => (
          <TableRow
            key={`${trace.traceId}-${trace.startNanos}`}
            interactive
            {...rowKeyboardAttrs("traces", index)}
            onClick={() => onOpen(trace.traceId)}
            className={cn(
              trace.hasError && "shadow-[inset_1px_0_0_0_var(--color-rose-500)]",
              activeRow === index && "bg-accent/60"
            )}
          >
            <TableCell>
              <div className="flex min-w-0 flex-col gap-1">
                <div className="flex min-w-0 items-center gap-2">
                  <span className="truncate font-medium">{trace.rootName}</span>
                  {trace.hasError ? <Badge variant="rose">errors</Badge> : null}
                </div>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span className="inline-flex items-center gap-1.5">
                    <ServiceDot name={trace.service || "unknown"} />
                    <Badge variant="outline">{trace.service || "unknown"}</Badge>
                  </span>
                  <span className="font-mono">{trace.traceId.slice(0, 16)}</span>
                </div>
              </div>
            </TableCell>
            <TableCell className="text-right tabular-nums">{trace.spanCount}</TableCell>
            <TableCell className="text-right tabular-nums">
              <HeatCell value={Number(trace.durationNs)} scale={durationScale}>
                {formatDurationNs(trace.durationNs)}
              </HeatCell>
            </TableCell>
            <TableCell className="text-right text-muted-foreground tabular-nums">
              <span title={formatTimeInRange(trace.startNanos, range)}>
                <RelativeTime nanos={trace.startNanos} />
              </span>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
