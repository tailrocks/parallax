import { useEffect, useState } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"
import { graphql, gqlString } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"

interface MetricPoint {
  tsNanos: string
  value: number
}

interface Panel {
  title: string
  unit: string
  points: MetricPoint[]
}

const stripConfig = {
  value: { label: "value", color: "var(--chart-1)" },
} satisfies ChartConfig

/** The cross-signal correlation strip: well-known process metrics in a
 * window around an anchor (trace, issue event, run) — run-scoped when a run
 * id is known, service-scoped otherwise. Renders nothing when the window
 * holds no points (graceful absence, same rule as the bundle). */
export function MetricStrip({
  title,
  service,
  runId,
  fromNanos,
  toNanos,
  stepSeconds,
}: {
  title: string
  service?: string | undefined
  runId?: string | undefined
  fromNanos: string
  toNanos: string
  stepSeconds: number
}) {
  const [panels, setPanels] = useState<Panel[] | null>(null)

  useEffect(() => {
    const scope = [
      runId ? `runId: "${gqlString(runId)}"` : "",
      !runId && service ? `service: "${gqlString(service)}"` : "",
    ]
      .filter(Boolean)
      .join(", ")
    const args = `${scope ? `${scope}, ` : ""}fromNanos: "${fromNanos}", toNanos: "${toNanos}", stepSeconds: ${stepSeconds}`
    void graphql<Record<string, Array<{ points: MetricPoint[] }> | undefined>>(
      `{
        cpu: metricSeries(name: "process.cpu.utilization", ${args}) { points { tsNanos value } }
        memory: metricSeries(name: "process.memory.usage", ${args}) { points { tsNanos value } }
        tasks: metricSeries(name: "tokio.runtime.alive_tasks", ${args}) { points { tsNanos value } }
      }`
    )
      .then((data) => {
        setPanels([
          {
            title: "CPU",
            unit: "%",
            points: (data.cpu?.[0]?.points ?? []).map((p) => ({
              tsNanos: p.tsNanos,
              value: p.value * 100,
            })),
          },
          {
            title: "Memory",
            unit: "MiB",
            points: (data.memory?.[0]?.points ?? []).map((p) => ({
              tsNanos: p.tsNanos,
              value: p.value / (1024 * 1024),
            })),
          },
          {
            title: "Tokio alive tasks",
            unit: "",
            points: data.tasks?.[0]?.points ?? [],
          },
        ])
      })
      .catch(() => setPanels([]))
  }, [service, runId, fromNanos, toNanos, stepSeconds])

  if (!panels || panels.every((panel) => panel.points.length === 0)) {
    return null
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">
          {title}{" "}
          <span className="font-normal text-muted-foreground">
            ({runId ? "this run's points" : "service-scoped"})
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4 md:grid-cols-3">
          {panels
            .filter((panel) => panel.points.length > 0)
            .map((panel) => (
              <div key={panel.title} className="space-y-1">
                <p className="text-xs font-medium text-muted-foreground">
                  {panel.title}
                  {panel.unit ? ` (${panel.unit})` : ""}
                </p>
                <ChartContainer config={stripConfig} className="h-24 w-full">
                  <LineChart
                    data={panel.points.map((p) => ({
                      time: new Date(
                        Number(BigInt(p.tsNanos) / 1_000_000n)
                      ).toLocaleTimeString([], {
                        minute: "2-digit",
                        second: "2-digit",
                      }),
                      value: Number(p.value.toFixed(2)),
                    }))}
                    margin={{ left: 0, right: 8, top: 4 }}
                  >
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
                      stroke="var(--color-value)"
                      dot={false}
                      strokeWidth={1.5}
                    />
                  </LineChart>
                </ChartContainer>
              </div>
            ))}
        </div>
      </CardContent>
    </Card>
  )
}
