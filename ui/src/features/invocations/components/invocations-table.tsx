import { Link } from "@tanstack/react-router"

import { CopyButton } from "@/shared/console/copy-button"
import { RelativeTime } from "@/shared/console/relative-time"
import { ServiceDot } from "@/shared/console/service-dot"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { formatCount, formatDurationNs } from "@/shared/format"
import {
  appModeLabel,
  invocationDurationNs,
  invocationStatus,
} from "@/features/invocations/model/invocation"
import type { InvocationRow } from "@/features/invocations/model/invocation"
import { cn } from "@/lib/utils"

import { InvocationStatusBadge, OutcomeChip } from "./invocation-status-badge"

export function InvocationsTable({
  rows,
  detailSearch,
  onOpen,
}: {
  rows: InvocationRow[]
  detailSearch: Record<string, string>
  onOpen: (invocationId: string) => void
}) {
  return (
    <div className="overflow-x-auto rounded-lg border bg-card">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Invocation</TableHead>
            <TableHead>Command</TableHead>
            <TableHead className="w-28">Mode</TableHead>
            <TableHead className="w-40">Service</TableHead>
            <TableHead className="w-28">Status</TableHead>
            <TableHead className="w-28">Outcome</TableHead>
            <TableHead className="w-20 text-right">Traces</TableHead>
            <TableHead className="w-20 text-right">Errors</TableHead>
            <TableHead className="w-24 text-right">Sessions</TableHead>
            <TableHead className="w-28 text-right">Duration</TableHead>
            <TableHead className="w-32 text-right">Last seen</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((row) => (
            <InvocationTableRow
              key={row.invocationId}
              row={row}
              detailSearch={detailSearch}
              onOpen={onOpen}
            />
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function InvocationTableRow({
  row,
  detailSearch,
  onOpen,
}: {
  row: InvocationRow
  detailSearch: Record<string, string>
  onOpen: (invocationId: string) => void
}) {
  const status = invocationStatus(row)
  const errors = row.errorCount ?? 0
  const duration = invocationDurationNs(row, status)
  return (
    <TableRow
      className={cn("cursor-pointer", errors > 0 && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]")}
      onClick={() => onOpen(row.invocationId)}
    >
      <TableCell>
        <div className="flex min-w-0 items-center gap-2">
          <Link
            to="/invocations/$invocationId"
            params={{ invocationId: row.invocationId }}
            search={detailSearch}
            className="min-w-0 hover:underline"
            onClick={(event) => event.stopPropagation()}
          >
            <code className="block max-w-44 truncate text-xs" title={row.invocationId}>
              {row.invocationId}
            </code>
          </Link>
          <CopyButton value={row.invocationId} />
          <Badge variant="secondary">{row.source}</Badge>
        </div>
      </TableCell>
      <TableCell
        className="max-w-md truncate font-mono text-xs text-muted-foreground"
        title={row.command ?? undefined}
      >
        {row.command ?? <span className="italic">external telemetry</span>}
      </TableCell>
      <TableCell>
        {row.appMode ? (
          <Badge variant="outline">{appModeLabel(row.appMode)}</Badge>
        ) : (
          <span className="text-muted-foreground/40">-</span>
        )}
      </TableCell>
      <TableCell
        className="max-w-40 truncate text-xs text-muted-foreground"
        title={row.service ?? undefined}
      >
        {row.service ? (
          <span className="inline-flex min-w-0 items-center gap-1.5">
            <ServiceDot name={row.service} />
            <span className="truncate">{row.service}</span>
          </span>
        ) : (
          "-"
        )}
      </TableCell>
      <TableCell>
        <InvocationStatusBadge status={status} exitCode={row.exitCode} />
      </TableCell>
      <TableCell>
        <OutcomeChip outcome={row.outcome} />
      </TableCell>
      <TableCell className="text-right tabular-nums">
        {row.traceCount == null ? "-" : formatCount(row.traceCount)}
      </TableCell>
      <TableCell
        className={cn(
          "text-right tabular-nums",
          errors > 0 ? "text-rose-600 dark:text-rose-400" : "text-muted-foreground/40"
        )}
      >
        {row.errorCount == null ? "-" : formatCount(errors)}
      </TableCell>
      <TableCell className="text-right tabular-nums">
        {row.sessionCount == null || row.sessionCount === 0 ? "-" : formatCount(row.sessionCount)}
      </TableCell>
      <TableCell className="text-right tabular-nums">
        {status === "running" ? "..." : duration ? formatDurationNs(duration) : "-"}
      </TableCell>
      <TableCell className="text-right text-muted-foreground">
        <RelativeTime nanos={row.lastNanos} />
      </TableCell>
    </TableRow>
  )
}
