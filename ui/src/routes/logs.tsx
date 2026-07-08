import {
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import {
  IconArticleFilled,
  IconColumns,
  IconPlayerPlayFilled,
  IconPlayerStopFilled,
  IconRefresh,
  IconX,
} from "@tabler/icons-react"
import { useEffect, useMemo, useState } from "react"
import {
  Bar,
  BarChart,
  CartesianGrid,
  ReferenceArea,
  XAxis,
  YAxis,
} from "recharts"
import { z } from "zod"

import { EmptyState } from "@/components/console/empty-state"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/components/console/range-picker"
import { TableSkeleton } from "@/components/console/skeletons"
import {
  LogsTable,
  parseLogColumns,
  serializeLogColumns,
} from "@/components/logs-table"
import type { LogDoc, OptionalLogColumn } from "@/components/logs-table"
import { PageHeader } from "@/components/page-header"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import type { ChartConfig } from "@/components/ui/chart"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { gqlString, graphql } from "@/lib/api"
import { formatCount, formatTimeInRange } from "@/lib/format"
import { resolveRangeSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface LogsData {
  services: string[]
  logs: LogDoc[]
  logCountSeries: SeriesPoint[]
}

interface LogsSearch {
  q?: string | undefined
  service?: string | undefined
  sev?: number | undefined
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  live?: boolean | undefined
  cols?: string | undefined
}

const PAGE_SIZE = 500
const SEVERITIES = [
  { label: "All severities", value: undefined },
  { label: "Debug+", value: 5 },
  { label: "Info+", value: 9 },
  { label: "Warn+", value: 13 },
  { label: "Error+", value: 17 },
] as const

const logsSearchSchema = z.object({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  sev: z.unknown().optional(),
  range: z.unknown().optional(),
  from: z.unknown().optional(),
  to: z.unknown().optional(),
  live: z.unknown().optional(),
  cols: z.unknown().optional(),
})

export function validateLogsSearch(
  search: Record<string, unknown>
): LogsSearch {
  const parsed = logsSearchSchema.parse(search)
  const severity = Number(parsed.sev)
  return {
    q: typeof parsed.q === "string" && parsed.q ? parsed.q : undefined,
    service:
      typeof parsed.service === "string" && parsed.service
        ? parsed.service
        : undefined,
    sev: Number.isFinite(severity) && severity > 0 ? severity : undefined,
    range: typeof parsed.range === "string" ? parsed.range : undefined,
    from: typeof parsed.from === "string" ? parsed.from : undefined,
    to: typeof parsed.to === "string" ? parsed.to : undefined,
    live: parsed.live === "1" || parsed.live === true,
    cols: typeof parsed.cols === "string" ? parsed.cols : undefined,
  }
}

export const Route = createFileRoute("/logs")({
  validateSearch: validateLogsSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => loadLogs(deps),
  component: LogsPage,
})

export function stepSecondsForRange(range: ResolvedRange): number {
  const spanNs = BigInt(range.toNanos) - BigInt(range.fromNanos)
  return Math.max(30, Math.round(Number(spanNs / 1_000_000_000n) / 60))
}

export function bucketWindow(
  points: readonly SeriesPoint[],
  index: number,
  stepSeconds: number
) {
  const point = points[index]
  if (!point) return null
  return {
    fromNanos: point.tsNanos,
    toNanos: (
      BigInt(point.tsNanos) +
      BigInt(stepSeconds) * 1_000_000_000n
    ).toString(),
  }
}

export function dragWindow(
  points: readonly SeriesPoint[],
  start: number,
  end: number,
  stepSeconds: number
) {
  const low = Math.min(start, end)
  const high = Math.max(start, end)
  const from = points[low]
  const to = bucketWindow(points, high, stepSeconds)
  if (!from || !to) return null
  return { fromNanos: from.tsNanos, toNanos: to.toNanos }
}

export async function loadLogs(search: LogsSearch): Promise<LogsData> {
  const range = resolveRangeSearch(search)
  const stepSeconds = stepSecondsForRange(range)
  const filters = [
    search.service ? `service: "${gqlString(search.service)}"` : "",
    search.sev ? `severityMin: ${search.sev}` : "",
    search.q ? `query: "${gqlString(search.q)}"` : "",
  ].filter(Boolean)
  if (search.live) {
    return graphql<LogsData>(
      `{ services logs(limit: 0) { tsNanos service severityNum severityText body traceId spanId runId scopeName attributes resource } logCountSeries(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) { tsNanos value } }`
    )
  }
  const logArgs = [
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    ...filters,
    `limit: ${PAGE_SIZE}`,
  ].join(", ")
  const seriesArgs = [
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    ...filters,
    `stepSeconds: ${stepSeconds}`,
  ].join(", ")
  return graphql<LogsData>(`{
    services
    logs(${logArgs}) {
      tsNanos service severityNum severityText body traceId spanId runId scopeName attributes resource
    }
    logCountSeries(${seriesArgs}) { tsNanos value }
  }`)
}

function LogsPage() {
  const data = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: "/logs" })
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const delayedLoading = useDelayedLoading(routerLoading)
  const range = resolveRangeSearch(search)
  const stepSeconds = stepSecondsForRange(range)
  const [logs, setLogs] = useState<LogDoc[]>(data.logs)
  const [pendingQuery, setPendingQuery] = useState(search.q ?? "")
  const [olderLoading, setOlderLoading] = useState(false)
  const [olderError, setOlderError] = useState<string | null>(null)
  const [exhausted, setExhausted] = useState(data.logs.length < PAGE_SIZE)
  const [dragStart, setDragStart] = useState<number | null>(null)
  const [dragEnd, setDragEnd] = useState<number | null>(null)
  const live = search.live === true
  const columns = parseLogColumns(search.cols)

  useEffect(() => {
    setLogs(data.logs)
    setExhausted(data.logs.length < PAGE_SIZE)
  }, [data.logs])

  useEffect(() => setPendingQuery(search.q ?? ""), [search.q])

  useEffect(() => {
    if (!live) return
    const params = new URLSearchParams()
    if (search.service) params.set("service", search.service)
    if (search.sev) params.set("severity_min", String(search.sev))
    if (search.q) params.set("q", search.q)
    const source = new EventSource(`/v1/logs/stream?${params}`)
    let buffer: LogDoc[] = []
    source.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) buffer.push(...(batch as LogDoc[]))
      } catch {
        // skip malformed frames
      }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      setLogs((current) =>
        [...incoming.reverse(), ...current].slice(0, PAGE_SIZE)
      )
    }, 250)
    return () => {
      source.close()
      clearInterval(flush)
    }
  }, [live, search.service, search.sev, search.q])

  const update = (patch: Partial<LogsSearch>) =>
    void navigate({
      search: (current) => ({ ...current, ...patch }),
    })

  const setRange = (next: ResolvedRange) => {
    update({ range: next.key, from: next.fromNanos, to: next.toNanos })
  }

  const loadOlder = async () => {
    const oldest = logs.at(-1)
    if (!oldest) return
    setOlderLoading(true)
    setOlderError(null)
    try {
      const args = [
        `fromNanos: "${range.fromNanos}"`,
        `toNanos: "${(BigInt(oldest.tsNanos) - 1n).toString()}"`,
        search.service ? `service: "${gqlString(search.service)}"` : "",
        search.sev ? `severityMin: ${search.sev}` : "",
        search.q ? `query: "${gqlString(search.q)}"` : "",
        `limit: ${PAGE_SIZE}`,
      ]
        .filter(Boolean)
        .join(", ")
      const more = await graphql<{ logs: LogDoc[] }>(`{ logs(${args}) {
        tsNanos service severityNum severityText body traceId spanId runId scopeName attributes resource
      } }`)
      setLogs((current) => [...current, ...more.logs])
      if (more.logs.length < PAGE_SIZE) setExhausted(true)
    } catch (err) {
      setOlderError(err instanceof Error ? err.message : String(err))
    } finally {
      setOlderLoading(false)
    }
  }

  const total = data.logCountSeries.reduce((sum, point) => sum + point.value, 0)
  const customWindow = search.from && search.to

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconArticleFilled}
        title="Logs"
        description="Search, tail, and narrow log records by histogram window."
        actions={
          <>
            <RangePicker value={range} onChange={setRange} />
            <Button
              type="button"
              variant={live ? "secondary" : "outline"}
              size="sm"
              onClick={() => update({ live: live ? undefined : true })}
            >
              {live ? <IconPlayerStopFilled /> : <IconPlayerPlayFilled />}
              {live ? "Live" : "Query"}
            </Button>
          </>
        }
      />

      <div className="flex flex-wrap items-center gap-2 rounded-xl border border-border/70 bg-muted/20 p-3">
        <Select
          value={search.service ?? "all"}
          onValueChange={(value) =>
            update({
              service: !value || value === "all" ? undefined : value,
            })
          }
        >
          <SelectTrigger className="w-48">
            <SelectValue placeholder="All services" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All services</SelectItem>
            {data.services.map((service) => (
              <SelectItem key={service} value={service}>
                {service}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select
          value={String(search.sev ?? 0)}
          onValueChange={(value) => update({ sev: Number(value) || undefined })}
        >
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {SEVERITIES.map((severity) => (
              <SelectItem
                key={severity.value ?? 0}
                value={String(severity.value ?? 0)}
              >
                {severity.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <form
          className="flex min-w-64 flex-1 gap-2"
          onSubmit={(event) => {
            event.preventDefault()
            update({ q: pendingQuery.trim() || undefined })
          }}
        >
          <Input
            value={pendingQuery}
            onChange={(event) => setPendingQuery(event.target.value)}
            placeholder="Filter log bodies"
          />
        </form>
        <ColumnMenu
          columns={columns}
          onChange={(next) => update({ cols: serializeLogColumns(next) })}
        />
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => update({})}
        >
          <IconRefresh />
          Refresh
        </Button>
      </div>

      <HistogramCard
        live={live}
        range={range}
        series={data.logCountSeries}
        stepSeconds={stepSeconds}
        dragStart={dragStart}
        dragEnd={dragEnd}
        onDragStart={setDragStart}
        onDragEnd={setDragEnd}
        onWindow={(fromNanos, toNanos) =>
          update({ range: "custom", from: fromNanos, to: toNanos })
        }
        onReset={() =>
          update({ from: undefined, to: undefined, range: undefined })
        }
        customWindow={Boolean(customWindow)}
        total={total}
      />

      {delayedLoading ? (
        <TableSkeleton rows={8} />
      ) : logs.length === 0 ? (
        <EmptyState
          title={
            search.q || search.service || search.sev
              ? "No matching logs"
              : "No logs yet"
          }
          description={
            search.q || search.service || search.sev
              ? "Clear filters or widen the time range."
              : "Send OTLP logs to http://127.0.0.1:4317 or run through parallax run."
          }
          icon={IconArticleFilled}
          className="rounded-xl border border-dashed"
        />
      ) : (
        <div className="overflow-hidden rounded-xl border border-border/70">
          {live ? (
            <div className="flex items-center gap-2 border-b border-border/70 px-3 py-2 text-xs">
              <span className="size-2 animate-pulse rounded-full bg-emerald-500" />
              <Badge variant="emerald">Live</Badge>
              <span className="text-muted-foreground">
                {formatCount(logs.length)} records buffered
              </span>
            </div>
          ) : null}
          <LogsTable logs={logs} range={range} columns={columns} />
          {!live && !exhausted ? (
            <div className="flex flex-col gap-2 border-t border-border/70 p-2">
              {olderError ? (
                <p className="px-2 text-sm text-destructive">{olderError}</p>
              ) : null}
              <Button
                type="button"
                variant="ghost"
                className="w-full"
                onClick={() => void loadOlder()}
                disabled={olderLoading}
              >
                Load older
              </Button>
            </div>
          ) : null}
        </div>
      )}
    </div>
  )
}

const histogramConfig = {
  value: { label: "logs", color: "var(--chart-2)" },
} satisfies ChartConfig

function HistogramCard({
  live,
  range,
  series,
  stepSeconds,
  dragStart,
  dragEnd,
  onDragStart,
  onDragEnd,
  onWindow,
  onReset,
  customWindow,
  total,
}: {
  live: boolean
  range: ResolvedRange
  series: SeriesPoint[]
  stepSeconds: number
  dragStart: number | null
  dragEnd: number | null
  onDragStart: (index: number | null) => void
  onDragEnd: (index: number | null) => void
  onWindow: (fromNanos: string, toNanos: string) => void
  onReset: () => void
  customWindow: boolean
  total: number
}) {
  const chartData = useMemo(
    () =>
      series.map((point, index) => ({
        ...point,
        index,
        time: formatTimeInRange(point.tsNanos, range),
      })),
    [series, range]
  )
  const referenceStart =
    dragStart != null && dragEnd != null
      ? chartData[Math.min(dragStart, dragEnd)]?.time
      : undefined
  const referenceEnd =
    dragStart != null && dragEnd != null
      ? chartData[Math.max(dragStart, dragEnd)]?.time
      : undefined

  const indexFromState = (state: unknown) =>
    typeof (state as { activeTooltipIndex?: unknown }).activeTooltipIndex ===
    "number"
      ? (state as { activeTooltipIndex: number }).activeTooltipIndex
      : null

  return (
    <div className="rounded-xl border border-border/70 bg-card p-4">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-sm font-medium">Log volume</h2>
          <p className="text-xs text-muted-foreground">
            {live
              ? "Histogram paused while tailing live logs."
              : `${formatCount(total)} records in window`}
          </p>
        </div>
        {customWindow ? (
          <Button type="button" variant="ghost" size="xs" onClick={onReset}>
            <IconX />
            Reset window
          </Button>
        ) : null}
      </div>
      <ChartContainer config={histogramConfig} className="h-[180px] w-full">
        <BarChart
          data={chartData}
          margin={{ left: 8, right: 8, top: 4 }}
          onClick={(state) => {
            if (live) return
            const index = indexFromState(state)
            if (index == null) return
            const window = bucketWindow(series, index, stepSeconds)
            if (window) onWindow(window.fromNanos, window.toNanos)
          }}
          onMouseDown={(state) => {
            if (live) return
            onDragStart(indexFromState(state))
            onDragEnd(indexFromState(state))
          }}
          onMouseMove={(state) => {
            if (live || dragStart == null) return
            onDragEnd(indexFromState(state))
          }}
          onMouseUp={() => {
            if (live || dragStart == null || dragEnd == null) {
              onDragStart(null)
              onDragEnd(null)
              return
            }
            const window = dragWindow(series, dragStart, dragEnd, stepSeconds)
            onDragStart(null)
            onDragEnd(null)
            if (window) onWindow(window.fromNanos, window.toNanos)
          }}
        >
          <CartesianGrid vertical={false} />
          <XAxis
            dataKey="time"
            tickLine={false}
            axisLine={false}
            minTickGap={48}
          />
          <YAxis tickLine={false} axisLine={false} width={48} />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Bar
            dataKey="value"
            fill="var(--color-value)"
            radius={2}
            opacity={live ? 0.35 : 1}
          />
          {referenceStart && referenceEnd ? (
            <ReferenceArea
              x1={referenceStart}
              x2={referenceEnd}
              fill="var(--muted-foreground)"
              fillOpacity={0.12}
            />
          ) : null}
        </BarChart>
      </ChartContainer>
    </div>
  )
}

function ColumnMenu({
  columns,
  onChange,
}: {
  columns: OptionalLogColumn[]
  onChange: (columns: OptionalLogColumn[]) => void
}) {
  const toggle = (column: OptionalLogColumn) => {
    const next = columns.includes(column)
      ? columns.filter((current) => current !== column)
      : [...columns, column]
    onChange(next)
  }
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button type="button" variant="outline" size="sm" />}
      >
        <IconColumns />
        Columns
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-44">
        <DropdownMenuLabel>Optional columns</DropdownMenuLabel>
        <DropdownMenuGroup>
          {(["service", "trace", "scope"] as OptionalLogColumn[]).map(
            (column) => (
              <DropdownMenuCheckboxItem
                key={column}
                checked={columns.includes(column)}
                onClick={(event) => {
                  event.preventDefault()
                  toggle(column)
                }}
              >
                {column}
              </DropdownMenuCheckboxItem>
            )
          )}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
