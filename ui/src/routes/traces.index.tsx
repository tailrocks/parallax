import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { graphql, relativeTime } from "@/lib/api"
import type { TraceSummary } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

export const Route = createFileRoute("/traces/")({
  loader: () =>
    graphql<{ recentTraces: TraceSummary[] }>(
      `{ recentTraces {
           traceId rootName service startNanos durationNs spanCount hasError
         } }`,
    ),
  component: TraceLookup,
})

/** Lifecycle-4 entry point: paste a trace id, or pick a recent one. */
function TraceLookup() {
  const { recentTraces } = Route.useLoaderData()
  const [traceId, setTraceId] = useState("")
  const navigate = useNavigate()
  return (
    <div className="space-y-6">
      <div className="max-w-xl space-y-4">
        <h1 className="text-lg font-semibold">Trace lookup</h1>
        <p className="text-sm text-muted-foreground">
          Paste a trace ID — from an error page, a log line, or{" "}
          <code>parallax issue context</code> — to see the full cross-service
          picture.
        </p>
        <form
          className="flex gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            const id = traceId.trim()
            if (id) {
              navigate({ to: "/traces/$traceId", params: { traceId: id } })
            }
          }}
        >
          <Input
            value={traceId}
            onChange={(event) => setTraceId(event.target.value)}
            placeholder="4bf92f3577b34da6a3ce929d0e0e4736"
            className="font-mono"
          />
          <Button type="submit">Open</Button>
        </form>
      </div>

      <div className="space-y-2">
        <h2 className="text-sm font-semibold">Latest traces</h2>
        {recentTraces.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No traces yet — point an OTLP exporter at this machine.
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Root span</TableHead>
                <TableHead className="w-32">Service</TableHead>
                <TableHead className="w-24 text-right">Spans</TableHead>
                <TableHead className="w-28 text-right">Duration</TableHead>
                <TableHead className="w-28">Started</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {recentTraces.map((trace) => (
                <TableRow key={trace.traceId}>
                  <TableCell>
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: trace.traceId }}
                      className="underline underline-offset-4"
                    >
                      {trace.rootName}
                    </Link>{" "}
                    {trace.hasError ? (
                      <Badge variant="destructive">error</Badge>
                    ) : null}
                  </TableCell>
                  <TableCell>{trace.service}</TableCell>
                  <TableCell className="text-right tabular-nums">
                    {trace.spanCount}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {(Number(trace.durationNs) / 1e6).toFixed(1)}ms
                  </TableCell>
                  <TableCell>{relativeTime(trace.startNanos)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  )
}
