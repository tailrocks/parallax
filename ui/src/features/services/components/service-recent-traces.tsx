import { Link } from "@tanstack/react-router"
import { IconAffiliate } from "@tabler/icons-react"
import { useMemo } from "react"

import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { RelativeTime } from "@/components/console/relative-time"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { TraceSummary } from "@/features/services/model/service-detail"
import { formatDurationNs } from "@/lib/format"
import { rangeLinkSearch, type ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

export function ServiceRecentTraces({
  traces,
  range,
}: {
  traces: readonly TraceSummary[]
  range: ResolvedRange
}) {
  const durations = traces.map((trace) => Number(trace.durationNs))
  const scale = useMemo(() => buildHeatScale(durations), [durations])
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Recent traces</CardTitle>
      </CardHeader>
      <CardContent>
        {traces.length === 0 ? (
          <EmptyState
            className="min-h-40"
            icon={IconAffiliate}
            title="No recent traces"
            description="Change the range or send spans for this service."
          />
        ) : (
          <div className="overflow-hidden rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Root</TableHead>
                  <TableHead className="w-32 text-right">Duration</TableHead>
                  <TableHead className="w-32 text-right">When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {traces.map((trace) => (
                  <TableRow
                    key={trace.traceId}
                    className={cn(
                      trace.hasError &&
                        "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                    )}
                  >
                    <TableCell>
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: trace.traceId }}
                        search={rangeLinkSearch(range)}
                        className="font-medium hover:underline"
                      >
                        {trace.rootName}
                      </Link>
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
        )}
      </CardContent>
    </Card>
  )
}
