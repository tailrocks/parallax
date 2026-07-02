import { Link, createFileRoute } from "@tanstack/react-router"
import {
  ActivityIcon,
  CheckCircle2Icon,
  TerminalSquareIcon,
  TimerIcon,
} from "lucide-react"
import { graphql, relativeTime } from "@/lib/api"
import type { ObservedRun, Run } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { KpiCard } from "@/components/kpi-card"
import { PageHeading } from "@/components/page-heading"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

interface RunRow {
  runId: string
  source: string
  detail: string
  status: string
  exitCode: number | null
  lastNanos: string
}

export const Route = createFileRoute("/runs/")({
  loader: async () => {
    const { runs, observedRuns } = await graphql<{
      runs: Run[]
      observedRuns: ObservedRun[]
    }>(`
      {
        runs {
          runId
          command
          status
          exitCode
          startedAtNanos
        }
        observedRuns {
          runId
          service
          firstNanos
          lastNanos
          spanCount
          logCount
        }
      }
    `)
    // One list: wrapper-registered runs win on id collision; everything an
    // external tool exported under parallax.run.id still shows up.
    const rows = new Map<string, RunRow>()
    for (const run of observedRuns) {
      rows.set(run.runId, {
        runId: run.runId,
        source: run.service,
        detail: `${run.spanCount} span(s) · ${run.logCount} log(s)`,
        status: "observed",
        exitCode: null,
        lastNanos: run.lastNanos,
      })
    }
    for (const run of runs) {
      const observed = rows.get(run.runId)
      rows.set(run.runId, {
        runId: run.runId,
        source: "wrapper",
        detail: run.command ?? observed?.detail ?? "—",
        status: run.status,
        exitCode: run.exitCode,
        lastNanos: observed?.lastNanos ?? run.startedAtNanos,
      })
    }
    const merged = [...rows.values()].sort((a, b) =>
      Number(BigInt(b.lastNanos) - BigInt(a.lastNanos))
    )
    return { merged }
  },
  component: RunsPage,
})

function RunsPage() {
  const { merged } = Route.useLoaderData()
  const finished = merged.filter((run) => run.status === "finished").length
  const observed = merged.filter((run) => run.status === "observed").length
  const failed = merged.filter(
    (run) => run.exitCode && run.exitCode !== 0
  ).length
  const last = merged[0]

  return (
    <div className="grid gap-5">
      <PageHeading
        eyebrow="Execution context"
        title="Runs"
        description="CLI wrappers and externally observed run ids, unified into one context timeline."
      />
      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <KpiCard
          icon={TerminalSquareIcon}
          label="Visible runs"
          value={String(merged.length)}
          detail="merged wrapper + observed"
          tone="violet"
          bars={merged.map(() => 1)}
        />
        <KpiCard
          icon={CheckCircle2Icon}
          label="Finished"
          value={String(finished)}
          detail="wrapper reported"
          tone="green"
        />
        <KpiCard
          icon={ActivityIcon}
          label="Observed only"
          value={String(observed)}
          detail="from telemetry"
          tone="blue"
        />
        <KpiCard
          icon={TimerIcon}
          label="Latest"
          value={last ? relativeTime(last.lastNanos) : "-"}
          detail={
            failed ? `${failed} failed exit(s)` : "no failed exits loaded"
          }
          tone={failed ? "rose" : "orange"}
        />
      </section>
      {merged.length === 0 ? (
        <div className="parallax-panel p-6">
          <p className="text-sm font-medium">No runs captured yet</p>
          <p className="mt-2 max-w-xl text-sm text-muted-foreground">
            Wrap a command or point any tool exporting{" "}
            <code className="font-mono text-foreground">parallax.run.id</code>{" "}
            at this server.
          </p>
          <code className="mt-4 block w-fit rounded-xl border border-border/70 bg-background/80 px-3 py-2 font-mono text-xs text-foreground">
            parallax run start -- cargo test
          </code>
        </div>
      ) : (
        <div className="parallax-panel overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Run</TableHead>
                <TableHead>Source</TableHead>
                <TableHead>Command / activity</TableHead>
                <TableHead className="w-28">Status</TableHead>
                <TableHead className="w-20 text-right">Exit</TableHead>
                <TableHead className="w-28">Last seen</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {merged.map((run) => (
                <TableRow key={run.runId}>
                  <TableCell className="font-mono text-xs">
                    <Link
                      to="/runs/$runId"
                      params={{ runId: run.runId }}
                      className="text-foreground underline underline-offset-4"
                    >
                      {run.runId}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="rounded-full">
                      {run.source}
                    </Badge>
                  </TableCell>
                  <TableCell className="max-w-md truncate">
                    {run.detail}
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        run.status === "finished" ? "secondary" : "outline"
                      }
                      className="rounded-full"
                    >
                      {run.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {run.exitCode ?? "-"}
                  </TableCell>
                  <TableCell>{relativeTime(run.lastNanos)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}
