import { Link, createFileRoute } from "@tanstack/react-router"
import { useMemo } from "react"
import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts"
import {
  IconBellPlus,
  IconChartLine,
  IconLayoutDashboard,
} from "@tabler/icons-react"

import { Button } from "@/components/ui/button"

import { RangePicker } from "@/features/time-range"
import { PageHeader } from "@/shared/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { WhereClauseEditor } from "@/shared/console/where-clause-editor"
import { gqlString, graphqlCached } from "@/lib/api"
import {
  coerceAggregation,
  inferMetricKind,
  legalAggregations,
  type MetricAggregation,
  type MetricKind,
} from "@/lib/metric-aggregation"
import {
  mergeRangeSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  type ResolvedRange,
} from "@/lib/range"
import {
  serializeWhereClause,
  whereClauseFromSearch,
  type WhereFilter,
} from "@/lib/where-clause"

// Plan 168 metric detail view (preliminary). Runs on the existing GraphQL
// primitives (metricSeries / histogramQuantile / metricLabels); the
// metricCatalog/metricQuery single entry point, breakdown click-to-filter,
// where-filter, incomplete-bucket dashed tail, and graduation buttons land
// with the full plan.

interface MetricDetailSearch {
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  agg?: string | undefined
  where?: string | undefined
  groupBy?: string | undefined
  step?: string | undefined
  kind?: string | undefined
}

function searchString(value: unknown) {
  return typeof value === "string" && value ? value : undefined
}

const NO_GROUP = "__none__"
const STEP_OPTIONS = ["30", "60", "300", "900"] as const

// Full contract legality table (lib/metric-aggregation.ts): the plan-168
// metricQuery backend accepts gauge avg|min|max|last, sum sum|rate|increase,
// histogram p50|p95|p99|avg. Backend kinds are gauge|sum|histogram; summary
// maps to histogram semantics and unknown to gauge.
function backendKind(kind: MetricKind): "gauge" | "sum" | "histogram" {
  switch (kind) {
    case "sum":
      return "sum"
    case "histogram":
    case "summary":
      return "histogram"
    case "gauge":
    case "unknown":
      return "gauge"
  }
}

function supportedAggregations(kind: MetricKind): MetricAggregation[] {
  return [...legalAggregations(backendKind(kind))]
}

function resolveAggregation(
  kind: MetricKind,
  raw: string | undefined
): MetricAggregation {
  return coerceAggregation(backendKind(kind), raw) ?? "avg"
}

interface SeriesOut {
  groupValue: string | null
  points: Array<{ tsNanos: string; value: number }>
}

interface DetailData {
  labels: string[]
  series: SeriesOut[]
  range: ResolvedRange
}

async function loadDetail(
  metricName: string,
  search: MetricDetailSearch
): Promise<DetailData> {
  const range = resolveRangeSearch(search)
  const kind = (search.kind as MetricKind) || inferMetricKind(metricName)
  const agg = resolveAggregation(kind, search.agg)
  const stepSeconds = Number(search.step ?? "60") || 60
  const name = `"${gqlString(metricName)}"`
  const window = `fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}"`
  const groupBy = search.groupBy
    ? `, groupBy: "${gqlString(search.groupBy)}"`
    : ""
  const whereFilters = whereClauseFromSearch(search.where)
  const where =
    whereFilters.length > 0
      ? `, attributeFilters: [${whereFilters
          .map(
            (filter) =>
              `{key: "${gqlString(filter.key)}", op: "${gqlString(filter.op)}", value: "${gqlString(filter.value)}"}`
          )
          .join(", ")}]`
      : ""
  try {
    const data = await graphqlCached<{
      metricLabels: string[]
      metricQuery: {
        kind: string
        effectiveStepSeconds: number
        series: SeriesOut[]
      }
    }>(`{
      metricLabels(name: ${name})
      metricQuery(name: ${name}, kind: "${gqlString(backendKind(kind))}", agg: "${gqlString(agg)}", ${window}, stepSeconds: ${stepSeconds}${groupBy}${where}) {
        kind
        effectiveStepSeconds
        series { groupValue points { tsNanos value } }
      }
    }`)
    return {
      labels: data.metricLabels,
      series: data.metricQuery.series,
      range,
    }
  } catch {
    // Fall back to legacy metricSeries / histogramQuantile paths.
  }
  if ((kind === "histogram" || kind === "summary") && agg.startsWith("p")) {
    const q = Number(agg.slice(1)) / 100
    const data = await graphqlCached<{
      metricLabels: string[]
      histogramQuantile: Array<{ tsNanos: string; value: number }>
    }>(`{
      metricLabels(name: ${name})
      histogramQuantile(name: ${name}, ${window}, q: ${q}, stepSeconds: ${stepSeconds}) { tsNanos value }
    }`)
    return {
      labels: data.metricLabels,
      series: [{ groupValue: null, points: data.histogramQuantile }],
      range,
    }
  }
  const data = await graphqlCached<{
    metricLabels: string[]
    metricSeries: SeriesOut[]
  }>(`{
    metricLabels(name: ${name})
    metricSeries(name: ${name}, ${window}, agg: "${gqlString(agg)}", stepSeconds: ${stepSeconds}${groupBy}) {
      groupValue
      points { tsNanos value }
    }
  }`)
  return { labels: data.metricLabels, series: data.metricSeries, range }
}

export const Route = createFileRoute("/metrics/$metricName")({
  validateSearch: (search: Record<string, unknown>): MetricDetailSearch => ({
    ...rangeSearchSchema.parse(search),
    agg: searchString(search["agg"]),
    where: searchString(search["where"]),
    groupBy: searchString(search["groupBy"]),
    step: searchString(search["step"]),
    kind: searchString(search["kind"]),
  }),
  loaderDeps: ({ search }) => search,
  loader: ({ params, deps }) => loadDetail(params.metricName, deps),
  component: MetricDetailPage,
})

function MetricDetailPage() {
  const { metricName } = Route.useParams()
  const { labels, series, range } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = Route.useNavigate()

  const kind = (search.kind as MetricKind) || inferMetricKind(metricName)
  const legal = supportedAggregations(kind)
  const agg = resolveAggregation(kind, search.agg)

  const groups = useMemo(
    () =>
      series.map((entry, index) => entry.groupValue ?? `series-${index + 1}`),
    [series]
  )
  const rows = useMemo(() => {
    const byTime = new Map<string, Record<string, string | number>>()
    series.forEach((entry, index) => {
      const key = entry.groupValue ?? `series-${index + 1}`
      // The newest bucket is usually incomplete: render its segment as a
      // dashed continuation series instead of a confident solid drop.
      const tailStart = Math.max(entry.points.length - 2, 0)
      entry.points.forEach((point, pointIndex) => {
        const time = new Date(
          Number(BigInt(point.tsNanos) / 1_000_000n)
        ).toLocaleTimeString()
        const row = byTime.get(point.tsNanos) ?? { time }
        if (pointIndex < entry.points.length - 1) {
          row[key] = point.value
        }
        if (pointIndex >= tailStart && entry.points.length > 1) {
          row[`${key}__tail`] = point.value
        }
        byTime.set(point.tsNanos, row)
      })
    })
    return Array.from(byTime.entries())
      .sort(([a], [b]) => (BigInt(a) < BigInt(b) ? -1 : 1))
      .map(([, row]) => row)
  }, [series])

  const config = Object.fromEntries(
    groups.map((group, index) => [
      group,
      { label: group, color: `var(--chart-${(index % 5) + 1})` },
    ])
  ) satisfies ChartConfig

  const setSearch = (patch: Partial<MetricDetailSearch>) =>
    void navigate({ search: (prev) => ({ ...prev, ...patch }) })

  const whereFilters = whereClauseFromSearch(search.where)
  const applyWhere = (filters: WhereFilter[]) =>
    setSearch({ where: serializeWhereClause(filters) || undefined })
  // Breakdown click-to-filter: pin one group value as a where filter.
  const filterToGroup = (value: string) => {
    if (!search.groupBy || value.startsWith("series-")) return
    applyWhere([
      ...whereFilters.filter((filter) => filter.key !== search.groupBy),
      { key: search.groupBy, op: "=", value },
    ])
  }

  return (
    <div className="space-y-4 p-4">
      <PageHeader
        titleLeading={<IconChartLine className="size-5" />}
        title={metricName}
        description={
          <span className="flex items-center gap-2">
            <Badge variant="outline">{kind}</Badge>
            <Badge variant="secondary">{agg}</Badge>
          </span>
        }
        titleTrailing={
          <RangePicker
            value={range}
            onChange={(next) =>
              void navigate({
                search: (prev) => mergeRangeSearch(prev, next),
              })
            }
          />
        }
      />
      <div className="flex flex-wrap items-center gap-2">
        <Select
          value={agg}
          onValueChange={(next) =>
            setSearch({ agg: next as MetricAggregation })
          }
        >
          <SelectTrigger size="sm" className="w-32">
            <SelectValue placeholder="Aggregation" />
          </SelectTrigger>
          <SelectContent>
            {legal.map((option) => (
              <SelectItem key={option} value={option}>
                {option}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {backendKind(kind) !== "histogram" ? (
          <Select
            value={search.groupBy ?? NO_GROUP}
            onValueChange={(next) =>
              setSearch({
                groupBy: next == null || next === NO_GROUP ? undefined : next,
              })
            }
          >
            <SelectTrigger size="sm" className="w-44">
              <SelectValue placeholder="Group by" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={NO_GROUP}>No grouping</SelectItem>
              {labels.map((label) => (
                <SelectItem key={label} value={label}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : null}
        <Select
          value={search.step ?? "60"}
          onValueChange={(next) => setSearch({ step: next ?? undefined })}
        >
          <SelectTrigger size="sm" className="w-28">
            <SelectValue placeholder="Step" />
          </SelectTrigger>
          <SelectContent>
            {STEP_OPTIONS.map((option) => (
              <SelectItem key={option} value={option}>
                {option}s
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <WhereClauseEditor
          filters={whereFilters}
          onApply={applyWhere}
          keySuggestions={labels}
          className="min-w-64 flex-1"
        />
        <Button
          size="sm"
          variant="outline"
          render={
            <Link
              to="/dashboards"
              search={{
                widget_metric: metricName,
                widget_agg: agg,
                widget_group_by: search.groupBy,
              }}
            />
          }
        >
          <IconLayoutDashboard data-icon="inline-start" />
          Add to dashboard
        </Button>
        <Button
          size="sm"
          variant="outline"
          render={
            <Link
              to="/alerts"
              search={{
                signal_type: "metric",
                metric_name: metricName,
                metric_aggregation: agg,
              }}
            />
          }
        >
          <IconBellPlus data-icon="inline-start" />
          Create alert
        </Button>
      </div>
      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {agg} · {groups.length} series
          </CardTitle>
          {search.groupBy ? (
            <div className="flex flex-wrap gap-1 pt-1">
              {groups.map((group) => (
                <Badge
                  key={group}
                  variant="outline"
                  className="cursor-pointer"
                  onClick={() => filterToGroup(group)}
                >
                  {group}
                </Badge>
              ))}
            </div>
          ) : null}
        </CardHeader>
        <CardContent>
          <ChartContainer config={config} className="h-[280px] w-full">
            <LineChart data={rows} margin={{ left: 8, right: 8, top: 8 }}>
              <CartesianGrid vertical={false} />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                minTickGap={32}
              />
              <YAxis tickLine={false} axisLine={false} width={48} />
              <ChartTooltip content={<ChartTooltipContent />} />
              {groups.map((group) => (
                <Line
                  key={group}
                  dataKey={group}
                  stroke={`var(--color-${group})`}
                  dot={{ r: 2, strokeWidth: 0, fill: `var(--color-${group})` }}
                />
              ))}
              {groups.map((group) => (
                <Line
                  key={`${group}__tail`}
                  dataKey={`${group}__tail`}
                  stroke={`var(--color-${group})`}
                  strokeDasharray="4 4"
                  dot={false}
                  legendType="none"
                />
              ))}
            </LineChart>
          </ChartContainer>
        </CardContent>
      </Card>
    </div>
  )
}
