import { Link, createFileRoute, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import { IconLayoutDashboard, IconPlus, IconTrash } from "@tabler/icons-react"

import { EmptyState } from "@/components/console/empty-state"
import { RelativeTime } from "@/components/console/relative-time"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Combobox } from "@/components/ui/combobox"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { gqlString, graphql } from "@/lib/api"
import { formatCount } from "@/lib/format"

interface Dashboard {
  id: string
  name: string
  layout: string
  updatedAtNanos: string
}

export interface Widget {
  metric: string
  agg: string
  chart: string
  title: string
  groupBy?: string | undefined
  w?: number
  [key: string]: unknown
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
          updatedAtNanos
        }
        metricNames
      }
    `),
  component: DashboardsPage,
})

export function emptyWidget(): Widget {
  return { metric: "", agg: "avg", chart: "line", title: "", w: 1 }
}

export function parseLayout(layout: string): Widget[] {
  try {
    const parsed: unknown = JSON.parse(layout)
    return Array.isArray(parsed)
      ? parsed.filter(
          (item): item is Widget =>
            typeof item === "object" &&
            item !== null &&
            typeof (item as Widget).metric === "string"
        )
      : []
  } catch {
    return []
  }
}

export function serializeWidgets(widgets: Widget[]): string {
  return JSON.stringify(widgets)
}

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
    <div className="flex flex-wrap items-end gap-2">
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">Metric</label>
        <Combobox
          value={value.metric}
          options={metricNames}
          placeholder="Search metrics"
          onChange={(metric) =>
            onChange({
              ...value,
              metric,
              title: metric ? `${metric} (${value.agg})` : "",
            })
          }
        />
      </div>
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">Aggregation</label>
        <Select
          value={value.agg}
          onValueChange={(agg) =>
            onChange({
              ...value,
              agg: agg ?? "avg",
              title: `${value.metric} (${agg ?? "avg"})`,
            })
          }
        >
          <SelectTrigger size="sm" className="w-24">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {AGGS.map((agg) => (
              <SelectItem key={agg} value={agg}>
                {agg}
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
          <SelectTrigger size="sm" className="w-24">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {CHARTS.map((chart) => (
              <SelectItem key={chart} value={chart}>
                {chart}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">Group by</label>
        <Input
          value={value.groupBy ?? ""}
          onChange={(event) =>
            onChange({
              ...value,
              groupBy: event.target.value || undefined,
            })
          }
          placeholder="attribute"
          className="h-8 w-40"
        />
      </div>
    </div>
  )
}

function DashboardsPage() {
  const { dashboards, metricNames } = Route.useLoaderData()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [widgets, setWidgets] = useState<Widget[]>([emptyWidget()])
  const [error, setError] = useState<string | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)

  async function create() {
    setError(null)
    try {
      const valid = widgets.filter((widget) => widget.metric)
      const { dashboardSave } = await graphql<{
        dashboardSave: { id: string }
      }>(
        `mutation { dashboardSave(name: "${gqlString(name)}",
           layout: "${gqlString(serializeWidgets(valid))}") { id } }`
      )
      setName("")
      setWidgets([emptyWidget()])
      setOpen(false)
      await router.navigate({
        to: "/dashboards/$dashboardId",
        params: { dashboardId: dashboardSave.id },
      })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  async function remove(id: string) {
    setDeleteError(null)
    try {
      await graphql(`mutation { dashboardDelete(id: "${gqlString(id)}") }`)
      await router.invalidate()
    } catch (err) {
      setDeleteError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconLayoutDashboard}
        iconClassName="text-fuchsia-500"
        title="Dashboards"
        description="Saved metric views built from telemetry already ingested."
        actions={
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button />}>
              <IconPlus />
              New dashboard
            </DialogTrigger>
            <DialogContent className="sm:max-w-3xl">
              <DialogHeader>
                <DialogTitle>New dashboard</DialogTitle>
                <DialogDescription>
                  Choose metrics, aggregation, and chart type.
                </DialogDescription>
              </DialogHeader>
              <div className="flex flex-col gap-4">
                <Input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="checkout ops"
                />
                {widgets.map((widget, index) => (
                  <WidgetPicker
                    key={index}
                    metricNames={metricNames}
                    value={widget}
                    onChange={(next) =>
                      setWidgets((current) =>
                        current.map((item, i) => (i === index ? next : item))
                      )
                    }
                  />
                ))}
                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    setWidgets((current) => [...current, emptyWidget()])
                  }
                >
                  <IconPlus />
                  Add widget
                </Button>
                {error ? (
                  <p className="text-sm text-destructive">{error}</p>
                ) : null}
              </div>
              <DialogFooter>
                <Button
                  disabled={
                    !name.trim() || widgets.every((widget) => !widget.metric)
                  }
                  onClick={create}
                >
                  Create
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        }
      />

      {deleteError ? (
        <p className="text-sm text-destructive">{deleteError}</p>
      ) : null}

      {dashboards.length === 0 ? (
        <EmptyState
          icon={IconLayoutDashboard}
          title="Create your first dashboard"
          description="Dashboards appear here after you save a metric layout."
        />
      ) : (
        <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {dashboards.map((dashboard) => (
            <li key={dashboard.id}>
              <Card>
                <CardHeader className="flex-row items-center justify-between">
                  <CardTitle className="truncate text-sm">
                    <Link
                      to="/dashboards/$dashboardId"
                      params={{ dashboardId: dashboard.id }}
                      className="hover:underline"
                    >
                      {dashboard.name}
                    </Link>
                  </CardTitle>
                  <AlertDialog>
                    <AlertDialogTrigger
                      render={
                        <Button variant="ghost-destructive" size="icon-xs" />
                      }
                    >
                      <IconTrash />
                      <span className="sr-only">Delete</span>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Delete dashboard?</AlertDialogTitle>
                        <AlertDialogDescription>
                          Delete {dashboard.name}. This cannot be undone.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                          variant="destructive"
                          onClick={() => void remove(dashboard.id)}
                        >
                          Delete
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </CardHeader>
                <CardContent className="flex items-center justify-between gap-2">
                  <Badge variant="secondary">
                    {formatCount(parseLayout(dashboard.layout).length)} widgets
                  </Badge>
                  <span className="text-xs text-muted-foreground">
                    <RelativeTime nanos={dashboard.updatedAtNanos} />
                  </span>
                </CardContent>
              </Card>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
