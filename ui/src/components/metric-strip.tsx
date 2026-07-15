import { useEffect, useMemo, useState } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"
import { graphql, gqlString } from "@/lib/api"
import { usePageVisible } from "@/lib/use-visible"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { formatBytes, formatTimeShort } from "@/lib/format"
import {
  CPU_METRICS,
  MEMORY_METRICS,
  TOKIO_RUNTIME_ALIVE_TASKS,
} from "@/shared/semconv"

interface MetricPoint {
  tsNanos: string
  value: number
}

interface Panel {
  title: string
  unit: string
  key: "cpu" | "memory" | "tasks"
  points: MetricPoint[]
}

const stripConfig = {
  cpu: { label: "CPU", color: "var(--chart-1)" },
  memory: { label: "Memory", color: "var(--chart-2)" },
  tasks: { label: "Tasks", color: "var(--chart-3)" },
} satisfies ChartConfig

function isAbortError(error: unknown): boolean {
  return (
    (error instanceof DOMException ||
      (typeof error === "object" && error !== null && "name" in error)) &&
    (error as { name?: unknown }).name === "AbortError"
  )
}

function MetricPanel({ panel }: { panel: Panel }) {
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
  live = false,
}: {
  title: string
  service?: string | undefined
  runId?: string | undefined
  fromNanos: string
  toNanos: string
  stepSeconds: number
  live?: boolean
}) {
  const [panels, setPanels] = useState<Panel[] | null>(null)
  const pageVisible = usePageVisible()

  useEffect(() => {
    let cancelled = false
    let activeController: AbortController | null = null
    const fetchPanels = () => {
      activeController?.abort()
      const controller = new AbortController()
      activeController = controller
      const to = live
        ? ((BigInt(Date.now()) + 30_000n) * 1_000_000n).toString()
        : toNanos
      const scope = [
        runId ? `runId: "${gqlString(runId)}"` : "",
        !runId && service ? `service: "${gqlString(service)}"` : "",
      ]
        .filter(Boolean)
        .join(", ")
      const args = `${scope ? `${scope}, ` : ""}fromNanos: "${fromNanos}", toNanos: "${to}", stepSeconds: ${stepSeconds}`
      void graphql<
        Record<string, Array<{ points: MetricPoint[] }> | undefined>
      >(
        `{
        cpu: metricSeries(name: "${CPU_METRICS[0]}", ${args}) { points { tsNanos value } }
        memory: metricSeries(name: "${MEMORY_METRICS[0]}", ${args}) { points { tsNanos value } }
        tasks: metricSeries(name: "${TOKIO_RUNTIME_ALIVE_TASKS}", ${args}) { points { tsNanos value } }
      }`,
        { signal: controller.signal }
      )
        .then((data) => {
          if (cancelled || controller.signal.aborted) return
          setPanels([
            {
              title: "CPU",
              unit: "%",
              key: "cpu",
              points: (data["cpu"]?.[0]?.points ?? []).map((p) => ({
                tsNanos: p.tsNanos,
                value: p.value * 100,
              })),
            },
            {
              title: "Memory",
              unit: "bytes",
              key: "memory",
              points: data["memory"]?.[0]?.points ?? [],
            },
            {
              title: "Tokio alive tasks",
              unit: "",
              key: "tasks",
              points: data["tasks"]?.[0]?.points ?? [],
            },
          ])
        })
        .catch((error: unknown) => {
          if (cancelled || isAbortError(error)) return
          setPanels([])
        })
    }
    // Always fetch once when deps change; only poll while visible + live.
    if (pageVisible) fetchPanels()
    const timer =
      live && pageVisible ? setInterval(fetchPanels, 5000) : undefined
    return () => {
      cancelled = true
      activeController?.abort()
      if (timer) clearInterval(timer)
    }
  }, [service, runId, fromNanos, toNanos, stepSeconds, live, pageVisible])

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
              <MetricPanel key={panel.title} panel={panel} />
            ))}
        </div>
      </CardContent>
    </Card>
  )
}
