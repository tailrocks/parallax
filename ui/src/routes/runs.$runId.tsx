import { Link, createFileRoute } from "@tanstack/react-router"
import { useEffect, useState } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import type { LogRecord } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"

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
      logsByRun: LogRecord[]
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
           tsNanos service severityText body traceId
         }
         bundle(runId: "${gqlString(params.runId)}") { markdown } }`
    ),
  component: RunDetailPage,
})

interface MetricPoint {
  tsNanos: string
  value: number
}

const runMetricsConfig = {
  value: { label: "value", color: "var(--chart-1)" },
} satisfies ChartConfig

/** Run-scoped process metrics: every point whose OTLP resource carried this
 * `parallax.run.id` — CPU and memory on the same timeline as the run's
 * traces and logs (cross-analytics). Hidden when the run exported none. */
function RunMetrics({
  runId,
  fromNanos,
  toNanos,
  live,
}: {
  runId: string
  fromNanos: string
  toNanos: string
  live: boolean
}) {
  const [panels, setPanels] = useState<Array<{
    title: string
    unit: string
    points: MetricPoint[]
  }> | null>(null)

  useEffect(() => {
    const fetchPanels = () => {
      // Live keeps the window's tail at "now" so new points keep arriving.
      const to = live
        ? ((BigInt(Date.now()) + 30_000n) * 1_000_000n).toString()
        : toNanos
      const args = `runId: "${gqlString(runId)}", fromNanos: "${fromNanos}", toNanos: "${to}", stepSeconds: 5`
      void graphql<
        Record<string, Array<{ points: MetricPoint[] }> | undefined>
      >(
        `{
          cpu: metricSeries(name: "process.cpu.utilization", ${args}) { points { tsNanos value } }
          memory: metricSeries(name: "process.memory.usage", ${args}) { points { tsNanos value } }
          tasks: metricSeries(name: "tokio.runtime.alive_tasks", ${args}) { points { tsNanos value } }
        }`
      ).then((data) => {
        setPanels([
          {
            title: "CPU",
            unit: "%",
            points: (data.cpu?.[0]?.points ?? []).map((p) => ({
              tsNanos: p.tsNanos,
              value: p.value * 100,
            })),
          },
          {
            title: "Memory",
            unit: "MiB",
            points: (data.memory?.[0]?.points ?? []).map((p) => ({
              tsNanos: p.tsNanos,
              value: p.value / (1024 * 1024),
            })),
          },
          {
            title: "Tokio alive tasks",
            unit: "",
            points: data.tasks?.[0]?.points ?? [],
          },
        ])
      })
    }
    fetchPanels()
    if (!live) return
    const timer = setInterval(fetchPanels, 5000)
    return () => clearInterval(timer)
  }, [runId, fromNanos, toNanos, live])

  if (!panels || panels.every((panel) => panel.points.length === 0)) {
    return null
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          Process metrics{" "}
          <span className="font-normal text-muted-foreground">
            (points tagged with this run id)
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4 md:grid-cols-3">
          {panels
            .filter((panel) => panel.points.length > 0)
            .map((panel) => (
              <div key={panel.title} className="space-y-1">
                <p className="text-xs font-medium text-muted-foreground">
                  {panel.title}
                  {panel.unit ? ` (${panel.unit})` : ""}
                </p>
                <ChartContainer
                  config={runMetricsConfig}
                  className="h-24 w-full"
                >
                  <LineChart
                    data={panel.points.map((p) => ({
                      time: new Date(
                        Number(BigInt(p.tsNanos) / 1_000_000n)
                      ).toLocaleTimeString([], {
                        minute: "2-digit",
                        second: "2-digit",
                      }),
                      value: Number(p.value.toFixed(2)),
                    }))}
                    margin={{ left: 0, right: 8, top: 4 }}
                  >
                    <CartesianGrid vertical={false} />
                    <XAxis
                      dataKey="time"
                      tickLine={false}
                      axisLine={false}
                      minTickGap={32}
                    />
                    <YAxis tickLine={false} axisLine={false} width={44} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line
                      dataKey="value"
                      stroke="var(--color-value)"
                      dot={false}
                      strokeWidth={1.5}
                    />
                  </LineChart>
                </ChartContainer>
              </div>
            ))}
        </div>
      </CardContent>
    </Card>
  )
}

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

interface LiveLog {
  tsNanos: string
  service: string
  severityText: string
  body: string
  traceId: string
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
  const [liveLogs, setLiveLogs] = useState<LiveLog[]>([])
  const [liveSpans, setLiveSpans] = useState<LiveSpan[]>([])
  const [polledRun, setPolledRun] = useState<RunRecordData | null>(null)
  const run = polledRun ?? loadedRun

  // Log tail: append newest-last (the card reads top-to-bottom).
  useEffect(() => {
    if (!live) return
    const logSource = new EventSource(
      `/v1/logs/stream?run_id=${encodeURIComponent(runId)}`
    )
    let logBuffer: LiveLog[] = []
    logSource.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) logBuffer.push(...(batch as LiveLog[]))
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
        setLiveLogs((current) => [...current, ...incoming].slice(-300))
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
              <span className="h-2 w-2 animate-pulse rounded-full bg-green-500" />
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

      {run ? (
        <RunMetrics
          runId={runId}
          fromNanos={(BigInt(run.startedAtNanos) - 30_000_000_000n).toString()}
          toNanos={(
            (run.endedAtNanos
              ? BigInt(run.endedAtNanos)
              : BigInt(Date.now()) * 1_000_000n) + 30_000_000_000n
          ).toString()}
          live={live}
        />
      ) : null}

      {live && liveSpans.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Live spans{" "}
              <span className="font-normal text-muted-foreground">
                (newest first, as they finish)
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-1 font-mono text-xs">
              {liveSpans.map((span, index) => (
                <li
                  key={`${span.spanId}-${index}`}
                  className="flex flex-wrap items-center gap-2"
                >
                  <span className="shrink-0 text-muted-foreground">
                    {relativeTime(span.tsNanos)}
                  </span>
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: span.traceId }}
                    className="underline underline-offset-4"
                  >
                    {span.name}
                  </Link>
                  <span className="text-muted-foreground">
                    {(Number(span.durationNs) / 1e6).toFixed(1)}ms
                  </span>
                  {span.statusCode === "STATUS_CODE_ERROR" ? (
                    <Badge variant="destructive">error</Badge>
                  ) : null}
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
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

      {logsByRun.length > 0 || liveLogs.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Logs{" "}
              <span className="font-normal text-muted-foreground">
                (newest last{live ? ", streaming" : ""})
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-1 font-mono text-xs">
              {[...logsByRun, ...liveLogs].slice(-500).map((log, index) => (
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
