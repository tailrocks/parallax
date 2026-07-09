import { useMemo } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import type { RuntimeMetric } from "@/lib/api"
import { formatTimeShort } from "@/lib/format"

const runtimeConfig = {
  value: { label: "Value", color: "var(--chart-1)" },
} satisfies ChartConfig

function formatMetricName(name: string) {
  return name
    .replace(/^tokio\.runtime\./, "tokio ")
    .replace(/^db\.client\.connection\./, "db connection ")
    .replace(/\./g, " ")
}

function displayValue(metric: RuntimeMetric, value: number) {
  if (metric.unit === "bytes") return value / (1024 * 1024)
  if (metric.unit === "ratio") return value * 100
  return value
}

function displayUnit(unit: string | null) {
  if (unit === "bytes") return "MiB"
  if (unit === "ratio") return "%"
  return unit ?? ""
}

function RuntimeChart({ metric }: { metric: RuntimeMetric }) {
  const data = useMemo(
    () =>
      metric.points.map((point) => ({
        time: formatTimeShort(point.tsNanos, {
          minute: "2-digit",
          second: "2-digit",
        }),
        value: Number(displayValue(metric, point.value).toFixed(2)),
      })),
    [metric]
  )
  const unit = displayUnit(metric.unit)

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between gap-2">
        <p className="truncate text-xs font-medium text-muted-foreground">
          {formatMetricName(metric.metric)}
        </p>
        {unit ? (
          <span className="text-xs text-muted-foreground">{unit}</span>
        ) : null}
      </div>
      <ChartContainer config={runtimeConfig} className="h-24 w-full">
        <LineChart data={data} margin={{ left: 0, right: 8, top: 4 }}>
          <CartesianGrid vertical={false} />
          <XAxis
            dataKey="time"
            tickLine={false}
            axisLine={false}
            minTickGap={32}
          />
          <YAxis tickLine={false} axisLine={false} width={44} />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Line
            dataKey="value"
            name="value"
            stroke="var(--color-value)"
            dot={false}
            strokeWidth={1.5}
          />
        </LineChart>
      </ChartContainer>
    </div>
  )
}

export function RuntimeSnapshotCard({
  title = "Runtime",
  metrics,
}: {
  title?: string
  metrics: RuntimeMetric[]
}) {
  const grouped = useMemo(() => {
    const rows = new Map<string, RuntimeMetric[]>()
    for (const metric of metrics.filter((row) => row.points.length > 0)) {
      rows.set(metric.family, [...(rows.get(metric.family) ?? []), metric])
    }
    return Array.from(rows.entries())
  }, [metrics])

  if (grouped.length === 0) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-5">
        {grouped.map(([family, rows]) => (
          <section key={family} className="space-y-3">
            <h3 className="text-xs font-medium text-muted-foreground uppercase">
              {family}
            </h3>
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
              {rows.map((metric) => (
                <RuntimeChart key={metric.metric} metric={metric} />
              ))}
            </div>
          </section>
        ))}
      </CardContent>
    </Card>
  )
}
