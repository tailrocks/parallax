import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  IconActivityHeartbeat,
  IconAffiliate,
  IconAlertTriangleFilled,
  IconGaugeFilled,
  IconServer,
} from "@tabler/icons-react"
import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
  XAxis,
  YAxis,
} from "recharts"

import { EmptyState } from "@/components/console/empty-state"
import { HeatCell } from "@/components/console/heat-cell"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import {
  CardSparkline,
  PillMeter,
  StatCard,
} from "@/components/console/stat-card"
import {
  ChartLegend,
  makeEdgeTick,
  thinTicks,
} from "@/components/console/trend"
import { navItem } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { gqlString, graphql } from "@/lib/api"
import type { TraceSummary } from "@/lib/api"
import { formatCount, formatDurationNs, formatPercent } from "@/lib/format"
import { rangeSearchSchema, resolveRangeSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface SpanRed {
  rate: SeriesPoint[]
  errorRate: SeriesPoint[]
  p50: SeriesPoint[]
  p95: SeriesPoint[]
  p99: SeriesPoint[]
}

interface ServiceOverview {
  cpu: SeriesPoint[]
  memory: SeriesPoint[]
  requestRate: SeriesPoint[]
  errorRate: SeriesPoint[]
  latencyP50: SeriesPoint[]
  latencyP95: SeriesPoint[]
  latencyP99: SeriesPoint[]
}

export interface ServiceDetailData {
  red: SpanRed
  overview: ServiceOverview
  tracesPage: { items: TraceSummary[] }
}

type MetricChartPoint = {
  label: string
  tsNanos: string
  [key: string]: string | number
}

const chartConfig = {
  requests: { label: "Requests", color: "var(--chart-1)" },
  errors: { label: "Errors", color: "var(--chart-5)" },
  p50Band: { label: "p50", color: "var(--chart-2)" },
  p95Band: { label: "p95", color: "var(--chart-3)" },
  p99Band: { label: "p99", color: "var(--chart-5)" },
  cpu: { label: "CPU", color: "var(--chart-1)" },
  memory: { label: "Memory", color: "var(--chart-2)" },
} satisfies ChartConfig

export const Route = createFileRoute("/services/$service")({
  validateSearch: (search: Record<string, unknown>) =>
    rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: ({ params, deps }) =>
    loadServiceDetail(params.service, resolveRangeSearch(deps)),
  component: ServiceDetailPage,
})

export function stepSecondsForRange(range: ResolvedRange): number {
  const spanNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  const seconds = Number(spanNs / 1_000_000_000n)
  return Math.max(30, Math.round(seconds / 60))
}

export async function loadServiceDetail(service: string, range: ResolvedRange) {
  const escaped = gqlString(service)
  const stepSeconds = stepSecondsForRange(range)
  return graphql<ServiceDetailData>(`
    {
      red: serviceRed(service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        rate { tsNanos value }
        errorRate { tsNanos value }
        p50 { tsNanos value }
        p95 { tsNanos value }
        p99 { tsNanos value }
      }
      overview: serviceOverview(service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        cpu { tsNanos value }
        memory { tsNanos value }
        requestRate { tsNanos value }
        errorRate { tsNanos value }
        latencyP50 { tsNanos value }
        latencyP95 { tsNanos value }
        latencyP99 { tsNanos value }
      }
      tracesPage(service: "${escaped}", sort: START_DESC, limit: 10, fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        items { traceId rootName service startNanos durationNs spanCount hasError }
      }
    }
  `)
}

export function totalSeries(points: SeriesPoint[]): number {
  return points.reduce((sum, point) => sum + point.value, 0)
}

function latestValue(points: SeriesPoint[]): number | null {
  return points.at(-1)?.value ?? null
}

export function latestErrorRate(red: SpanRed): number {
  return latestValue(red.errorRate) ?? 0
}

export function latencyBands(red: SpanRed) {
  const points = new Map<
    string,
    { tsNanos: string; p50: number; p95: number; p99: number }
  >()
  for (const point of red.p50) {
    points.set(point.tsNanos, {
      tsNanos: point.tsNanos,
      p50: point.value,
      p95: point.value,
      p99: point.value,
    })
  }
  for (const point of red.p95) {
    const row = points.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p95 = point.value
    points.set(point.tsNanos, row)
  }
  for (const point of red.p99) {
    const row = points.get(point.tsNanos) ?? {
      tsNanos: point.tsNanos,
      p50: 0,
      p95: 0,
      p99: 0,
    }
    row.p99 = point.value
    points.set(point.tsNanos, row)
  }
  return Array.from(points.values())
    .sort((a, b) => (BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1))
    .map((point) => ({
      ...point,
      label: formatChartTime(point.tsNanos),
      p50Band: Math.max(point.p50, 0),
      p95Band: Math.max(point.p95 - point.p50, 0),
      p99Band: Math.max(point.p99 - point.p95, 0),
    }))
}

function formatChartTime(tsNanos: string) {
  return new Date(Number(BigInt(tsNanos) / 1_000_000n)).toLocaleTimeString(
    undefined,
    { hour: "2-digit", minute: "2-digit" }
  )
}

function toLineData(
  series: Record<string, SeriesPoint[]>,
  mapValue: (key: string, value: number, tsNanos: string) => number = (
    _key,
    value
  ) => value
): MetricChartPoint[] {
  const rows = new Map<string, MetricChartPoint>()
  for (const [key, points] of Object.entries(series)) {
    for (const point of points) {
      const row = rows.get(point.tsNanos) ?? {
        tsNanos: point.tsNanos,
        label: formatChartTime(point.tsNanos),
      }
      row[key] = mapValue(key, point.value, point.tsNanos)
      rows.set(point.tsNanos, row)
    }
  }
  return Array.from(rows.values()).sort((a, b) =>
    BigInt(a.tsNanos) < BigInt(b.tsNanos) ? -1 : 1
  )
}

function ServiceDetailPage() {
  const data = Route.useLoaderData()
  const params = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const range = resolveRangeSearch(search)

  return (
    <ServiceDetailContent
      service={params.service}
      data={data}
      range={range}
      onRange={(next) =>
        navigate({
          search: { range: next.key },
        })
      }
    />
  )
}

export function ServiceDetailContent({
  service,
  data,
  range,
  onRange,
}: {
  service: string
  data: ServiceDetailData
  range: ResolvedRange
  onRange: (range: ResolvedRange) => void
}) {
  const hasRed =
    data.red.rate.length > 0 ||
    data.red.errorRate.length > 0 ||
    data.red.p95.length > 0
  const traces = data.tracesPage.items
  const noData = !hasRed && traces.length === 0
  const lastSeen = traces[0]?.startNanos
  const servicesBack = navItem("/services")!

  if (noData) {
    return (
      <div className="space-y-4">
        <PageHeader
          back={servicesBack}
          title={service}
          actions={<RangePicker value={range} onChange={onRange} />}
        />
        <EmptyState
          icon={IconServer}
          title="Service not found"
          description="No spans, errors, or metrics matched this service in the selected window."
        />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <PageHeader
        back={servicesBack}
        title={service}
        actions={<RangePicker value={range} onChange={onRange} />}
      />

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          icon={IconActivityHeartbeat}
          label="Requests"
          value={formatCount(Math.round(totalSeries(data.red.rate)))}
          hint="span-derived"
          chart={<CardSparkline data={data.red.rate} />}
        />
        <StatCard
          icon={IconAlertTriangleFilled}
          iconClassName="text-rose-500"
          label="Error rate"
          value={formatPercent(latestErrorRate(data.red))}
          hint={<PillMeter value={latestErrorRate(data.red)} />}
        />
        <StatCard
          icon={IconGaugeFilled}
          label="p95 latency"
          value={
            latestValue(data.red.p95) == null
              ? "-"
              : formatDurationNs((latestValue(data.red.p95) ?? 0) * 1_000_000)
          }
          hint={
            latestValue(data.red.p50) == null
              ? undefined
              : `p50 ${formatDurationNs((latestValue(data.red.p50) ?? 0) * 1_000_000)}`
          }
          chart={<CardSparkline data={data.red.p95} />}
        />
        <StatCard
          icon={IconAffiliate}
          label="Last seen"
          value={lastSeen ? <RelativeTime nanos={lastSeen} /> : <span>-</span>}
          hint={`${formatCount(traces.length)} recent traces`}
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <RequestsChart red={data.red} />
        <LatencyChart red={data.red} overview={data.overview} />
      </div>

      <InfraBand overview={data.overview} />
      <RecentTraces traces={traces} />
    </div>
  )
}

function RequestsChart({ red }: { red: SpanRed }) {
  const errorsByTs = new Map(red.errorRate.map((p) => [p.tsNanos, p.value]))
  const data = toLineData(
    {
      requests: red.rate,
      errors: red.rate,
    },
    (key, value, tsNanos) =>
      key === "errors" ? value * (errorsByTs.get(tsNanos) ?? 0) : value
  )
  const ticks = thinTicks(
    data.map((row) => row.label),
    7
  )
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Requests & errors</CardTitle>
        <ChartLegend
          items={[
            {
              key: "requests",
              label: "Requests",
              color: "var(--chart-1)",
            },
            { key: "errors", label: "Errors", color: "var(--chart-5)" },
          ]}
        />
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[220px] w-full">
          <LineChart data={data} margin={{ left: 8, right: 8, top: 8 }}>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="label"
              tickLine={false}
              axisLine={false}
              ticks={ticks}
              tickFormatter={(value, index) =>
                makeEdgeTick(value, index, ticks)
              }
            />
            <YAxis tickLine={false} axisLine={false} width={48} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Line
              dataKey="requests"
              stroke="var(--color-requests)"
              dot={false}
              strokeWidth={1.7}
            />
            <Line
              dataKey="errors"
              stroke="var(--color-errors)"
              dot={false}
              strokeWidth={1.7}
            />
          </LineChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

function LatencyChart({
  red,
  overview,
}: {
  red: SpanRed
  overview: ServiceOverview
}) {
  const redBands = latencyBands(red)
  const appData =
    redBands.length > 0
      ? redBands
      : latencyBands({
          rate: [],
          errorRate: [],
          p50: overview.latencyP50,
          p95: overview.latencyP95,
          p99: overview.latencyP99,
        })
  const ticks = thinTicks(
    appData.map((row) => row.label),
    7
  )
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Latency</CardTitle>
        {redBands.length === 0 && appData.length > 0 ? (
          <Badge variant="secondary">app histogram</Badge>
        ) : null}
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[220px] w-full">
          <AreaChart data={appData} margin={{ left: 8, right: 8, top: 8 }}>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="label"
              tickLine={false}
              axisLine={false}
              ticks={ticks}
              tickFormatter={(value, index) =>
                makeEdgeTick(value, index, ticks)
              }
            />
            <YAxis tickLine={false} axisLine={false} width={48} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Area
              dataKey="p50Band"
              stackId="latency"
              stroke="var(--color-p50Band)"
              fill="var(--color-p50Band)"
              fillOpacity={0.3}
            />
            <Area
              dataKey="p95Band"
              stackId="latency"
              stroke="var(--color-p95Band)"
              fill="var(--color-p95Band)"
              fillOpacity={0.25}
            />
            <Area
              dataKey="p99Band"
              stackId="latency"
              stroke="var(--color-p99Band)"
              fill="var(--color-p99Band)"
              fillOpacity={0.2}
            />
          </AreaChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

function InfraBand({ overview }: { overview: ServiceOverview }) {
  if (overview.cpu.length === 0 && overview.memory.length === 0) return null
  const data = toLineData({
    cpu: overview.cpu,
    memory: overview.memory,
  })
  const ticks = thinTicks(
    data.map((row) => row.label),
    7
  )
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Runtime metrics</CardTitle>
        <ChartLegend
          items={[
            { key: "cpu", label: "CPU", color: "var(--chart-1)" },
            { key: "memory", label: "Memory", color: "var(--chart-2)" },
          ]}
        />
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[220px] w-full">
          <LineChart data={data} margin={{ left: 8, right: 8, top: 8 }}>
            <CartesianGrid vertical={false} />
            <XAxis
              dataKey="label"
              tickLine={false}
              axisLine={false}
              ticks={ticks}
              tickFormatter={(value, index) =>
                makeEdgeTick(value, index, ticks)
              }
            />
            <YAxis tickLine={false} axisLine={false} width={48} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Line
              dataKey="cpu"
              stroke="var(--color-cpu)"
              dot={false}
              strokeWidth={1.7}
            />
            <Line
              dataKey="memory"
              stroke="var(--color-memory)"
              dot={false}
              strokeWidth={1.7}
            />
          </LineChart>
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

function RecentTraces({ traces }: { traces: TraceSummary[] }) {
  const durations = traces.map((trace) => Number(trace.durationNs))
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Recent traces</CardTitle>
      </CardHeader>
      <CardContent>
        {traces.length === 0 ? (
          <EmptyState
            className="min-h-40"
            icon={IconAffiliate}
            title="No recent traces"
            description="Change the range or send spans for this service."
          />
        ) : (
          <div className="overflow-hidden rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Root</TableHead>
                  <TableHead className="w-32 text-right">Duration</TableHead>
                  <TableHead className="w-32 text-right">When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {traces.map((trace) => (
                  <TableRow
                    key={trace.traceId}
                    className={cn(
                      trace.hasError &&
                        "shadow-[inset_3px_0_0_rgba(244,63,94,0.85)]"
                    )}
                  >
                    <TableCell>
                      <Link
                        to="/traces/$traceId"
                        params={{ traceId: trace.traceId }}
                        className="font-medium hover:underline"
                      >
                        {trace.rootName}
                      </Link>
                    </TableCell>
                    <TableCell className="text-right">
                      <HeatCell
                        value={Number(trace.durationNs)}
                        values={durations}
                      >
                        {formatDurationNs(trace.durationNs)}
                      </HeatCell>
                    </TableCell>
                    <TableCell className="text-right text-muted-foreground">
                      <RelativeTime nanos={trace.startNanos} />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
