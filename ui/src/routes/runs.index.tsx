import { Link, createFileRoute } from "@tanstack/react-router"
import { graphql, relativeTime } from "@/lib/api"
import type { ObservedRun, Run } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
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
      Number(BigInt(b.lastNanos) - BigInt(a.lastNanos)),
    )
    return { merged }
  },
  component: RunsPage,
})

function RunsPage() {
  const { merged } = Route.useLoaderData()
  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">Runs</h1>
      {merged.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No runs yet — wrap a command (
          <code>parallax run start -- cargo test</code>) or point any tool
          exporting a <code>parallax.run.id</code> at this server.
        </p>
      ) : (
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
                    className="underline underline-offset-4"
                  >
                    {run.runId}
                  </Link>
                </TableCell>
                <TableCell>
                  <Badge variant="outline">{run.source}</Badge>
                </TableCell>
                <TableCell className="max-w-md truncate">
                  {run.detail}
                </TableCell>
                <TableCell>
                  <Badge
                    variant={
                      run.status === "finished" ? "secondary" : "outline"
                    }
                  >
                    {run.status}
                  </Badge>
                </TableCell>
                <TableCell className="text-right tabular-nums">
                  {run.exitCode ?? "—"}
                </TableCell>
                <TableCell>{relativeTime(run.lastNanos)}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  )
}
