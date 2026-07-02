import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import {
  BarChart3,
  Gauge,
  LayoutDashboard,
  Plus,
  Settings2,
} from "lucide-react"
import { graphql, gqlString } from "@/lib/api"
import { KpiCard } from "@/components/kpi-card"
import { PageHeading } from "@/components/page-heading"
import { Badge } from "@/components/ui/badge"
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
  groupBy?: string | undefined
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
      <div className="flex flex-col gap-1">
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
      <div className="flex flex-col gap-1">
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
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">Chart</label>
        <Select
          value={value.chart}
          onValueChange={(chart) =>
            onChange({ ...value, chart: chart ?? "line" })
          }
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
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">
          Group by attribute
        </label>
        <Input
          value={value.groupBy ?? ""}
          onChange={(e) => {
            const groupBy = e.target.value
            const next = { ...value }
            if (groupBy) {
              next.groupBy = groupBy
            } else {
              delete next.groupBy
            }
            onChange(next)
          }}
          placeholder="e.g. payment.method"
          className="w-44"
        />
      </div>
      <div className="flex flex-col gap-1">
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

  const chartCount = dashboards.reduce(
    (count, dashboard) => count + safeWidgetCount(dashboard.layout),
    0
  )

  return (
    <div className="flex flex-col gap-4">
      <PageHeading
        eyebrow="Metric dashboards"
        title="Dashboards"
        description="Build compact telemetry views from the metrics Parallax already receives."
      />

      <div className="grid gap-3 md:grid-cols-4">
        <KpiCard
          icon={LayoutDashboard}
          label="Dashboards"
          value={dashboards.length.toLocaleString()}
          detail="saved views"
          tone="blue"
        />
        <KpiCard
          icon={BarChart3}
          label="Charts"
          value={chartCount.toLocaleString()}
          detail="configured widgets"
          tone="orange"
        />
        <KpiCard
          icon={Gauge}
          label="Metrics"
          value={metricNames.length.toLocaleString()}
          detail="available series"
          tone="green"
        />
        <KpiCard
          icon={Settings2}
          label="Builder"
          value={widget.metric ? "Ready" : "Pick"}
          detail={widget.metric || "choose a metric"}
          tone="violet"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <Plus />
            New dashboard
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap items-end gap-2">
          <div className="flex flex-col gap-1">
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
        <div className="parallax-panel p-6 text-sm text-muted-foreground">
          No dashboards yet — pick one of the metrics your apps already send.
        </div>
      ) : (
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {dashboards.map((dashboard) => (
            <li key={dashboard.id} className="relative">
              <Link
                to="/dashboards/$dashboardId"
                params={{ dashboardId: dashboard.id }}
                className="parallax-panel block min-h-32 p-4 pr-20 transition hover:border-primary/50 hover:bg-muted/30"
              >
                <span className="grid size-9 place-items-center rounded-2xl bg-[color-mix(in_srgb,var(--brand-blue),transparent_18%)] text-white">
                  <LayoutDashboard />
                </span>
                <span className="mt-4 block truncate font-medium">
                  {dashboard.name}
                </span>
                <span className="mt-2 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <Badge variant="outline">
                    {safeWidgetCount(dashboard.layout)} chart(s)
                  </Badge>
                  <span>last hour window</span>
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
