import {
  createFileRoute,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import {
  IconArticleFilled,
  IconBookmark,
  IconColumns,
  IconDeviceFloppy,
  IconHistory,
  IconPlayerPlayFilled,
  IconPlayerStopFilled,
  IconRefresh,
  IconTrash,
  IconX,
} from "@tabler/icons-react"
import { useEffect, useMemo, useRef, useState } from "react"
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
import { useChartBrush } from "@/components/console/use-chart-brush"
import {
  LogsTable,
  OPTIONAL_LOG_COLUMNS,
  parseLogColumns,
  serializeLogColumns,
} from "@/components/logs-table"
import type { LogDoc, OptionalLogColumn } from "@/components/logs-table"
import { useLiveStream } from "@/hooks/use-live-stream"
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
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
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
import { gqlString, graphql, graphqlCached, LOG_FIELDS } from "@/lib/api"
import { formatCount, formatDateTime, formatTimeInRange } from "@/lib/format"
import { resolveRangeSearch, updateRangeSearch } from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface LogsData {
  services: string[]
  logs: LogDoc[]
  logCountSeries: SeriesPoint[]
  savedViews: SavedView[]
}

export interface SavedView {
  id: string
  name: string
  page: string
  state: string
  updatedAtNanos: string
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
  anchor?: string | undefined
}

const PAGE_SIZE = 500
const SEVERITIES = [
  { label: "All severities", value: undefined },
  { label: "Debug+", value: 5 },
  { label: "Info+", value: 9 },
  { label: "Warn+", value: 13 },
  { label: "Error+", value: 17 },
] as const

let logKeySequence = 0

function assignLogKeys(logs: LogDoc[]): LogDoc[] {
  return logs.map((log) =>
    log._key ? log : { ...log, _key: `log-${logKeySequence++}` }
  )
}

const logsSearchSchema = z.object({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  sev: z.unknown().optional(),
  range: z.unknown().optional(),
  from: z.unknown().optional(),
  to: z.unknown().optional(),
  live: z.unknown().optional(),
  cols: z.unknown().optional(),
  anchor: z.unknown().optional(),
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
    anchor:
      typeof parsed.anchor === "string" && /^\d+$/.test(parsed.anchor)
        ? parsed.anchor
        : undefined,
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

export function contextWindow(
  anchorNanos: string,
  windowSeconds = 30
): ResolvedRange {
  const anchor = BigInt(anchorNanos)
  const width = BigInt(windowSeconds) * 1_000_000_000n
  const from = anchor > width ? anchor - width : 0n
  return {
    key: "custom",
    fromNanos: from.toString(),
    toNanos: (anchor + width).toString(),
  }
}

export function parseSavedViewState(state: string): LogsSearch {
  const params = new URLSearchParams(
    state.startsWith("?") ? state.slice(1) : state
  )
  const raw: Record<string, unknown> = {}
  params.forEach((value, key) => {
    raw[key] = value
  })
  return validateLogsSearch(raw)
}

function serializeLogsSearch(search: LogsSearch) {
  const params = new URLSearchParams()
  if (search.q) params.set("q", search.q)
  if (search.service) params.set("service", search.service)
  if (search.sev) params.set("sev", String(search.sev))
  if (search.range) params.set("range", search.range)
  if (search.from) params.set("from", search.from)
  if (search.to) params.set("to", search.to)
  if (search.live) params.set("live", "1")
  if (search.cols) params.set("cols", search.cols)
  if (search.anchor) params.set("anchor", search.anchor)
  const value = params.toString()
  return value ? `?${value}` : ""
}

export async function loadLogs(search: LogsSearch): Promise<LogsData> {
  const range = search.anchor
    ? contextWindow(search.anchor)
    : resolveRangeSearch(search)
  const stepSeconds = stepSecondsForRange(range)
  const filters = [
    search.service ? `service: "${gqlString(search.service)}"` : "",
    search.sev ? `severityMin: ${search.sev}` : "",
    search.q ? `query: "${gqlString(search.q)}"` : "",
  ].filter(Boolean)
  if (search.live) {
    return graphqlCached<LogsData>(
      `{ services savedViews(page: "/logs") { id name page state updatedAtNanos } logs(limit: 0) { ${LOG_FIELDS} } logCountSeries(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) { tsNanos value } }`
    )
  }
  const logsQuery = search.anchor
    ? `logs: logsAround(anchorNanos: "${search.anchor}", windowSeconds: 30, ${search.service ? `service: "${gqlString(search.service)}", ` : ""}limit: ${PAGE_SIZE}) { ${LOG_FIELDS} }`
    : `logs(${[
        `fromNanos: "${range.fromNanos}"`,
        `toNanos: "${range.toNanos}"`,
        ...filters,
        `limit: ${PAGE_SIZE}`,
      ].join(", ")}) { ${LOG_FIELDS} }`
  const seriesArgs = [
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    ...filters,
    `stepSeconds: ${stepSeconds}`,
  ].join(", ")
  return graphqlCached<LogsData>(`{
    services
    savedViews(page: "/logs") { id name page state updatedAtNanos }
    ${logsQuery}
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
  const range = search.anchor
    ? contextWindow(search.anchor)
    : resolveRangeSearch(search)
  const stepSeconds = stepSecondsForRange(range)
  const keyedDataLogs = useMemo(() => assignLogKeys(data.logs), [data.logs])
  const [logs, setLogs] = useState<LogDoc[]>(keyedDataLogs)
  const [pendingQuery, setPendingQuery] = useState(search.q ?? "")
  const [olderLoading, setOlderLoading] = useState(false)
  const [olderError, setOlderError] = useState<string | null>(null)
  const [exhausted, setExhausted] = useState(data.logs.length < PAGE_SIZE)
  const [savedViews, setSavedViews] = useState(data.savedViews)
  const [viewError, setViewError] = useState<string | null>(null)
  const [saveOpen, setSaveOpen] = useState(false)
  const [saveName, setSaveName] = useState("")
  const [savingView, setSavingView] = useState(false)
  const logsGeneration = useRef(0)
  const live = search.live === true
  const columns = parseLogColumns(search.cols)

  useEffect(() => {
    logsGeneration.current += 1
    setLogs(keyedDataLogs)
    setExhausted(keyedDataLogs.length < PAGE_SIZE)
  }, [keyedDataLogs])

  useEffect(() => setSavedViews(data.savedViews), [data.savedViews])

  useEffect(() => setPendingQuery(search.q ?? ""), [search.q])

  const streamUrl = useMemo(() => {
    if (!live) return null
    const params = new URLSearchParams()
    if (search.service) params.set("service", search.service)
    if (search.sev) params.set("severity_min", String(search.sev))
    if (search.q) params.set("q", search.q)
    return `/v1/logs/stream?${params}`
  }, [live, search.service, search.sev, search.q])

  const streamStatus = useLiveStream<LogDoc>({
    url: streamUrl,
    parse: (frame) => {
      const batch: unknown = JSON.parse(frame)
      return Array.isArray(batch) ? assignLogKeys(batch as LogDoc[]) : []
    },
    onBatch: (incoming) => {
      setLogs((current) =>
        [...incoming.reverse(), ...current].slice(0, PAGE_SIZE)
      )
    },
  })

  const update = (patch: Partial<LogsSearch>) =>
    void navigate({
      search: (current) => ({ ...current, ...patch }),
    })

  const showContext = (log: LogDoc) => {
    const window = contextWindow(log.tsNanos)
    update({
      anchor: log.tsNanos,
      range: "custom",
      from: window.fromNanos,
      to: window.toNanos,
      live: undefined,
      q: undefined,
      sev: undefined,
    })
  }

  const selectSavedView = (view: SavedView) => {
    setViewError(null)
    try {
      const next = parseSavedViewState(view.state)
      void navigate({ search: () => next })
    } catch (err) {
      setViewError(err instanceof Error ? err.message : String(err))
    }
  }

  const deleteSavedView = async (id: string) => {
    setViewError(null)
    try {
      await graphql<{
        savedViewDelete: boolean
      }>(`mutation { savedViewDelete(id: "${gqlString(id)}") }`)
      setSavedViews((current) => current.filter((view) => view.id !== id))
    } catch (err) {
      setViewError(err instanceof Error ? err.message : String(err))
    }
  }

  const saveCurrentView = async () => {
    const name = saveName.trim()
    if (!name) return
    setSavingView(true)
    setViewError(null)
    try {
      const state = serializeLogsSearch(search)
      const result = await graphql<{
        savedViewSave: SavedView
      }>(`mutation { savedViewSave(name: "${gqlString(name)}", page: "/logs", state: "${gqlString(state)}") { id name page state updatedAtNanos } }`)
      setSavedViews((current) => [
        result.savedViewSave,
        ...current.filter((view) => view.id !== result.savedViewSave.id),
      ])
      setSaveOpen(false)
      setSaveName("")
    } catch (err) {
      setViewError(err instanceof Error ? err.message : String(err))
    } finally {
      setSavingView(false)
    }
  }

  const setRange = (next: ResolvedRange) => {
    update(updateRangeSearch(next))
  }

  const loadOlder = async () => {
    const oldest = logs.at(-1)
    if (!oldest) return
    const generation = logsGeneration.current
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
        ${LOG_FIELDS}
      } }`)
      // Window/filters changed while in flight — drop the stale page.
      if (logsGeneration.current !== generation) return
      setLogs((current) => [...current, ...assignLogKeys(more.logs)])
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
        <SavedViewsMenu
          views={savedViews}
          onSelect={selectSavedView}
          onDelete={(id) => void deleteSavedView(id)}
          onSave={() => {
            setSaveName("")
            setSaveOpen(true)
          }}
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

      {viewError ? (
        <p className="text-sm text-destructive">{viewError}</p>
      ) : null}

      <Dialog open={saveOpen} onOpenChange={setSaveOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Save view</DialogTitle>
          </DialogHeader>
          <Input
            value={saveName}
            onChange={(event) => setSaveName(event.target.value)}
            placeholder="View name"
            autoFocus
          />
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setSaveOpen(false)}
            >
              Cancel
            </Button>
            <Button
              type="button"
              onClick={() => void saveCurrentView()}
              disabled={savingView || !saveName.trim()}
            >
              <IconDeviceFloppy />
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {search.anchor ? (
        <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-border/70 bg-accent/25 px-3 py-2">
          <div className="flex items-center gap-2 text-sm">
            <IconHistory className="size-4 text-muted-foreground" />
            <span>Context around {formatDateTime(search.anchor)}</span>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => update({ anchor: undefined })}
          >
            <IconX />
            Reset
          </Button>
        </div>
      ) : null}

      <HistogramCard
        live={live}
        range={range}
        series={data.logCountSeries}
        stepSeconds={stepSeconds}
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
              : "No log records in this window — send OTLP logs to 127.0.0.1:4317/4318 or wrap a command with parallax invocation start."
          }
          icon={IconArticleFilled}
          className="rounded-xl border border-dashed"
        />
      ) : (
        <div className="content-enter overflow-hidden rounded-xl border border-border/70">
          {live ? (
            <div className="flex items-center gap-2 border-b border-border/70 px-3 py-2 text-xs">
              {streamStatus === "open" ? (
                <>
                  <span className="size-2 animate-pulse rounded-full bg-emerald-500" />
                  <Badge variant="emerald">Live</Badge>
                </>
              ) : streamStatus === "error" ? (
                <Badge variant="amber">reconnecting…</Badge>
              ) : (
                <Badge variant="secondary">connecting…</Badge>
              )}
              <span className="text-muted-foreground">
                {formatCount(logs.length)} records buffered
              </span>
            </div>
          ) : null}
          <LogsTable
            logs={logs}
            range={range}
            columns={columns}
            anchorNanos={search.anchor}
            onShowContext={showContext}
          />
          {!live && !search.anchor && !exhausted ? (
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
  onWindow,
  onReset,
  customWindow,
  total,
}: {
  live: boolean
  range: ResolvedRange
  series: SeriesPoint[]
  stepSeconds: number
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
  const brush = useChartBrush({
    series: chartData,
    stepSeconds,
    disabled: live,
    onWindow,
    getReferenceValue: (point) => point.time,
  })

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
          {...brush.chartHandlers}
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
          {brush.referenceRange ? (
            <ReferenceArea
              x1={brush.referenceRange.x1}
              x2={brush.referenceRange.x2}
              fill="var(--muted-foreground)"
              fillOpacity={0.12}
            />
          ) : null}
        </BarChart>
      </ChartContainer>
    </div>
  )
}

export function ColumnMenu({
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
          {OPTIONAL_LOG_COLUMNS.map((column) => (
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
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export function SavedViewsMenu({
  views,
  onSelect,
  onDelete,
  onSave,
}: {
  views: SavedView[]
  onSelect: (view: SavedView) => void
  onDelete: (id: string) => void
  onSave: () => void
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button type="button" variant="outline" size="sm" />}
      >
        <IconBookmark />
        Views
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel>Saved views</DropdownMenuLabel>
        <DropdownMenuGroup>
          {views.length === 0 ? (
            <DropdownMenuItem disabled>No saved views</DropdownMenuItem>
          ) : (
            views.map((view) => (
              <DropdownMenuItem key={view.id} onClick={() => onSelect(view)}>
                <IconBookmark />
                <span className="truncate">{view.name}</span>
              </DropdownMenuItem>
            ))
          )}
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={onSave}>
          <IconDeviceFloppy />
          Save current view
        </DropdownMenuItem>
        {views.length > 0 ? (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuLabel>Delete view</DropdownMenuLabel>
            {views.map((view) => (
              <DropdownMenuItem
                key={`delete-${view.id}`}
                variant="destructive"
                onClick={(event) => {
                  event.preventDefault()
                  onDelete(view.id)
                }}
              >
                <IconTrash />
                <span className="truncate">{view.name}</span>
              </DropdownMenuItem>
            ))}
          </>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
