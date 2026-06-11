import { createFileRoute } from "@tanstack/react-router"
import { graphql, relativeTime } from "@/lib/api"
import type { Run } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export const Route = createFileRoute("/runs")({
  loader: () =>
    graphql<{ runs: Run[] }>(`
      {
        runs {
          runId
          command
          status
          exitCode
          startedAtNanos
        }
      }
    `),
  component: RunsPage,
})

function RunsPage() {
  const { runs } = Route.useLoaderData()
  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">Runs</h1>
      {runs.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No runs yet — wrap a command:{" "}
          <code>parallax run start -- cargo test</code>
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Run</TableHead>
              <TableHead>Command</TableHead>
              <TableHead className="w-28">Status</TableHead>
              <TableHead className="w-20 text-right">Exit</TableHead>
              <TableHead className="w-28">Started</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {runs.map((run) => (
              <TableRow key={run.runId}>
                <TableCell className="font-mono text-xs">{run.runId}</TableCell>
                <TableCell className="max-w-md truncate">
                  {run.command ?? "—"}
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
                <TableCell>{relativeTime(run.startedAtNanos)}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </div>
  )
}
