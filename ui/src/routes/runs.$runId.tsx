import { Link, createFileRoute } from "@tanstack/react-router"
import { graphql, relativeTime } from "@/lib/api"
import type { LogRecord, Span } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

interface RunTrace {
  traceId: string
  spans: Span[]
}

export const Route = createFileRoute("/runs/$runId")({
  loader: ({ params }) =>
    graphql<{ tracesByRun: RunTrace[]; logsByRun: LogRecord[] }>(
      `{ tracesByRun(runId: "${params.runId}") {
           traceId
           spans { tsNanos service name kind statusCode durationNs spanId parentSpanId }
         }
         logsByRun(runId: "${params.runId}") {
           tsNanos service severityText body traceId
         } }`,
    ),
  component: RunDetailPage,
})

function durationMs(spans: Span[]): string {
  const max = spans.reduce(
    (acc, span) => Math.max(acc, Number(span.durationNs)),
    0,
  )
  return (max / 1e6).toFixed(1)
}

function RunDetailPage() {
  const { tracesByRun, logsByRun } = Route.useLoaderData()
  const { runId } = Route.useParams()
  const empty = tracesByRun.length === 0 && logsByRun.length === 0
  return (
    <div className="space-y-4">
      <div className="space-y-1">
        <h1 className="font-mono text-lg font-semibold">{runId}</h1>
        <p className="text-sm text-muted-foreground">
          {tracesByRun.length} trace(s) · {logsByRun.length} log(s) · agent
          handoff: <code>parallax logs --run {runId}</code>
        </p>
      </div>

      {empty ? (
        <p className="text-sm text-muted-foreground">
          Nothing recorded under this run id yet. If the run is live, telemetry
          arrives in batches — refresh in a few seconds.
        </p>
      ) : null}

      {tracesByRun.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Traces</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-3">
              {tracesByRun.map((trace) => {
                const root =
                  trace.spans.find((span) => !span.parentSpanId) ??
                  trace.spans[0]
                const failed = trace.spans.some(
                  (span) => span.statusCode === "STATUS_CODE_ERROR",
                )
                return (
                  <li
                    key={trace.traceId}
                    className="flex flex-wrap items-center gap-2 border-b pb-3 text-sm last:border-b-0"
                  >
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: trace.traceId }}
                      className="font-medium underline underline-offset-4"
                    >
                      {root?.name ?? trace.traceId}
                    </Link>
                    {failed ? <Badge variant="destructive">error</Badge> : null}
                    <span className="text-xs text-muted-foreground">
                      {trace.spans.length} span(s) · {durationMs(trace.spans)}
                      ms ·{" "}
                      {root ? relativeTime(root.tsNanos) : ""}
                    </span>
                    <code className="text-xs text-muted-foreground">
                      {trace.traceId}
                    </code>
                  </li>
                )
              })}
            </ul>
          </CardContent>
        </Card>
      ) : null}

      {logsByRun.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Logs{" "}
              <span className="font-normal text-muted-foreground">
                (newest last)
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-1 font-mono text-xs">
              {logsByRun.map((log, index) => (
                <li key={`${log.tsNanos}-${index}`} className="flex gap-2">
                  <span className="shrink-0 text-muted-foreground">
                    {relativeTime(log.tsNanos)}
                  </span>
                  <span className="shrink-0 font-semibold">
                    {log.severityText}
                  </span>
                  <span className="break-all">{log.body}</span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      ) : null}
    </div>
  )
}
