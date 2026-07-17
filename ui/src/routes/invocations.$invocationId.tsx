import { useEffect, useMemo, useState } from "react"
import { createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  IconActivity,
  IconAlertTriangleFilled,
  IconClock,
  IconPlayerPause,
  IconPlayerPlay,
  IconTerminal2,
  IconUsers,
} from "@tabler/icons-react"

import { CopyButton } from "@/components/console/copy-button"
import type { AgentSessionData } from "@/components/console/agent-session"
import { EmptyState } from "@/components/console/empty-state"
import { InvocationErrorsTab } from "@/components/console/invocations/invocation-errors-tab"
import type { InvocationIssue } from "@/components/console/invocations/invocation-errors-tab"
import {
  InvocationStatusBadge,
  OutcomeChip,
} from "@/components/console/invocations/invocation-status-badge"
import { InvocationTracesTab } from "@/components/console/invocations/invocation-traces-tab"
import { JobsCyclesTab } from "@/components/console/invocations/jobs-cycles-tab"
import { SessionsTab } from "@/components/console/invocations/sessions-tab"
import type { JourneyError } from "@/components/console/invocations/session-journey"
import { PinButton } from "@/components/console/pin-button"
import { StatCard } from "@/components/console/stat-card"
import { StoryTimeline } from "@/components/console/story-timeline"
import { useLiveStream } from "@/hooks/use-live-stream"
import { LogsTable } from "@/components/logs-table"
import type { LogDoc } from "@/components/logs-table"
import { MetricStrip } from "@/components/metric-strip"
import { navItem } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { gqlString, graphql, graphqlCached, LOG_FIELDS } from "@/lib/api"
import type {
  BackgroundCycle,
  Conversation,
  Job,
  LiveSpan,
  ScreenVisit,
  Session,
  StoryBeat,
  TraceSummary,
  UiAction,
} from "@/lib/api"
import { formatCount, formatDurationNs } from "@/lib/format"
import {
  appModeLabel,
  invocationDurationNs,
  invocationStatus,
} from "@/lib/invocation"
import { rangeSearchSchema, resolveRangeSearch } from "@/lib/range"
import { usePageVisible } from "@/lib/use-visible"
import { cn } from "@/lib/utils"
import { z } from "zod"

const TABS = [
  "overview",
  "traces",
  "logs",
  "errors",
  "sessions",
  "jobs",
] as const
type HubTab = (typeof TABS)[number]

interface InvocationRecordData {
  invocationId: string
  command: string | null
  appMode: string | null
  outcome: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  errorCount: number
  traceCount: number
  sessionCount: number
  issues: Array<
    InvocationIssue & { lastSeenNanos: string; lastTraceId: string | null }
  >
  errorEvents: Array<{
    tsNanos: string
    title: string
    fingerprint: string
    traceId: string | null
  }>
}

interface HubSearch {
  tab?: HubTab
  live?: boolean
  range?: string
  from?: string
  to?: string
}

const DAY_NS = 86_400_000_000_000n

function windowFor(record: InvocationRecordData | null, nowMs: number) {
  const toNanos = (BigInt(nowMs) * 1_000_000n + 60_000_000_000n).toString()
  const fromNanos =
    record?.startedAtNanos && /^\d+$/.test(record.startedAtNanos)
      ? (BigInt(record.startedAtNanos) - 60_000_000_000n).toString()
      : (BigInt(nowMs) * 1_000_000n - DAY_NS).toString()
  return { fromNanos, toNanos }
}

const RECORD_QUERY = (escaped: string) =>
  `{ invocation(invocationId: "${escaped}") {
       invocationId command appMode outcome status exitCode
       startedAtNanos endedAtNanos errorCount traceCount sessionCount
       issues { fingerprint title errorType status eventCount lastSeenNanos lastTraceId }
       errorEvents { tsNanos title fingerprint traceId }
     } }`

export async function loadInvocationHub(
  invocationId: string,
  nowMs = Date.now()
) {
  const escaped = gqlString(invocationId)
  const { invocation } = await graphqlCached<{
    invocation: InvocationRecordData | null
  }>(RECORD_QUERY(escaped))
  const { fromNanos, toNanos } = windowFor(invocation, nowMs)
  const rest = await graphqlCached<{
    tracesByInvocation: TraceSummary[]
    logsByInvocation: LogDoc[]
    story: StoryBeat[]
    sessions: Session[]
    screenVisits: ScreenVisit[]
    uiActions: UiAction[]
    backgroundCycles: BackgroundCycle[]
    jobs: Job[]
    conversations: Conversation[]
    agentSession: AgentSessionData | null
  }>(
    `{ tracesByInvocation(invocationId: "${escaped}") {
         traceId rootName service startNanos durationNs spanCount hasError
       }
       logsByInvocation(invocationId: "${escaped}", limit: 200) {
         ${LOG_FIELDS}
       }
       story(invocationId: "${escaped}") {
         tsNanos lane kind title traceId spanId severity durationNs
       }
       sessions(invocationId: "${escaped}") {
         sessionId previousSessionId startNanos endNanos
       }
       screenVisits(invocationId: "${escaped}") {
         screenId visitId sessionId navigationSequence transitionReason
         enteredNanos exitedNanos
       }
       uiActions(invocationId: "${escaped}") {
         name screenId widgetName sessionId traceId startNanos durationMs outcome hasError
       }
       backgroundCycles(invocationId: "${escaped}", fromNanos: "${fromNanos}", toNanos: "${toNanos}") {
         name count errorCount p50Ms p95Ms lastNanos lastTraceId
       }
       jobs(invocationId: "${escaped}", fromNanos: "${fromNanos}", toNanos: "${toNanos}") {
         jobId jobType producedNanos lastTraceId
         attempts { startNanos durationMs outcome hasError traceId }
       }
       conversations(invocationId: "${escaped}") {
         conversationId agentName providerName firstNanos lastNanos
         spanCount inputTokens outputTokens
       }
       agentSession(invocationId: "${escaped}") {
         rootSpanId truncated totalInputTokens totalOutputTokens errorCount
         steps {
           spanId traceId kind name startNanos durationNs isError
           genAiOperation inputTokens outputTokens
         }
       } }`
  )
  return { record: invocation, ...rest }
}

const hubSearchSchema = rangeSearchSchema.extend({
  tab: z.unknown().optional(),
  live: z.unknown().optional(),
})

export const Route = createFileRoute("/invocations/$invocationId")({
  validateSearch: (search: Record<string, unknown>): HubSearch => {
    const parsed = hubSearchSchema.parse(search)
    const result: HubSearch = {}
    const tab = TABS.find((value) => value === parsed.tab)
    if (tab && tab !== "overview") result.tab = tab
    if (parsed.live === true || parsed.live === "true") result.live = true
    if (parsed.range) result.range = parsed.range
    if (parsed.from) result.from = parsed.from
    if (parsed.to) result.to = parsed.to
    return result
  },
  loader: ({ params }) => loadInvocationHub(params.invocationId),
  component: InvocationHubPage,
})

function InvocationHubPage() {
  const data = Route.useLoaderData()
  const { invocationId } = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const live = search.live === true
  const pageVisible = usePageVisible()
  const [liveLogs, setLiveLogs] = useState<LogDoc[]>([])
  const [liveSpans, setLiveSpans] = useState<LiveSpan[]>([])
  const [polledRecord, setPolledRecord] = useState<InvocationRecordData | null>(
    null
  )
  const record = polledRecord ?? data.record

  const logStatus = useLiveStream<LogDoc>({
    url: live
      ? `/v1/logs/stream?invocation_id=${encodeURIComponent(invocationId)}`
      : null,
    parse: (payload) => {
      const batch: unknown = JSON.parse(payload)
      return Array.isArray(batch) ? (batch as LogDoc[]) : []
    },
    onBatch: (incoming) =>
      setLiveLogs((current) =>
        [...incoming.reverse(), ...current].slice(0, 300)
      ),
  })
  const spanStatus = useLiveStream<LiveSpan>({
    url: live
      ? `/v1/traces/stream?invocation_id=${encodeURIComponent(invocationId)}`
      : null,
    parse: (payload) => {
      const batch: unknown = JSON.parse(payload)
      return Array.isArray(batch) ? (batch as LiveSpan[]) : []
    },
    onBatch: (incoming) =>
      setLiveSpans((current) =>
        [...incoming.reverse(), ...current].slice(0, 300)
      ),
  })

  useEffect(() => {
    if (!live || !pageVisible) return
    const timer = setInterval(() => {
      void graphql<{ invocation: InvocationRecordData | null }>(
        RECORD_QUERY(gqlString(invocationId))
      )
        .then((next) => {
          if (next.invocation) setPolledRecord(next.invocation)
        })
        // Live polling tolerates transient API failures; next tick retries.
        .catch(() => {})
    }, 10_000)
    return () => clearInterval(timer)
  }, [live, invocationId, pageVisible])

  return (
    <InvocationHubContent
      invocationId={invocationId}
      record={record}
      data={data}
      live={live}
      streamActive={logStatus === "open" || spanStatus === "open"}
      liveLogs={liveLogs}
      liveSpans={liveSpans}
      activeTab={search.tab ?? "overview"}
      onTab={(tab) =>
        void navigate({
          search: (current) => {
            const next = { ...current }
            delete next.tab
            const parsed = TABS.find((value) => value === tab)
            if (parsed && parsed !== "overview") next.tab = parsed
            return next
          },
        })
      }
      onLive={() =>
        void navigate({
          search: (current) => {
            const next = { ...current }
            if (live) delete next.live
            else next.live = true
            return next
          },
        })
      }
    />
  )
}

export function InvocationHubContent({
  invocationId,
  record,
  data,
  live,
  streamActive = false,
  liveLogs,
  liveSpans,
  activeTab,
  onTab,
  onLive,
}: {
  invocationId: string
  record: InvocationRecordData | null
  data: Omit<Awaited<ReturnType<typeof loadInvocationHub>>, "record">
  live: boolean
  streamActive?: boolean
  liveLogs: LogDoc[]
  liveSpans: LiveSpan[]
  activeTab: HubTab
  onTab: (tab: string) => void
  onLive: () => void
}) {
  const search = { range: undefined, from: undefined, to: undefined }
  const range = resolveRangeSearch(search)
  const back = navItem("/invocations")!
  const empty =
    !record &&
    data.tracesByInvocation.length === 0 &&
    data.logsByInvocation.length === 0
  const logs = useMemo(
    () =>
      [...(live ? liveLogs : []), ...data.logsByInvocation]
        .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? 1 : -1))
        .slice(0, 500),
    [data.logsByInvocation, liveLogs, live]
  )
  // Per-occurrence events with intact nanosecond timestamps: journey beats
  // place each occurrence at its exact time (an issue's ms-truncated
  // last-seen mis-attributed between-screen errors to the previous screen).
  const journeyErrors: JourneyError[] = (record?.errorEvents ?? []).map(
    (event) => ({
      tsNanos: event.tsNanos,
      title: event.title,
      fingerprint: event.fingerprint,
      traceId: event.traceId ?? null,
    })
  )

  if (empty) {
    return (
      <EmptyState
        icon={IconTerminal2}
        title="Invocation not found"
        description="No registered invocation, traces, or logs exist for this id yet."
      />
    )
  }

  const status = record
    ? invocationStatus({
        endedAtNanos: record.endedAtNanos,
        exitCode: record.exitCode,
        outcome: record.outcome,
        lastNanos: record.endedAtNanos ?? record.startedAtNanos,
        startedAtNanos: record.startedAtNanos,
      })
    : null

  return (
    <div className="space-y-4">
      <PageHeader
        back={back}
        title={invocationId}
        titleTrailing={<CopyButton value={invocationId} />}
        description={
          <span className="inline-flex flex-wrap items-center gap-2">
            {record?.command ? (
              <code className="max-w-xl truncate" title={record.command}>
                {record.command}
              </code>
            ) : (
              "Observed CLI invocation"
            )}
            {record?.appMode ? (
              <Badge variant="outline">{appModeLabel(record.appMode)}</Badge>
            ) : null}
            {record ? <OutcomeChip outcome={record.outcome} /> : null}
          </span>
        }
        actions={
          <>
            <PinButton kind="run" label={invocationId} />
            <Button
              size="sm"
              variant={live ? "secondary" : "outline"}
              onClick={onLive}
            >
              {live ? <IconPlayerPause /> : <IconPlayerPlay />}
              {live ? "Live" : "Go live"}
              {live && streamActive ? (
                <span className="size-1.5 animate-pulse rounded-full bg-emerald-500" />
              ) : null}
            </Button>
          </>
        }
      />

      {record && status ? <HubStats record={record} status={status} /> : null}

      <Tabs value={activeTab} onValueChange={onTab}>
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="traces">Traces</TabsTrigger>
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="errors">Errors</TabsTrigger>
          <TabsTrigger value="sessions">Sessions &amp; UI</TabsTrigger>
          <TabsTrigger value="jobs">Jobs &amp; Cycles</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="flex flex-col gap-4">
          {record ? (
            <MetricStrip
              title="Process metrics"
              invocationId={invocationId}
              fromNanos={(
                BigInt(record.startedAtNanos) - 30_000_000_000n
              ).toString()}
              toNanos={(
                (record.endedAtNanos
                  ? BigInt(record.endedAtNanos)
                  : BigInt(Date.now()) * 1_000_000n) + 30_000_000_000n
              ).toString()}
              stepSeconds={5}
              live={live}
            />
          ) : null}
          {data.story.length > 0 ? (
            <Card>
              <CardHeader>
                <CardTitle className="text-sm">Story</CardTitle>
              </CardHeader>
              <CardContent>
                <StoryTimeline beats={data.story} />
              </CardContent>
            </Card>
          ) : (
            <EmptyState
              icon={IconActivity}
              title="No story yet"
              description="Nothing yet — this invocation has not emitted spans or logs."
            />
          )}
        </TabsContent>

        <TabsContent value="traces">
          <InvocationTracesTab
            traces={data.tracesByInvocation}
            liveSpans={liveSpans}
            live={live}
            range={range}
          />
        </TabsContent>

        <TabsContent value="logs">
          {logs.length === 0 ? (
            <EmptyState
              icon={IconActivity}
              title="No logs"
              description="Nothing yet — this invocation has not emitted log records."
            />
          ) : (
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
          )}
        </TabsContent>

        <TabsContent value="errors">
          <InvocationErrorsTab issues={record?.issues ?? []} range={range} />
        </TabsContent>

        <TabsContent value="sessions">
          <SessionsTab
            sessions={data.sessions}
            visits={data.screenVisits}
            actions={data.uiActions}
            conversations={data.conversations}
            errors={journeyErrors}
            agentSession={data.agentSession}
          />
        </TabsContent>

        <TabsContent value="jobs">
          <JobsCyclesTab cycles={data.backgroundCycles} jobs={data.jobs} />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function HubStats({
  record,
  status,
}: {
  record: InvocationRecordData
  status: ReturnType<typeof invocationStatus>
}) {
  const duration = invocationDurationNs(
    {
      startedAtNanos: record.startedAtNanos,
      endedAtNanos: record.endedAtNanos,
      lastNanos: record.endedAtNanos ?? record.startedAtNanos,
    },
    status
  )
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
      <StatCard
        label="Status"
        value={
          <InvocationStatusBadge status={status} exitCode={record.exitCode} />
        }
      />
      <StatCard
        icon={IconAlertTriangleFilled}
        iconClassName="text-rose-500"
        label="Errors"
        value={
          <span
            className={cn(
              record.errorCount > 0
                ? "text-rose-600 dark:text-rose-400"
                : undefined
            )}
          >
            {formatCount(record.errorCount)}
          </span>
        }
      />
      <StatCard
        icon={IconActivity}
        label="Traces"
        value={formatCount(record.traceCount)}
      />
      <StatCard
        icon={IconUsers}
        label="Sessions"
        value={formatCount(record.sessionCount)}
      />
      <StatCard
        icon={IconClock}
        label="Duration"
        value={
          status === "running"
            ? "..."
            : duration
              ? formatDurationNs(duration)
              : "-"
        }
      />
    </div>
  )
}
