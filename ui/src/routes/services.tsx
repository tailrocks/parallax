import { createFileRoute, useNavigate } from "@tanstack/react-router"
import {
  Area,
  AreaChart,
  CartesianGrid,
  Line,
  LineChart,
  XAxis,
  YAxis,
} from "recharts"
import { graphql, gqlString } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface ChartPoint {
  time: string
  [key: string]: string | number
}

interface PanelData {
  title: string
  unit: string
  series: { key: string; points: SeriesPoint[] }[]
}

// The predefined service overview (scope §2.5a): process CPU/memory plus
// HTTP and gRPC server latency percentiles, all from OTel semconv metric
// names. Panels render only when the service actually sends the metric.
const LATENCY_QS = [
  { key: "p50", q: 0.5 },
  { key: "p95", q: 0.95 },
  { key: "p99", q: 0.99 },
] as const

export const Route = createFileRoute("/services")({
  validateSearch: (search: Record<string, unknown>) => ({
    service: typeof search.service === "string" ? search.service : undefined,
  }),
  loaderDeps: ({ search }) => ({ service: search.service }),
  loader: async ({ deps }) => {
    const { services } = await graphql<{ services: string[] }>(`
      {
        services
      }
    `)
    const service = deps.service ?? services[0]
    if (!service) return { services, service: undefined, panels: [] }

    const nowNanos = BigInt(Date.now()) * 1_000_000n
    const fromNanos = nowNanos - 3_600n * 1_000_000_000n
    const range = `fromNanos: "${fromNanos}", toNanos: "${nowNanos}",
                   service: "${gqlString(service)}"`

    const point = (name: string, agg: string) =>
      graphql<{ metricSeries: { points: SeriesPoint[] }[] }>(
        `{ metricSeries(name: "${name}", agg: "${agg}", ${range}) { points { tsNanos value } } }`
      ).then((r) => r.metricSeries[0]?.points ?? [])
    const quantiles = (name: string) =>
      Promise.all(
        LATENCY_QS.map(({ key, q }) =>
          graphql<{ histogramQuantile: SeriesPoint[] }>(
            `{ histogramQuantile(name: "${name}", q: ${q}, ${range}) { tsNanos value } }`
          ).then((r) => ({ key, points: r.histogramQuantile }))
        )
      )

    const [cpu, memory, http, grpc] = await Promise.all([
      point("process.cpu.utilization", "avg"),
      point("process.memory.usage", "avg"),
      quantiles("http.server.request.duration"),
      quantiles("rpc.server.duration"),
    ])

    const panels: PanelData[] = [
      {
        title: "CPU utilization",
        unit: "ratio",
        series: [{ key: "cpu", points: cpu }],
      },
      {
        title: "Memory usage",
        unit: "bytes",
        series: [{ key: "memory", points: memory }],
      },
      { title: "HTTP server duration", unit: "s", series: http },
      { title: "gRPC server duration", unit: "s", series: grpc },
    ].filter((panel) => panel.series.some((s) => s.points.length > 0))

    return { services, service, panels }
  },
  component: ServicesPage,
})

function toChartData(series: PanelData["series"]): ChartPoint[] {
  const byTime = new Map<string, ChartPoint>()
  for (const { key, points } of series) {
    for (const p of points) {
      const time = new Date(Number(p.tsNanos) / 1e6).toLocaleTimeString()
      const row = byTime.get(time) ?? { time }
      row[key] = p.value
      byTime.set(time, row)
    }
  }
  return [...byTime.values()]
}

const PALETTE = ["var(--chart-1)", "var(--chart-2)", "var(--chart-3)"]

function Panel({ panel }: { panel: PanelData }) {
  const config: ChartConfig = Object.fromEntries(
    panel.series.map((s, i) => [
      s.key,
      { label: s.key, color: PALETTE[i % PALETTE.length] ?? "var(--chart-1)" },
    ])
  )
  const data = toChartData(panel.series)
  const multi = panel.series.length > 1
  const firstKey = panel.series[0]?.key ?? "value"
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          {panel.title}{" "}
          <span className="font-normal text-muted-foreground">
            ({panel.unit})
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={config} className="h-56 w-full">
          {multi ? (
            <LineChart data={data} margin={{ left: 8, right: 8, top: 8 }}>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                minTickGap={32}
              />
              <YAxis tickLine={false} axisLine={false} width={56} />
              <ChartTooltip content={<ChartTooltipContent />} />
              {panel.series.map((s) => (
                <Line
                  key={s.key}
                  dataKey={s.key}
                  stroke={`var(--color-${s.key})`}
                  dot={false}
                />
              ))}
            </LineChart>
          ) : (
            <AreaChart data={data} margin={{ left: 8, right: 8, top: 8 }}>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                minTickGap={32}
              />
              <YAxis tickLine={false} axisLine={false} width={56} />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Area
                dataKey={firstKey}
                stroke={`var(--color-${firstKey})`}
                fill={`var(--color-${firstKey})`}
                fillOpacity={0.2}
              />
            </AreaChart>
          )}
        </ChartContainer>
      </CardContent>
    </Card>
  )
}

function ServicesPage() {
  const { services, service, panels } = Route.useLoaderData()
  const navigate = useNavigate({ from: Route.fullPath })

  if (!service) {
    return (
      <p className="text-sm text-muted-foreground">
        No services yet — point an app's OTLP exporter at this machine and its
        overview appears here.
      </p>
    )
  }
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold">Service overview</h1>
        <Select
          value={service}
          onValueChange={(value) =>
            value && navigate({ search: { service: value } })
          }
        >
          <SelectTrigger className="w-56">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {services.map((s) => (
              <SelectItem key={s} value={s}>
                {s}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {panels.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {service} sent no overview metrics in the last hour. The predefined
          charts read the OTel semconv names process.cpu.utilization,
          process.memory.usage, http.server.request.duration and
          rpc.server.duration — wire them per the conventions page, or chart any
          custom metric from Dashboards.
        </p>
      ) : (
        <div className="grid gap-4 lg:grid-cols-2">
          {panels.map((panel) => (
            <Panel key={panel.title} panel={panel} />
          ))}
        </div>
      )}
    </div>
  )
}
