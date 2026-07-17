import { useMemo, useState } from "react"
import { Link, useNavigate } from "@tanstack/react-router"
import {
  IconChartLine,
  IconActivity,
  IconAlertTriangleFilled,
  IconAffiliateFilled,
  IconGaugeFilled,
  IconNotes,
  IconX,
} from "@tabler/icons-react"
import {
  Area,
  AreaChart,
  CartesianGrid,
  ReferenceArea,
  XAxis,
  YAxis,
} from "recharts"

import { CopyButton } from "@/components/console/copy-button"
import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { RangePicker } from "@/features/time-range"
import { RelativeTime } from "@/components/console/relative-time"
import { ScrollFade } from "@/components/console/scroll-fade"
import {
  StatCard,
  CardSparkline,
  PillMeter,
} from "@/components/console/stat-card"
import { TopMovers } from "@/components/console/top-movers"
import type { ServiceMoverInput } from "@/components/console/top-movers"
import {
  ChartLegend,
  makeEdgeTick,
  thinTicks,
} from "@/components/console/trend"
import { PageHeader } from "@/shared/components/page-header"
import { navItem } from "@/shared/navigation"
import { Badge } from "@/components/ui/badge"
import { Button, buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { graphqlCached } from "@/lib/api"
import type { Issue, TraceSummary } from "@/lib/api"
import {
  formatCount,
  formatDelta,
  formatDurationNs,
  formatPercent,
  formatTimeInRange,
} from "@/lib/format"
import {
  customRange,
  mergeRangeSearch,
  rangeLinkSearch,
  resolvePreset,
  resolveRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"
import { useChartBrush } from "@/components/console/use-chart-brush"
import {
  mergeSignalSeries,
  sampleLatencyData,
  sampleSignalData,
} from "@/lib/overview-chart-helpers"

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

type IssueRow = Pick<
  Issue,
  | "fingerprint"
  | "title"
  | "service"
  | "lastSeenNanos"
  | "eventCount"
  | "status"
>

type TraceRow = TraceSummary

export interface OverviewData {
  overview: OverviewTotals
  previousOverview: OverviewTotals
  spansSeries: SeriesPoint[]
  errorsSeries: SeriesPoint[]
  red: SpanRed
  previousRed: SpanRed
  servicesNow: ServiceMoverInput[]
  servicesPrev: ServiceMoverInput[]
  issues: { items: IssueRow[] }
  tracesPage: { items: TraceRow[] }
}

type VisibleSeries = "all" | "spans" | "errors"
type VisibleLatency = "all" | "p50" | "p95" | "p99"

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
  return graphqlCached<OverviewData>(`
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
      servicesNow: serviceList(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        name spanCount errorCount p95Ms
      }
      servicesPrev: serviceList(fromNanos: "${previous.fromNanos}", toNanos: "${previous.toNanos}") {
        name spanCount errorCount p95Ms
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

export function OverviewPage({
  data,
  search,
}: {
  data: OverviewData
  search: {
    range?: string | undefined
    from?: string | undefined
    to?: string | undefined
  }
}) {
  const range = resolveRangeSearch(search)
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

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <Link
          to="/traces"
          search={rangeLinkSearch(range)}
          className="block rounded-lg transition outline-none hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
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
          className="block rounded-lg transition outline-none hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
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
          to="/metrics"
          className="block rounded-lg transition outline-none hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
        >
          <StatCard
            label="Metric points"
            value={formatCount(count(data.overview.metricPointCount))}
            hint="finite samples in window"
            delta={formatDelta(
              count(data.overview.metricPointCount),
              count(data.previousOverview.metricPointCount)
            )}
            icon={IconChartLine}
            iconClassName="text-emerald-500"
          />
        </Link>
        <Link
          to="/issues"
          search={{ status: "open", ...rangeLinkSearch(range) }}
          className="block rounded-lg transition outline-none hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
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
          className="block rounded-lg transition outline-none hover:-translate-y-0.5 focus-visible:ring-[1.5px] focus-visible:ring-ring/50"
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

      <TopMovers
        now={data.servicesNow}
        previous={data.servicesPrev}
        range={range}
      />

      <section className="grid gap-4 lg:grid-cols-2">
        <SignalTrendCard
          range={range}
          spans={data.spansSeries}
          errors={data.errorsSeries}
          visible={visibleSeries}
          onVisibleChange={setVisibleSeries}
          onRangeChange={onRangeChange}
        />
        <LatencyTrendCard
          range={range}
          red={data.red}
          visible={visibleLatency}
          onVisibleChange={setVisibleLatency}
          onRangeChange={onRangeChange}
        />
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <RecentIssuesCard issues={data.issues.items} range={range} />
        <SlowestTracesCard traces={data.tracesPage.items} range={range} />
      </section>
    </div>
  )
}

const signalConfig = {
  spans: { label: "Spans", color: "var(--chart-throughput)" },
  errors: { label: "Errors", color: "var(--chart-error)" },
} satisfies ChartConfig

function SignalTrendCard({
  range,
  spans,
  errors,
  visible,
  onVisibleChange,
  onRangeChange,
}: {
  range: ResolvedRange
  spans: SeriesPoint[]
  errors: SeriesPoint[]
  visible: VisibleSeries
  onVisibleChange: (value: VisibleSeries) => void
  onRangeChange: (range: ResolvedRange) => void
}) {
  const chartData = mergeSignalSeries(range, spans, errors)
  const empty = spans.length === 0 && errors.length === 0
  const displayData = empty ? sampleSignalData(range) : chartData
  const brush = useChartBrush({
    series: chartData,
    stepSeconds: stepSecondsForRange(range),
    disabled: empty,
    onWindow: (fromNanos, toNanos) =>
      onRangeChange(customRange(fromNanos, toNanos)),
    getReferenceValue: (point) => point.time,
  })
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
                {
                  key: "spans",
                  label: "Spans",
                  color: "var(--chart-throughput)",
                },
                {
                  key: "errors",
                  label: "Errors",
                  color: "var(--destructive)",
                },
              ]}
            />
            {range.key === "custom" ? (
              <Button
                type="button"
                variant="ghost"
                size="xs"
                onClick={() => onRangeChange(resolvePreset())}
              >
                <IconX />
                Reset window
              </Button>
            ) : null}
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
              {...brush.chartHandlers}
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
              {brush.referenceRange ? (
                <ReferenceArea
                  x1={brush.referenceRange.x1}
                  x2={brush.referenceRange.x2}
                  fill="var(--muted-foreground)"
                  fillOpacity={0.12}
                />
              ) : null}
            </AreaChart>
          </ChartContainer>
          {empty ? <EmptyChartOverlay /> : null}
        </div>
      </CardContent>
    </Card>
  )
}

const latencyConfig = {
  p50Band: { label: "p50", color: "var(--chart-p50)" },
  p95Band: { label: "p95", color: "var(--chart-p95)" },
  p99Band: { label: "p99", color: "var(--chart-p99)" },
} satisfies ChartConfig

function LatencyTrendCard({
  range,
  red,
  visible,
  onVisibleChange,
  onRangeChange,
}: {
  range: ResolvedRange
  red: SpanRed
  visible: VisibleLatency
  onVisibleChange: (value: VisibleLatency) => void
  onRangeChange: (range: ResolvedRange) => void
}) {
  const bands = latencyBands(red)
  const empty = bands.length === 0
  const displayData = (empty ? sampleLatencyData(range) : bands).map(
    (point) => ({
      ...point,
      time: formatTimeInRange(point.tsNanos, range),
    })
  )
  const brush = useChartBrush({
    series: empty ? [] : displayData,
    stepSeconds: stepSecondsForRange(range),
    disabled: empty,
    onWindow: (fromNanos, toNanos) =>
      onRangeChange(customRange(fromNanos, toNanos)),
    getReferenceValue: (point) => point.time,
  })
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
                { key: "p50", label: "p50", color: "var(--chart-p50)" },
                { key: "p95", label: "p95", color: "var(--chart-p95)" },
                { key: "p99", label: "p99", color: "var(--chart-p99)" },
              ]}
            />
            {range.key === "custom" ? (
              <Button
                type="button"
                variant="ghost"
                size="xs"
                onClick={() => onRangeChange(resolvePreset())}
              >
                <IconX />
                Reset window
              </Button>
            ) : null}
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
              {...brush.chartHandlers}
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
              {brush.referenceRange ? (
                <ReferenceArea
                  x1={brush.referenceRange.x1}
                  x2={brush.referenceRange.x2}
                  fill="var(--muted-foreground)"
                  fillOpacity={0.12}
                />
              ) : null}
            </AreaChart>
          </ChartContainer>
          {empty ? <EmptyChartOverlay /> : null}
        </div>
      </CardContent>
    </Card>
  )
}

function EmptyChartOverlay() {
  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
      <Badge variant="secondary">No data in this range</Badge>
    </div>
  )
}

function RecentIssuesCard({
  issues,
  range,
}: {
  issues: IssueRow[]
  range: ResolvedRange
}) {
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
                  search={rangeLinkSearch(range)}
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

function SlowestTracesCard({
  traces,
  range,
}: {
  traces: TraceRow[]
  range: ResolvedRange
}) {
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
                  search={rangeLinkSearch(range)}
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
