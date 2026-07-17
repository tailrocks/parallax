import { useMemo, useRef, useState } from "react"
import {
  Link,
  createFileRoute,
  useNavigate,
  useRouter,
} from "@tanstack/react-router"
import {
  IconArrowUpRight,
  IconBug,
  IconClock,
  IconHash,
  IconHistory,
} from "@tabler/icons-react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts"

import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { PinButton } from "@/components/console/pin-button"
import { RangePicker } from "@/features/time-range"
import { RelativeTime } from "@/components/console/relative-time"
import { CardSparkline, StatCard } from "@/components/console/stat-card"
import { navItem } from "@/components/nav"
import { PageHeader } from "@/shared/components/page-header"
import { MetricStrip } from "@/features/runtime-metrics"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { gqlString, graphql, graphqlCached } from "@/lib/api"
import type { ErrorEvent, Issue } from "@/lib/api"
import {
  formatCount,
  formatDateTime,
  formatDelta,
  formatTimeInRange,
} from "@/lib/format"
import { parseStacktrace, structuredFrameCount } from "@/lib/stacktrace"
import type { Frame } from "@/lib/stacktrace"
import {
  mergeRangeSearch,
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

type IssueEvent = ErrorEvent & { service: string }
type IssueDetail = Issue & { tags: string; events: IssueEvent[] }

interface TrendPoint {
  tsNanos: string
  count: number
}

interface BreadcrumbLog {
  tsNanos: string
  severityText: string
  body: string
}

interface LoaderData {
  issue: IssueDetail | null
  issueTrend: TrendPoint[]
  resource: Record<string, unknown>
  breadcrumbs: BreadcrumbLog[]
  traceRunId: string | null
  releaseVersion: string | null
}

const trendConfig = {
  count: { label: "events", color: "var(--destructive)" },
} satisfies ChartConfig

export const Route = createFileRoute("/issues/$fingerprint")({
  validateSearch: (search: Record<string, unknown>) =>
    rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: ({ params, deps }) =>
    loadIssueDetail(params.fingerprint, resolveRangeSearch(deps)),
  component: IssueDetailPage,
})

function rangeHours(range: ResolvedRange): number {
  const ns = BigInt(range.toNanos) - BigInt(range.fromNanos)
  return Math.max(1, Math.ceil(Number(ns / 3_600_000_000_000n)))
}

function shortRunId(invocationId: string) {
  return invocationId.length > 8
    ? `${invocationId.slice(0, 8)}...`
    : invocationId
}

export async function loadIssueDetail(
  fingerprint: string,
  range: ResolvedRange
): Promise<LoaderData> {
  const escaped = gqlString(fingerprint)
  const { issue, issueTrend } = await graphqlCached<{
    issue: IssueDetail | null
    issueTrend: TrendPoint[]
  }>(
    `{ issue(fingerprint: "${escaped}") {
         fingerprint title errorType culprit service status
         firstSeenNanos lastSeenNanos eventCount lastTraceId tags
         events(limit: 20, fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
           tsNanos service message stacktrace source traceId spanId attributes
         }
       }
       issueTrend(fingerprint: "${escaped}", hours: ${rangeHours(range)}) { tsNanos count } }`
  )

  let resource: Record<string, unknown> = {}
  let breadcrumbs: BreadcrumbLog[] = []
  let traceRunId: string | null = null
  let releaseVersion: string | null = null
  const traceId = issue?.lastTraceId
  if (traceId) {
    try {
      const correlated = await graphqlCached<{
        trace: {
          spans: { resource: string; invocationId: string | null }[]
        } | null
        logsByTrace: BreadcrumbLog[]
      }>(
        `{ trace(traceId: "${gqlString(traceId)}") { spans { resource invocationId } }
           logsByTrace(traceId: "${gqlString(traceId)}") { tsNanos severityText body } }`
      )
      resource = JSON.parse(
        correlated.trace?.spans[0]?.resource ?? "{}"
      ) as Record<string, unknown>
      const version = resource["service.version"]
      releaseVersion =
        typeof version === "string" && version.trim() ? version.trim() : null
      breadcrumbs = correlated.logsByTrace.slice(-12)
      traceRunId =
        correlated.trace?.spans.find((s) => s.invocationId)?.invocationId ??
        null
    } catch {
      // Trace may have aged out; issue detail still renders.
    }
  }
  return {
    issue,
    issueTrend,
    resource,
    breadcrumbs,
    traceRunId,
    releaseVersion,
  }
}

function IssueDetailPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)
  return (
    <IssueDetailContent
      data={data}
      range={range}
      onRange={(next) =>
        void navigate({
          search: (current) => mergeRangeSearch(current, next),
        })
      }
    />
  )
}

function issueDelta(trend: TrendPoint[]) {
  if (trend.length < 2) return null
  const midpoint = Math.floor(trend.length / 2)
  const previous = trend
    .slice(0, midpoint)
    .reduce((sum, point) => sum + point.count, 0)
  const current = trend
    .slice(midpoint)
    .reduce((sum, point) => sum + point.count, 0)
  return formatDelta(current, previous)
}

export function IssueDetailContent({
  data,
  range,
  onRange,
}: {
  data: LoaderData
  range: ResolvedRange
  onRange: (range: ResolvedRange) => void
}) {
  const {
    issue,
    issueTrend,
    resource,
    breadcrumbs,
    traceRunId,
    releaseVersion,
  } = data
  const router = useRouter()
  const [mutating, setMutating] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [bucket, setBucket] = useState<string | null>(null)
  const [bucketEvents, setBucketEvents] = useState<IssueEvent[] | null>(null)
  const occurrencesRef = useRef<HTMLDivElement>(null)
  const bucketRequestRef = useRef<string | null>(null)

  const issuesBack = navItem("/issues")

  if (!issue) {
    return (
      <EmptyState
        icon={IconBug}
        title="Issue not found"
        description="No issue matches this fingerprint."
      />
    )
  }

  const currentIssue = issue
  const latest = currentIssue.events[0]
  const shownEvents = bucketEvents ?? currentIssue.events
  const command = `parallax issue context ${currentIssue.fingerprint}`

  async function setStatus(status: "open" | "resolved") {
    setMutating(true)
    setActionError(null)
    try {
      await graphql(`mutation { issueSetStatus(fingerprint: "${gqlString(currentIssue.fingerprint)}", status: "${status}") { status } }`)
      await router.invalidate()
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err))
    } finally {
      setMutating(false)
    }
  }

  async function filterBucket(tsNanos: string | null) {
    setActionError(null)
    setBucket(tsNanos)
    if (!tsNanos) {
      bucketRequestRef.current = null
      setBucketEvents(null)
      return
    }
    bucketRequestRef.current = tsNanos
    try {
      const from = BigInt(tsNanos)
      const to = from + 3_600_000_000_000n
      const { issue: scoped } = await graphql<{
        issue: { events: IssueEvent[] } | null
      }>(`{ issue(fingerprint: "${gqlString(currentIssue.fingerprint)}") {
             events(limit: 50, fromNanos: "${from}", toNanos: "${to}") {
               tsNanos service message stacktrace source traceId spanId attributes
             }
           } }`)
      // Another bucket was selected (or cleared) while this fetch was in flight.
      if (bucketRequestRef.current !== tsNanos) return
      setBucketEvents(scoped?.events ?? [])
      occurrencesRef.current?.scrollIntoView({
        behavior: "smooth",
        block: "start",
      })
    } catch (err) {
      setActionError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <div className="space-y-4">
      <PageHeader
        {...(issuesBack ? { back: issuesBack } : {})}
        title={currentIssue.errorType || currentIssue.title}
        titleTrailing={<CopyButton value={currentIssue.fingerprint} />}
        description={currentIssue.title}
        actions={
          <>
            <PinButton
              kind="issue"
              label={currentIssue.title || currentIssue.fingerprint}
            />
            <Button
              size="sm"
              variant="outline"
              disabled={mutating}
              onClick={() =>
                void setStatus(
                  currentIssue.status === "open" ? "resolved" : "open"
                )
              }
            >
              {currentIssue.status === "open" ? "Resolve" : "Reopen"}
            </Button>
            <RangePicker value={range} onChange={onRange} />
          </>
        }
      />

      <div className="flex flex-wrap items-center gap-2">
        <Link
          to="/services/$service"
          params={{ service: issue.service }}
          search={rangeLinkSearch(range)}
          className="inline-flex"
        >
          <Badge variant="outline">{issue.service}</Badge>
        </Link>
        {traceRunId ? (
          <Link
            to="/invocations/$invocationId"
            params={{ invocationId: traceRunId }}
            search={rangeLinkSearch(range)}
            className="inline-flex"
          >
            <Badge variant="secondary">run {shortRunId(traceRunId)}</Badge>
          </Link>
        ) : null}
        {releaseVersion ? (
          <Badge variant="secondary">release {releaseVersion}</Badge>
        ) : null}
        <Badge variant={issue.status === "open" ? "rose" : "emerald"}>
          {issue.status}
        </Badge>
        <Badge variant="secondary">
          first <RelativeTime nanos={issue.firstSeenNanos} />
        </Badge>
        <Badge variant="secondary">
          last <RelativeTime nanos={issue.lastSeenNanos} />
        </Badge>
      </div>

      {actionError ? (
        <p className="text-sm text-destructive">{actionError}</p>
      ) : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={IconHash}
          label="Events"
          value={formatCount(issue.eventCount)}
          hint="total occurrences"
          chart={
            <CardSparkline data={issueTrend.map((p) => ({ value: p.count }))} />
          }
        />
        <StatCard
          icon={IconClock}
          label="First seen"
          value={<RelativeTime nanos={issue.firstSeenNanos} />}
          hint={formatDateTime(issue.firstSeenNanos)}
        />
        <StatCard
          icon={IconHistory}
          label="Last seen"
          value={<RelativeTime nanos={issue.lastSeenNanos} />}
          hint={formatDateTime(issue.lastSeenNanos)}
        />
        <StatCard
          icon={IconBug}
          label="Trend"
          value={formatCount(
            issueTrend.reduce((sum, point) => sum + point.count, 0)
          )}
          hint="selected range"
          delta={issueDelta(issueTrend)}
          deltaInverted
        />
      </div>

      <TrendChart
        trend={issueTrend}
        onBucket={(tsNanos) => void filterBucket(tsNanos)}
        activeBucket={bucket}
      />

      {latest ? (
        <StacktraceCard event={latest} culprit={issue.culprit} range={range} />
      ) : null}

      {latest ? (
        <MetricStrip
          title="Metrics around latest event"
          service={issue.service}
          invocationId={traceRunId ?? undefined}
          fromNanos={(BigInt(latest.tsNanos) - 300_000_000_000n).toString()}
          toNanos={(BigInt(latest.tsNanos) + 300_000_000_000n).toString()}
          stepSeconds={30}
        />
      ) : null}

      <TagsTable tags={issue.tags} />
      <ContextSections resource={resource} />
      <Breadcrumbs logs={breadcrumbs} range={range} />

      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle className="text-sm">Agent handoff</CardTitle>
          <CopyButton value={command} />
        </CardHeader>
        <CardContent>
          <code className="block rounded-md border bg-muted/40 p-3 font-mono text-xs">
            {command}
          </code>
        </CardContent>
      </Card>

      <Occurrences
        refEl={occurrencesRef}
        events={shownEvents}
        bucket={bucket}
        range={range}
      />
    </div>
  )
}

function TrendChart({
  trend,
  onBucket,
  activeBucket,
}: {
  trend: TrendPoint[]
  onBucket: (tsNanos: string | null) => void
  activeBucket: string | null
}) {
  if (trend.length === 0) return null
  const data = trend.map((point) => ({
    ...point,
    time: formatTimeInRange(point.tsNanos, {
      fromNanos: trend[0]?.tsNanos ?? point.tsNanos,
      toNanos: trend.at(-1)?.tsNanos ?? point.tsNanos,
    }),
  }))
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Occurrence trend</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={trendConfig} className="h-[180px] w-full">
          <BarChart
            data={data}
            margin={{ left: 8, right: 8, top: 8 }}
            onClick={(state) => {
              const payloadState = state as {
                activePayload?: Array<{ payload?: { tsNanos?: unknown } }>
              }
              const ts = payloadState.activePayload?.[0]?.payload?.tsNanos as
                | string
                | undefined
              if (ts) void onBucket(ts === activeBucket ? null : ts)
            }}
          >
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="time"
              tickLine={false}
              axisLine={false}
              minTickGap={48}
            />
            <YAxis tickLine={false} axisLine={false} width={40} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="count" fill="var(--color-count)" radius={3} />
          </BarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

function StacktraceCard({
  event,
  culprit,
  range,
}: {
  event: IssueEvent
  culprit: string | null
  range: ResolvedRange
}) {
  const [showLibraries, setShowLibraries] = useState(false)
  const frames = parseStacktrace(event.stacktrace)
  const structured = structuredFrameCount(frames)
  const libraryCount = frames.filter((frame) => frame.isApp === false).length
  const visibleFrames = showLibraries
    ? frames
    : frames.filter((frame) => frame.isApp !== false)

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Latest event stacktrace</CardTitle>
        {event.stacktrace ? <CopyButton value={event.stacktrace} /> : null}
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm">{event.message}</p>
        {event.traceId ? (
          <Link
            to="/traces/$traceId"
            params={{ traceId: event.traceId }}
            search={rangeLinkSearch(range)}
            className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            Open trace {event.traceId.slice(0, 16)}
            <IconArrowUpRight className="size-3.5" />
          </Link>
        ) : null}
        {event.stacktrace && structured >= 2 ? (
          <div className="overflow-hidden rounded-lg border">
            {visibleFrames.map((frame, index) => (
              <FrameRow
                key={`${frame.raw}-${index}`}
                frame={frame}
                culprit={culprit}
              />
            ))}
            {libraryCount > 0 ? (
              <button
                type="button"
                className="w-full border-t bg-muted/30 px-3 py-2 text-left text-xs text-muted-foreground hover:text-foreground"
                onClick={() => setShowLibraries((value) => !value)}
              >
                {showLibraries ? "Hide" : "Show"} {libraryCount} library frames
              </button>
            ) : null}
          </div>
        ) : event.stacktrace ? (
          <pre className="max-h-96 overflow-auto rounded-md border bg-muted/30 p-3 text-xs leading-relaxed">
            {event.stacktrace}
          </pre>
        ) : (
          <p className="text-sm text-muted-foreground">
            No stacktrace captured.
          </p>
        )}
      </CardContent>
    </Card>
  )
}

function FrameRow({
  frame,
  culprit,
}: {
  frame: Frame
  culprit: string | null
}) {
  const highlighted =
    Boolean(culprit) &&
    Boolean(
      frame.raw.includes(culprit ?? "") ||
      frame.fn?.includes(culprit ?? "") ||
      frame.file?.includes(culprit ?? "")
    )
  return (
    <div
      className={cn(
        "grid gap-1 border-b px-3 py-2 last:border-b-0 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]",
        frame.isApp === false && "text-muted-foreground",
        frame.isApp !== false && "font-medium",
        highlighted && "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
      )}
    >
      <span className="truncate font-mono text-xs">
        {frame.file ? (
          <>
            {frame.file}
            {frame.line ? `:${frame.line}` : ""}
            {frame.col ? `:${frame.col}` : ""}
          </>
        ) : (
          frame.raw
        )}
      </span>
      <span className="truncate text-xs text-muted-foreground">
        {frame.fn ?? frame.raw}
      </span>
    </div>
  )
}

const CONTEXT_SECTIONS: [string, (key: string) => boolean][] = [
  ["Runtime", (key) => key.startsWith("process.runtime.")],
  [
    "Process",
    (key) => key.startsWith("process.") && !key.startsWith("process.runtime."),
  ],
  ["OS / Host", (key) => key.startsWith("os.") || key.startsWith("host.")],
  ["SDK", (key) => key.startsWith("telemetry.")],
]

function ContextSections({ resource }: { resource: Record<string, unknown> }) {
  const entries = Object.entries(resource).map(
    ([key, value]) =>
      [key, typeof value === "string" ? value : JSON.stringify(value)] as const
  )
  const sections = CONTEXT_SECTIONS.map(([title, match]) => ({
    title,
    rows: entries.filter(([key]) => match(key)),
  })).filter((section) => section.rows.length > 0)
  if (sections.length === 0) return null
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Runtime context</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4 sm:grid-cols-2">
        {sections.map((section) => (
          <div key={section.title}>
            <p className="mb-1 text-xs font-medium text-muted-foreground">
              {section.title}
            </p>
            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5 text-xs">
              {section.rows.map(([key, value]) => (
                <div key={key} className="contents">
                  <dt className="font-mono text-muted-foreground">{key}</dt>
                  <dd className="font-mono break-all">{value}</dd>
                </div>
              ))}
            </dl>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}

function TagsTable({ tags }: { tags: string }) {
  let parsed: Record<string, Record<string, number>> = {}
  try {
    parsed = JSON.parse(tags) as Record<string, Record<string, number>>
  } catch {
    return null
  }
  const keys = Object.keys(parsed)
  if (keys.length === 0) return null
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Tags</CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
          {keys.map((key) => (
            <div key={key} className="contents">
              <dt className="font-mono text-muted-foreground">{key}</dt>
              <dd className="flex flex-wrap gap-1">
                {Object.entries(parsed[key] ?? {})
                  .sort(([, a], [, b]) => b - a)
                  .map(([value, count]) => (
                    <Badge key={value} variant="secondary">
                      {value}
                      <span className="ml-1 text-muted-foreground">
                        x{count}
                      </span>
                    </Badge>
                  ))}
              </dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  )
}

function Breadcrumbs({
  logs,
  range,
}: {
  logs: BreadcrumbLog[]
  range: ResolvedRange
}) {
  if (logs.length === 0) return null
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Logs around latest event</CardTitle>
      </CardHeader>
      <CardContent>
        <ul className="space-y-1 font-mono text-xs">
          {logs.map((log, index) => (
            <li
              key={`${log.tsNanos}-${index}`}
              className="grid gap-2 sm:grid-cols-[90px_80px_1fr]"
            >
              <span className="text-muted-foreground">
                {formatTimeInRange(log.tsNanos, range)}
              </span>
              <Badge variant="secondary">{log.severityText}</Badge>
              <span className="break-all">{log.body}</span>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}

function Occurrences({
  refEl,
  events,
  bucket,
  range,
}: {
  refEl: React.RefObject<HTMLDivElement | null>
  events: IssueEvent[]
  bucket: string | null
  range: ResolvedRange
}) {
  const durations = events.map((event) => Number(event.tsNanos))
  const scale = useMemo(() => buildHeatScale(durations), [durations])
  return (
    <Card ref={refEl}>
      <CardHeader>
        <CardTitle className="text-sm">
          Occurrences
          {bucket ? (
            <span className="ml-2 font-normal text-muted-foreground">
              selected hour ({events.length})
            </span>
          ) : null}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {events.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No occurrences in this window.
          </p>
        ) : (
          <ul className="space-y-2 text-sm">
            {events.map((event) => (
              <li
                key={`${event.tsNanos}-${event.spanId}`}
                className="grid gap-2 rounded-lg border bg-muted/20 px-3 py-2 md:grid-cols-[minmax(0,1fr)_auto]"
              >
                <span className="min-w-0 truncate">{event.message}</span>
                <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
                  <HeatCell value={Number(event.tsNanos)} scale={scale}>
                    {formatTimeInRange(event.tsNanos, range)}
                  </HeatCell>
                  <Badge variant="outline">{event.service}</Badge>
                  {event.traceId ? (
                    <Link
                      to="/traces/$traceId"
                      params={{ traceId: event.traceId }}
                      search={rangeLinkSearch(range)}
                      className="inline-flex items-center gap-1 hover:text-foreground"
                    >
                      trace
                      <IconArrowUpRight className="size-3" />
                    </Link>
                  ) : null}
                </span>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}
