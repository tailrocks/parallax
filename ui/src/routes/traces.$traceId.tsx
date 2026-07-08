import { useMemo, useState } from "react"
import type { ReactNode } from "react"
import { createFileRoute, Link } from "@tanstack/react-router"
import {
  IconAlertTriangle,
  IconAffiliate,
  IconArticle,
  IconClock,
  IconExternalLink,
  IconHash,
  IconServer,
} from "@tabler/icons-react"

import {
  TraceWaterfall,
  WHOLE_TRACE_ID,
} from "@/components/console/trace-waterfall"
import type { WaterfallSpan } from "@/components/console/trace-waterfall"
import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { PageHeader } from "@/components/page-header"
import { MetricStrip } from "@/components/metric-strip"
import { navItem } from "@/components/nav"
import { Badge } from "@/components/ui/badge"
import { buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { graphql, gqlString } from "@/lib/api"
import { formatDateTime, formatDurationNs } from "@/lib/format"
import { computeWindow } from "@/lib/trace-tree"
import { cn } from "@/lib/utils"

interface TraceSpan extends WaterfallSpan {
  tsNanos: string
  traceId: string
  runId: string | null
  links: string
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

type JsonRecord = Record<string, unknown>
type KeyValues = Array<[string, ReactNode]>
type StringKeyValues = Array<[string, string]>

interface SpanLink {
  traceId: string
  spanId?: string
  attributes?: JsonRecord
}

interface SpanEvent {
  name: string
  timeUnixNano?: string
  attributes?: JsonRecord
}

export const Route = createFileRoute("/traces/$traceId")({
  loader: ({ params }) => {
    const traceId = gqlString(params.traceId)
    return graphql<{
      trace: { spans: TraceSpan[] } | null
      logsByTrace: TraceLog[]
    }>(
      `{ trace(traceId: "${traceId}") {
           spans { tsNanos service traceId name kind statusCode statusMessage durationNs
                   spanId parentSpanId runId links events attributes resource }
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

function parseLinks(json: string): SpanLink[] {
  try {
    const value: unknown = JSON.parse(json)
    return Array.isArray(value)
      ? value.filter(
          (link): link is SpanLink =>
            !!link && typeof link === "object" && "traceId" in link
        )
      : []
  } catch {
    return []
  }
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
  const { trace, logsByTrace } = Route.useLoaderData()
  const { traceId } = Route.useParams()
  const [selectedId, setSelectedId] = useState<string | null>(WHOLE_TRACE_ID)
  const orderedLogs = useMemo(
    () =>
      [...logsByTrace].sort((a, b) =>
        BigInt(a.tsNanos) < BigInt(b.tsNanos) ? 1 : -1
      ),
    [logsByTrace]
  )

  if (!trace || trace.spans.length === 0) {
    return (
      <EmptyState
        title="Trace not found"
        description={traceId}
        icon={IconAffiliate}
      />
    )
  }

  const spans = trace.spans
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
  const spanLinks = spans.flatMap((span) => parseLinks(span.links))
  const spanEvents = spans.flatMap((span) => parseEvents(span.events))
  const selectedSpan =
    selectedId && selectedId !== WHOLE_TRACE_ID
      ? (spans.find((span) => span.spanId === selectedId) ?? null)
      : null

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
                className="inline-flex"
              >
                <Badge variant="outline">{service}</Badge>
              </Link>
            ))}
          </span>
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

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_420px]">
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Waterfall</CardTitle>
            </CardHeader>
            <CardContent>
              <TraceWaterfall
                spans={spans}
                selectedId={selectedId}
                onSelect={setSelectedId}
              />
            </CardContent>
          </Card>

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
          logs={orderedLogs}
          onSelectSpan={setSelectedId}
        />
      </div>
    </div>
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

function TraceInspector({
  traceId,
  spans,
  selectedSpan,
  logs,
  onSelectSpan,
}: {
  traceId: string
  spans: TraceSpan[]
  selectedSpan: TraceSpan | null
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
  const links = parseLinks(selectedSpan.links)
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
          <div className="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-rose-700 dark:text-rose-300">
            {selectedSpan.statusMessage || "Span ended with error status."}
          </div>
        ) : null}

        <KeyValueList
          entries={[
            [
              "service",
              <Link
                key={selectedSpan.service}
                to="/services/$service"
                params={{ service: selectedSpan.service }}
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
            <ul className="space-y-2">
              {events.map((event, index) => (
                <li
                  key={`${event.name}-${index}`}
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
                          typeof value === "string"
                            ? value
                            : JSON.stringify(value),
                        ]
                      )}
                    />
                  ) : null}
                </li>
              ))}
            </ul>
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
            <ul className="space-y-1 font-mono">
              {links.map((link) => (
                <li key={`${link.traceId}-${link.spanId ?? ""}`}>
                  <Link
                    to="/traces/$traceId"
                    params={{ traceId: link.traceId }}
                    className="underline underline-offset-4"
                  >
                    {link.traceId}
                  </Link>
                </li>
              ))}
            </ul>
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
