import { createFileRoute } from "@tanstack/react-router"
import { graphql, type LogRecord, type Span } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export const Route = createFileRoute("/traces/$traceId")({
  loader: ({ params }) =>
    graphql<{ trace: { spans: Span[] } | null; logsByTrace: LogRecord[] }>(
      `{ trace(traceId: "${params.traceId}") {
           spans { tsNanos service name kind statusCode durationNs spanId parentSpanId }
         }
         logsByTrace(traceId: "${params.traceId}") { tsNanos service severityText body } }`,
    ),
  component: TracePage,
})

/** Waterfall-lite: offset/width bars computed from span timestamps. */
function TracePage() {
  const { trace, logsByTrace } = Route.useLoaderData()
  const { traceId } = Route.useParams()
  if (!trace) {
    return <p className="text-sm text-muted-foreground">Trace not found.</p>
  }
  const spans = trace.spans
  const start = Math.min(...spans.map((s) => Number(s.tsNanos)))
  const end = Math.max(
    ...spans.map((s) => Number(s.tsNanos) + Number(s.durationNs)),
  )
  const total = Math.max(1, end - start)

  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">
        Trace <code className="text-base">{traceId}</code>
      </h1>
      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {spans.length} span(s) · {(total / 1e6).toFixed(1)}ms
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-1.5">
          {spans.map((span) => {
            const offset = ((Number(span.tsNanos) - start) / total) * 100
            const width = Math.max(
              0.5,
              (Number(span.durationNs) / total) * 100,
            )
            const failed = span.statusCode === "STATUS_CODE_ERROR"
            return (
              <div key={span.spanId} className="space-y-0.5">
                <div className="flex items-center justify-between gap-2 text-xs">
                  <span className="truncate">
                    <Badge variant="outline" className="mr-1">
                      {span.service}
                    </Badge>
                    {span.name}
                  </span>
                  <span className="shrink-0 tabular-nums text-muted-foreground">
                    {(Number(span.durationNs) / 1e6).toFixed(2)}ms
                  </span>
                </div>
                <div className="h-2 w-full rounded bg-muted">
                  <div
                    className={`h-2 rounded ${failed ? "bg-destructive" : "bg-primary"}`}
                    style={{ marginLeft: `${offset}%`, width: `${width}%` }}
                  />
                </div>
              </div>
            )
          })}
        </CardContent>
      </Card>

      {logsByTrace.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Correlated logs</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-1 font-mono text-xs">
              {logsByTrace.map((log, index) => (
                <li key={index} className="flex gap-2">
                  <span className="shrink-0 text-muted-foreground">
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
