import {
  createFileRoute,
  notFound,
  useNavigate,
  useRouter,
} from "@tanstack/react-router"
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

import { EmptyState } from "@/components/console/empty-state"
import { RangePicker } from "@/components/console/range-picker"
import { ChartLegend } from "@/components/console/trend"
import { navItem } from "@/components/nav"
import { PageHeader } from "@/components/page-header"
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
import { loadWidgetSeries as loadWidgetSeriesDecoded } from "@/features/dashboards/api/widget-series-api"
import type { WidgetSeries } from "@/features/dashboards/api/widget-series-schema"
import { gqlString, graphql, graphqlCached } from "@/lib/api"
import { formatCount, formatTimeInRange } from "@/lib/format"
import {
  mergeRangeSearch,
  resolveRangeSearch,
  rangeSearchSchema,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"
import {
  WidgetPicker,
  emptyWidget,
  parseLayout,
  serializeWidgets,
} from "./dashboards.index"
import type { Widget } from "./dashboards.index"

type Series = WidgetSeries

interface WidgetData {
  widget: Widget
  groups: string[]
  rows: Record<string, number | string>[]
}

const MAX_GROUPS = 5

/** Plan 152 dynamic dashboard series adapter (AST + alias-aware decode). */
export async function loadWidgetSeries(
  widgets: Widget[],
  range: ResolvedRange,
  fetch?: (
    query: string,
    init?: { signal?: AbortSignal }
  ) => Promise<Record<string, unknown>>
): Promise<Series[][]> {
  if (fetch) {
    return loadWidgetSeriesDecoded(widgets, range, fetch)
  }
  return loadWidgetSeriesDecoded(widgets, range)
}

export const Route = createFileRoute("/dashboards/$dashboardId")({
  validateSearch: (search: Record<string, unknown>) =>
    rangeSearchSchema.parse(search),
  loaderDeps: ({ search }) => search,
  loader: async ({ params, deps }) => {
    const range = resolveRangeSearch(deps)
    const { dashboard, metricNames } = await graphqlCached<{
      dashboard: { id: string; name: string; layout: string } | null
      metricNames: string[]
    }>(
      `{ dashboard(id: "${gqlString(params.dashboardId)}") { id name layout }
         metricNames }`
    )
    if (!dashboard) throw notFound()
    const widgets = parseLayout(dashboard.layout)
    const seriesList = await loadWidgetSeries(widgets, range)
    const data = widgets.map((widget, index) =>
      toWidgetData(widget, seriesList[index] ?? [], range)
    )
    return {
      id: dashboard.id,
      name: dashboard.name,
      widgets,
      data,
      metricNames,
      range,
    }
  },
  component: DashboardPage,
})

function toWidgetData(
  widget: Widget,
  series: Series[],
  range: ResolvedRange
): WidgetData {
  const kept = series.slice(0, MAX_GROUPS)
  const groups = kept.map(
    (s, i) => s.groupValue ?? (i === 0 ? "value" : `#${i}`)
  )
  const byTime = new Map<string, Record<string, number | string>>()
  kept.forEach((s, index) => {
    const group = groups[index]
    if (!group) return
    for (const point of s.points) {
      const row = byTime.get(point.tsNanos) ?? {
        time: formatTimeInRange(point.tsNanos, range),
      }
      row[group] = point.value
      byTime.set(point.tsNanos, row)
    }
  })
  const entries = [...byTime.entries()].sort(([a], [b]) =>
    BigInt(a) < BigInt(b) ? -1 : 1
  )
  const rows = fillBucketGaps(entries, range)
  return { widget, groups, rows }
}

/** Insert empty rows for skipped buckets so a gauge that stopped reporting
 * renders a line BREAK instead of silently bridging the gap (corpus id
 * m-shapes). Bucket step = the smallest observed inter-point delta. */
const MAX_FILLED_ROWS = 2_000

function fillBucketGaps(
  entries: Array<[string, Record<string, number | string>]>,
  range: ResolvedRange
): Array<Record<string, number | string>> {
  if (entries.length < 2) return entries.map(([, row]) => row)
  let step = 0n
  for (let i = 1; i < entries.length; i += 1) {
    const delta = BigInt(entries[i]![0]) - BigInt(entries[i - 1]![0])
    if (delta > 0n && (step === 0n || delta < step)) step = delta
  }
  if (step === 0n) return entries.map(([, row]) => row)
  const rows: Array<Record<string, number | string>> = []
  for (let i = 0; i < entries.length; i += 1) {
    const [ts, row] = entries[i]!
    if (i > 0) {
      let cursor = BigInt(entries[i - 1]![0]) + step
      const end = BigInt(ts)
      while (cursor < end && rows.length < MAX_FILLED_ROWS) {
        rows.push({ time: formatTimeInRange(cursor.toString(), range) })
        cursor += step
      }
    }
    rows.push(row)
  }
  return rows
}

function DashboardPage() {
  const { id, name, widgets, data, metricNames, range } = Route.useLoaderData()
  const router = useRouter()
  const navigate = useNavigate({ from: Route.fullPath })
  const [editing, setEditing] = useState(false)
  const [draft, setDraft] = useState<Widget[]>(widgets)
  const [addition, setAddition] = useState<Widget>(emptyWidget())
  const [error, setError] = useState<string | null>(null)
  const dashboardsBack = navItem("/dashboards")

  async function save(layout: Widget[]) {
    setError(null)
    try {
      await graphql(`mutation { dashboardSave(name: "${gqlString(name)}",
           layout: "${gqlString(serializeWidgets(layout))}",
           id: "${gqlString(id)}") { id } }`)
      setEditing(false)
      await router.invalidate()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  async function removeDashboard() {
    setError(null)
    try {
      await graphql(`mutation { dashboardDelete(id: "${gqlString(id)}") }`)
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
