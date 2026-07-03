import { Link, createFileRoute } from "@tanstack/react-router"
import { useEffect, useMemo, useState } from "react"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import { LiveEventStack, LiveStreamPanel } from "@/components/live-stream-panel"
import { LogsTable } from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { MetricStrip } from "@/components/metric-strip"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
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
  endedAtNanos: string | null
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
      logsByRun: LogDoc[]
      bundle: { markdown: string } | null
    }>(
      `{ run(runId: "${gqlString(params.runId)}") {
           runId command status exitCode startedAtNanos endedAtNanos
           errorCount traceCount
           issues { fingerprint title status eventCount }
         }
         tracesByRun(runId: "${gqlString(params.runId)}") {
           traceId rootName service startNanos durationNs spanCount hasError
         }
         logsByRun(runId: "${gqlString(params.runId)}", limit: 200) {
           tsNanos service severityNum severityText body traceId spanId
           runId scopeName attributes resource
         }
         bundle(runId: "${gqlString(params.runId)}") { markdown } }`
    ),
  component: RunDetailPage,
})

/** One finished span from the live feed (`/v1/traces/stream?run_id=…`). */
interface LiveSpan {
  tsNanos: string
  service: string
  traceId: string
  spanId: string
  name: string
  statusCode: string
  durationNs: string
}

function RunDetailPage() {
  const {
    run: loadedRun,
    tracesByRun,
    logsByRun,
    bundle,
  } = Route.useLoaderData()
  const { runId } = Route.useParams()
  // Live mode: explicit, never default (a tail costs subscriptions). It
  // streams this run's new logs and finished spans over SSE, repolls the
  // metrics card, and refreshes the run record — the observation entrance
  // for "is my run doing the right thing, right now".
  const [live, setLive] = useState(false)
  const [liveLogs, setLiveLogs] = useState<LogDoc[]>([])
  const [liveSpans, setLiveSpans] = useState<LiveSpan[]>([])
  const [polledRun, setPolledRun] = useState<RunRecordData | null>(null)
  const run = polledRun ?? loadedRun

  // Loaded + live logs as one newest-first list for the shared logs table.
  const runLogs = useMemo(
    () =>
      [...logsByRun, ...liveLogs]
        .sort((a, b) =>
          BigInt(a.tsNanos) < BigInt(b.tsNanos)
            ? 1
            : BigInt(a.tsNanos) > BigInt(b.tsNanos)
              ? -1
              : 0
        )
        .slice(0, 500),
    [logsByRun, liveLogs]
  )

  // Log tail: newest first (every run-page surface reads newest-on-top).
  useEffect(() => {
    if (!live) return
    const logSource = new EventSource(
      `/v1/logs/stream?run_id=${encodeURIComponent(runId)}`
    )
    let logBuffer: LogDoc[] = []
    logSource.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) logBuffer.push(...(batch as LogDoc[]))
      } catch {
        // skip malformed frames
      }
    }
    const spanSource = new EventSource(
      `/v1/traces/stream?run_id=${encodeURIComponent(runId)}`
    )
    let spanBuffer: LiveSpan[] = []
    spanSource.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) spanBuffer.push(...(batch as LiveSpan[]))
      } catch {
        // skip malformed frames
      }
    }
    const flush = setInterval(() => {
      if (logBuffer.length > 0) {
        const incoming = logBuffer
        logBuffer = []
        setLiveLogs((current) =>
          [...incoming.reverse(), ...current].slice(0, 300)
        )
      }
      if (spanBuffer.length > 0) {
        const incoming = spanBuffer
        spanBuffer = []
        setLiveSpans((current) =>
          [...incoming.reverse(), ...current].slice(0, 300)
        )
      }
    }, 250)
    return () => {
      logSource.close()
      spanSource.close()
      clearInterval(flush)
    }
  }, [live, runId])

  // Run record poll: status flips running → finished, counts move.
  useEffect(() => {
    if (!live) return
    const timer = setInterval(() => {
      void graphql<{ run: RunRecordData | null }>(
        `{ run(runId: "${gqlString(runId)}") {
             runId command status exitCode startedAtNanos endedAtNanos
             errorCount traceCount
             issues { fingerprint title status eventCount }
           } }`
      ).then((data) => {
        if (data.run) setPolledRun(data.run)
      })
    }, 10_000)
    return () => clearInterval(timer)
  }, [live, runId])

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
          <Button
            size="sm"
            variant={live ? "destructive" : "default"}
            onClick={() => setLive((current) => !current)}
          >
            {live ? "Stop live" : "Go live"}
          </Button>
          {live ? (
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <span className="size-2 animate-pulse rounded-full bg-(--brand-green)" />
              streaming logs + spans · metrics every 5s
            </span>
          ) : null}
        </div>
        <p className="text-sm text-muted-foreground">
          {run?.command ? <code className="mr-2">{run.command}</code> : null}
          {run ? `started ${relativeTime(run.startedAtNanos)} · ` : ""}
          {run
            ? `${run.traceCount} trace(s) · ${run.errorCount} error(s) · `
            : ""}
          {logsByRun.length + liveLogs.length} log(s) · agent handoff:{" "}
          <code>parallax run bundle {runId}</code>
        </p>
      </div>

      {empty ? (
        <p className="text-sm text-muted-foreground">
          Nothing recorded under this run id yet. If the run is live, telemetry
          arrives in batches — refresh in a few seconds, or press Go live to
          stream it as it lands.
        </p>
      ) : null}

      {live ? (
        <LiveStreamPanel
          title="Run observation stream"
          description="Streaming this run's logs and finished spans while the metrics window follows now."
          count={liveLogs.length + liveSpans.length}
          endpoint={`/v1/*/stream?run_id=${runId}`}
          active
        >
          <LiveEventStack
            items={[
              ...liveSpans.map((span) => ({
                id: `span-${span.spanId}-${span.tsNanos}`,
                title: span.name,
                meta: `${relativeTime(span.tsNanos)} · span · ${(
                  Number(span.durationNs) / 1e6
                ).toFixed(1)}ms`,
                status:
                  span.statusCode === "STATUS_CODE_ERROR"
                    ? ("error" as const)
                    : ("ok" as const),
                detail: `trace ${span.traceId.slice(0, 16)}`,
              })),
              ...liveLogs.map((log) => ({
                id: `log-${log.tsNanos}-${log.spanId}`,
                title: log.body,
                meta: `${relativeTime(log.tsNanos)} · log · ${log.severityText}`,
                status:
                  log.severityNum >= 17 ? ("error" as const) : ("ok" as const),
                detail: log.traceId,
              })),
            ].sort((a, b) => a.id.localeCompare(b.id))}
          />
        </LiveStreamPanel>
      ) : null}

      {run ? (
        <MetricStrip
          title="Process metrics"
          runId={runId}
          fromNanos={(BigInt(run.startedAtNanos) - 30_000_000_000n).toString()}
          toNanos={(
            (run.endedAtNanos
              ? BigInt(run.endedAtNanos)
              : BigInt(Date.now()) * 1_000_000n) + 30_000_000_000n
          ).toString()}
          stepSeconds={5}
          live={live}
        />
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

      {runLogs.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Logs{" "}
              <span className="font-normal text-muted-foreground">
                (newest first{live ? ", streaming" : ""})
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <LogsTable logs={runLogs} />
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
