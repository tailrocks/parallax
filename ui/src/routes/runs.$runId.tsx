import { useEffect, useMemo, useState } from "react"
import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  IconActivity,
  IconAlertTriangleFilled,
  IconArrowUpRight,
  IconDownload,
  IconPlayerPause,
  IconPlayerPlay,
  IconTerminal2,
  IconClock,
} from "@tabler/icons-react"

import { CopyButton } from "@/components/console/copy-button"
import { AgentSessionCard } from "@/components/console/agent-session"
import type { AgentSessionData } from "@/components/console/agent-session"
import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { PinButton } from "@/components/console/pin-button"
import { RelativeTime } from "@/components/console/relative-time"
import { ScrollFade } from "@/components/console/scroll-fade"
import { StatCard } from "@/components/console/stat-card"
import { StoryTimeline } from "@/components/console/story-timeline"
import { LiveEventStack, LiveStreamPanel } from "@/components/live-stream-panel"
import { LogsTable } from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { MetricStrip } from "@/components/metric-strip"
import { navItem } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
import { RuntimeSnapshotCard } from "@/components/runtime-snapshot"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { gqlString, graphql } from "@/lib/api"
import type { RuntimeMetric, StoryBeat } from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import { cn } from "@/lib/utils"
import { RunStatusBadge, durationNs } from "@/routes/runs.index"
import type { RunRow } from "@/routes/runs.index"

interface RunIssue {
  fingerprint: string
  title: string
  status: string
  eventCount: number
  errorType?: string | null
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

interface LiveSpan {
  tsNanos: string
  service: string
  traceId: string
  spanId: string
  name: string
  statusCode: string
  durationNs: string
}

type RunDetailTab = "overview" | "story"

interface RunDetailSearch {
  tab?: RunDetailTab | undefined
}

export const Route = createFileRoute("/runs/$runId")({
  validateSearch: (search: Record<string, unknown>): RunDetailSearch => ({
    tab: search.tab === "story" ? "story" : undefined,
  }),
  loader: ({ params }) => {
    const toNanos = (
      BigInt(Date.now()) * 1_000_000n +
      60_000_000_000n
    ).toString()
    return graphql<{
      run: RunRecordData | null
      tracesByRun: RunTraceSummary[]
      logsByRun: LogDoc[]
      bundle: { markdown: string } | null
      story: StoryBeat[]
      runtimeSnapshot: RuntimeMetric[]
      agentSession: AgentSessionData | null
    }>(
      `{ run(runId: "${gqlString(params.runId)}") {
           runId command status exitCode startedAtNanos endedAtNanos
           errorCount traceCount
           issues { fingerprint title errorType status eventCount }
         }
         tracesByRun(runId: "${gqlString(params.runId)}") {
           traceId rootName service startNanos durationNs spanCount hasError
         }
         logsByRun(runId: "${gqlString(params.runId)}", limit: 200) {
           tsNanos eventName observedTsNanos service severityNum severityText body traceId spanId
           runId scopeName attributes resource
         }
         story(runId: "${gqlString(params.runId)}") {
           tsNanos lane kind title traceId spanId severity durationNs
         }
         runtimeSnapshot(runId: "${gqlString(params.runId)}", fromNanos: "0", toNanos: "${toNanos}", stepSeconds: 5) {
           family metric unit points { tsNanos value }
         }
         agentSession(runId: "${gqlString(params.runId)}") {
           rootSpanId truncated totalInputTokens totalOutputTokens errorCount
           steps {
             spanId traceId kind name startNanos durationNs isError
             genAiOperation inputTokens outputTokens
           }
         }
         bundle(runId: "${gqlString(params.runId)}") { markdown } }`
    )
  },
  component: RunDetailPage,
})

function runRow(run: RunRecordData, runId: string): RunRow {
  return {
    runId,
    source: "cli",
    command: run.command,
    service: null,
    status: run.status === "running" ? "running" : "finished",
    exitCode: run.exitCode,
    startedAtNanos: run.startedAtNanos,
    endedAtNanos: run.endedAtNanos,
    lastNanos: run.endedAtNanos ?? run.startedAtNanos,
    errorCount: run.errorCount,
    traceCount: run.traceCount,
    spanCount: 0,
    logCount: 0,
  }
}

function RunDetailPage() {
  const {
    run: loadedRun,
    tracesByRun,
    logsByRun,
    bundle,
    story,
    runtimeSnapshot,
    agentSession,
  } = Route.useLoaderData()
  const { runId } = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const [live, setLive] = useState(false)
  const [liveLogs, setLiveLogs] = useState<LogDoc[]>([])
  const [liveSpans, setLiveSpans] = useState<LiveSpan[]>([])
  const [polledRun, setPolledRun] = useState<RunRecordData | null>(null)
  const run = polledRun ?? loadedRun

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

  useEffect(() => {
    if (!live) return
    const timer = setInterval(() => {
      void graphql<{ run: RunRecordData | null }>(
        `{ run(runId: "${gqlString(runId)}") {
             runId command status exitCode startedAtNanos endedAtNanos
             errorCount traceCount
             issues { fingerprint title errorType status eventCount }
           } }`
      )
        .then((data) => {
          if (data.run) setPolledRun(data.run)
        })
        // Live polling tolerates transient API failures; next interval retries.
        .catch(() => {})
    }, 10_000)
    return () => clearInterval(timer)
  }, [live, runId])

  return (
    <RunDetailContent
      runId={runId}
      run={run}
      traces={tracesByRun}
      logs={runLogs}
      bundle={bundle}
      story={story}
      runtimeSnapshot={runtimeSnapshot}
      agentSession={agentSession}
      activeTab={search.tab === "story" ? "story" : "overview"}
      onTab={(value) =>
        navigate({
          search: (current) => ({
            ...current,
            tab: value === "story" ? "story" : undefined,
          }),
        })
      }
      live={live}
      liveLogs={liveLogs}
      liveSpans={liveSpans}
      onLive={() => setLive((current) => !current)}
    />
  )
}

export function RunDetailContent({
  runId,
  run,
  traces,
  logs,
  bundle,
  story = [],
  runtimeSnapshot,
  agentSession = null,
  activeTab = "overview",
  onTab = () => {},
  live,
  liveLogs,
  liveSpans,
  onLive,
}: {
  runId: string
  run: RunRecordData | null
  traces: RunTraceSummary[]
  logs: LogDoc[]
  bundle: { markdown: string } | null
  story?: StoryBeat[]
  runtimeSnapshot: RuntimeMetric[]
  agentSession?: AgentSessionData | null
  activeTab?: RunDetailTab
  onTab?: (value: string) => void
  live: boolean
  liveLogs: LogDoc[]
  liveSpans: LiveSpan[]
  onLive: () => void
}) {
  const empty = !run && traces.length === 0 && logs.length === 0
  const runsBack = navItem("/runs")!
  const row = run ? runRow(run, runId) : null

  if (empty) {
    return (
      <EmptyState
        icon={IconTerminal2}
        title="Run not found"
        description="No registered run, traces, or logs exist for this run id yet."
      />
    )
  }

  return (
    <div className="space-y-4">
      <PageHeader
        back={runsBack}
        title={runId}
        titleTrailing={<CopyButton value={runId} />}
        description={
          run?.command ? <code>{run.command}</code> : "Observed telemetry run"
        }
        actions={
          <>
            <PinButton kind="run" label={runId} />
            <Button
              size="sm"
              variant={live ? "secondary" : "outline"}
              onClick={onLive}
            >
              {live ? <IconPlayerPause /> : <IconPlayerPlay />}
              {live ? "Following" : "Follow live"}
              {live ? (
                <span className="size-1.5 animate-pulse rounded-full bg-emerald-500" />
              ) : null}
            </Button>
            {bundle ? (
              <DownloadBundle runId={runId} markdown={bundle.markdown} />
            ) : null}
          </>
        }
      />

      {run && row ? <RunStats run={run} row={row} /> : null}

      <Tabs value={activeTab} onValueChange={onTab}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="story">Story</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" className="flex flex-col gap-4">
          {live ? (
            <LiveStreamPanel
              title="Run observation stream"
              description="Streaming this run's logs and finished spans while metrics follow now."
              count={liveLogs.length + liveSpans.length}
              endpoint={`/v1/*/stream?run_id=${runId}`}
              active
            >
              <LiveEventStack
                items={[
                  ...liveSpans.map((span) => ({
                    id: `span-${span.spanId}-${span.tsNanos}`,
                    title: span.name,
                    meta: `${formatDurationNs(span.durationNs)} · span · ${span.service}`,
                    status:
                      span.statusCode === "STATUS_CODE_ERROR"
                        ? ("error" as const)
                        : ("ok" as const),
                    detail: `trace ${span.traceId.slice(0, 16)}`,
                  })),
                  ...liveLogs.map((log) => ({
                    id: `log-${log.tsNanos}-${log.spanId}`,
                    title: log.body,
                    meta: `log · ${log.severityText} · ${log.service}`,
                    status:
                      log.severityNum >= 17
                        ? ("error" as const)
                        : ("ok" as const),
                    detail: log.traceId,
                  })),
                ].sort((a, b) => a.id.localeCompare(b.id))}
              />
            </LiveStreamPanel>
          ) : null}

          {agentSession ? <AgentSessionCard session={agentSession} /> : null}

          {run ? (
            <MetricStrip
              title="Process metrics"
              runId={runId}
              fromNanos={(
                BigInt(run.startedAtNanos) - 30_000_000_000n
              ).toString()}
              toNanos={(
                (run.endedAtNanos
                  ? BigInt(run.endedAtNanos)
                  : BigInt(Date.now()) * 1_000_000n) + 30_000_000_000n
              ).toString()}
              stepSeconds={5}
              live={live}
            />
          ) : null}
          <RuntimeSnapshotCard metrics={runtimeSnapshot} />

          {run?.issues.length ? <IssuesCard issues={run.issues} /> : null}
          {traces.length ? <TracesCard traces={traces} /> : null}
          {logs.length ? (
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
                <LogsTable logs={logs} />
              </CardContent>
            </Card>
          ) : null}
          {bundle ? (
            <BundleCard runId={runId} markdown={bundle.markdown} />
          ) : null}
        </TabsContent>
        <TabsContent value="story">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Story</CardTitle>
            </CardHeader>
            <CardContent>
              <StoryTimeline beats={story} />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}

function RunStats({ run, row }: { run: RunRecordData; row: RunRow }) {
  const duration = durationNs(row)
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <StatCard label="Status" value={<RunStatusBadge row={row} />} />
      <StatCard
        icon={IconAlertTriangleFilled}
        iconClassName="text-rose-500"
        label="Errors"
        value={
          <span
            className={cn(
              run.errorCount > 0
                ? "text-rose-600 dark:text-rose-400"
                : undefined
            )}
          >
            {formatCount(run.errorCount)}
          </span>
        }
      />
      <StatCard
        icon={IconActivity}
        label="Traces"
        value={formatCount(run.traceCount)}
      />
      <StatCard
        icon={IconClock}
        label="Duration"
        value={
          row.status === "running"
            ? "..."
            : duration
              ? formatDurationNs(duration)
              : "-"
        }
      />
    </div>
  )
}

function IssuesCard({ issues }: { issues: RunIssue[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Issues in this run</CardTitle>
      </CardHeader>
      <CardContent>
        <ul className="space-y-2 text-sm">
          {issues.map((issue) => (
            <li
              key={issue.fingerprint}
              className="grid gap-2 rounded-lg border bg-muted/20 px-3 py-2 md:grid-cols-[minmax(0,1fr)_auto]"
            >
              <Link
                to="/issues/$fingerprint"
                params={{ fingerprint: issue.fingerprint }}
                className="min-w-0 truncate font-medium hover:underline"
              >
                {issue.errorType ? `${issue.errorType}: ` : ""}
                {issue.title}
              </Link>
              <span className="flex items-center gap-2 text-xs text-muted-foreground">
                <Badge variant={issue.status === "open" ? "rose" : "emerald"}>
                  {issue.status}
                </Badge>
                {formatCount(issue.eventCount)} events
              </span>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}

function TracesCard({ traces }: { traces: RunTraceSummary[] }) {
  const durations = traces.map((trace) => Number(trace.durationNs))
  const scale = useMemo(() => buildHeatScale(durations), [durations])
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Traces</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="overflow-hidden rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Root</TableHead>
                <TableHead className="w-24 text-right">Spans</TableHead>
                <TableHead className="w-32 text-right">Duration</TableHead>
                <TableHead className="w-32 text-right">When</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {traces.map((trace) => (
                <TableRow
                  key={trace.traceId}
                  className={cn(
                    trace.hasError &&
                      "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                  )}
                >
                  <TableCell>
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: trace.traceId }}
                      className="inline-flex items-center gap-1 font-medium hover:underline"
                    >
                      {trace.rootName || trace.traceId}
                      <IconArrowUpRight className="size-3.5" />
                    </Link>
                    <div className="text-xs text-muted-foreground">
                      {trace.service}
                    </div>
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {formatCount(trace.spanCount)}
                  </TableCell>
                  <TableCell className="text-right">
                    <HeatCell value={Number(trace.durationNs)} scale={scale}>
                      {formatDurationNs(trace.durationNs)}
                    </HeatCell>
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    <RelativeTime nanos={trace.startNanos} />
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

function BundleCard({ runId, markdown }: { runId: string; markdown: string }) {
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Evidence bundle</CardTitle>
        <div className="flex items-center gap-1">
          <CopyButton value={markdown} />
          <DownloadBundle runId={runId} markdown={markdown} />
        </div>
      </CardHeader>
      <CardContent>
        <ScrollFade className="max-h-96 overflow-auto rounded-md border bg-muted/30 p-3">
          <pre className="text-xs leading-relaxed whitespace-pre-wrap">
            {markdown}
          </pre>
        </ScrollFade>
      </CardContent>
    </Card>
  )
}

function DownloadBundle({
  runId,
  markdown,
}: {
  runId: string
  markdown: string
}) {
  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      onClick={() => {
        const blob = new Blob([markdown], {
          type: "text/markdown;charset=utf-8",
        })
        const url = URL.createObjectURL(blob)
        const anchor = document.createElement("a")
        anchor.href = url
        anchor.download = `parallax-bundle-${runId}.md`
        anchor.click()
        URL.revokeObjectURL(url)
      }}
    >
      <IconDownload />
      Download
    </Button>
  )
}
