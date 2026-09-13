import { useEffect, useRef, useState } from "react"
import { Link, useNavigate, useRouter } from "@tanstack/react-router"
import { IconArrowUpRight, IconBug, IconClock, IconHash, IconHistory } from "@tabler/icons-react"

import { CopyButton } from "@/shared/console/copy-button"
import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { SectionError } from "@/shared/console/error-state"
import { CardSparkline, StatCard } from "@/shared/console/stat-card"
import { navItem } from "@/shared/navigation"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  loadIssueCorrelation,
  loadIssueOccurrences,
  setIssueStatus,
} from "@/features/issues/api/issues-api"
import {
  issueDelta,
  shortRunId,
  parseIssueAttributes,
  type IssueDetailData,
  type IssueEvent,
} from "@/features/issues/model/issue-detail"
import {
  parseStacktrace,
  structuredFrameCount,
  type Frame,
} from "@/features/issues/model/stacktrace"
import { issueNeedsAttention, issueStatusBadgeVariant } from "@/features/issues/model/issue-status"
import { issueGroupingCard } from "@/features/issues/components/grouping-card"
import {
  CorrelationCard,
  type IssueCorrelationState,
} from "@/features/issues/components/correlation-card"
import { Occurrences } from "@/features/issues/components/occurrences"
import { LongValue } from "@/features/issues/components/long-value"
import { TrendChart } from "@/features/issues/components/issue-trend-chart"
import { PinButton } from "@/features/investigations"
import { MetricStrip } from "@/features/runtime-metrics"
import { RangePicker } from "@/features/time-range"
import { formatCount, formatDateTime } from "@/shared/format"
import {
  mergeRangeSearch,
  rangeLinkSearch,
  resolveRangeSearch,
  type ResolvedRange,
} from "@/domain/range"
import { cn } from "@/lib/utils"
import { PageHeader } from "@/shared/components/page-header"
import type { IssuesSearch } from "@/features/issues/model/issues-search"

function eventKey(event: IssueEvent): string {
  return `${event.tsNanos}:${event.spanId}`
}

export function IssueDetailRoutePage({
  data,
  search,
}: {
  data: IssueDetailData
  search: IssuesSearch
}) {
  const navigate = useNavigate({ from: "/issues/$service/$fingerprint" })
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
  const { issue, issueTrend } = data
  const router = useRouter()
  const [mutating, setMutating] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [bucket, setBucket] = useState<string | null>(null)
  const [bucketEvents, setBucketEvents] = useState<IssueEvent[] | null>(null)
  const [selectedEventKey, setSelectedEventKey] = useState<string | null>(null)
  const [correlation, setCorrelation] = useState<IssueCorrelationState>({ status: "loading" })
  const [correlationAttempt, setCorrelationAttempt] = useState(0)
  const occurrencesRef = useRef<HTMLDivElement>(null)
  const bucketRequestRef = useRef<string | null>(null)
  const issuesBack = navItem("/issues")

  const latest = issue?.events[0]
  const shownEvents = bucketEvents ?? issue?.events ?? []
  const selectedEvent = shownEvents.find((event) => eventKey(event) === selectedEventKey) ?? latest
  const selectedTraceId = selectedEvent?.traceId ?? ""
  const correlationInvocationId =
    correlation.status === "ready" ? correlation.correlation.invocationId : null

  useEffect(() => {
    if (!selectedTraceId) {
      setCorrelation({ status: "no-trace" })
      return
    }

    let active = true
    setCorrelation({ status: "loading" })
    loadIssueCorrelation(selectedTraceId)
      .then((result) => {
        if (active) setCorrelation(result)
      })
      .catch((error: unknown) => {
        if (active) {
          setCorrelation({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          })
        }
      })

    return () => {
      active = false
    }
  }, [correlationAttempt, selectedTraceId])

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
  const command = `parallax issue context ${currentIssue.fingerprint}`

  async function setStatus(status: "open" | "resolved") {
    setMutating(true)
    setActionError(null)
    try {
      await setIssueStatus(currentIssue.service, currentIssue.fingerprint, status)
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
        currentIssue.service,
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
              onClick={() =>
                void setStatus(issueNeedsAttention(currentIssue.status) ? "resolved" : "open")
              }
            >
              {issueNeedsAttention(currentIssue.status) ? "Resolve" : "Reopen"}
            </Button>
            <RangePicker value={range} onChange={onRange} />
          </>
        }
      />
      {issueGroupingCard(currentIssue)}
      <div className="flex flex-wrap items-center gap-2">
        <Link
          to="/services/$service"
          params={{ service: currentIssue.service }}
          search={rangeLinkSearch(range)}
          className="inline-flex"
        >
          <Badge variant="outline">{currentIssue.service}</Badge>
        </Link>
        {correlationInvocationId ? (
          <Link
            to="/invocations/$invocationId"
            params={{ invocationId: correlationInvocationId }}
            search={rangeLinkSearch(range)}
            className="inline-flex"
          >
            <Badge variant="secondary">run {shortRunId(correlationInvocationId)}</Badge>
          </Link>
        ) : null}
        <Badge variant={issueStatusBadgeVariant(currentIssue.status)}>
          {currentIssue.status}
        </Badge>
        <Badge variant="secondary">
          first <RelativeTime nanos={currentIssue.firstSeenNanos} />
        </Badge>
        <Badge variant="secondary">
          last <RelativeTime nanos={currentIssue.lastSeenNanos} />
        </Badge>
      </div>

      {actionError ? <SectionError message={actionError} /> : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={IconHash}
          label="Events"
          value={formatCount(currentIssue.eventCount)}
          hint="total occurrences"
          chart={<CardSparkline data={issueTrend.map((p) => ({ value: p.count }))} />}
        />
        <StatCard
          icon={IconClock}
          label="First seen"
          value={<RelativeTime nanos={currentIssue.firstSeenNanos} />}
          hint={formatDateTime(currentIssue.firstSeenNanos)}
        />
        <StatCard
          icon={IconHistory}
          label="Last seen"
          value={<RelativeTime nanos={currentIssue.lastSeenNanos} />}
          hint={formatDateTime(currentIssue.lastSeenNanos)}
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
      {selectedEvent ? <AttributesCard event={selectedEvent} /> : null}
      {selectedEvent ? (
        <StacktraceCard event={selectedEvent} culprit={currentIssue.culprit} range={range} />
      ) : null}
      {selectedEvent ? (
        <MetricStrip
          title="Metrics around selected event"
          service={currentIssue.service}
          invocationId={
            correlation.status === "ready"
              ? (correlation.correlation.invocationId ?? undefined)
              : undefined
          }
          fromNanos={(BigInt(selectedEvent.tsNanos) - 300_000_000_000n).toString()}
          toNanos={(BigInt(selectedEvent.tsNanos) + 300_000_000_000n).toString()}
          stepSeconds={30}
        />
      ) : null}

      <TagsTable tags={currentIssue.tags} />
      <CorrelationCard
        state={correlation}
        traceId={selectedTraceId}
        range={range}
        onRetry={() => setCorrelationAttempt((attempt) => attempt + 1)}
      />

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
        selectedEvent={selectedEvent ?? null}
        onSelect={(event) => setSelectedEventKey(eventKey(event))}
        bucket={bucket}
        range={range}
      />
    </div>
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
        <CardTitle className="text-sm">Selected occurrence stacktrace</CardTitle>
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

function AttributesCard({ event }: { event: IssueEvent }) {
  const attributes = parseIssueAttributes(event.attributes)

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Attributes</CardTitle>
      </CardHeader>
      <CardContent>
        {attributes.kind === "empty" ? (
          <p className="text-sm text-muted-foreground">No attributes captured.</p>
        ) : attributes.kind === "raw" ? (
          <div className="space-y-2">
            <p className="text-xs text-muted-foreground">Attributes could not be parsed.</p>
            <LongValue value={attributes.raw} />
          </div>
        ) : (
          <dl className="grid gap-x-4 gap-y-1 text-xs md:grid-cols-[minmax(0,180px)_minmax(0,1fr)]">
            {attributes.entries.map((entry) => (
              <div key={entry.key} className="contents">
                <dt title={entry.key} className="min-w-0 truncate font-mono text-muted-foreground">
                  {entry.key}
                </dt>
                <dd className="min-w-0">
                  <LongValue value={entry.value} />
                </dd>
              </div>
            ))}
          </dl>
        )}
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
