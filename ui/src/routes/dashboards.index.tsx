import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import { graphql, gqlString } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

interface Dashboard {
  id: string
  name: string
  layout: string
}

export interface Widget {
  metric: string
  agg: string
  chart: string
  title: string
  /** Attribute key to split the series by (optional). */
  groupBy?: string
  /** Grid width in columns (1 or 2). */
  w?: number
}

export const AGGS = ["avg", "sum", "min", "max", "rate"] as const
export const CHARTS = ["line", "area", "bar"] as const

export const Route = createFileRoute("/dashboards/")({
  loader: () =>
    graphql<{ dashboards: Dashboard[]; metricNames: string[] }>(`
      {
        dashboards {
          id
          name
          layout
        }
        metricNames
      }
    `),
  component: DashboardsPage,
})

/** The shared widget form row (create here, add-widget on the detail page). */
export function WidgetPicker({
  metricNames,
  value,
  onChange,
}: {
  metricNames: string[]
  value: Widget
  onChange: (widget: Widget) => void
}) {
  return (
    <>
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">Metric</label>
        <Select
          value={value.metric}
          onValueChange={(metric) =>
            onChange({
              ...value,
              metric: metric ?? "",
              title: `${metric} (${value.agg})`,
            })
          }
        >
          <SelectTrigger className="w-64">
            <SelectValue placeholder="pick a metric you send" />
          </SelectTrigger>
          <SelectContent>
            {metricNames.map((m) => (
              <SelectItem key={m} value={m}>
                {m}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">Aggregation</label>
        <Select
          value={value.agg}
          onValueChange={(agg) =>
            onChange({
              ...value,
              agg: agg ?? "avg",
              title: `${value.metric} (${agg})`,
            })
          }
        >
          <SelectTrigger className="w-24">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {AGGS.map((a) => (
              <SelectItem key={a} value={a}>
                {a}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">Chart</label>
        <Select
          value={value.chart}
          onValueChange={(chart) => onChange({ ...value, chart: chart ?? "line" })}
        >
          <SelectTrigger className="w-24">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {CHARTS.map((c) => (
              <SelectItem key={c} value={c}>
                {c}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">
          Group by attribute
        </label>
        <Input
          value={value.groupBy ?? ""}
          onChange={(e) =>
            onChange({ ...value, groupBy: e.target.value || undefined })
          }
          placeholder="e.g. payment.method"
          className="w-44"
        />
      </div>
      <div className="space-y-1">
        <label className="text-xs text-muted-foreground">Width</label>
        <Select
          value={String(value.w ?? 1)}
          onValueChange={(w) => onChange({ ...value, w: Number(w ?? "1") })}
        >
          <SelectTrigger className="w-24">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="1">half</SelectItem>
            <SelectItem value="2">full</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </>
  )
}

export function emptyWidget(): Widget {
  return { metric: "", agg: "avg", chart: "line", title: "", w: 1 }
}

function DashboardsPage() {
  const { dashboards, metricNames } = Route.useLoaderData()
  const router = useRouter()
  const [name, setName] = useState("")
  const [widget, setWidget] = useState<Widget>(emptyWidget())
  const [error, setError] = useState<string | null>(null)

  async function create() {
    setError(null)
    try {
      await graphql<{ dashboardSave: { id: string } }>(
        `mutation { dashboardSave(name: "${gqlString(name)}",
           layout: "${gqlString(JSON.stringify([widget]))}") { id } }`
      )
      setName("")
      setWidget(emptyWidget())
      await router.invalidate()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  async function remove(id: string) {
    setError(null)
    try {
      await graphql(`mutation { dashboardDelete(id: "${gqlString(id)}") }`)
      await router.invalidate()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  return (
    <div className="space-y-4">
      <h1 className="text-lg font-semibold">Dashboards</h1>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">New dashboard</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap items-end gap-2">
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Name</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="checkout ops"
              className="w-44"
            />
          </div>
          <WidgetPicker
            metricNames={metricNames}
            value={widget}
            onChange={setWidget}
          />
          <Button onClick={create} disabled={!name.trim() || !widget.metric}>
            Create
          </Button>
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
        </CardContent>
      </Card>

      {dashboards.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No dashboards yet — pick one of the metrics your apps already send.
        </p>
      ) : (
        <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          {dashboards.map((dashboard) => (
            <li key={dashboard.id} className="relative">
              <Link
                to="/dashboards/$dashboardId"
                params={{ dashboardId: dashboard.id }}
                className="block rounded-lg border p-4 pr-16 hover:bg-muted"
              >
                <span className="font-medium">{dashboard.name}</span>
                <span className="block text-xs text-muted-foreground">
                  {safeWidgetCount(dashboard.layout)} chart(s)
                </span>
              </Link>
              <Button
                variant="ghost"
                size="sm"
                className="absolute top-3 right-2 text-muted-foreground hover:text-destructive"
                aria-label={`Delete ${dashboard.name}`}
                onClick={() => remove(dashboard.id)}
              >
                Delete
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}

function safeWidgetCount(layout: string): number {
  try {
    const parsed: unknown = JSON.parse(layout)
    return Array.isArray(parsed) ? parsed.length : 0
  } catch {
    return 0
  }
}
