import { createFileRoute, notFound } from "@tanstack/react-router"
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
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

interface Widget {
  metric: string
  agg: string
  chart: string
  title: string
}

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface WidgetData {
  widget: Widget
  points: { time: string; value: number }[]
}

export const Route = createFileRoute("/dashboards/$dashboardId")({
  loader: async ({ params }) => {
    const { dashboards } = await graphql<{
      dashboards: { id: string; name: string; layout: string }[]
    }>(`
      {
        dashboards {
          id
          name
          layout
        }
      }
    `)
    const dashboard = dashboards.find((d) => d.id === params.dashboardId)
    if (!dashboard) throw notFound()

    const widgets = parseLayout(dashboard.layout)
    const nowNanos = BigInt(Date.now()) * 1_000_000n
    const fromNanos = nowNanos - 3_600n * 1_000_000_000n // last hour

    const data: WidgetData[] = await Promise.all(
      widgets.map(async (widget) => {
        const { metricSeries } = await graphql<{ metricSeries: SeriesPoint[] }>(
          `{ metricSeries(name: "${gqlString(widget.metric)}",
               fromNanos: "${fromNanos}", toNanos: "${nowNanos}",
               agg: "${gqlString(widget.agg)}") { tsNanos value } }`
        )
        return {
          widget,
          points: metricSeries.map((p) => ({
            time: new Date(Number(p.tsNanos) / 1e6).toLocaleTimeString(),
            value: p.value,
          })),
        }
      })
    )
    return { name: dashboard.name, data }
  },
  component: DashboardPage,
})

function parseLayout(layout: string): Widget[] {
  try {
    const parsed: unknown = JSON.parse(layout)
    if (!Array.isArray(parsed)) return []
    return parsed.filter(
      (w): w is Widget =>
        typeof w === "object" &&
        w !== null &&
        typeof (w as Widget).metric === "string"
    )
  } catch {
    return []
  }
}

const chartConfig = {
  value: { label: "value", color: "var(--chart-1)" },
} satisfies ChartConfig

function WidgetChart({ data }: { data: WidgetData }) {
  const common = {
    data: data.points,
    margin: { left: 8, right: 8, top: 8 },
  }
  const axes = (
    <>
      <CartesianGrid vertical={false} />
      <XAxis dataKey="time" tickLine={false} axisLine={false} minTickGap={32} />
      <YAxis tickLine={false} axisLine={false} width={48} />
      <ChartTooltip content={<ChartTooltipContent />} />
    </>
  )
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">{data.widget.title}</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-56 w-full">
          {data.widget.chart === "bar" ? (
            <BarChart {...common}>
              {axes}
              <Bar dataKey="value" fill="var(--color-value)" radius={2} />
            </BarChart>
          ) : data.widget.chart === "area" ? (
            <AreaChart {...common}>
              {axes}
              <Area
                dataKey="value"
                fill="var(--color-value)"
                stroke="var(--color-value)"
                fillOpacity={0.2}
              />
            </AreaChart>
          ) : (
            <LineChart {...common}>
              {axes}
              <Line dataKey="value" stroke="var(--color-value)" dot={false} />
            </LineChart>
          )}
        </ChartContainer>
        {data.points.length === 0 ? (
          <p className="mt-2 text-xs text-muted-foreground">
            No points in the last hour for {data.widget.metric}.
          </p>
        ) : null}
      </CardContent>
    </Card>
  )
}

function DashboardPage() {
  const { name, data } = Route.useLoaderData()
  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">{name}</h1>
      <div className="grid gap-4 lg:grid-cols-2">
        {data.map((d) => (
          <WidgetChart key={d.widget.title} data={d} />
        ))}
      </div>
    </div>
  )
}
