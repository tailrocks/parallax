import { useNavigate, useRouter } from "@tanstack/react-router"
import { useState } from "react"
import {
  IconArrowDown,
  IconArrowUp,
  IconLayoutDashboard,
  IconPencil,
  IconPlus,
  IconTrash,
} from "@tabler/icons-react"
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

import { EmptyState } from "@/shared/console/empty-state"
import { RangePicker } from "@/features/time-range"
import { ChartLegend } from "@/shared/console/trend"
import { navItem } from "@/shared/navigation"
import { PageHeader } from "@/shared/components/page-header"
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
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import {
  deleteDashboard,
  saveDashboard,
} from "@/features/dashboards/api/dashboard-api"
import { WidgetPicker } from "@/features/dashboards/components/dashboards-page"
import {
  emptyWidget,
  serializeWidgets,
  type Widget,
} from "@/features/dashboards/model/widget"
import { formatCount } from "@/lib/format"
import { mergeRangeSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import type { WidgetData } from "@/features/dashboards/model/widget-data"
import { cn } from "@/lib/utils"

export function DashboardDetailPage({
  id,
  name,
  widgets,
  data,
  metricNames,
  range,
}: {
  id: string
  name: string
  widgets: Widget[]
  data: WidgetData[]
  metricNames: string[]
  range: ResolvedRange
}) {
  const router = useRouter()
  const navigate = useNavigate({ from: "/dashboards/$dashboardId" })
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState<Widget[]>(widgets)
  const [addition, setAddition] = useState<Widget>(emptyWidget())
  const [error, setError] = useState<string | null>(null)
  const dashboardsBack = navItem("/dashboards")

  async function save(layout: Widget[]) {
    setError(null)
    try {
      await saveDashboard({ name, layout: serializeWidgets(layout), id })
      setEditing(false)
      await router.invalidate()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  async function removeDashboard() {
    setError(null)
    try {
      await deleteDashboard(id)
      await router.navigate({ to: "/dashboards" })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
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
      data.find((item) => item.widget === widget) ?? {
        widget,
        groups: [],
        rows: [],
      }
  )

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        {...(dashboardsBack ? { back: dashboardsBack } : {})}
        title={name}
        actions={
          <>
            <RangePicker
              value={range}
              onChange={(next) =>
                void navigate({
                  search: (current) => mergeRangeSearch(current, next),
                })
              }
            />
            {editing ? (
              <>
                <Button size="sm" onClick={() => void save(draft)}>
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
                  <IconPencil />
                  Edit
                </Button>
                <AlertDialog>
                  <AlertDialogTrigger
                    render={<Button size="sm" variant="ghost-destructive" />}
                  >
                    <IconTrash />
                    Delete
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Delete dashboard?</AlertDialogTitle>
                      <AlertDialogDescription>
                        Delete {name}. This cannot be undone.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        variant="destructive"
                        onClick={() => void removeDashboard()}
                      >
                        Delete
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </>
            )}
          </>
        }
      />

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
              <IconPlus />
              Add
            </Button>
          </CardContent>
        </Card>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        {shownData.length === 0 ? (
          <EmptyState
            className="lg:col-span-2"
            icon={IconLayoutDashboard}
            title="No widgets yet"
            description="Enter edit mode and add a metric your app already sends."
          />
        ) : (
          shownData.map((item, index) => (
            <WidgetChart
              key={`${item.widget.metric}-${index}`}
              data={item}
              editing={editing}
              onRemove={() => setDraft(draft.filter((_, i) => i !== index))}
              onMove={(delta) => move(index, delta)}
            />
          ))
        )}
      </div>
    </div>
  )
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
  const common = { data: data.rows, margin: { left: 8, right: 8, top: 8 } }
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
    <Card className={cn("min-h-[260px]", wide && "lg:col-span-2")}>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle className="truncate text-sm">
          {data.widget.title || data.widget.metric}
        </CardTitle>
        {editing ? (
          <span className="flex gap-1">
            <Button variant="ghost" size="icon-xs" onClick={() => onMove(-1)}>
              <IconArrowUp />
              <span className="sr-only">Move widget up</span>
            </Button>
            <Button variant="ghost" size="icon-xs" onClick={() => onMove(1)}>
              <IconArrowDown />
              <span className="sr-only">Move widget down</span>
            </Button>
            <Button
              variant="ghost-destructive"
              size="icon-xs"
              onClick={onRemove}
            >
              <IconTrash />
              <span className="sr-only">Remove widget</span>
            </Button>
          </span>
        ) : null}
      </CardHeader>
      <CardContent>
        <div className="mb-3 flex flex-wrap items-center gap-2">
          <Badge variant="outline">{data.widget.agg}</Badge>
          <Badge variant="secondary">{data.widget.chart}</Badge>
          <span className="text-xs text-muted-foreground">
            {formatCount(data.rows.length)} points
          </span>
          {data.groups.length > 1 ? (
            <ChartLegend
              items={data.groups.map((group, index) => ({
                key: group,
                label: group,
                color: `var(--chart-${index + 1})`,
              }))}
            />
          ) : null}
        </div>
        <ChartContainer config={config} className="h-[220px] w-full">
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
                  // Small dots keep an isolated bucket visible when the gap
                  // fill breaks the line on both sides of it.
                  dot={{ r: 2, strokeWidth: 0, fill: `var(--color-${group})` }}
                />
              ))}
            </LineChart>
          )}
        </ChartContainer>
      </CardContent>
    </Card>
  )
}
