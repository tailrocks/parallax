import { Fragment } from "react"

import { IncidentBundlePanel } from "@/features/alerts/components/incident-bundle-panel"
import type { AlertIncidentRow } from "@/features/alerts/api/alerts-gql"
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

function SeverityBadge({ severity }: { severity: string }) {
  return <Badge variant={severity === "critical" ? "destructive" : "outline"}>{severity}</Badge>
}

export function IncidentsTable({ incidents }: { incidents: AlertIncidentRow[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Rule</TableHead>
          <TableHead>Group</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Severity</TableHead>
          <TableHead className="text-right">Last value</TableHead>
          <TableHead>First triggered</TableHead>
          <TableHead>Last triggered</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {incidents.map((incident) => (
          <Fragment key={incident.id}>
            <TableRow>
              <TableCell>
                <span className="font-medium">{incident.rule?.name ?? incident.ruleId}</span>
              </TableCell>
              <TableCell className="text-muted-foreground">{incident.groupKey || "—"}</TableCell>
              <TableCell>
                <Badge variant={incident.status === "open" ? "destructive" : "outline"}>
                  {incident.status}
                </Badge>
              </TableCell>
              <TableCell>
                <SeverityBadge severity={incident.severity} />
              </TableCell>
              <TableCell className="text-right tabular-nums">{incident.lastValue ?? "—"}</TableCell>
              <TableCell className="text-xs text-muted-foreground">
                <RelativeTime nanos={incident.firstTriggeredAtNanos} />
              </TableCell>
              <TableCell className="text-xs text-muted-foreground">
                <RelativeTime nanos={incident.lastTriggeredAtNanos} />
              </TableCell>
            </TableRow>
            <TableRow>
              <TableCell colSpan={7}>
                <IncidentBundlePanel
                  markdown={incident.bundle?.markdown}
                  canonicalHash={incident.bundle?.canonicalHash}
                />
              </TableCell>
            </TableRow>
          </Fragment>
        ))}
      </TableBody>
    </Table>
  )
}
