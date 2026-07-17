import { Link } from "@tanstack/react-router"
import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
  XAxis,
  YAxis,
} from "recharts"

import {
  ChartLegend,
  makeEdgeTick,
  thinTicks,
} from "@/components/console/trend"
import { Badge } from "@/components/ui/badge"
import { buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"
import {
  Popover,
  PopoverContent,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  PopoverTrigger,
} from "@/components/ui/popover"
import {
  exemplarMarkers,
  formatChartTime,
  latencyBands,
  toLineData,
  type MetricExemplar,
  type ServiceOverview,
  type SpanRed,
} from "@/features/services/model/service-detail"
import { rangeLinkSearch, type ResolvedRange } from "@/lib/range"

const chartConfig = {
  requests: { label: "Requests", color: "var(--chart-throughput)" },
  errors: { label: "Errors", color: "var(--chart-error)" },
  p50Band: { label: "p50", color: "var(--chart-p50)" },
  p95Band: { label: "p95", color: "var(--chart-p95)" },
  p99Band: { label: "p99", color: "var(--chart-p99)" },
  cpu: { label: "CPU", color: "var(--chart-1)" },
  memory: { label: "Memory", color: "var(--chart-2)" },
  exemplar: { label: "Exemplar", color: "var(--chart-4)" },
} satisfies ChartConfig

export function ServiceRequestsChart({ red }: { red: SpanRed }) {
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
              color: "var(--chart-throughput)",
            },
            { key: "errors", label: "Errors", color: "var(--chart-error)" },
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

export function ServiceLatencyChart({
  red,
  overview,
  exemplars,
  range,
}: {
  red: SpanRed
  overview: ServiceOverview
  exemplars: readonly MetricExemplar[]
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
