import { Link } from "@tanstack/react-router"

import { Chip } from "@/shared/console/chip"
import { TableCell } from "@/components/ui/table"
import { formatTimeInRange } from "@/shared/format"
import { type rangeLinkSearch, type ResolvedRange } from "@/domain/time-range/range"

export function LogTimeCell({
  tsNanos,
  range,
  onOpen,
}: {
  tsNanos: string
  range: ResolvedRange
  onOpen: () => void
}) {
  return (
    <TableCell className="font-mono text-xs whitespace-nowrap">
      <button
        type="button"
        className="text-left focus-visible:ring-[1.5px] focus-visible:ring-ring/50 focus-visible:outline-none"
        onClick={(event) => {
          event.stopPropagation()
          onOpen()
        }}
      >
        {formatTimeInRange(tsNanos, range)}
      </button>
    </TableCell>
  )
}

export function LogTraceCell({
  log,
  detailSearch,
}: {
  log: { traceId?: string }
  detailSearch: ReturnType<typeof rangeLinkSearch>
}) {
  return (
    <TableCell>
      {log.traceId ? (
        <Chip
          render={
            <Link
              to="/traces/$traceId"
              params={{ traceId: log.traceId }}
              search={detailSearch}
              aria-label={`Trace ${log.traceId}`}
              onClick={(event) => event.stopPropagation()}
            />
          }
          className="text-muted-foreground hover:text-foreground"
        >
          {log.traceId.slice(0, 8)}
        </Chip>
      ) : (
        <span className="text-muted-foreground">-</span>
      )}
    </TableCell>
  )
}
