import { Link } from "@tanstack/react-router"

import { RelativeTime } from "@/components/console/relative-time"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { MetricKind } from "@/lib/metric-aggregation"

export interface MetricsTableRow {
  name: string
  kind: MetricKind
  unit?: string | null
  services?: string[] | null
  pointCount?: string | null
  lastDatapointNanos?: string | null
}

export function MetricsTable({ rows }: { rows: MetricsTableRow[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead className="w-32">Kind</TableHead>
          <TableHead className="w-24">Unit</TableHead>
          <TableHead>Services</TableHead>
          <TableHead className="w-28 text-right">Datapoints</TableHead>
          <TableHead className="w-36">Last seen</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((row) => (
          <TableRow key={row.name}>
            <TableCell className="font-mono text-xs">
              <Link
                to="/metrics/$metricName"
                params={{ metricName: row.name }}
                search={{ kind: row.kind }}
                className="hover:underline"
              >
                {row.name}
              </Link>
            </TableCell>
            <TableCell>
              <Badge variant="outline">{row.kind}</Badge>
            </TableCell>
            <TableCell className="text-xs text-muted-foreground">
              {row.unit ?? "—"}
            </TableCell>
            <TableCell className="text-xs text-muted-foreground">
              {row.services?.join(", ") ?? "—"}
            </TableCell>
            <TableCell className="text-right text-xs tabular-nums">
              {row.pointCount ?? "—"}
            </TableCell>
            <TableCell className="text-xs text-muted-foreground">
              {row.lastDatapointNanos ? (
                <RelativeTime nanos={row.lastDatapointNanos} />
              ) : (
                "—"
              )}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
