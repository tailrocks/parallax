import { Link } from "@tanstack/react-router"
import { IconArrowUpRight, IconRepeat } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { BackgroundCycle, Job } from "@/features/invocations/model/wire"
import { formatCount } from "@/shared/format"
import { cn } from "@/lib/utils"

export function JobsCyclesTab({ cycles, jobs }: { cycles: BackgroundCycle[]; jobs: Job[] }) {
  if (cycles.length === 0 && jobs.length === 0) {
    return (
      <EmptyState
        icon={IconRepeat}
        title="No background work"
        description="Nothing yet — this invocation has not emitted background.cycle spans or job producer/consumer pairs."
      />
    )
  }
  return (
    <div className="flex flex-col gap-4">
      {cycles.length > 0 ? <CyclesCard cycles={cycles} /> : null}
      {jobs.length > 0 ? <JobsCard jobs={jobs} /> : null}
    </div>
  )
}

function CyclesCard({ cycles }: { cycles: BackgroundCycle[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Background cycles</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-hidden rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Cycle</TableHead>
                <TableHead className="w-20 text-right">Runs</TableHead>
                <TableHead className="w-20 text-right">Errors</TableHead>
                <TableHead className="w-24 text-right">p50</TableHead>
                <TableHead className="w-24 text-right">p95</TableHead>
                <TableHead className="w-32 text-right">Last</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {cycles.map((cycle) => (
                <TableRow
                  key={cycle.name}
                  className={cn(
                    cycle.errorCount > 0 && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                  )}
                >
                  <TableCell>
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: cycle.lastTraceId }}
                      className="inline-flex items-center gap-1 font-medium hover:underline"
                    >
                      {cycle.name}
                      <IconArrowUpRight className="size-3.5" />
                    </Link>
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatCount(cycle.count)}
                  </TableCell>
                  <TableCell
                    className={cn(
                      "text-right tabular-nums",
                      cycle.errorCount > 0
                        ? "text-rose-600 dark:text-rose-400"
                        : "text-muted-foreground/40"
                    )}
                  >
                    {formatCount(cycle.errorCount)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {cycle.p50Ms != null ? `${cycle.p50Ms.toFixed(1)} ms` : "-"}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {cycle.p95Ms != null ? `${cycle.p95Ms.toFixed(1)} ms` : "-"}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    <RelativeTime nanos={cycle.lastNanos} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

function JobsCard({ jobs }: { jobs: Job[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Jobs</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-hidden rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Job</TableHead>
                <TableHead className="w-40">Type</TableHead>
                <TableHead className="w-32 text-right">Produced</TableHead>
                <TableHead>Attempts</TableHead>
                <TableHead className="w-24 text-right">Trace</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {jobs.map((job) => (
                <TableRow key={job.jobId}>
                  <TableCell>
                    <code className="block max-w-36 truncate text-xs" title={job.jobId}>
                      {job.jobId}
                    </code>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {job.jobType ?? "-"}
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {job.producedNanos != null ? <RelativeTime nanos={job.producedNanos} /> : "-"}
                  </TableCell>
                  <TableCell>
                    {job.attempts.length === 0 ? (
                      <span className="text-xs text-muted-foreground">no consumer yet</span>
                    ) : (
                      <span className="flex flex-wrap gap-1">
                        {job.attempts.map((attempt, index) => (
                          <Link
                            key={`${attempt.traceId}-${attempt.startNanos}`}
                            to="/traces/$traceId"
                            params={{ traceId: attempt.traceId }}
                          >
                            <Badge
                              variant={
                                attempt.hasError ||
                                (attempt.outcome != null && attempt.outcome !== "success")
                                  ? "rose"
                                  : "emerald"
                              }
                            >
                              #{index + 1} {attempt.outcome ?? "done"}
                            </Badge>
                          </Link>
                        ))}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: job.lastTraceId }}
                      className="inline-flex items-center gap-1 text-xs hover:underline"
                    >
                      open
                      <IconArrowUpRight className="size-3.5" />
                    </Link>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}
