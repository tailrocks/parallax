import { useMemo } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"

import type { StripPanel } from "@/features/runtime-metrics/api/runtime-metrics-mapper"
import { useRuntimeMetrics } from "@/features/runtime-metrics/hooks/use-runtime-metrics"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { formatBytes, formatTimeShort } from "@/lib/format"

const stripConfig = {
  cpu: { label: "CPU", color: "var(--chart-1)" },
  memory: { label: "Memory", color: "var(--chart-2)" },
  tasks: { label: "Tasks", color: "var(--chart-3)" },
} satisfies ChartConfig

function MetricPanel({ panel }: { panel: StripPanel }) {
  const chartData = useMemo(
    () =>
      panel.points.map((p) => ({
        time: formatTimeShort(p.tsNanos, {
          minute: "2-digit",
          second: "2-digit",
        }),
        value: Number(p.value.toFixed(2)),
      })),
    [panel.points]
  )

  return (
    <div className="space-y-1">
      <p className="text-xs font-medium text-muted-foreground">
        {panel.title}
        {panel.unit ? ` (${panel.unit})` : ""}
      </p>
      <ChartContainer config={stripConfig} className="h-24 w-full">
        <LineChart data={chartData} margin={{ left: 0, right: 8, top: 4 }}>
          <CartesianGrid vertical={false} />
          <XAxis
            dataKey="time"
            tickLine={false}
            axisLine={false}
            minTickGap={32}
          />
          <YAxis
            tickLine={false}
            axisLine={false}
            width={panel.key === "memory" ? 72 : 44}
            tickFormatter={(value) =>
              panel.key === "memory"
                ? formatBytes(Number(value))
                : String(value)
            }
          />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Line
            dataKey="value"
            name={panel.key}
            stroke={`var(--color-${panel.key})`}
            dot={false}
            strokeWidth={1.5}
          />
        </LineChart>
      </ChartContainer>
    </div>
  )
}

/** Cross-signal correlation strip: well-known process metrics around an
 * anchor. Run-scoped when invocationId is known, else service-scoped.
 * Renders nothing when the window holds no points. */
export function MetricStrip({
  title,
  service,
  invocationId,
  fromNanos,
  toNanos,
  stepSeconds,
  live = false,
}: {
  title: string
  service?: string | undefined
  invocationId?: string | undefined
  fromNanos: string
  toNanos: string
  stepSeconds: number
  live?: boolean
}) {
  const panels = useRuntimeMetrics({
    service,
    invocationId,
    fromNanos,
    toNanos,
    stepSeconds,
    live,
  })

  if (!panels || panels.every((panel) => panel.points.length === 0)) {
    return null
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          {title}{" "}
          <span className="font-normal text-muted-foreground">
            ({invocationId ? "this run's points" : "service-scoped"})
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4 md:grid-cols-3">
          {panels
            .filter((panel) => panel.points.length > 0)
            .map((panel) => (
              <MetricPanel key={panel.title} panel={panel} />
            ))}
        </div>
      </CardContent>
    </Card>
  )
}
