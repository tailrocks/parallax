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

interface Widget {
  metric: string
  agg: string
  chart: string
  title: string
}

const AGGS = ["avg", "sum", "min", "max", "rate"] as const
const CHARTS = ["line", "area", "bar"] as const

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

function DashboardsPage() {
  const { dashboards, metricNames } = Route.useLoaderData()
  const router = useRouter()
  const [name, setName] = useState("")
  const [metric, setMetric] = useState("")
  const [agg, setAgg] = useState<string>("avg")
  const [chart, setChart] = useState<string>("line")
  const [error, setError] = useState<string | null>(null)

  async function create() {
    setError(null)
    const widget: Widget = {
      metric,
      agg,
      chart,
      title: `${metric} (${agg})`,
    }
    try {
      await graphql<{ dashboardSave: string }>(
        `mutation { dashboardSave(name: "${gqlString(name)}",
           layout: "${gqlString(JSON.stringify([widget]))}") }`
      )
      setName("")
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
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Metric</label>
            <Select value={metric} onValueChange={(v) => setMetric(v ?? "")}>
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
            <Select value={agg} onValueChange={(v) => setAgg(v ?? "avg")}>
              <SelectTrigger className="w-28">
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
            <Select value={chart} onValueChange={(v) => setChart(v ?? "line")}>
              <SelectTrigger className="w-28">
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
          <Button onClick={create} disabled={!name.trim() || !metric}>
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
            <li key={dashboard.id}>
              <Link
                to="/dashboards/$dashboardId"
                params={{ dashboardId: dashboard.id }}
                className="block rounded-lg border p-4 hover:bg-muted"
              >
                <span className="font-medium">{dashboard.name}</span>
                <span className="block text-xs text-muted-foreground">
                  {safeWidgetCount(dashboard.layout)} chart(s)
                </span>
              </Link>
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
