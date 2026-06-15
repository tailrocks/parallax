import { useState } from "react"
import {
  createFileRoute,
  notFound,
  useNavigate,
  useRouter,
} from "@tanstack/react-router"
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
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import { WidgetPicker, emptyWidget } from "./dashboards.index"
import type { Widget } from "./dashboards.index"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface Series {
  groupValue: string | null
  points: SeriesPoint[]
}

/** Recharts rows: one row per timestamp, one column per group. */
interface WidgetData {
  widget: Widget
  groups: string[]
  rows: Record<string, number | string>[]
}

const MAX_GROUPS = 5

export const Route = createFileRoute("/dashboards/$dashboardId")({
  loader: async ({ params }) => {
    const { dashboard, metricNames } = await graphql<{
      dashboard: { id: string; name: string; layout: string } | null
      metricNames: string[]
    }>(
      `{ dashboard(id: "${gqlString(params.dashboardId)}") { id name layout }
         metricNames }`
    )
    if (!dashboard) throw notFound()

    const widgets = parseLayout(dashboard.layout)
    const nowNanos = BigInt(Date.now()) * 1_000_000n
    const fromNanos = nowNanos - 3_600n * 1_000_000_000n // last hour

    const data: WidgetData[] = await Promise.all(
      widgets.map(async (widget) => {
        const { metricSeries } = await graphql<{ metricSeries: Series[] }>(
          `{ metricSeries(name: "${gqlString(widget.metric)}",
               fromNanos: "${fromNanos}", toNanos: "${nowNanos}",
               agg: "${gqlString(widget.agg)}"${
                 widget.groupBy
                   ? `, groupBy: "${gqlString(widget.groupBy)}"`
                   : ""
               }) { groupValue points { tsNanos value } } }`
        )
        return toWidgetData(widget, metricSeries)
      })
    )
    return {
      id: dashboard.id,
      name: dashboard.name,
      widgets,
      data,
      metricNames,
    }
  },
  component: DashboardPage,
})

function toWidgetData(widget: Widget, series: Series[]): WidgetData {
  const kept = series.slice(0, MAX_GROUPS)
  const groups = kept.map(
    (s, i) => s.groupValue ?? (i === 0 ? "value" : `#${i}`)
  )
  const byTime = new Map<string, Record<string, number | string>>()
  kept.forEach((s, index) => {
    const group = groups[index]
    if (!group) return
    for (const point of s.points) {
      const time = new Date(Number(point.tsNanos) / 1e6).toLocaleTimeString()
      const row = byTime.get(point.tsNanos) ?? { time }
      row[group] = point.value
      byTime.set(point.tsNanos, row)
    }
  })
  const rows = [...byTime.entries()]
    .sort(([a], [b]) => (BigInt(a) < BigInt(b) ? -1 : 1))
    .map(([, row]) => row)
  return { widget, groups, rows }
}

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

function WidgetChart({
  data,
  editing,
  onRemove,
  onMove,
}: {
  data: WidgetData
  editing: boolean
  onRemove: () => void
  onMove: (delta: -1 | 1) => void
}) {
  const config = Object.fromEntries(
    data.groups.map((group, index) => [
      group,
      { label: group, color: `var(--chart-${index + 1})` },
    ])
  ) satisfies ChartConfig
  const common = {
    data: data.rows,
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
  const wide = (data.widget.w ?? 1) >= 2
  return (
    <Card className={wide ? "lg:col-span-2" : ""}>
      <CardHeader>
        <CardTitle className="flex items-center justify-between gap-2 text-sm">
          <span className="truncate">
            {data.widget.title || data.widget.metric}
            {data.widget.groupBy ? (
              <span className="ml-1 font-normal text-muted-foreground">
                by {data.widget.groupBy}
              </span>
            ) : null}
          </span>
          {editing ? (
            <span className="flex shrink-0 gap-1">
              <Button variant="ghost" size="sm" onClick={() => onMove(-1)}>
                ↑
              </Button>
              <Button variant="ghost" size="sm" onClick={() => onMove(1)}>
                ↓
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="text-destructive"
                onClick={onRemove}
              >
                Remove
              </Button>
            </span>
          ) : null}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={config} className="h-56 w-full">
          {data.widget.chart === "bar" ? (
            <BarChart {...common}>
              {axes}
              {data.groups.map((group) => (
                <Bar
                  key={group}
                  dataKey={group}
                  fill={`var(--color-${group})`}
                  radius={2}
                />
              ))}
            </BarChart>
          ) : data.widget.chart === "area" ? (
            <AreaChart {...common}>
              {axes}
              {data.groups.map((group) => (
                <Area
                  key={group}
                  dataKey={group}
                  fill={`var(--color-${group})`}
                  stroke={`var(--color-${group})`}
                  fillOpacity={0.2}
                />
              ))}
            </AreaChart>
          ) : (
            <LineChart {...common}>
              {axes}
              {data.groups.map((group) => (
                <Line
                  key={group}
                  dataKey={group}
                  stroke={`var(--color-${group})`}
                  dot={false}
                />
              ))}
            </LineChart>
          )}
        </ChartContainer>
        {data.rows.length === 0 ? (
          <p className="mt-2 text-xs text-muted-foreground">
            No points in the last hour for {data.widget.metric}.
          </p>
        ) : null}
      </CardContent>
    </Card>
  )
}

function DashboardPage() {
  const { id, name, widgets, data, metricNames } = Route.useLoaderData()
  const router = useRouter()
  const navigate = useNavigate()
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState<Widget[]>(widgets)
  const [addition, setAddition] = useState<Widget>(emptyWidget())
  const [error, setError] = useState<string | null>(null)

  async function save(layout: Widget[]) {
    setError(null)
    try {
      await graphql(
        `mutation { dashboardSave(name: "${gqlString(name)}",
           layout: "${gqlString(JSON.stringify(layout))}",
           id: "${gqlString(id)}") { id } }`
      )
      setEditing(false)
      await router.invalidate()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  async function removeDashboard() {
    setError(null)
    try {
      await graphql(`mutation { dashboardDelete(id: "${gqlString(id)}") }`)
      await navigate({ to: "/dashboards" })
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  function move(index: number, delta: -1 | 1) {
    const next = [...draft]
    const target = index + delta
    if (target < 0 || target >= next.length) return
    const current = next[index]
    const other = next[target]
    if (!current || !other) return
    ;[next[index], next[target]] = [other, current]
    setDraft(next)
  }

  const shown = editing ? draft : widgets
  const shownData = shown.map(
    (widget) =>
      data.find((d) => d.widget === widget) ?? { widget, groups: [], rows: [] }
  )

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-lg font-semibold">{name}</h1>
        <span className="flex gap-2">
          {editing ? (
            <>
              <Button size="sm" onClick={() => save(draft)}>
                Save
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setDraft(widgets)
                  setEditing(false)
                }}
              >
                Cancel
              </Button>
            </>
          ) : (
            <>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setDraft(widgets)
                  setEditing(true)
                }}
              >
                Edit
              </Button>
              <Button
                size="sm"
                variant="outline"
                className="text-destructive"
                onClick={removeDashboard}
              >
                Delete
              </Button>
            </>
          )}
        </span>
      </div>
      {error ? <p className="text-sm text-destructive">{error}</p> : null}

      {editing ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Add widget</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-wrap items-end gap-2">
            <WidgetPicker
              metricNames={metricNames}
              value={addition}
              onChange={setAddition}
            />
            <Button
              variant="outline"
              disabled={!addition.metric}
              onClick={() => {
                setDraft([...draft, addition])
                setAddition(emptyWidget())
              }}
            >
              Add
            </Button>
          </CardContent>
        </Card>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        {shownData.map((d, index) => (
          <WidgetChart
            key={`${d.widget.metric}-${index}`}
            data={d}
            editing={editing}
            onRemove={() => setDraft(draft.filter((_, i) => i !== index))}
            onMove={(delta) => move(index, delta)}
          />
        ))}
      </div>
    </div>
  )
}
