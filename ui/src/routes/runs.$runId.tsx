import { Link, createFileRoute } from "@tanstack/react-router"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import type { LogRecord } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

interface RunIssue {
  fingerprint: string
  title: string
  status: string
  eventCount: number
}

interface RunRecordData {
  runId: string
  command: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
  errorCount: number
  traceCount: number
  issues: RunIssue[]
}

interface RunTraceSummary {
  traceId: string
  rootName: string
  service: string
  startNanos: string
  durationNs: string
  spanCount: number
  hasError: boolean
}

export const Route = createFileRoute("/runs/$runId")({
  loader: ({ params }) =>
    graphql<{
      run: RunRecordData | null
      tracesByRun: RunTraceSummary[]
      logsByRun: LogRecord[]
      bundle: { markdown: string } | null
    }>(
      `{ run(runId: "${gqlString(params.runId)}") {
           runId command status exitCode startedAtNanos
           errorCount traceCount
           issues { fingerprint title status eventCount }
         }
         tracesByRun(runId: "${gqlString(params.runId)}") {
           traceId rootName service startNanos durationNs spanCount hasError
         }
         logsByRun(runId: "${gqlString(params.runId)}", limit: 200) {
           tsNanos service severityText body traceId
         }
         bundle(runId: "${gqlString(params.runId)}") { markdown } }`
    ),
  component: RunDetailPage,
})

function RunDetailPage() {
  const { run, tracesByRun, logsByRun, bundle } = Route.useLoaderData()
  const { runId } = Route.useParams()
  const empty = !run && tracesByRun.length === 0 && logsByRun.length === 0
  return (
    <div className="space-y-4">
      <div className="space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          <h1 className="font-mono text-lg font-semibold">{runId}</h1>
          {run ? (
            <Badge
              variant={
                run.status === "running"
                  ? "default"
                  : run.status === "external"
                    ? "outline"
                    : "secondary"
              }
            >
              {run.status}
            </Badge>
          ) : null}
          {run?.exitCode != null ? (
            <Badge variant={run.exitCode === 0 ? "secondary" : "destructive"}>
              exit {run.exitCode}
            </Badge>
          ) : null}
        </div>
        <p className="text-sm text-muted-foreground">
          {run?.command ? <code className="mr-2">{run.command}</code> : null}
          {run ? `started ${relativeTime(run.startedAtNanos)} · ` : ""}
          {run ? `${run.traceCount} trace(s) · ${run.errorCount} error(s) · ` : ""}
          {logsByRun.length} log(s) · agent handoff:{" "}
          <code>parallax run bundle {runId}</code>
        </p>
      </div>

      {empty ? (
        <p className="text-sm text-muted-foreground">
          Nothing recorded under this run id yet. If the run is live, telemetry
          arrives in batches — refresh in a few seconds.
        </p>
      ) : null}

      {run && run.issues.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Issues in this run</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              {run.issues.map((issue) => (
                <li
                  key={issue.fingerprint}
                  className="flex flex-wrap items-center gap-2"
                >
                  <Link
                    to="/issues/$fingerprint"
                    params={{ fingerprint: issue.fingerprint }}
                    className="font-medium underline underline-offset-4"
                  >
                    {issue.title}
                  </Link>
                  <Badge
                    variant={
                      issue.status === "open" ? "destructive" : "secondary"
                    }
                  >
                    {issue.status}
                  </Badge>
                  <span className="text-xs text-muted-foreground">
                    {issue.eventCount} event(s)
                  </span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      ) : null}

      {tracesByRun.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Traces</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-3">
              {tracesByRun.map((trace) => (
                <li
                  key={trace.traceId}
                  className="flex flex-wrap items-center gap-2 border-b pb-3 text-sm last:border-b-0"
                >
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: trace.traceId }}
                    className="font-medium underline underline-offset-4"
                  >
                    {trace.rootName || trace.traceId}
                  </Link>
                  <Badge variant="outline">{trace.service}</Badge>
                  {trace.hasError ? (
                    <Badge variant="destructive">error</Badge>
                  ) : null}
                  <span className="text-xs text-muted-foreground">
                    {trace.spanCount} span(s) ·{" "}
                    {(Number(trace.durationNs) / 1e6).toFixed(1)}ms ·{" "}
                    {relativeTime(trace.startNanos)}
                  </span>
                  <code className="text-xs text-muted-foreground">
                    {trace.traceId}
                  </code>
                </li>
              ))}
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

      {bundle ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Evidence bundle{" "}
              <span className="font-normal text-muted-foreground">
                (what <code>parallax run bundle {runId}</code> hands the agent)
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="max-h-96 overflow-auto rounded-md bg-muted p-3 text-xs leading-relaxed whitespace-pre-wrap">
              {bundle.markdown}
            </pre>
          </CardContent>
        </Card>
      ) : null}
    </div>
  )
}
