import { Link, useRouter } from "@tanstack/react-router"
import { useEffect, useState } from "react"
import { IconLayoutDashboard, IconPlus, IconTrash } from "@tabler/icons-react"

import { EmptyState } from "@/shared/console/empty-state"
import { RelativeTime } from "@/shared/console/relative-time"
import { PageHeader } from "@/shared/components/page-header"
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
import { deleteDashboard, saveDashboard } from "@/features/dashboards/api/dashboard-api"
import type { Dashboard } from "@/features/dashboards/model/dashboard"
import {
  dashboardRangeSearch,
  type DashboardSearch,
} from "@/features/dashboards/model/dashboard-search"
import {
  AGGS,
  CHARTS,
  emptyWidget,
  parseLayout,
  serializeWidgets,
  type Widget,
} from "@/features/dashboards/model/widget"
import { formatCount } from "@/shared/format"
import { gqlString, graphql } from "@/platform/graphql/transport"

const NO_GROUP = "__none__"
const ALL_VALUES = "__all__"

export function WidgetPicker({
  metricNames,
  value,
  onChange,
}: {
  metricNames: string[]
  value: Widget
  onChange: (widget: Widget) => void
}) {
  const [labels, setLabels] = useState<string[]>([])
  const [labelValues, setLabelValues] = useState<string[]>([])

  useEffect(() => {
    if (!value.metric) {
      setLabels([])
      return
    }
    let ignore = false
    void graphql<{
      metricLabels: string[]
    }>(`{ metricLabels(name: "${gqlString(value.metric)}") }`)
      .then((data) => {
        if (!ignore) setLabels(data.metricLabels)
      })
      .catch(() => {
        if (!ignore) setLabels([])
      })
    return () => {
      ignore = true
    }
  }, [value.metric])

  useEffect(() => {
    if (!value.metric || !value.groupBy) {
      setLabelValues([])
      return
    }
    let ignore = false
    const toNanos = (BigInt(Date.now()) * 1_000_000n).toString()
    void graphql<{
      metricLabelValues: string[]
    }>(`{ metricLabelValues(name: "${gqlString(value.metric)}", label: "${gqlString(value.groupBy)}", fromNanos: "0", toNanos: "${toNanos}") }`)
      .then((data) => {
        if (!ignore) setLabelValues(data.metricLabelValues)
      })
      .catch(() => {
        if (!ignore) setLabelValues([])
      })
    return () => {
      ignore = true
    }
  }, [value.metric, value.groupBy])

  const groupOptions =
    value.groupBy && !labels.includes(value.groupBy) ? [value.groupBy, ...labels] : labels
  const valueOptions =
    value.filterValue && !labelValues.includes(value.filterValue)
      ? [value.filterValue, ...labelValues]
      : labelValues

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
              groupBy: undefined,
              filterValue: undefined,
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
          onValueChange={(chart) => onChange({ ...value, chart: chart ?? "line" })}
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
        <Select
          value={value.groupBy ?? NO_GROUP}
          onValueChange={(groupBy) =>
            onChange({
              ...value,
              groupBy: !groupBy || groupBy === NO_GROUP ? undefined : groupBy,
              filterValue: undefined,
            })
          }
        >
          <SelectTrigger size="sm" className="w-44">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={NO_GROUP}>none</SelectItem>
            {groupOptions.map((label) => (
              <SelectItem key={label} value={label}>
                {label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {value.groupBy ? (
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">Filter value</label>
          <Select
            value={value.filterValue ?? ALL_VALUES}
            onValueChange={(filterValue) =>
              onChange({
                ...value,
                filterValue: !filterValue || filterValue === ALL_VALUES ? undefined : filterValue,
              })
            }
          >
            <SelectTrigger size="sm" className="w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUES}>all</SelectItem>
              {valueOptions.map((labelValue) => (
                <SelectItem key={labelValue} value={labelValue}>
                  {labelValue}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      ) : null}
    </div>
  )
}

export function DashboardsPage({
  dashboards,
  metricNames,
  search,
}: {
  dashboards: Dashboard[]
  metricNames: string[]
  search: DashboardSearch
}) {
  const router = useRouter()
  const detailSearch = dashboardRangeSearch(search)
  // Metric-explorer graduation (plan 168): pre-fill and open the create
  // dialog with the explored query as the first widget.
  const graduationWidget: Widget | null = search.widget_metric
    ? {
        ...emptyWidget(),
        metric: search.widget_metric,
        agg: search.widget_agg ?? "avg",
        groupBy: search.widget_group_by,
        title: `${search.widget_metric} (${search.widget_agg ?? "avg"})`,
      }
    : null
  const [deleteError, setDeleteError] = useState<string | null>(null)

  async function remove(id: string) {
    setDeleteError(null)
    try {
      await deleteDashboard(id)
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
          <DashboardCreateDialog
            metricNames={metricNames}
            initialWidget={graduationWidget}
            detailSearch={detailSearch}
            onCreated={(dashboardId, createdSearch) =>
              router.navigate({
                to: "/dashboards/$dashboardId",
                params: { dashboardId },
                search: createdSearch,
              })
            }
          />
        }
      />

      {deleteError ? <p className="text-sm text-destructive">{deleteError}</p> : null}

      {dashboards.length === 0 ? (
        <EmptyState
          icon={IconLayoutDashboard}
          title="Create your first dashboard"
          description="Dashboards appear here after you save a metric layout."
        />
      ) : (
        <DashboardCards dashboards={dashboards} detailSearch={detailSearch} onRemove={remove} />
      )}
    </div>
  )
}

export function DashboardCreateDialog({
  metricNames,
  detailSearch,
  initialWidget = null,
  onCreated,
}: {
  metricNames: string[]
  initialWidget?: Widget | null
  detailSearch: ReturnType<typeof dashboardRangeSearch>
  onCreated: (
    dashboardId: string,
    search: ReturnType<typeof dashboardRangeSearch>
  ) => void | Promise<void>
}) {
  const [open, setOpen] = useState(Boolean(initialWidget))
  const [name, setName] = useState("")
  const [widgets, setWidgets] = useState<Widget[]>([initialWidget ?? emptyWidget()])
  const [error, setError] = useState<string | null>(null)

  async function create() {
    setError(null)
    try {
      const valid = widgets.filter((widget) => widget.metric)
      const created = await saveDashboard({
        name,
        layout: serializeWidgets(valid),
      })
      setName("")
      setWidgets([emptyWidget()])
      setOpen(false)
      await onCreated(created.id, detailSearch)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={<Button />}>
        <IconPlus />
        New dashboard
      </DialogTrigger>
      <DialogContent className="sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>New dashboard</DialogTitle>
          <DialogDescription>Choose metrics, aggregation, and chart type.</DialogDescription>
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
                setWidgets((current) => current.map((item, i) => (i === index ? next : item)))
              }
            />
          ))}
          <Button
            type="button"
            variant="outline"
            onClick={() => setWidgets((current) => [...current, emptyWidget()])}
          >
            <IconPlus />
            Add widget
          </Button>
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
        </div>
        <DialogFooter>
          <Button
            disabled={!name.trim() || widgets.every((widget) => !widget.metric)}
            onClick={() => void create()}
          >
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export function DashboardCards({
  dashboards,
  detailSearch,
  onRemove,
}: {
  dashboards: Dashboard[]
  detailSearch: ReturnType<typeof dashboardRangeSearch>
  onRemove: (id: string) => void | Promise<void>
}) {
  return (
    <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {dashboards.map((dashboard) => (
        <li key={dashboard.id}>
          <Card>
            <CardHeader className="flex-row items-center justify-between">
              <CardTitle className="truncate text-sm">
                <Link
                  to="/dashboards/$dashboardId"
                  params={{ dashboardId: dashboard.id }}
                  search={detailSearch}
                  className="hover:underline"
                >
                  {dashboard.name}
                </Link>
              </CardTitle>
              <AlertDialog>
                <AlertDialogTrigger render={<Button variant="ghost-destructive" size="icon-xs" />}>
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
                      onClick={() => void onRemove(dashboard.id)}
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
  )
}
