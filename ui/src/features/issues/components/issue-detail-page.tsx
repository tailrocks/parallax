import { useMemo, useRef, useState } from "react"
import { Link, useNavigate, useRouter } from "@tanstack/react-router"
import { IconArrowUpRight, IconBug, IconClock, IconHash, IconHistory } from "@tabler/icons-react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts"
import { CopyButton } from "@/shared/console/copy-button"
import { EmptyState } from "@/shared/console/empty-state"
import { HeatCell, buildHeatScale } from "@/shared/console/heat-cell"
import { RelativeTime } from "@/shared/console/relative-time"
import { CardSparkline, StatCard } from "@/shared/console/stat-card"
import { navItem } from "@/shared/navigation"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from "@/components/ui/chart"
import { loadIssueOccurrences, setIssueStatus } from "@/features/issues/api/issues-api"
import {
  issueDelta,
  shortRunId,
  type BreadcrumbLog,
  type IssueDetailData,
  type IssueEvent,
} from "@/features/issues/model/issue-detail"
import type { TrendPoint } from "@/features/issues/model/issue-summary"
import {
  parseStacktrace,
  structuredFrameCount,
  type Frame,
} from "@/features/issues/model/stacktrace"
import { issueGroupingCard } from "@/features/issues/components/grouping-card"
import { PinButton } from "@/features/investigations"
import { MetricStrip } from "@/features/runtime-metrics"
import { RangePicker } from "@/features/time-range"
import { formatCount, formatDateTime, formatTimeInRange } from "@/shared/format"
import { mergeRangeSearch, rangeLinkSearch, resolveRangeSearch, type ResolvedRange } from "@/domain/range"
import { cn } from "@/lib/utils"
import { PageHeader } from "@/shared/components/page-header"
import type { IssuesSearch } from "@/features/issues/model/issues-search"

const trendConfig = {
  count: { label: "events", color: "var(--destructive)" },
} satisfies ChartConfig
export function IssueDetailRoutePage({
  data,
  search,
}: {
  data: IssueDetailData
  search: IssuesSearch
}) {
  const navigate = useNavigate({ from: "/issues/$fingerprint" })
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

export function IssueDetailContent({
  data,
  range,
  onRange,
}: {
  data: IssueDetailData
  range: ResolvedRange
  onRange: (range: ResolvedRange) => void
}) {
  const { issue, issueTrend, resource, breadcrumbs, traceRunId, releaseVersion } = data
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
      await setIssueStatus(currentIssue.fingerprint, status)
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
      const events = await loadIssueOccurrences(
        currentIssue.fingerprint,
        from.toString(),
        to.toString()
      )
      if (bucketRequestRef.current !== tsNanos) return
      setBucketEvents([...events])
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
            <PinButton kind="issue" label={currentIssue.title || currentIssue.fingerprint} />
            <Button
              size="sm"
              variant="outline"
              disabled={mutating}
              onClick={() => void setStatus(currentIssue.status === "open" ? "resolved" : "open")}
            >
              {currentIssue.status === "open" ? "Resolve" : "Reopen"}
            </Button>
            <RangePicker value={range} onChange={onRange} />
          </>
        }
      />
      {issueGroupingCard(currentIssue)}
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
        {releaseVersion ? <Badge variant="secondary">release {releaseVersion}</Badge> : null}
        <Badge variant={issue.status === "open" ? "rose" : "emerald"}>{issue.status}</Badge>
        <Badge variant="secondary">
          first <RelativeTime nanos={issue.firstSeenNanos} />
        </Badge>
        <Badge variant="secondary">
          last <RelativeTime nanos={issue.lastSeenNanos} />
        </Badge>
      </div>

      {actionError ? <p className="text-sm text-destructive">{actionError}</p> : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={IconHash}
          label="Events"
          value={formatCount(issue.eventCount)}
          hint="total occurrences"
          chart={<CardSparkline data={issueTrend.map((p) => ({ value: p.count }))} />}
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
          value={formatCount(issueTrend.reduce((sum, point) => sum + point.count, 0))}
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
      {latest ? <StacktraceCard event={latest} culprit={issue.culprit} range={range} /> : null}
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

      <Occurrences refEl={occurrencesRef} events={shownEvents} bucket={bucket} range={range} />
    </div>
  )
}

function TrendChart({
  trend,
  onBucket,
  activeBucket,
}: {
  trend: readonly TrendPoint[]
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
              const ts = payloadState.activePayload?.[0]?.payload?.tsNanos as string | undefined
              if (ts) void onBucket(ts === activeBucket ? null : ts)
            }}
          >
            <CartesianGrid vertical={false} />
            <XAxis dataKey="time" tickLine={false} axisLine={false} minTickGap={48} />
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
  const visibleFrames = showLibraries ? frames : frames.filter((frame) => frame.isApp !== false)

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
              <FrameRow key={`${frame.raw}-${index}`} frame={frame} culprit={culprit} />
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
          <p className="text-sm text-muted-foreground">No stacktrace captured.</p>
        )}
      </CardContent>
    </Card>
  )
}

function FrameRow({ frame, culprit }: { frame: Frame; culprit: string | null }) {
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
      <span className="truncate text-xs text-muted-foreground">{frame.fn ?? frame.raw}</span>
    </div>
  )
}

const CONTEXT_SECTIONS: [string, (key: string) => boolean][] = [
  ["Runtime", (key) => key.startsWith("process.runtime.")],
  ["Process", (key) => key.startsWith("process.") && !key.startsWith("process.runtime.")],
  ["OS / Host", (key) => key.startsWith("os.") || key.startsWith("host.")],
  ["SDK", (key) => key.startsWith("telemetry.")],
]

function ContextSections({ resource }: { resource: Record<string, unknown> }) {
  const entries = Object.entries(resource).map(
    ([key, value]) => [key, typeof value === "string" ? value : JSON.stringify(value)] as const
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
            <p className="mb-1 text-xs font-medium text-muted-foreground">{section.title}</p>
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
                      <span className="ml-1 text-muted-foreground">x{count}</span>
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

function Breadcrumbs({ logs, range }: { logs: readonly BreadcrumbLog[]; range: ResolvedRange }) {
  if (logs.length === 0) return null
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Logs around latest event</CardTitle>
      </CardHeader>
      <CardContent>
        <ul className="space-y-1 font-mono text-xs">
          {logs.map((log, index) => (
            <li key={`${log.tsNanos}-${index}`} className="grid gap-2 sm:grid-cols-[90px_80px_1fr]">
              <span className="text-muted-foreground">{formatTimeInRange(log.tsNanos, range)}</span>
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
  events: readonly IssueEvent[]
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
          <p className="text-sm text-muted-foreground">No occurrences in this window.</p>
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
