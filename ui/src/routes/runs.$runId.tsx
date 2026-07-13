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
import { useLiveStream } from "@/hooks/use-live-stream"
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
import { gqlString, graphql, graphqlCached, LOG_FIELDS } from "@/lib/api"
import type { RuntimeMetric, StoryBeat } from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import { rangeLinkSearch, resolveRangeSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { usePageVisible } from "@/lib/use-visible"
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
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
}

function searchString(value: unknown) {
  if (typeof value === "string") return value
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  return undefined
}

const DAY_NS = 86_400_000_000_000n

/** Bound runtimeSnapshot lower edge: run start, or now−24h if unknown. */
export function snapshotFromNanos(
  startedAtNanos: string | null | undefined,
  nowMs = Date.now()
): string {
  if (startedAtNanos && /^\d+$/.test(startedAtNanos)) return startedAtNanos
  return (BigInt(nowMs) * 1_000_000n - DAY_NS).toString()
}

export async function loadRunDetail(runId: string, nowMs = Date.now()) {
  const escaped = gqlString(runId)
  const { run } = await graphqlCached<{ run: RunRecordData | null }>(
    `{ run(runId: "${escaped}") {
         runId command status exitCode startedAtNanos endedAtNanos
         errorCount traceCount
         issues { fingerprint title errorType status eventCount }
       } }`
  )
  const toNanos = (BigInt(nowMs) * 1_000_000n + 60_000_000_000n).toString()
  const fromNanos = snapshotFromNanos(run?.startedAtNanos, nowMs)
  const rest = await graphqlCached<{
    tracesByRun: RunTraceSummary[]
    logsByRun: LogDoc[]
    story: StoryBeat[]
    runtimeSnapshot: RuntimeMetric[]
    agentSession: AgentSessionData | null
  }>(
    `{ tracesByRun(runId: "${escaped}") {
         traceId rootName service startNanos durationNs spanCount hasError
       }
       logsByRun(runId: "${escaped}", limit: 200) {
         ${LOG_FIELDS}
       }
       story(runId: "${escaped}") {
         tsNanos lane kind title traceId spanId severity durationNs
       }
       runtimeSnapshot(runId: "${escaped}", fromNanos: "${fromNanos}", toNanos: "${toNanos}", stepSeconds: 5) {
         family metric unit points { tsNanos value }
       }
       agentSession(runId: "${escaped}") {
         rootSpanId truncated totalInputTokens totalOutputTokens errorCount
         steps {
           spanId traceId kind name startNanos durationNs isError
           genAiOperation inputTokens outputTokens
         }
       } }`
  )
  return {
    run,
    tracesByRun: rest.tracesByRun,
    logsByRun: rest.logsByRun,
    story: rest.story,
    runtimeSnapshot: rest.runtimeSnapshot,
    agentSession: rest.agentSession,
    snapshotFromNanos: fromNanos,
  }
}

export const Route = createFileRoute("/runs/$runId")({
  validateSearch: (search: Record<string, unknown>): RunDetailSearch => ({
    tab: search["tab"] === "story" ? "story" : undefined,
    range: searchString(search["range"]),
    from: searchString(search["from"]),
    to: searchString(search["to"]),
  }),
  loader: ({ params }) => loadRunDetail(params.runId),
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
    story,
    runtimeSnapshot,
    agentSession,
  } = Route.useLoaderData()
  const { runId } = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)
  const [live, setLive] = useState(false)
  const [liveLogs, setLiveLogs] = useState<LogDoc[]>([])
  const [liveSpans, setLiveSpans] = useState<LiveSpan[]>([])
  const [polledRun, setPolledRun] = useState<RunRecordData | null>(null)
  const pageVisible = usePageVisible()
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

  const logStreamUrl = live
    ? `/v1/logs/stream?run_id=${encodeURIComponent(runId)}`
    : null
  const spanStreamUrl = live
    ? `/v1/traces/stream?run_id=${encodeURIComponent(runId)}`
    : null

  const logStatus = useLiveStream<LogDoc>({
    url: logStreamUrl,
    parse: (data) => {
      const batch: unknown = JSON.parse(data)
      return Array.isArray(batch) ? (batch as LogDoc[]) : []
    },
    onBatch: (incoming) => {
      setLiveLogs((current) =>
        [...incoming.reverse(), ...current].slice(0, 300)
      )
    },
  })

  const spanStatus = useLiveStream<LiveSpan>({
    url: spanStreamUrl,
    parse: (data) => {
      const batch: unknown = JSON.parse(data)
      return Array.isArray(batch) ? (batch as LiveSpan[]) : []
    },
    onBatch: (incoming) => {
      setLiveSpans((current) =>
        [...incoming.reverse(), ...current].slice(0, 300)
      )
    },
  })

  const streamActive = logStatus === "open" || spanStatus === "open"

  useEffect(() => {
    if (!live || !pageVisible) return
    const timer = setInterval(() => {
      void graphql<{
        run: RunRecordData | null
      }>(`{ run(runId: "${gqlString(runId)}") {
             runId command status exitCode startedAtNanos endedAtNanos
             errorCount traceCount
             issues { fingerprint title errorType status eventCount }
           } }`)
        .then((data) => {
          if (data.run) setPolledRun(data.run)
        })
        // Live polling tolerates transient API failures; next interval retries.
        .catch(() => {})
    }, 10_000)
    return () => clearInterval(timer)
  }, [live, runId, pageVisible])

  return (
    <RunDetailContent
      runId={runId}
      run={run}
      traces={tracesByRun}
      logs={runLogs}
      story={story}
      runtimeSnapshot={runtimeSnapshot}
      agentSession={agentSession}
      range={range}
      activeTab={search.tab === "story" ? "story" : "overview"}
      onTab={(value) =>
        void navigate({
          search: (current) => ({
            ...current,
            tab: value === "story" ? "story" : undefined,
          }),
        })
      }
      live={live}
      liveLogs={liveLogs}
      liveSpans={liveSpans}
      streamActive={streamActive}
      onLive={() => setLive((current) => !current)}
    />
  )
}

export function RunDetailContent({
  runId,
  run,
  traces,
  logs,
  /** Optional preloaded markdown; when omitted the card loads on mount. */
  bundle = null,
  story = [],
  runtimeSnapshot,
  agentSession = null,
  range,
  activeTab = "overview",
  onTab = () => {},
  live,
  liveLogs,
  liveSpans,
  streamActive = false,
  onLive,
}: {
  runId: string
  run: RunRecordData | null
  traces: RunTraceSummary[]
  logs: LogDoc[]
  bundle?: { markdown: string } | null
  story?: StoryBeat[]
  runtimeSnapshot: RuntimeMetric[]
  agentSession?: AgentSessionData | null
  range: ResolvedRange
  activeTab?: RunDetailTab
  onTab?: (value: string) => void
  live: boolean
  liveLogs: LogDoc[]
  liveSpans: LiveSpan[]
  streamActive?: boolean
  onLive: () => void
}) {
  const empty = !run && traces.length === 0 && logs.length === 0
  const runsBack = navItem("/runs")!
  const row = run ? runRow(run, runId) : null
  const [lazyBundle, setLazyBundle] = useState<string | null>(
    bundle?.markdown ?? null
  )
  const [bundleError, setBundleError] = useState<string | null>(null)
  const [bundleLoading, setBundleLoading] = useState(false)

  useEffect(() => {
    if (bundle?.markdown) {
      setLazyBundle(bundle.markdown)
      return
    }
    let cancelled = false
    setBundleLoading(true)
    setBundleError(null)
    void graphql<{
      bundle: { markdown: string } | null
    }>(`{ bundle(runId: "${gqlString(runId)}") { markdown } }`)
      .then((data) => {
        if (cancelled) return
        setLazyBundle(data.bundle?.markdown ?? null)
      })
      .catch((error: unknown) => {
        if (cancelled) return
        setBundleError(error instanceof Error ? error.message : String(error))
      })
      .finally(() => {
        if (!cancelled) setBundleLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [runId, bundle?.markdown])

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
            {lazyBundle ? (
              <DownloadBundle runId={runId} markdown={lazyBundle} />
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
              active={streamActive}
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

          {run?.issues.length ? (
            <IssuesCard issues={run.issues} range={range} />
          ) : null}
          {traces.length ? <TracesCard traces={traces} range={range} /> : null}
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
                <LogsTable logs={logs} range={range} />
              </CardContent>
            </Card>
          ) : null}
          <BundleCard
            runId={runId}
            markdown={lazyBundle}
            loading={bundleLoading}
            error={bundleError}
          />
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

function IssuesCard({
  issues,
  range,
}: {
  issues: RunIssue[]
  range: ResolvedRange
}) {
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
                search={rangeLinkSearch(range)}
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

function TracesCard({
  traces,
  range,
}: {
  traces: RunTraceSummary[]
  range: ResolvedRange
}) {
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
                      search={rangeLinkSearch(range)}
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

function BundleCard({
  runId,
  markdown,
  loading,
  error,
}: {
  runId: string
  markdown: string | null
  loading: boolean
  error: string | null
}) {
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Evidence bundle</CardTitle>
        {markdown ? (
          <div className="flex items-center gap-1">
            <CopyButton value={markdown} />
            <DownloadBundle runId={runId} markdown={markdown} />
          </div>
        ) : null}
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="space-y-2 rounded-md border bg-muted/30 p-3">
            <div className="h-3 w-2/3 animate-pulse rounded bg-muted" />
            <div className="h-3 w-full animate-pulse rounded bg-muted" />
            <div className="h-3 w-5/6 animate-pulse rounded bg-muted" />
          </div>
        ) : error ? (
          <p className="text-sm text-destructive">{error}</p>
        ) : markdown ? (
          <ScrollFade className="max-h-96 overflow-auto rounded-md border bg-muted/30 p-3">
            <pre className="text-xs leading-relaxed whitespace-pre-wrap">
              {markdown}
            </pre>
          </ScrollFade>
        ) : (
          <p className="text-sm text-muted-foreground">
            No evidence bundle for this run.
          </p>
        )}
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
