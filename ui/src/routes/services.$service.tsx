import { Link, createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  IconActivityHeartbeat,
  IconAffiliate,
  IconAlertTriangleFilled,
  IconArticle,
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
import { useMemo } from "react"

import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
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
import { RuntimeSnapshotCard } from "@/components/runtime-snapshot"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { buttonVariants } from "@/components/ui/button"
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { gqlString, graphqlCached } from "@/lib/api"
import type { RuntimeMetric, ServiceCatalogRow, TraceSummary } from "@/lib/api"
import {
  formatCount,
  formatDateTime,
  formatDurationNs,
  formatPercent,
  formatTimeShort,
} from "@/lib/format"
import {
  mergeRangeSearch,
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"
import * as Semconv from "@/shared/semconv"
type SeriesPoint = { tsNanos: string; value: number }
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

export interface MetricExemplar {
  tsNanos: string
  service: string
  name: string
  value: number
  traceId: string
  spanId: string
  runId: string | null
  attributes: string
}

export interface ReleaseWindow {
  version: string
  firstSeenNanos: string
  lastSeenNanos: string
  spanCount: string
}

export interface ServiceDetailData {
  red: SpanRed
  overview: ServiceOverview
  releases: ReleaseWindow[]
  serviceCatalog: ServiceCatalogRow[]
  httpDurationExemplars: MetricExemplar[]
  rpcDurationExemplars: MetricExemplar[]
  runtimeSnapshot: RuntimeMetric[]
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
  exemplar: { label: "Exemplar", color: "var(--chart-4)" },
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
  return graphqlCached<ServiceDetailData>(`
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
      releases(service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        version firstSeenNanos lastSeenNanos spanCount
      }
      serviceCatalog(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") {
        name
        serviceVersion
        serviceNamespace
        deploymentEnvironment
        telemetrySdkLanguage
        telemetrySdkName
        telemetrySdkVersion
        lastSeenNanos
        instanceCount
      }
      httpDurationExemplars: metricExemplars(name: "${Semconv.HTTP_SERVER_REQUEST_DURATION}", service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", limit: 50) {
        tsNanos service name value traceId spanId runId attributes
      }
      rpcDurationExemplars: metricExemplars(name: "${Semconv.REQUEST_DURATION_METRICS[1]}", service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", limit: 50) {
        tsNanos service name value traceId spanId runId attributes
      }
      runtimeSnapshot(service: "${escaped}", fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) {
        family metric unit points { tsNanos value }
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
  return formatTimeShort(tsNanos)
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

type ExemplarMarker = {
  exemplar: MetricExemplar
  x: number
  y: number
}

function exemplarMarkers(
  exemplars: MetricExemplar[],
  data: Array<{ tsNanos: string; p50?: number; p95?: number; p99?: number }>,
  range: ResolvedRange
): ExemplarMarker[] {
  const from = BigInt(range.fromNanos)
  const to = BigInt(range.toNanos)
  const span = to - from
  if (span <= 0n) return []
  const chartMax = data.reduce(
    (max, row) => Math.max(max, row.p50 ?? 0, row.p95 ?? 0, row.p99 ?? 0),
    0
  )
  const exemplarMax = exemplars.reduce(
    (max, exemplar) =>
      Number.isFinite(exemplar.value) ? Math.max(max, exemplar.value) : max,
    0
  )
  const maxValue = Math.max(chartMax, exemplarMax, 1)
  return exemplars
    .filter((exemplar) => exemplar.traceId && exemplar.spanId)
    .map((exemplar) => {
      const ts = BigInt(exemplar.tsNanos)
      const clampedTs = ts < from ? from : ts > to ? to : ts
      const x = Number(((clampedTs - from) * 10_000n) / span) / 100
      const ratio = Number.isFinite(exemplar.value)
        ? Math.max(0, Math.min(1, exemplar.value / maxValue))
        : 0
      return {
        exemplar,
        x: Math.max(5, Math.min(95, x)),
        y: Math.max(12, Math.min(86, 86 - ratio * 70)),
      }
    })
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
        void navigate({
          search: (current) => mergeRangeSearch(current, next),
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
  const identity = data.serviceCatalog.find((row) => row.name === service)
  const noData =
    !hasRed && traces.length === 0 && data.runtimeSnapshot.length === 0
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
        actions={
          <>
            <Link
              to="/traces"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconAffiliate />
              Traces
            </Link>
            <Link
              to="/logs"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconArticle />
              Logs
            </Link>
            <Link
              to="/issues"
              search={{ service, ...rangeLinkSearch(range) }}
              className={buttonVariants({ variant: "outline", size: "sm" })}
            >
              <IconAlertTriangleFilled />
              Issues
            </Link>
            <RangePicker value={range} onChange={onRange} />
          </>
        }
      />

      <ReleaseStrip releases={data.releases} range={range} />

      <IdentityCard identity={identity} fallbackLastSeen={lastSeen} />

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
        <LatencyChart
          red={data.red}
          overview={data.overview}
          exemplars={[
            ...data.httpDurationExemplars,
            ...data.rpcDurationExemplars,
          ]}
          range={range}
        />
      </div>

      <RuntimeSnapshotCard metrics={data.runtimeSnapshot} />
      <RecentTraces traces={traces} range={range} />
    </div>
  )
}

function IdentityCard({
  identity,
  fallbackLastSeen,
}: {
  identity: ServiceCatalogRow | undefined
  fallbackLastSeen: string | undefined
}) {
  const sdk = [identity?.telemetrySdkName, identity?.telemetrySdkVersion]
    .filter(Boolean)
    .join(" ")
  const identityLastSeen = identity?.lastSeenNanos ?? fallbackLastSeen
  const values = [
    ["Version", identity?.serviceVersion],
    ["Namespace", identity?.serviceNamespace],
    ["Environment", identity?.deploymentEnvironment],
    ["Runtime", identity?.telemetrySdkLanguage],
    ["SDK", sdk || null],
    ["Instances", formatCount(Number(identity?.instanceCount ?? 0))],
    [
      "Last seen",
      identityLastSeen ? <RelativeTime nanos={identityLastSeen} /> : null,
    ],
  ] satisfies Array<[string, React.ReactNode]>

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">Identity</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {values.map(([label, value]) => (
          <div key={label} className="space-y-1">
            <div className="text-xs text-muted-foreground">{label}</div>
            <div className="text-sm font-medium">
              {value || (
                <span className="text-muted-foreground">not emitted</span>
              )}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}

function ReleaseStrip({
  releases,
  range,
}: {
  releases: ReleaseWindow[]
  range: ResolvedRange
}) {
  const segments = useMemo(() => {
    const from = BigInt(range.fromNanos)
    const to = BigInt(range.toNanos)
    const total = to - from
    if (total <= 0n) return []
    return releases.map((release) => {
      const first = BigInt(release.firstSeenNanos)
      const last = BigInt(release.lastSeenNanos)
      const start = first < from ? from : first > to ? to : first
      const end = last < from ? from : last > to ? to : last
      const left = Number(((start - from) * 10_000n) / total) / 100
      const duration = end > start ? end - start : 1n
      const rawWidth = Number((duration * 10_000n) / total) / 100
      const width = Math.max(4, Math.min(100 - left, rawWidth))
      return {
        ...release,
        left,
        width,
        title: `${release.version}: ${formatDateTime(release.firstSeenNanos)} - ${formatDateTime(release.lastSeenNanos)} (${formatCount(Number(release.spanCount))} spans)`,
      }
    })
  }, [range.fromNanos, range.toNanos, releases])

  if (segments.length === 0) return null

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-medium">Releases</h2>
        <Badge variant="secondary">{segments.length} versions</Badge>
      </div>
      <div className="relative h-9 overflow-hidden rounded-md border bg-muted/30">
        {segments.map((segment) => (
          <div
            key={`${segment.version}-${segment.firstSeenNanos}`}
            className="absolute inset-y-1 flex min-w-12 items-center justify-center truncate rounded-sm border border-primary/30 bg-primary/15 px-2 text-xs font-medium text-primary"
            style={{
              left: `${segment.left}%`,
              width: `${segment.width}%`,
            }}
            title={segment.title}
          >
            {segment.version}
          </div>
        ))}
      </div>
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
                makeEdgeTick(String(value), index, ticks)
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
  exemplars,
  range,
}: {
  red: SpanRed
  overview: ServiceOverview
  exemplars: MetricExemplar[]
  range: ResolvedRange
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
  const markers = exemplarMarkers(exemplars, appData, range)
  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="text-sm">Latency</CardTitle>
        <div className="flex items-center gap-2">
          {markers.length > 0 ? (
            <Badge variant="secondary">{markers.length} exemplars</Badge>
          ) : null}
          {redBands.length === 0 && appData.length > 0 ? (
            <Badge variant="secondary">app histogram</Badge>
          ) : null}
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="relative">
          <ChartContainer config={chartConfig} className="h-[220px] w-full">
            <AreaChart data={appData} margin={{ left: 8, right: 8, top: 8 }}>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="label"
                tickLine={false}
                axisLine={false}
                ticks={ticks}
                tickFormatter={(value, index) =>
                  makeEdgeTick(String(value), index, ticks)
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
          {markers.map((marker) => (
            <Popover
              key={`${marker.exemplar.traceId}-${marker.exemplar.spanId}-${marker.exemplar.tsNanos}`}
            >
              <PopoverTrigger
                render={
                  <button
                    type="button"
                    className="absolute size-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary shadow-sm ring-2 ring-background transition-transform outline-none hover:scale-125 focus-visible:ring-ring"
                    style={{ left: `${marker.x}%`, top: `${marker.y}%` }}
                    aria-label={`Trace exemplar ${marker.exemplar.traceId}`}
                  />
                }
              />
              <PopoverContent align="center" className="w-80">
                <PopoverHeader>
                  <PopoverTitle>Trace exemplar</PopoverTitle>
                  <PopoverDescription>
                    {formatChartTime(marker.exemplar.tsNanos)}
                  </PopoverDescription>
                </PopoverHeader>
                <div className="grid gap-2 text-xs">
                  <div className="grid grid-cols-[64px_1fr] gap-2">
                    <span className="text-muted-foreground">trace</span>
                    <span className="truncate font-mono">
                      {marker.exemplar.traceId}
                    </span>
                  </div>
                  <div className="grid grid-cols-[64px_1fr] gap-2">
                    <span className="text-muted-foreground">span</span>
                    <span className="truncate font-mono">
                      {marker.exemplar.spanId}
                    </span>
                  </div>
                  <div className="grid grid-cols-[64px_1fr] gap-2">
                    <span className="text-muted-foreground">value</span>
                    <span>{marker.exemplar.value.toLocaleString()}</span>
                  </div>
                </div>
                <Link
                  to="/traces/$traceId"
                  params={{ traceId: marker.exemplar.traceId }}
                  search={rangeLinkSearch(range)}
                  className={buttonVariants({ variant: "outline", size: "sm" })}
                >
                  Open trace
                </Link>
              </PopoverContent>
            </Popover>
          ))}
        </div>
        {markers.length === 0 ? (
          <p className="text-xs text-muted-foreground">
            No trace exemplar attached; showing traces near this timestamp
          </p>
        ) : null}
      </CardContent>
    </Card>
  )
}

function RecentTraces({
  traces,
  range,
}: {
  traces: TraceSummary[]
  range: ResolvedRange
}) {
  const durations = traces.map((trace) => Number(trace.durationNs))
  const scale = useMemo(() => buildHeatScale(durations), [durations])
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
                        search={rangeLinkSearch(range)}
                        className="font-medium hover:underline"
                      >
                        {trace.rootName}
                      </Link>
                    </TableCell>
                    <TableCell className="text-right">
                      <HeatCell value={Number(trace.durationNs)} scale={scale}>
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
