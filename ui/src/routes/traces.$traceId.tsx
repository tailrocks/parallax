import { useMemo, useState } from "react"
import type { FormEvent, ReactNode } from "react"
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import {
  IconAlertTriangle,
  IconAffiliate,
  IconArticle,
  IconClock,
  IconExternalLink,
  IconGitCompare,
  IconHash,
  IconRoute,
  IconServer,
} from "@tabler/icons-react"

import {
  TraceWaterfall,
  WHOLE_TRACE_ID,
} from "@/components/console/trace-waterfall"
import type {
  TraceViewMode,
  WaterfallSpan,
} from "@/components/console/trace-waterfall"
import { GraphqlOperationCard } from "@/components/console/graphql-operation"
import { PinButton } from "@/components/console/pin-button"
import { RpcStreamCard } from "@/components/console/rpc-stream"
import { EvidenceGapsCard } from "@/components/console/evidence-gaps"
import { StoryTimeline } from "@/components/console/story-timeline"
import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { PageHeader } from "@/components/page-header"
import { MetricStrip } from "@/components/metric-strip"
import { navItem } from "@/components/nav"
import { Badge } from "@/components/ui/badge"
import { Button, buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Separator } from "@/components/ui/separator"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { Spinner } from "@/components/ui/spinner"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import { graphql, gqlString } from "@/lib/api"
import type {
  CriticalPath,
  EvidenceGap,
  SpanLink,
  StoryBeat,
  TraceDiff,
  TraceDiffSpan,
  TraceSummary,
} from "@/lib/api"
import { formatDateTime, formatDurationNs } from "@/lib/format"
import { buildGraphqlOperations } from "@/lib/graphql-trace"
import type { GraphqlOperation, GraphqlTraceSpan } from "@/lib/graphql-trace"
import {
  buildRpcStreams,
  grpcStatusLabel,
  messagingSummary,
  parseGrpcStatusCode,
} from "@/lib/rpc-trace"
import type {
  MessagingSummary,
  RpcStreamInfo,
  RpcTraceSpan,
} from "@/lib/rpc-trace"
import { computeWindow, detectSkew } from "@/lib/trace-tree"
import type { SkewReport } from "@/lib/trace-tree"
import { rangeLinkSearch, resolveRangeSearch } from "@/lib/range"
import { cn } from "@/lib/utils"

interface TraceSpan extends WaterfallSpan, GraphqlTraceSpan, RpcTraceSpan {
  tsNanos: string
  traceId: string
  runId: string | null
  links: string
  typedLinks: SpanLink[]
  events: string
  attributes: string
  resource: string
}

interface TraceLog {
  tsNanos: string
  service: string
  severityText: string
  body: string
  spanId: string
}

type TraceDetailTab = "waterfall" | "story"

interface TraceDetailSearch {
  tab?: TraceDetailTab | undefined
  view?: TraceViewMode | undefined
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
}

type TraceRangeSearch = ReturnType<typeof rangeLinkSearch>

type JsonRecord = Record<string, unknown>
type KeyValues = Array<[string, ReactNode]>
type StringKeyValues = Array<[string, string]>

const TRACE_DIFF_SPAN_FIELDS =
  "spanId service name kind statusCode durationNs depth matchKey"
const EMPTY_TRACE_SPANS: TraceSpan[] = []
const INSPECTOR_LIST_CAP = 25
const TRACE_VIEW_MODES = ["tree", "errors", "lanes"] as const

function isTraceViewMode(value: unknown): value is TraceViewMode {
  return (
    typeof value === "string" &&
    TRACE_VIEW_MODES.includes(value as TraceViewMode)
  )
}

function searchString(value: unknown) {
  if (typeof value === "string") return value
  if (typeof value === "number" && Number.isFinite(value)) return String(value)
  return undefined
}

export function validateTraceDetailSearch(
  search: Record<string, unknown>
): TraceDetailSearch {
  return {
    tab: search.tab === "story" ? "story" : undefined,
    view: isTraceViewMode(search.view) ? search.view : undefined,
    range: searchString(search.range),
    from: searchString(search.from),
    to: searchString(search.to),
  }
}

export interface SpanEvent {
  name: string
  timeUnixNano?: string
  attributes?: JsonRecord
}

interface TraceEventRow {
  spanId: string
  spanName: string
  service: string
  name: string
  timeUnixNano: string
  attributes: string
}

interface TraceEventsResult {
  events: TraceEventRow[]
  truncated: boolean
  skippedSpans: number
}

export const Route = createFileRoute("/traces/$traceId")({
  validateSearch: validateTraceDetailSearch,
  loader: ({ params }) => {
    const traceId = gqlString(params.traceId)
    return graphql<{
      trace: { spans: TraceSpan[] } | null
      logsByTrace: TraceLog[]
      linkedTraces: TraceSummary[]
      story: StoryBeat[]
      evidenceGaps: EvidenceGap[]
      rpcTraceEvents: TraceEventsResult
    }>(
      `{ trace(traceId: "${traceId}") {
           spans { tsNanos service traceId name kind statusCode statusMessage durationNs
                   spanId parentSpanId runId links typedLinks { traceId spanId attributes }
                   events attributes resource }
         }
         linkedTraces(traceId: "${traceId}") {
           traceId rootName service startNanos durationNs spanCount hasError
         }
         story(traceId: "${traceId}") {
           tsNanos lane kind title traceId spanId severity durationNs
         }
         evidenceGaps(traceId: "${traceId}") {
           kind subject detail
         }
         rpcTraceEvents: traceEvents(traceId: "${traceId}", namePrefix: "rpc", limit: 500) {
           truncated skippedSpans
           events { spanId spanName service name timeUnixNano attributes }
         }
         logsByTrace(traceId: "${traceId}") { tsNanos service severityText body spanId } }`
    )
  },
  component: TracePage,
})

function parseJsonRecord(json: string): JsonRecord {
  try {
    const parsed: unknown = JSON.parse(json)
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as JsonRecord)
      : {}
  } catch {
    return {}
  }
}

function parseKeyValues(json: string): StringKeyValues {
  return Object.entries(parseJsonRecord(json))
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([key, value]) => [
      key,
      typeof value === "string" ? value : JSON.stringify(value),
    ])
}

function parseEvents(json: string): SpanEvent[] {
  try {
    const value: unknown = JSON.parse(json)
    return Array.isArray(value)
      ? value.filter(
          (event): event is SpanEvent =>
            !!event && typeof event === "object" && "name" in event
        )
      : []
  } catch {
    return []
  }
}

function valueFor(entries: StringKeyValues, key: string): string | null {
  return entries.find(([entryKey]) => entryKey === key)?.[1] ?? null
}

function TracePage() {
  const {
    trace,
    logsByTrace,
    linkedTraces,
    story,
    evidenceGaps,
    rpcTraceEvents,
  } = Route.useLoaderData()
  const { traceId } = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const activeTab: TraceDetailTab =
    search.tab === "story" ? "story" : "waterfall"
  const waterfallView: TraceViewMode = search.view ?? "tree"
  const detailRangeSearch = rangeLinkSearch(resolveRangeSearch(search))
  const [selectedId, setSelectedId] = useState<string | null>(WHOLE_TRACE_ID)
  const [criticalEnabled, setCriticalEnabled] = useState(false)
  const [criticalPath, setCriticalPath] = useState<CriticalPath | null>(null)
  const [criticalLoading, setCriticalLoading] = useState(false)
  const [criticalError, setCriticalError] = useState<string | null>(null)
  const [compareOpen, setCompareOpen] = useState(false)
  const [compareTraceId, setCompareTraceId] = useState("")
  const [compareResult, setCompareResult] = useState<TraceDiff | null>(null)
  const [compareLoading, setCompareLoading] = useState(false)
  const [compareError, setCompareError] = useState<string | null>(null)
  const criticalIds = useMemo(
    () =>
      criticalEnabled && criticalPath
        ? new Set(criticalPath.hops.map((hop) => hop.spanId))
        : undefined,
    [criticalEnabled, criticalPath]
  )
  const orderedLogs = useMemo(
    () =>
      [...logsByTrace].sort((a, b) =>
        BigInt(a.tsNanos) < BigInt(b.tsNanos) ? 1 : -1
      ),
    [logsByTrace]
  )
  const linkedTraceById = useMemo(
    () =>
      new Map(
        linkedTraces.map((traceSummary) => [traceSummary.traceId, traceSummary])
      ),
    [linkedTraces]
  )
  const spans = trace?.spans ?? EMPTY_TRACE_SPANS
  const graphqlOperations = useMemo(
    () => buildGraphqlOperations(spans),
    [spans]
  )
  const rpcStreams = useMemo(
    () =>
      buildRpcStreams(spans, rpcTraceEvents.events, rpcTraceEvents.truncated),
    [rpcTraceEvents.events, rpcTraceEvents.truncated, spans]
  )
  const messaging = useMemo(() => messagingSummary(spans), [spans])
  const skewReport = useMemo(() => detectSkew(spans), [spans])

  if (!trace || spans.length === 0) {
    return (
      <EmptyState
        title="Trace not found"
        description={traceId}
        icon={IconAffiliate}
      />
    )
  }

  const window = computeWindow(spans)
  const rootSpan =
    spans.find((span) => !span.parentSpanId) ??
    [...spans].sort((a, b) =>
      BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1
    )[0]!
  const tracesBack = navItem("/traces")
  const runId = spans.find((span) => span.runId)?.runId ?? null
  const services = Array.from(new Set(spans.map((span) => span.service))).sort()
  const failedSpans = spans.filter(
    (span) => span.statusCode === "STATUS_CODE_ERROR"
  )
  const spanLinks = spans.flatMap((span) => span.typedLinks)
  const spanEvents = spans.flatMap((span) => parseEvents(span.events))
  const selectedSpan =
    selectedId && selectedId !== WHOLE_TRACE_ID
      ? (spans.find((span) => span.spanId === selectedId) ?? null)
      : null

  const loadCriticalPath = async () => {
    setCriticalLoading(true)
    setCriticalError(null)
    try {
      const data = await graphql<{ traceCriticalPath: CriticalPath }>(
        `{ traceCriticalPath(traceId: "${gqlString(traceId)}") {
             totalGatedNs unattached
             hops { spanId selfTimeNs gatedByChild clockSuspect }
           } }`
      )
      setCriticalPath(data.traceCriticalPath)
    } catch (error) {
      setCriticalError(error instanceof Error ? error.message : String(error))
    } finally {
      setCriticalLoading(false)
    }
  }

  const toggleCriticalPath = () => {
    const next = !criticalEnabled
    setCriticalEnabled(next)
    if (next && !criticalPath && !criticalLoading) void loadCriticalPath()
    if (!next) setCriticalError(null)
  }

  const runCompare = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const otherTraceId = compareTraceId.trim()
    if (!otherTraceId) {
      setCompareError("Trace id required.")
      return
    }
    setCompareLoading(true)
    setCompareError(null)
    try {
      const data = await graphql<{ traceCompare: TraceDiff }>(
        `{ traceCompare(traceIdA: "${gqlString(traceId)}", traceIdB: "${gqlString(
          otherTraceId
        )}") {
             added { ${TRACE_DIFF_SPAN_FIELDS} }
             removed { ${TRACE_DIFF_SPAN_FIELDS} }
             changed {
               durationDeltaNs durationDeltaPct statusChanged
               before { ${TRACE_DIFF_SPAN_FIELDS} }
               after { ${TRACE_DIFF_SPAN_FIELDS} }
             }
           } }`
      )
      setCompareResult(data.traceCompare)
    } catch (error) {
      setCompareError(error instanceof Error ? error.message : String(error))
    } finally {
      setCompareLoading(false)
    }
  }

  const setActiveTab = (value: string) => {
    void navigate({
      search: (current) => ({
        ...current,
        tab: value === "story" ? "story" : undefined,
      }),
    })
  }

  const setWaterfallView = (value: TraceViewMode) => {
    void navigate({
      search: (current) => ({
        ...current,
        view: value === "tree" ? undefined : value,
      }),
    })
  }

  return (
    <div className="space-y-4">
      <PageHeader
        {...(tracesBack ? { back: tracesBack } : {})}
        title={rootSpan.name || "Trace"}
        titleTrailing={<CopyButton value={traceId} />}
        description={
          <span className="flex flex-wrap items-center gap-1.5">
            <span className="font-mono">{traceId}</span>
            {runId ? (
              <Link
                to="/runs/$runId"
                params={{ runId }}
                search={detailRangeSearch}
                className={buttonVariants({ variant: "outline", size: "xs" })}
              >
                <IconExternalLink />
                run {runId.slice(0, 12)}
              </Link>
            ) : null}
            {services.map((service) => (
              <Link
                key={service}
                to="/services/$service"
                params={{ service }}
                search={detailRangeSearch}
                className="inline-flex"
              >
                <Badge variant="outline">{service}</Badge>
              </Link>
            ))}
          </span>
        }
        actions={
          <div className="flex flex-wrap items-center justify-end gap-2">
            <PinButton kind="trace" label={rootSpan.name || traceId} />
            <Button
              type="button"
              variant={criticalEnabled ? "secondary" : "outline"}
              size="sm"
              aria-pressed={criticalEnabled}
              onClick={toggleCriticalPath}
              disabled={criticalLoading}
            >
              {criticalLoading ? (
                <Spinner data-icon="inline-start" />
              ) : (
                <IconRoute data-icon="inline-start" />
              )}
              Critical path
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setCompareOpen(true)}
            >
              <IconGitCompare data-icon="inline-start" />
              Compare
            </Button>
          </div>
        }
      />

      <SummaryStrip
        spans={spans}
        logs={logsByTrace}
        durationNs={window.durationNs.toString()}
        serviceCount={services.length}
        linkCount={spanLinks.length}
        eventCount={spanEvents.length}
      />
      {messaging ? <MessagingSummaryLine summary={messaging} /> : null}

      {failedSpans.length > 0 ? (
        <button
          type="button"
          onClick={() => setSelectedId(failedSpans[0]!.spanId)}
          className="flex w-full items-center justify-between gap-3 rounded-lg border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-left text-sm text-rose-700 dark:text-rose-300"
        >
          <span className="flex min-w-0 items-center gap-2">
            <IconAlertTriangle className="size-4 shrink-0" />
            <span className="truncate">
              {failedSpans.length} errored span
              {failedSpans.length === 1 ? "" : "s"} in this trace
            </span>
          </span>
          <span className="text-xs font-medium">Open first</span>
        </button>
      ) : null}

      <ClockSkewBanner report={skewReport} />

      <EvidenceGapsCard gaps={evidenceGaps} />

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="waterfall">Waterfall</TabsTrigger>
          <TabsTrigger value="story">Story</TabsTrigger>
        </TabsList>
        <TabsContent value="waterfall">
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_420px]">
            <div className="space-y-4">
              <Card>
                <CardHeader className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                  <CardTitle className="text-sm">Waterfall</CardTitle>
                  <TraceViewModeToggle
                    value={waterfallView}
                    onChange={setWaterfallView}
                  />
                </CardHeader>
                <CardContent className="flex flex-col gap-3">
                  {criticalEnabled ? (
                    <CriticalPathSummary
                      criticalPath={criticalPath}
                      loading={criticalLoading}
                      error={criticalError}
                      totalNs={window.durationNs.toString()}
                    />
                  ) : null}
                  <TraceWaterfall
                    spans={spans}
                    selectedId={selectedId}
                    onSelect={setSelectedId}
                    highlightIds={criticalIds}
                    mode={waterfallView}
                  />
                </CardContent>
              </Card>

              <TraceGraphqlSection
                operations={graphqlOperations}
                onSelect={setSelectedId}
              />

              <TraceRpcSection streams={rpcStreams} />

              {orderedLogs.length > 0 ? (
                <Card>
                  <CardHeader>
                    <CardTitle className="text-sm">
                      Trace logs{" "}
                      <span className="font-normal text-muted-foreground">
                        ({orderedLogs.length})
                      </span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ul className="space-y-1.5">
                      {orderedLogs.map((log, index) => (
                        <TraceLogRow
                          key={`${log.tsNanos}-${index}`}
                          log={log}
                          onSelectSpan={setSelectedId}
                        />
                      ))}
                    </ul>
                  </CardContent>
                </Card>
              ) : null}

              <MetricStrip
                title="Metrics around this trace"
                service={rootSpan.service}
                runId={runId ?? undefined}
                fromNanos={(window.startNs - 300_000_000_000n).toString()}
                toNanos={(
                  window.startNs +
                  window.durationNs +
                  300_000_000_000n
                ).toString()}
                stepSeconds={30}
              />
            </div>

            <TraceInspector
              traceId={traceId}
              spans={spans}
              selectedSpan={selectedSpan}
              linkedTraceById={linkedTraceById}
              rangeSearch={detailRangeSearch}
              logs={orderedLogs}
              onSelectSpan={setSelectedId}
            />
          </div>
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

      <Sheet open={compareOpen} onOpenChange={setCompareOpen}>
        <SheetContent className="overflow-y-auto sm:max-w-2xl">
          <SheetHeader>
            <SheetTitle>Compare traces</SheetTitle>
            <SheetDescription>
              Diff this trace against another trace id.
            </SheetDescription>
          </SheetHeader>
          <div className="flex flex-col gap-4 px-6 pb-6">
            <form onSubmit={runCompare}>
              <FieldGroup className="gap-3">
                <Field data-invalid={compareError ? true : undefined}>
                  <FieldLabel htmlFor="trace-compare-id">
                    Compare with
                  </FieldLabel>
                  <Input
                    id="trace-compare-id"
                    value={compareTraceId}
                    onChange={(event) => {
                      setCompareTraceId(event.target.value)
                      setCompareError(null)
                    }}
                    placeholder="trace id"
                    aria-invalid={compareError ? true : undefined}
                  />
                  <FieldDescription>
                    Current trace: <span className="font-mono">{traceId}</span>
                  </FieldDescription>
                  <FieldError>{compareError}</FieldError>
                </Field>
                <Button
                  type="submit"
                  className="w-fit"
                  disabled={
                    compareLoading || compareTraceId.trim().length === 0
                  }
                >
                  {compareLoading ? (
                    <Spinner data-icon="inline-start" />
                  ) : (
                    <IconGitCompare data-icon="inline-start" />
                  )}
                  Run compare
                </Button>
              </FieldGroup>
            </form>
            {compareResult ? <TraceCompareResult diff={compareResult} /> : null}
          </div>
        </SheetContent>
      </Sheet>
    </div>
  )
}

export function TraceGraphqlSection({
  operations,
  onSelect,
}: {
  operations: GraphqlOperation[]
  onSelect: (spanId: string) => void
}) {
  if (operations.length === 0) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">GraphQL</CardTitle>
      </CardHeader>
      <CardContent>
        <GraphqlOperationCard operations={operations} onSelect={onSelect} />
      </CardContent>
    </Card>
  )
}

export function TraceRpcSection({ streams }: { streams: RpcStreamInfo[] }) {
  if (streams.length === 0) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">RPC streams</CardTitle>
      </CardHeader>
      <CardContent>
        <RpcStreamCard streams={streams} />
      </CardContent>
    </Card>
  )
}

export function TraceViewModeToggle({
  value,
  onChange,
}: {
  value: TraceViewMode
  onChange: (value: TraceViewMode) => void
}) {
  const options: Array<{ value: TraceViewMode; label: string }> = [
    { value: "tree", label: "Tree" },
    { value: "errors", label: "Errors" },
    { value: "lanes", label: "Lanes" },
  ]

  return (
    <ToggleGroup
      value={[value]}
      onValueChange={(values) => {
        const next = values[0]
        if (isTraceViewMode(next)) onChange(next)
      }}
      variant="outline"
      size="sm"
      spacing={0}
      aria-label="Trace view mode"
    >
      {options.map((option) => (
        <ToggleGroupItem
          key={option.value}
          value={option.value}
          aria-label={`${option.label} view`}
        >
          {option.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}

export function ClockSkewBanner({ report }: { report: SkewReport }) {
  if (report.suspectPairs.length === 0) return null

  return (
    <div className="flex w-full items-center gap-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-left text-sm text-amber-800 dark:text-amber-200">
      <IconAlertTriangle className="size-4 shrink-0" />
      <span>
        Clock skew suspected: {report.suspectPairs.length.toLocaleString()}{" "}
        parent/child pair
        {report.suspectPairs.length === 1 ? "" : "s"} disagree by up to{" "}
        {Math.round(report.maxDriftMs).toLocaleString()} ms across services -
        span order may be misleading.
      </span>
    </div>
  )
}

function MessagingSummaryLine({ summary }: { summary: MessagingSummary }) {
  return (
    <p className="rounded-lg border border-border/70 bg-muted/20 px-3 py-2 text-sm text-muted-foreground">
      {summary.producer.toLocaleString()} producer /{" "}
      {summary.consumer.toLocaleString()} consumer spans - max batch{" "}
      {summary.batchMax.toLocaleString()}
    </p>
  )
}

function SummaryStrip({
  spans,
  logs,
  durationNs,
  serviceCount,
  linkCount,
  eventCount,
}: {
  spans: TraceSpan[]
  logs: TraceLog[]
  durationNs: string
  serviceCount: number
  linkCount: number
  eventCount: number
}) {
  const errorCount = spans.filter(
    (span) => span.statusCode === "STATUS_CODE_ERROR"
  ).length
  const items = [
    { label: "Spans", value: spans.length.toLocaleString(), icon: IconHash },
    { label: "Duration", value: formatDurationNs(durationNs), icon: IconClock },
    {
      label: "Services",
      value: serviceCount.toLocaleString(),
      icon: IconServer,
    },
    {
      label: "Errors",
      value: errorCount.toLocaleString(),
      icon: IconAlertTriangle,
      tone: errorCount > 0 ? "text-rose-600" : undefined,
    },
    { label: "Logs", value: logs.length.toLocaleString(), icon: IconArticle },
    {
      label: "Links/events",
      value: `${linkCount.toLocaleString()}/${eventCount.toLocaleString()}`,
      icon: IconAffiliate,
    },
  ]

  return (
    <div className="grid gap-2 rounded-xl border border-border/70 bg-muted/20 p-2 sm:grid-cols-2 lg:grid-cols-6">
      {items.map((item) => {
        const Icon = item.icon
        return (
          <div key={item.label} className="min-w-0 rounded-lg px-3 py-2">
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Icon className={cn("size-3.5", item.tone)} />
              {item.label}
            </div>
            <div className="mt-1 truncate text-sm font-medium tabular-nums">
              {item.value}
            </div>
          </div>
        )
      })}
    </div>
  )
}

function TraceLogRow({
  log,
  onSelectSpan,
}: {
  log: TraceLog
  onSelectSpan: (spanId: string) => void
}) {
  return (
    <li className="grid gap-2 rounded-lg border border-border/70 bg-background/60 px-3 py-2 text-xs md:grid-cols-[9rem_7rem_minmax(0,1fr)_auto]">
      <span className="text-muted-foreground tabular-nums">
        {formatDateTime(log.tsNanos)}
      </span>
      <Badge variant={log.severityText === "ERROR" ? "rose" : "secondary"}>
        {log.severityText || "log"}
      </Badge>
      <span className="min-w-0 font-mono break-words">{log.body}</span>
      {log.spanId ? (
        <button
          type="button"
          onClick={() => onSelectSpan(log.spanId)}
          className="font-mono text-muted-foreground underline underline-offset-4 hover:text-foreground"
        >
          {log.spanId.slice(0, 8)}
        </button>
      ) : null}
    </li>
  )
}

function CriticalPathSummary({
  criticalPath,
  loading,
  error,
  totalNs,
}: {
  criticalPath: CriticalPath | null
  loading: boolean
  error: string | null
  totalNs: string
}) {
  if (loading) {
    return (
      <p className="flex items-center gap-2 text-sm text-muted-foreground">
        <Spinner />
        Loading critical path
      </p>
    )
  }
  if (error) {
    return (
      <p className="text-sm text-destructive" role="alert">
        {error}
      </p>
    )
  }
  if (!criticalPath) return null

  const clockSuspect = criticalPath.hops.some((hop) => hop.clockSuspect)
  return (
    <p
      className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground"
      data-testid="critical-path-summary"
    >
      <span>
        {criticalPath.hops.length.toLocaleString()} spans gate{" "}
        {formatDurationNs(criticalPath.totalGatedNs)} of{" "}
        {formatDurationNs(totalNs)} total
      </span>
      {criticalPath.unattached.length > 0 ? (
        <Badge variant="outline">
          {criticalPath.unattached.length.toLocaleString()} unattached
        </Badge>
      ) : null}
      {clockSuspect ? <Badge variant="secondary">clock suspect</Badge> : null}
    </p>
  )
}

function statusLabel(statusCode: string): string {
  return statusCode.replace("STATUS_CODE_", "") || "UNSET"
}

function formatSignedDurationNs(value: string): string {
  const ns = BigInt(value)
  const abs = ns < 0n ? -ns : ns
  const sign = ns > 0n ? "+" : ns < 0n ? "-" : ""
  return `${sign}${formatDurationNs(abs.toString())}`
}

function formatPctDelta(value: number): string {
  if (!Number.isFinite(value)) return "-"
  const sign = value > 0 ? "+" : ""
  const abs = Math.abs(value)
  return `${sign}${value.toFixed(abs < 10 ? 1 : 0)}%`
}

function DiffSpanSection({
  title,
  spans,
  emptyLabel,
}: {
  title: string
  spans: TraceDiffSpan[]
  emptyLabel: string
}) {
  return (
    <section className="flex flex-col gap-2">
      <h3 className="text-sm font-medium">
        {title}{" "}
        <span className="font-normal text-muted-foreground">
          ({spans.length.toLocaleString()})
        </span>
      </h3>
      {spans.length === 0 ? (
        <p className="text-sm text-muted-foreground">{emptyLabel}</p>
      ) : (
        <Table density="compact">
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Service</TableHead>
              <TableHead align="right">Duration</TableHead>
              <TableHead>Status</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {spans.map((span) => (
              <TableRow key={span.matchKey}>
                <TableCell className="max-w-[16rem] truncate">
                  {span.name}
                </TableCell>
                <TableCell>{span.service}</TableCell>
                <TableCell align="right">
                  {formatDurationNs(span.durationNs)}
                </TableCell>
                <TableCell>
                  <Badge
                    variant={
                      span.statusCode === "STATUS_CODE_ERROR"
                        ? "rose"
                        : "secondary"
                    }
                  >
                    {statusLabel(span.statusCode)}
                  </Badge>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </section>
  )
}

export function TraceCompareResult({ diff }: { diff: TraceDiff }) {
  const empty =
    diff.added.length === 0 &&
    diff.removed.length === 0 &&
    diff.changed.length === 0

  return (
    <div className="flex flex-col gap-5" data-testid="trace-compare-result">
      {empty ? (
        <p className="text-sm text-muted-foreground">Structurally identical.</p>
      ) : null}
      <DiffSpanSection
        title="Added"
        spans={diff.added}
        emptyLabel="No added spans."
      />
      <DiffSpanSection
        title="Removed"
        spans={diff.removed}
        emptyLabel="No removed spans."
      />
      <section className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">
          Changed{" "}
          <span className="font-normal text-muted-foreground">
            ({diff.changed.length.toLocaleString()})
          </span>
        </h3>
        {diff.changed.length === 0 ? (
          <p className="text-sm text-muted-foreground">No changed spans.</p>
        ) : (
          <Table density="compact">
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Service</TableHead>
                <TableHead align="right">Before</TableHead>
                <TableHead align="right">After</TableHead>
                <TableHead align="right">Delta</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {diff.changed.map((change) => (
                <TableRow key={change.after.matchKey}>
                  <TableCell className="max-w-[14rem] truncate">
                    {change.after.name}
                  </TableCell>
                  <TableCell>{change.after.service}</TableCell>
                  <TableCell align="right">
                    {formatDurationNs(change.before.durationNs)}
                  </TableCell>
                  <TableCell align="right">
                    {formatDurationNs(change.after.durationNs)}
                  </TableCell>
                  <TableCell align="right">
                    {formatSignedDurationNs(change.durationDeltaNs)}{" "}
                    <span className="text-muted-foreground">
                      ({formatPctDelta(change.durationDeltaPct)})
                    </span>
                  </TableCell>
                  <TableCell>
                    {change.statusChanged ? (
                      <span className="flex flex-wrap gap-1">
                        <Badge variant="outline">
                          {statusLabel(change.before.statusCode)}
                        </Badge>
                        <Badge
                          variant={
                            change.after.statusCode === "STATUS_CODE_ERROR"
                              ? "rose"
                              : "secondary"
                          }
                        >
                          {statusLabel(change.after.statusCode)}
                        </Badge>
                      </span>
                    ) : (
                      <Badge variant="secondary">duration</Badge>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </section>
    </div>
  )
}

export function LinkedTraceEdges({
  links,
  linkedTraceById,
  rangeSearch,
}: {
  links: SpanLink[]
  linkedTraceById: ReadonlyMap<string, TraceSummary>
  rangeSearch?: TraceRangeSearch
}) {
  return (
    <ul className="space-y-2">
      {links.map((link) => {
        const target = linkedTraceById.get(link.traceId)
        return (
          <li
            key={`${link.traceId}-${link.spanId}`}
            data-testid="trace-link-edge"
          >
            <Link
              to="/traces/$traceId"
              params={{ traceId: link.traceId }}
              {...(rangeSearch ? { search: rangeSearch } : {})}
              className="block rounded-lg border border-border/70 bg-background/60 p-2 hover:bg-muted/60"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-1.5">
                    <Badge variant="outline">
                      {target?.service ?? "unknown service"}
                    </Badge>
                    {target?.hasError ? (
                      <Badge variant="rose">error</Badge>
                    ) : null}
                  </div>
                  <p className="mt-1 font-medium break-words">
                    {target?.rootName ?? "Unresolved linked trace"}
                  </p>
                </div>
                <IconExternalLink className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
              </div>
              <div className="mt-2 flex flex-wrap gap-2 font-mono text-[11px] text-muted-foreground">
                <span>{link.traceId}</span>
                {target ? (
                  <>
                    <span>{target.spanCount.toLocaleString()} spans</span>
                    <span>{formatDurationNs(target.durationNs)}</span>
                  </>
                ) : null}
              </div>
            </Link>
            <dl className="mt-1 grid gap-1 pl-2 font-mono text-[11px] text-muted-foreground">
              <div className="flex gap-2">
                <dt>span</dt>
                <dd>{link.spanId || "-"}</dd>
              </div>
              {link.attributes && link.attributes !== "{}" ? (
                <div className="flex gap-2">
                  <dt>attributes</dt>
                  <dd className="break-all">{link.attributes}</dd>
                </div>
              ) : null}
            </dl>
          </li>
        )
      })}
    </ul>
  )
}

export function TraceErrorCallout({
  statusMessage,
  grpcStatusCode,
}: {
  statusMessage: string
  grpcStatusCode: string | null
}) {
  const grpcLabel = grpcStatusLabel(parseGrpcStatusCode(grpcStatusCode))
  return (
    <div className="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-rose-700 dark:text-rose-300">
      {statusMessage || "Span ended with error status."}
      {grpcLabel ? <span className="ml-1 font-medium">{grpcLabel}</span> : null}
    </div>
  )
}

export function InspectorEventList({ events }: { events: SpanEvent[] }) {
  const [showAll, setShowAll] = useState(false)
  const visible = showAll ? events : events.slice(0, INSPECTOR_LIST_CAP)
  return (
    <div className="space-y-2">
      <ul className="space-y-2">
        {visible.map((event, index) => (
          <li
            key={`${event.name}-${index}`}
            data-testid="inspector-event"
            className="rounded-lg border border-border/70 bg-background/60 p-2"
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium">{event.name}</span>
              {event.timeUnixNano ? (
                <span className="text-muted-foreground tabular-nums">
                  {formatDateTime(event.timeUnixNano)}
                </span>
              ) : null}
            </div>
            {event.attributes ? (
              <KeyValueList
                entries={Object.entries(event.attributes).map(
                  ([key, value]) => [
                    key,
                    typeof value === "string" ? value : JSON.stringify(value),
                  ]
                )}
              />
            ) : null}
          </li>
        ))}
      </ul>
      {events.length > INSPECTOR_LIST_CAP ? (
        <Button
          type="button"
          variant="outline"
          size="xs"
          onClick={() => setShowAll((current) => !current)}
        >
          {showAll
            ? "Show fewer events"
            : `Show all ${events.length.toLocaleString()} events`}
        </Button>
      ) : null}
    </div>
  )
}

export function InspectorLinksList({
  links,
  linkedTraceById,
  rangeSearch,
}: {
  links: SpanLink[]
  linkedTraceById: ReadonlyMap<string, TraceSummary>
  rangeSearch?: TraceRangeSearch
}) {
  const [showAll, setShowAll] = useState(false)
  const visible = showAll ? links : links.slice(0, INSPECTOR_LIST_CAP)
  return (
    <div className="space-y-2">
      <LinkedTraceEdges
        links={visible}
        linkedTraceById={linkedTraceById}
        {...(rangeSearch ? { rangeSearch } : {})}
      />
      {links.length > INSPECTOR_LIST_CAP ? (
        <Button
          type="button"
          variant="outline"
          size="xs"
          onClick={() => setShowAll((current) => !current)}
        >
          {showAll
            ? "Show fewer links"
            : `Show all ${links.length.toLocaleString()} links`}
        </Button>
      ) : null}
    </div>
  )
}

function TraceInspector({
  traceId,
  spans,
  selectedSpan,
  linkedTraceById,
  rangeSearch,
  logs,
  onSelectSpan,
}: {
  traceId: string
  spans: TraceSpan[]
  selectedSpan: TraceSpan | null
  linkedTraceById: ReadonlyMap<string, TraceSummary>
  rangeSearch: TraceRangeSearch
  logs: TraceLog[]
  onSelectSpan: (spanId: string) => void
}) {
  if (!selectedSpan) {
    return (
      <Card className="h-fit xl:sticky xl:top-4">
        <CardHeader>
          <CardTitle className="flex items-center justify-between gap-2 text-sm">
            <span>Trace</span>
            <CopyButton value={traceId} />
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-xs">
          <KeyValueList
            entries={[
              ["trace id", traceId],
              ["spans", spans.length.toLocaleString()],
              [
                "services",
                Array.from(new Set(spans.map((s) => s.service))).join(", "),
              ],
              ["logs", logs.length.toLocaleString()],
            ]}
          />
          <Separator />
          <div>
            <p className="mb-1 font-medium text-muted-foreground">Span index</p>
            <div className="flex flex-wrap gap-1">
              {spans.map((span) => (
                <button
                  key={span.spanId}
                  type="button"
                  onClick={() => onSelectSpan(span.spanId)}
                  className="rounded-full border border-border/70 px-2 py-1 font-mono text-[11px] hover:bg-muted"
                >
                  {span.spanId.slice(0, 8)}
                </button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>
    )
  }

  const attributes = parseKeyValues(selectedSpan.attributes)
  const resource = parseKeyValues(selectedSpan.resource)
  const links = selectedSpan.typedLinks
  const events = parseEvents(selectedSpan.events)
  const spanLogs = logs.filter((log) => log.spanId === selectedSpan.spanId)
  const dbQuery = valueFor(attributes, "db.query.text")
  const stacktrace =
    valueFor(attributes, "exception.stacktrace") ??
    events
      .map((event) =>
        event.attributes
          ? String(event.attributes["exception.stacktrace"] ?? "")
          : ""
      )
      .find(Boolean) ??
    null

  return (
    <Card className="h-fit xl:sticky xl:top-4">
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-2 text-sm">
          <span className="min-w-0 truncate">{selectedSpan.name}</span>
          <span className="flex shrink-0 items-center gap-1">
            <Badge
              variant={
                selectedSpan.statusCode === "STATUS_CODE_ERROR"
                  ? "rose"
                  : "secondary"
              }
            >
              {selectedSpan.statusCode.replace("STATUS_CODE_", "") || "UNSET"}
            </Badge>
            <CopyButton value={selectedSpan.spanId} />
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 text-xs">
        {selectedSpan.statusCode === "STATUS_CODE_ERROR" ? (
          <TraceErrorCallout
            statusMessage={selectedSpan.statusMessage}
            grpcStatusCode={valueFor(attributes, "rpc.grpc.status_code")}
          />
        ) : null}

        <KeyValueList
          entries={[
            [
              "service",
              <Link
                key={selectedSpan.service}
                to="/services/$service"
                params={{ service: selectedSpan.service }}
                search={rangeSearch}
                className="font-mono underline underline-offset-4"
              >
                {selectedSpan.service}
              </Link>,
            ],
            ["kind", selectedSpan.kind.replace("SPAN_KIND_", "")],
            ["duration", formatDurationNs(selectedSpan.durationNs)],
            ["start", formatDateTime(selectedSpan.tsNanos)],
            ["span id", selectedSpan.spanId],
            ["parent id", selectedSpan.parentSpanId ?? "-"],
          ]}
        />

        {dbQuery ? (
          <InspectorCode title="db.query.text" value={dbQuery} copy />
        ) : null}

        {events.length > 0 ? (
          <InspectorSection title={`Events (${events.length})`}>
            <InspectorEventList key={selectedSpan.spanId} events={events} />
          </InspectorSection>
        ) : null}

        <InspectorSection title={`Attributes (${attributes.length})`}>
          {attributes.length > 0 ? (
            <KeyValueList entries={attributes} />
          ) : (
            <p className="text-muted-foreground">No attributes.</p>
          )}
        </InspectorSection>

        <InspectorSection title={`Resource (${resource.length})`}>
          {resource.length > 0 ? (
            <KeyValueList entries={resource} />
          ) : (
            <p className="text-muted-foreground">No resource attributes.</p>
          )}
        </InspectorSection>

        {links.length > 0 ? (
          <InspectorSection title={`Links (${links.length})`}>
            <InspectorLinksList
              key={selectedSpan.spanId}
              links={links}
              linkedTraceById={linkedTraceById}
              rangeSearch={rangeSearch}
            />
            <details className="mt-2 rounded-lg border border-border/70 bg-background/60 p-2">
              <summary className="cursor-pointer text-muted-foreground">
                Raw links JSON
              </summary>
              <pre className="mt-2 overflow-x-auto font-mono text-[11px] whitespace-pre-wrap">
                {selectedSpan.links}
              </pre>
            </details>
          </InspectorSection>
        ) : null}

        {spanLogs.length > 0 ? (
          <InspectorSection title={`Logs (${spanLogs.length})`}>
            <ul className="space-y-1.5">
              {spanLogs.map((log, index) => (
                <li
                  key={`${log.tsNanos}-${index}`}
                  className="rounded-lg border border-border/70 bg-background/60 p-2"
                >
                  <div className="mb-1 flex items-center justify-between gap-2">
                    <Badge
                      variant={
                        log.severityText === "ERROR" ? "rose" : "secondary"
                      }
                    >
                      {log.severityText || "log"}
                    </Badge>
                    <span className="text-muted-foreground tabular-nums">
                      {formatDateTime(log.tsNanos)}
                    </span>
                  </div>
                  <p className="font-mono break-words">{log.body}</p>
                </li>
              ))}
            </ul>
          </InspectorSection>
        ) : null}

        {stacktrace ? (
          <InspectorCode title="exception.stacktrace" value={stacktrace} copy />
        ) : null}
      </CardContent>
    </Card>
  )
}

function InspectorSection({
  title,
  children,
}: {
  title: string
  children: React.ReactNode
}) {
  return (
    <div>
      <Separator className="my-2" />
      <p className="mb-1 font-medium text-muted-foreground">{title}</p>
      {children}
    </div>
  )
}

function InspectorCode({
  title,
  value,
  copy = false,
}: {
  title: string
  value: string
  copy?: boolean
}) {
  return (
    <div>
      <div className="mb-1 flex items-center justify-between gap-2">
        <p className="font-medium text-muted-foreground">{title}</p>
        {copy ? <CopyButton value={value} /> : null}
      </div>
      <pre className="max-h-60 overflow-auto rounded-lg border border-border/70 bg-background/70 p-2 font-mono text-[11px] whitespace-pre-wrap">
        {value}
      </pre>
    </div>
  )
}

function KeyValueList({ entries }: { entries: KeyValues }) {
  return (
    <dl className="grid grid-cols-[7.5rem_minmax(0,1fr)] gap-x-3 gap-y-1">
      {entries.map(([key, value]) => (
        <div key={key} className="contents">
          <dt className="min-w-0 truncate font-mono text-muted-foreground">
            {key}
          </dt>
          <dd className="min-w-0 font-mono break-words">{value}</dd>
        </div>
      ))}
    </dl>
  )
}
