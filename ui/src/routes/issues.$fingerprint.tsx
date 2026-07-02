import { useState } from "react"
import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { AlertTriangle, Activity, Clock3, Hash } from "lucide-react"
import { Bar, BarChart, XAxis } from "recharts"
import { graphql, gqlString, relativeTime } from "@/lib/api"
import type { ErrorEvent, Issue } from "@/lib/api"
import { KpiCard } from "@/components/kpi-card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { MetricStrip } from "@/components/metric-strip"
import { PageHeading } from "@/components/page-heading"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"

type IssueDetail = Issue & { tags: string; events: ErrorEvent[] }

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
  /** Run id carried by the latest event's trace, for run-scoped metrics. */
  traceRunId: string | null
}

export const Route = createFileRoute("/issues/$fingerprint")({
  loader: async ({ params }): Promise<LoaderData> => {
    const { issue, issueTrend } = await graphql<{
      issue: IssueDetail | null
      issueTrend: TrendPoint[]
    }>(
      `{ issue(fingerprint: "${params.fingerprint}") {
           fingerprint title errorType culprit service status
           firstSeenNanos lastSeenNanos eventCount lastTraceId tags
           events(limit: 20) { tsNanos message stacktrace source traceId spanId attributes }
         }
         issueTrend(fingerprint: "${params.fingerprint}") { tsNanos count } }`
    )
    // Context sections (runtime/OS/process/SDK) come from the resource of
    // the latest event's trace; breadcrumbs are that trace's logs.
    let resource: Record<string, unknown> = {}
    let breadcrumbs: BreadcrumbLog[] = []
    let traceRunId: string | null = null
    const traceId = issue?.lastTraceId
    if (traceId) {
      try {
        const correlated = await graphql<{
          trace: { spans: { resource: string; runId: string | null }[] } | null
          logsByTrace: BreadcrumbLog[]
        }>(
          `{ trace(traceId: "${gqlString(traceId)}") { spans { resource runId } }
             logsByTrace(traceId: "${gqlString(traceId)}") { tsNanos severityText body } }`
        )
        resource = JSON.parse(
          correlated.trace?.spans[0]?.resource ?? "{}"
        ) as Record<string, unknown>
        breadcrumbs = correlated.logsByTrace.slice(-12)
        traceRunId = correlated.trace?.spans.find((s) => s.runId)?.runId ?? null
      } catch {
        // Trace may have aged out; the issue page still renders.
      }
    }
    return { issue, issueTrend, resource, breadcrumbs, traceRunId }
  },
  component: IssueDetailPage,
})

const trendConfig = {
  count: { label: "events", color: "var(--chart-1)" },
} satisfies ChartConfig

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
  const data = trend.map((p) => ({
    tsNanos: p.tsNanos,
    time: new Date(Number(p.tsNanos) / 1e6).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    }),
    count: p.count,
  }))
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          Occurrence trend{" "}
          <span className="font-normal text-muted-foreground">
            (last 24h; click a bar to filter
            {activeBucket ? "; click again to clear" : ""})
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={trendConfig} className="h-24 w-full">
          <BarChart
            data={data}
            margin={{ left: 0, right: 0, top: 4 }}
            onClick={(state) => {
              const payloadState = state as {
                activePayload?: Array<{ payload?: { tsNanos?: unknown } }>
              }
              const ts = payloadState.activePayload?.[0]?.payload?.tsNanos as
                | string
                | undefined
              if (ts) onBucket(ts === activeBucket ? null : ts)
            }}
          >
            <XAxis
              dataKey="time"
              tickLine={false}
              axisLine={false}
              minTickGap={48}
            />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="count" fill="var(--color-count)" radius={2} />
          </BarChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

/** Section groups for the resource-attribute context card. */
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
                        ×{count}
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

function IssueDetailPage() {
  const { issue, issueTrend, resource, breadcrumbs, traceRunId } =
    Route.useLoaderData()
  const router = useRouter()
  const [mutating, setMutating] = useState(false)
  const [bucket, setBucket] = useState<string | null>(null)
  const [bucketEvents, setBucketEvents] = useState<ErrorEvent[] | null>(null)
  if (!issue) {
    return <p className="text-sm text-muted-foreground">Issue not found.</p>
  }
  const latest = issue.events[0]
  const shownEvents = bucketEvents ?? issue.events

  async function setStatus(status: "open" | "resolved") {
    if (!issue) return
    setMutating(true)
    try {
      await graphql(
        `mutation { issueSetStatus(fingerprint: "${gqlString(issue.fingerprint)}", status: "${status}") { status } }`
      )
      await router.invalidate()
    } finally {
      setMutating(false)
    }
  }

  async function filterBucket(tsNanos: string | null) {
    setBucket(tsNanos)
    if (!tsNanos || !issue) {
      setBucketEvents(null)
      return
    }
    // Trend buckets are hourly: fetch the occurrences inside the clicked one.
    const from = BigInt(tsNanos)
    const to = from + 3_600_000_000_000n
    const { issue: scoped } = await graphql<{
      issue: { events: ErrorEvent[] } | null
    }>(
      `{ issue(fingerprint: "${gqlString(issue.fingerprint)}") {
           events(limit: 50, fromNanos: "${from}", toNanos: "${to}") {
             tsNanos message stacktrace source traceId spanId attributes }
         } }`
    )
    setBucketEvents(scoped?.events ?? [])
  }

  return (
    <div className="space-y-4">
      <PageHeading
        eyebrow="Issue detail"
        title={issue.title}
        description={`${issue.service} · ${issue.culprit || "unknown culprit"}`}
        action={
          <Button
            size="sm"
            variant={issue.status === "open" ? "default" : "outline"}
            disabled={mutating}
            onClick={() =>
              setStatus(issue.status === "open" ? "resolved" : "open")
            }
          >
            {issue.status === "open" ? "Resolve" : "Reopen"}
          </Button>
        }
      />

      <div className="flex flex-wrap items-center gap-2">
        <Badge variant={issue.status === "open" ? "destructive" : "secondary"}>
          {issue.status}
        </Badge>
        <Badge variant="outline">{issue.service}</Badge>
        <span className="text-muted-foreground">
          {issue.eventCount} events · first {relativeTime(issue.firstSeenNanos)}{" "}
          · last {relativeTime(issue.lastSeenNanos)}
        </span>
      </div>

      <div className="grid gap-3 md:grid-cols-4">
        <KpiCard
          icon={AlertTriangle}
          label="Status"
          value={issue.status}
          detail={issue.errorType || "error"}
          tone={issue.status === "open" ? "rose" : "green"}
        />
        <KpiCard
          icon={Hash}
          label="Events"
          value={issue.eventCount.toLocaleString()}
          detail="total occurrences"
          tone="orange"
          bars={issueTrend.map((point) => point.count)}
        />
        <KpiCard
          icon={Clock3}
          label="Last seen"
          value={relativeTime(issue.lastSeenNanos)}
          detail="newest event"
          tone="blue"
        />
        <KpiCard
          icon={Activity}
          label="Trace"
          value={issue.lastTraceId ? "Linked" : "None"}
          detail={issue.lastTraceId?.slice(0, 12) ?? "no trace id"}
          tone="violet"
        />
      </div>

      <TrendChart
        trend={issueTrend}
        onBucket={filterBucket}
        activeBucket={bucket}
      />

      {latest ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Latest event · {relativeTime(latest.tsNanos)} ·{" "}
              <span className="font-normal text-muted-foreground">
                {latest.source}
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <p className="text-sm">{latest.message}</p>
            {latest.stacktrace ? (
              <pre className="max-h-96 overflow-auto rounded-md border border-border/70 bg-background/70 p-3 text-xs leading-relaxed">
                {latest.stacktrace}
              </pre>
            ) : null}
            {latest.traceId ? (
              <Link
                to="/traces/$traceId"
                params={{ traceId: latest.traceId }}
                className="text-sm underline underline-offset-4"
              >
                Open trace {latest.traceId.slice(0, 16)}…
              </Link>
            ) : null}
            <div className="text-xs text-muted-foreground">
              Agent handoff:{" "}
              <code>parallax issue context {issue.fingerprint}</code>
            </div>
          </CardContent>
        </Card>
      ) : null}

      {latest ? (
        <MetricStrip
          title="Metrics around the latest event"
          service={issue.service}
          runId={traceRunId ?? undefined}
          fromNanos={(BigInt(latest.tsNanos) - 300_000_000_000n).toString()}
          toNanos={(BigInt(latest.tsNanos) + 300_000_000_000n).toString()}
          stepSeconds={30}
        />
      ) : null}

      <TagsTable tags={issue.tags} />
      <ContextSections resource={resource} />

      {breadcrumbs.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Logs around the latest event
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-1 font-mono text-xs">
              {breadcrumbs.map((log, index) => (
                <li key={index} className="flex gap-2">
                  <span className="w-14 shrink-0 text-muted-foreground">
                    {log.severityText}
                  </span>
                  <span className="break-all">{log.body}</span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            Occurrences
            {bucket ? (
              <span className="ml-2 font-normal text-muted-foreground">
                in the selected hour ({shownEvents.length})
              </span>
            ) : null}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {shownEvents.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No occurrences in this window.
            </p>
          ) : (
            <ul className="space-y-2 text-sm">
              {shownEvents.map((event) => (
                <li
                  key={`${event.tsNanos}-${event.spanId}`}
                  className="flex items-center justify-between gap-4 rounded-xl border border-border/70 bg-background/60 px-3 py-2"
                >
                  <span className="truncate">{event.message}</span>
                  <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
                    {event.traceId ? (
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: event.traceId }}
                        className="underline underline-offset-4"
                      >
                        trace
                      </Link>
                    ) : null}
                    {relativeTime(event.tsNanos)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
