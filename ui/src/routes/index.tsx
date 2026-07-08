import { useMemo, useState } from "react"
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import {
  IconActivity,
  IconAlertTriangleFilled,
  IconAffiliateFilled,
  IconGaugeFilled,
  IconNotes,
} from "@tabler/icons-react"
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from "recharts"

import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import { ScrollFade } from "@/components/console/scroll-fade"
import {
  StatCard,
  CardSparkline,
  PillMeter,
} from "@/components/console/stat-card"
import {
  ChartLegend,
  makeEdgeTick,
  thinTicks,
} from "@/components/console/trend"
import { PageHeader } from "@/components/page-header"
import { navItem } from "@/components/nav"
import { Badge } from "@/components/ui/badge"
import { buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { graphql } from "@/lib/api"
import {
  formatCount,
  formatDelta,
  formatDurationNs,
  formatPercent,
  formatTimeInRange,
} from "@/lib/format"
import {
  mergeRangeSearch,
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface OverviewTotals {
  spanCount: string
  traceCount: string
  logCount: string
  metricPointCount: string
  errorCount: string
  errorRate: number
  activeServices: number
}

interface SpanRed {
  rate: SeriesPoint[]
  errorRate: SeriesPoint[]
  p50: SeriesPoint[]
  p95: SeriesPoint[]
  p99: SeriesPoint[]
}

interface IssueRow {
  fingerprint: string
  title: string
  service: string
  lastSeenNanos: string
  eventCount: number
  status: string
}

interface TraceRow {
  traceId: string
  rootName: string
  service: string
  startNanos: string
  durationNs: string
  spanCount: number
  hasError: boolean
}

export interface OverviewData {
  overview: OverviewTotals
  previousOverview: OverviewTotals
  spansSeries: SeriesPoint[]
  errorsSeries: SeriesPoint[]
  red: SpanRed
  previousRed: SpanRed
  issues: { items: IssueRow[] }
  tracesPage: { items: TraceRow[] }
}

type VisibleSeries = "all" | "spans" | "errors"
type VisibleLatency = "all" | "p50" | "p95" | "p99"

export const Route = createFileRoute("/")({
  validateSearch: (search: Record<string, unknown>) =>
    rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadOverview(resolveRangeSearch(deps)),
  component: OverviewPage,
})

function previousRange(range: ResolvedRange): ResolvedRange {
  const from = BigInt(range.fromNanos)
  const to = BigInt(range.toNanos)
  const span = to - from
  return {
    key: "previous",
    fromNanos: (from - span).toString(),
    toNanos: from.toString(),
  }
}

export function stepSecondsForRange(range: ResolvedRange): number {
  const spanNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  const seconds = Number(spanNs / 1_000_000_000n)
  return Math.max(30, Math.round(seconds / 60))
}

export async function loadOverview(range: ResolvedRange) {
  const previous = previousRange(range)
  const stepSeconds = stepSecondsForRange(range)
  const previousStepSeconds = stepSecondsForRange(previous)
  return graphql<OverviewData>(`
    {
      overview(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        spanCount traceCount logCount metricPointCount errorCount errorRate activeServices
      }
      previousOverview: overview(fromNanos: "${previous.fromNanos}", toNanos: "${previous.toNanos}") {
        spanCount traceCount logCount metricPointCount errorCount errorRate activeServices
      }
      spansSeries: signalCountSeries(kind: SPANS, fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        tsNanos value
      }
      errorsSeries: signalCountSeries(kind: ERRORS, fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        tsNanos value
      }
      red: serviceRed(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        rate { tsNanos value }
        errorRate { tsNanos value }
        p50 { tsNanos value }
        p95 { tsNanos value }
        p99 { tsNanos value }
      }
      previousRed: serviceRed(fromNanos: "${previous.fromNanos}", toNanos: "${previous.toNanos}", stepSeconds: ${previousStepSeconds}) {
        rate { tsNanos value }
        errorRate { tsNanos value }
        p50 { tsNanos value }
        p95 { tsNanos value }
        p99 { tsNanos value }
      }
      issues(sort: LAST_SEEN, limit: 6) {
        items { fingerprint title service lastSeenNanos eventCount status }
      }
      tracesPage(sort: DURATION_DESC, limit: 6) {
        items { traceId rootName service startNanos durationNs spanCount hasError }
      }
    }
  `)
}

export function latencyBands(red: SpanRed) {
  const pointsByTs = new Map<
    string,
    { tsNanos: string; p50: number; p95: number; p99: number }
  >()
  for (const point of red.p50) {
    pointsByTs.set(point.tsNanos, {
      tsNanos: point.tsNanos,
      p50: point.value,
      p95: point.value,
      p99: point.value,
    })
  }
  for (const point of red.p95) {
    const row = pointsByTs.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p95 = point.value
    pointsByTs.set(point.tsNanos, row)
  }
  for (const point of red.p99) {
    const row = pointsByTs.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p99 = point.value
    pointsByTs.set(point.tsNanos, row)
  }
  return Array.from(pointsByTs.values())
    .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    .map((point) => ({
      ...point,
      p50Band: Math.max(point.p50, 0),
      p95Band: Math.max(point.p95 - point.p50, 0),
      p99Band: Math.max(point.p99 - point.p95, 0),
    }))
}

function latestValue(points: SeriesPoint[]): number | null {
  return (
    [...points].reverse().find((point) => Number.isFinite(point.value))
      ?.value ?? null
  )
}

function count(value: string): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : 0
}

function msToNs(value: number | null): string {
  if (value == null || !Number.isFinite(value)) return "0"
  return Math.max(0, Math.round(value * 1_000_000)).toString()
}

function OverviewPage() {
  const data = Route.useLoaderData()
  const range = resolveRangeSearch(Route.useSearch())
  const navigate = useNavigate({ from: "/" })

  return (
    <OverviewContent
      data={data}
      range={range}
      onRangeChange={(next) =>
        void navigate({
          search: (current) => mergeRangeSearch(current, next),
        })
      }
    />
  )
}

export function OverviewContent({
  data,
  range,
  onRangeChange,
}: {
  data: OverviewData
  range: ResolvedRange
  onRangeChange: (range: ResolvedRange) => void
}) {
  const [visibleSeries, setVisibleSeries] = useState<VisibleSeries>("all")
  const [visibleLatency, setVisibleLatency] = useState<VisibleLatency>("all")
  const p95 = latestValue(data.red.p95)
  const previousP95 = latestValue(data.previousRed.p95)
  const p50 = latestValue(data.red.p50)
  const overviewItem = navItem("/")
  const hasData = count(data.overview.spanCount) > 0

  if (!hasData) {
    return (
      <div className="flex flex-col gap-4">
        <PageHeader
          {...(overviewItem
            ? {
                icon: overviewItem.activeIcon,
                iconClassName: overviewItem.iconClassName,
              }
            : {})}
          title="Overview"
          description="Telemetry volume, health, latency, and recent activity."
          actions={<RangePicker value={range} onChange={onRangeChange} />}
        />
        <OnboardingCard />
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        {...(overviewItem
          ? {
              icon: overviewItem.activeIcon,
              iconClassName: overviewItem.iconClassName,
            }
          : {})}
        title="Overview"
        description="Telemetry volume, health, latency, and recent activity."
        actions={<RangePicker value={range} onChange={onRangeChange} />}
      />

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <Link
          to="/traces"
          search={rangeLinkSearch(range)}
          className="block rounded-lg outline-none transition hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
        >
          <StatCard
            label="Spans"
            value={formatCount(count(data.overview.spanCount))}
            hint={`${formatCount(count(data.overview.traceCount))} traces`}
            delta={formatDelta(
              count(data.overview.spanCount),
              count(data.previousOverview.spanCount)
            )}
            icon={IconAffiliateFilled}
            iconClassName="text-sky-500"
            chart={
              <CardSparkline
                data={data.spansSeries.map((point) => ({ value: point.value }))}
                className="text-sky-500"
              />
            }
          />
        </Link>
        <Link
          to="/logs"
          search={rangeLinkSearch(range)}
          className="block rounded-lg outline-none transition hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
        >
          <StatCard
            label="Logs"
            value={formatCount(count(data.overview.logCount))}
            hint={`${data.overview.activeServices.toLocaleString()} services`}
            delta={formatDelta(
              count(data.overview.logCount),
              count(data.previousOverview.logCount)
            )}
            icon={IconNotes}
            iconClassName="text-blue-500"
            chart={
              <CardSparkline
                data={data.red.rate.map((point) => ({ value: point.value }))}
                className="text-blue-500"
              />
            }
          />
        </Link>
        <Link
          to="/issues"
          search={{ status: "open", ...rangeLinkSearch(range) }}
          className="block rounded-lg outline-none transition hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
        >
          <StatCard
            label="Error rate"
            value={formatPercent(data.overview.errorRate)}
            hint={`${formatCount(count(data.overview.errorCount))} errors`}
            delta={formatDelta(
              data.overview.errorRate,
              data.previousOverview.errorRate
            )}
            deltaInverted
            icon={IconAlertTriangleFilled}
            iconClassName="text-rose-500"
            chart={<PillMeter value={data.overview.errorRate} />}
          />
        </Link>
        <Link
          to="/traces"
          search={{ sort: "DURATION_DESC", ...rangeLinkSearch(range) }}
          className="block rounded-lg outline-none transition hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
        >
          <StatCard
            label="p95 latency"
            value={formatDurationNs(msToNs(p95))}
            hint={`p50 ${formatDurationNs(msToNs(p50))}`}
            delta={formatDelta(p95 ?? 0, previousP95 ?? 0)}
            deltaInverted
            icon={IconGaugeFilled}
            iconClassName="text-violet-500"
            chart={
              <CardSparkline
                data={data.red.p95.map((point) => ({ value: point.value }))}
                className="text-violet-500"
              />
            }
          />
        </Link>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <SignalTrendCard
          range={range}
          spans={data.spansSeries}
          errors={data.errorsSeries}
          visible={visibleSeries}
          onVisibleChange={setVisibleSeries}
        />
        <LatencyTrendCard
          range={range}
          red={data.red}
          visible={visibleLatency}
          onVisibleChange={setVisibleLatency}
        />
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <RecentIssuesCard issues={data.issues.items} />
        <SlowestTracesCard traces={data.tracesPage.items} />
      </section>
    </div>
  )
}

const signalConfig = {
  spans: { label: "Spans", color: "var(--chart-2)" },
  errors: { label: "Errors", color: "var(--destructive)" },
} satisfies ChartConfig

function SignalTrendCard({
  range,
  spans,
  errors,
  visible,
  onVisibleChange,
}: {
  range: ResolvedRange
  spans: SeriesPoint[]
  errors: SeriesPoint[]
  visible: VisibleSeries
  onVisibleChange: (value: VisibleSeries) => void
}) {
  const chartData = mergeSignalSeries(range, spans, errors)
  const empty = spans.length === 0 && errors.length === 0
  const displayData = empty ? sampleSignalData(range) : chartData
  const ticks = thinTicks(
    displayData.map((point) => point.time),
    6
  )

  return (
    <Card size="sm">
      <CardHeader className="gap-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle>Spans & errors</CardTitle>
          <div className="flex flex-wrap items-center gap-2">
            <Link
              to="/traces"
              search={rangeLinkSearch(range)}
              className={buttonVariants({ variant: "ghost", size: "xs" })}
            >
              View traces
            </Link>
            <Link
              to="/issues"
              search={{ status: "open", ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "ghost", size: "xs" })}
            >
              View issues
            </Link>
            <ChartLegend
              {...(visible === "all" ? {} : { selected: visible })}
              onSelect={(key) =>
                onVisibleChange(
                  visible === key ? "all" : (key as VisibleSeries)
                )
              }
              items={[
                { key: "spans", label: "Spans", color: "var(--chart-2)" },
                {
                  key: "errors",
                  label: "Errors",
                  color: "var(--destructive)",
                },
              ]}
            />
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="relative h-[260px]">
          <ChartContainer
            config={signalConfig}
            className={cn("h-full w-full", empty && "opacity-50 blur-[3px]")}
          >
            <AreaChart
              data={displayData}
              margin={{ left: 0, right: 8, top: 8 }}
            >
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="time"
                ticks={ticks}
                tickFormatter={(value: string, index) =>
                  makeEdgeTick(value, index, ticks)
                }
                tickLine={false}
                axisLine={false}
              />
              <YAxis tickLine={false} axisLine={false} width={44} />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Area
                dataKey="spans"
                type="monotone"
                stroke="var(--color-spans)"
                fill="var(--color-spans)"
                fillOpacity={0.18}
                strokeOpacity={visible === "errors" ? 0.3 : 1}
                opacity={visible === "errors" ? 0.3 : 1}
              />
              <Area
                dataKey="errors"
                type="monotone"
                stroke="var(--color-errors)"
                fill="var(--color-errors)"
                fillOpacity={0.2}
                strokeOpacity={visible === "spans" ? 0.3 : 1}
                opacity={visible === "spans" ? 0.3 : 1}
              />
            </AreaChart>
          </ChartContainer>
          {empty ? <EmptyChartOverlay /> : null}
        </div>
      </CardContent>
    </Card>
  )
}

const latencyConfig = {
  p50Band: { label: "p50", color: "var(--chart-2)" },
  p95Band: { label: "p95", color: "var(--chart-4)" },
  p99Band: { label: "p99", color: "var(--chart-5)" },
} satisfies ChartConfig

function LatencyTrendCard({
  range,
  red,
  visible,
  onVisibleChange,
}: {
  range: ResolvedRange
  red: SpanRed
  visible: VisibleLatency
  onVisibleChange: (value: VisibleLatency) => void
}) {
  const bands = latencyBands(red)
  const empty = bands.length === 0
  const displayData = (empty ? sampleLatencyData(range) : bands).map(
    (point) => ({
      ...point,
      time: formatTimeInRange(point.tsNanos, range),
    })
  )
  const ticks = thinTicks(
    displayData.map((point) => point.time),
    6
  )

  return (
    <Card size="sm">
      <CardHeader className="gap-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle>Latency</CardTitle>
          <div className="flex flex-wrap items-center gap-2">
            <Link
              to="/traces"
              search={{ sort: "DURATION_DESC", ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "ghost", size: "xs" })}
            >
              View traces
            </Link>
            <ChartLegend
              {...(visible === "all" ? {} : { selected: visible })}
              onSelect={(key) =>
                onVisibleChange(
                  visible === key ? "all" : (key as VisibleLatency)
                )
              }
              items={[
                { key: "p50", label: "p50", color: "var(--chart-2)" },
                { key: "p95", label: "p95", color: "var(--chart-4)" },
                { key: "p99", label: "p99", color: "var(--chart-5)" },
              ]}
            />
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="relative h-[260px]">
          <ChartContainer
            config={latencyConfig}
            className={cn("h-full w-full", empty && "opacity-50 blur-[3px]")}
          >
            <AreaChart
              data={displayData}
              margin={{ left: 0, right: 8, top: 8 }}
            >
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="time"
                ticks={ticks}
                tickFormatter={(value: string, index) =>
                  makeEdgeTick(value, index, ticks)
                }
                tickLine={false}
                axisLine={false}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                width={52}
                tickFormatter={(value: number) =>
                  formatDurationNs(msToNs(value))
                }
              />
              <ChartTooltip
                content={
                  <ChartTooltipContent
                    formatter={(_, name, item) => {
                      const payload = item.payload as {
                        p50: number
                        p95: number
                        p99: number
                      }
                      const key = String(name).replace("Band", "")
                      const value = payload[key as "p50" | "p95" | "p99"]
                      return (
                        <div className="flex w-full items-center justify-between gap-4">
                          <span className="text-muted-foreground">{key}</span>
                          <span className="font-mono tabular-nums">
                            {formatDurationNs(msToNs(value))}
                          </span>
                        </div>
                      )
                    }}
                  />
                }
              />
              <Area
                dataKey="p50Band"
                stackId="latency"
                type="monotone"
                stroke="var(--color-p50Band)"
                fill="var(--color-p50Band)"
                fillOpacity={0.35}
                opacity={visible !== "all" && visible !== "p50" ? 0.3 : 1}
              />
              <Area
                dataKey="p95Band"
                stackId="latency"
                type="monotone"
                stroke="var(--color-p95Band)"
                fill="var(--color-p95Band)"
                fillOpacity={0.28}
                opacity={visible !== "all" && visible !== "p95" ? 0.3 : 1}
              />
              <Area
                dataKey="p99Band"
                stackId="latency"
                type="monotone"
                stroke="var(--color-p99Band)"
                fill="var(--color-p99Band)"
                fillOpacity={0.22}
                opacity={visible !== "all" && visible !== "p99" ? 0.3 : 1}
              />
            </AreaChart>
          </ChartContainer>
          {empty ? <EmptyChartOverlay /> : null}
        </div>
      </CardContent>
    </Card>
  )
}

function mergeSignalSeries(
  range: ResolvedRange,
  spans: SeriesPoint[],
  errors: SeriesPoint[]
) {
  const rows = new Map<
    string,
    { tsNanos: string; spans: number; errors: number }
  >()
  for (const point of spans) {
    rows.set(point.tsNanos, {
      tsNanos: point.tsNanos,
      spans: point.value,
      errors: 0,
    })
  }
  for (const point of errors) {
    const row = rows.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      spans: 0,
      errors: 0,
    }
    row.errors = point.value
    rows.set(point.tsNanos, row)
  }
  return Array.from(rows.values())
    .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    .map((point) => ({
      ...point,
      time: formatTimeInRange(point.tsNanos, range),
    }))
}

function sampleSignalData(range: ResolvedRange) {
  return sampleTimes(range).map((tsNanos, index) => ({
    tsNanos,
    time: formatTimeInRange(tsNanos, range),
    spans: [12, 18, 14, 28, 22, 32][index] ?? 0,
    errors: [0, 1, 0, 2, 1, 0][index] ?? 0,
  }))
}

function sampleLatencyData(range: ResolvedRange) {
  return sampleTimes(range).map((tsNanos, index) => {
    const p50 = [18, 21, 19, 24, 22, 20][index] ?? 20
    const p95 = p50 + ([38, 44, 40, 52, 48, 42][index] ?? 40)
    const p99 = p95 + ([40, 52, 44, 70, 58, 46][index] ?? 50)
    return {
      tsNanos,
      p50,
      p95,
      p99,
      p50Band: p50,
      p95Band: Math.max(p95 - p50, 0),
      p99Band: Math.max(p99 - p95, 0),
    }
  })
}

function sampleTimes(range: ResolvedRange) {
  const from = BigInt(range.fromNanos)
  const span = BigInt(range.toNanos) - from
  return Array.from({ length: 6 }, (_, index) =>
    (from + (span * BigInt(index)) / 5n).toString()
  )
}

function EmptyChartOverlay() {
  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
      <Badge variant="secondary">No data in this range</Badge>
    </div>
  )
}

function RecentIssuesCard({ issues }: { issues: IssueRow[] }) {
  return (
    <Card size="sm">
      <CardHeader>
        <CardTitle>Recent issues</CardTitle>
      </CardHeader>
      <CardContent>
        {issues.length === 0 ? (
          <EmptyState
            title="No recent issues"
            className="min-h-32 rounded-lg border border-dashed"
            icon={IconAlertTriangleFilled}
          />
        ) : (
          <ScrollFade className="max-h-88">
            <div className="divide-y divide-border/40">
              {issues.map((issue) => (
                <Link
                  key={issue.fingerprint}
                  to="/issues/$fingerprint"
                  params={{ fingerprint: issue.fingerprint }}
                  className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 py-3 text-sm hover:bg-muted/40"
                >
                  <div className="flex min-w-0 items-start gap-2">
                    <span
                      className={cn(
                        "mt-1.5 size-2 shrink-0 rounded-full",
                        issue.status === "open" ? "bg-rose-500" : "bg-muted"
                      )}
                    />
                    <div className="min-w-0">
                      <div className="truncate font-medium">{issue.title}</div>
                      <div className="text-xs text-muted-foreground">
                        {issue.service}
                      </div>
                    </div>
                  </div>
                  <div className="text-right text-xs text-muted-foreground">
                    <div className="font-mono text-foreground tabular-nums">
                      {formatCount(issue.eventCount)}
                    </div>
                    <RelativeTime nanos={issue.lastSeenNanos} />
                  </div>
                </Link>
              ))}
            </div>
          </ScrollFade>
        )}
      </CardContent>
    </Card>
  )
}

function SlowestTracesCard({ traces }: { traces: TraceRow[] }) {
  const values = traces.map((trace) => Number(trace.durationNs))
  const scale = useMemo(() => buildHeatScale(values), [values])
  return (
    <Card size="sm">
      <CardHeader>
        <CardTitle>Slowest traces</CardTitle>
      </CardHeader>
      <CardContent>
        {traces.length === 0 ? (
          <EmptyState
            title="No traces in range"
            className="min-h-32 rounded-lg border border-dashed"
            icon={IconActivity}
          />
        ) : (
          <ScrollFade className="max-h-88">
            <div className="divide-y divide-border/40">
              {traces.map((trace) => (
                <Link
                  key={trace.traceId}
                  to="/traces/$traceId"
                  params={{ traceId: trace.traceId }}
                  className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 py-3 text-sm hover:bg-muted/40"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium">{trace.rootName}</div>
                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                      <span>{trace.service}</span>
                      {trace.hasError ? (
                        <Badge variant="rose">error</Badge>
                      ) : null}
                    </div>
                  </div>
                  <div className="text-right text-xs text-muted-foreground">
                    <HeatCell value={Number(trace.durationNs)} scale={scale}>
                      {formatDurationNs(trace.durationNs)}
                    </HeatCell>
                    <div>{formatCount(trace.spanCount)} spans</div>
                  </div>
                </Link>
              ))}
            </div>
          </ScrollFade>
        )}
      </CardContent>
    </Card>
  )
}

function OnboardingCard() {
  const grpc = "http://127.0.0.1:4317"
  const http = "http://127.0.0.1:4318"
  const run = "parallax run -- <your command>"
  return (
    <Card>
      <CardHeader>
        <CardTitle>Send your first telemetry</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-3 text-sm md:grid-cols-3">
        {(
          [
            ["OTLP/gRPC", grpc],
            ["OTLP/HTTP", http],
            ["CLI", run],
          ] satisfies Array<[string, string]>
        ).map(([label, value]) => (
          <div
            key={label}
            className="rounded-lg border border-border/70 bg-background/60 p-3"
          >
            <div className="mb-1 text-xs font-medium text-muted-foreground">
              {label}
            </div>
            <div className="flex items-center justify-between gap-2">
              <code className="min-w-0 truncate font-mono text-xs">
                {value}
              </code>
              <CopyButton value={value} />
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
