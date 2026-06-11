import { Link, createFileRoute } from "@tanstack/react-router"
import { Bar, BarChart, XAxis } from "recharts"
import { graphql, relativeTime } from "@/lib/api"
import type { ErrorEvent, Issue } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"

type IssueDetail = Issue & { events: ErrorEvent[] }

interface TrendPoint {
  tsNanos: string
  count: number
}

export const Route = createFileRoute("/issues/$fingerprint")({
  loader: ({ params }) =>
    graphql<{ issue: IssueDetail | null; issueTrend: TrendPoint[] }>(
      `{ issue(fingerprint: "${params.fingerprint}") {
           fingerprint title errorType culprit service status
           firstSeenNanos lastSeenNanos eventCount lastTraceId
           events(limit: 20) { tsNanos message stacktrace source traceId spanId attributes }
         }
         issueTrend(fingerprint: "${params.fingerprint}") { tsNanos count } }`
    ),
  component: IssueDetailPage,
})

const trendConfig = {
  count: { label: "events", color: "var(--chart-1)" },
} satisfies ChartConfig

function TrendSparkline({ trend }: { trend: TrendPoint[] }) {
  if (trend.length === 0) return null
  const data = trend.map((p) => ({
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
          Trend{" "}
          <span className="font-normal text-muted-foreground">(last 24h)</span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={trendConfig} className="h-24 w-full">
          <BarChart data={data} margin={{ left: 0, right: 0, top: 4 }}>
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

function IssueDetailPage() {
  const { issue, issueTrend } = Route.useLoaderData()
  if (!issue) {
    return <p className="text-sm text-muted-foreground">Issue not found.</p>
  }
  const latest = issue.events[0]
  return (
    <div className="space-y-4">
      <div className="space-y-1">
        <h1 className="text-lg font-semibold">{issue.title}</h1>
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <Badge
            variant={issue.status === "open" ? "destructive" : "secondary"}
          >
            {issue.status}
          </Badge>
          <Badge variant="outline">{issue.service}</Badge>
          <span className="text-muted-foreground">
            {issue.eventCount} events · first{" "}
            {relativeTime(issue.firstSeenNanos)} · last{" "}
            {relativeTime(issue.lastSeenNanos)}
          </span>
        </div>
        {issue.culprit ? (
          <code className="text-xs text-muted-foreground">{issue.culprit}</code>
        ) : null}
      </div>

      <TrendSparkline trend={issueTrend} />

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
              <pre className="overflow-auto rounded-md bg-muted p-3 text-xs leading-relaxed">
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

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Occurrences</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-2 text-sm">
            {issue.events.map((event) => (
              <li
                key={`${event.tsNanos}-${event.spanId}`}
                className="flex items-center justify-between gap-4 border-b pb-2 last:border-b-0"
              >
                <span className="truncate">{event.message}</span>
                <span className="shrink-0 text-xs text-muted-foreground">
                  {relativeTime(event.tsNanos)}
                </span>
              </li>
            ))}
          </ul>
        </CardContent>
      </Card>
    </div>
  )
}
