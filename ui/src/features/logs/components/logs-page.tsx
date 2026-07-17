import { useNavigate, useRouterState } from "@tanstack/react-router"
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
import { Bar, BarChart, CartesianGrid, ReferenceArea, XAxis, YAxis } from "recharts"
import { EmptyState } from "@/shared/console/empty-state"
import { FacetSidebar, type Facet } from "@/shared/console/facet-sidebar"
import { useDelayedLoading } from "@/shared/console/hooks"
import { TableSkeleton } from "@/shared/console/skeletons"
import { useChartBrush } from "@/shared/console/use-chart-brush"
import { WhereClauseChips, WhereClauseEditor } from "@/shared/console/where-clause-editor"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
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
import {
  LogsTable,
  OPTIONAL_LOG_COLUMNS,
  parseLogColumns,
  serializeLogColumns,
} from "@/features/logs/components/logs-table"
import type { LogDoc, OptionalLogColumn } from "@/features/logs/components/logs-table"
import { contextWindow, stepSecondsForRange } from "@/features/logs/model/logs-range"
import { logStreamBatchDecoder } from "@/features/logs/api/log-stream-schema"
import { mergeLiveLogs } from "@/features/logs/model/merge-live-logs"
import {
  parseSavedViewState,
  serializeLogsSearch,
  type LogsSearch,
} from "@/features/logs/model/logs-search"
import { RangePicker } from "@/features/time-range"
import { gqlString, graphql, graphqlCached, LOG_FIELDS } from "../api/gql"
import { formatCount, formatDateTime, formatTimeInRange } from "@/shared/format"
import { resolveRangeSearch, updateRangeSearch, type ResolvedRange } from "@/domain/range"
import {
  serializeWhereClause,
  whereClauseFromSearch,
  type WhereFilter,
} from "@/shared/where-clause"
import { useLiveStream } from "@/platform/sse/use-live-stream"
import { PageHeader } from "@/shared/components/page-header"

interface SeriesPoint {
  tsNanos: string
  value: number
}

interface LogFacet {
  dimension: string
  values: Array<{ value: string; count: string }>
}

export interface LogsData {
  services: string[]
  logs: LogDoc[]
  logCountSeries: SeriesPoint[]
  logFacets: LogFacet[]
  logPatterns: LogPatternRow[]
  savedViews: SavedView[]
}

export interface SavedView {
  id: string
  name: string
  page: string
  state: string
  updatedAtNanos: string
}

interface LogPatternRow {
  template: string
  count: string
  severityMixJson: string
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
  return logs.map((log) => (log._key ? log : { ...log, _key: `log-${logKeySequence++}` }))
}

function logAttributeFilters(where: string | undefined): string | null {
  const filters = whereClauseFromSearch(where)
  if (filters.length === 0) return null
  const items = filters
    .map(
      (filter) =>
        `{key: "${gqlString(filter.key)}", op: "${gqlString(filter.op)}", value: "${gqlString(filter.value)}"}`
    )
    .join(", ")
  return `attributeFilters: [${items}]`
}

export async function loadLogs(search: LogsSearch): Promise<LogsData> {
  const range = search.anchor ? contextWindow(search.anchor) : resolveRangeSearch(search)
  const stepSeconds = stepSecondsForRange(range)
  const filters = [
    search.service ? `service: "${gqlString(search.service)}"` : "",
    search.sev ? `severityMin: ${search.sev}` : "",
    search.q ? `query: "${gqlString(search.q)}"` : "",
    logAttributeFilters(search.where) ?? "",
  ].filter(Boolean)
  if (search.live) {
    return graphqlCached<LogsData>(
      `{ services savedViews(page: "/logs") { id name page state updatedAtNanos } logs(limit: 0) { ${LOG_FIELDS} } logCountSeries(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", stepSeconds: ${stepSeconds}) { tsNanos value } logFacets(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") { dimension values { value count } } }`
    ).then((data) => ({
      ...data,
      logFacets: data.logFacets ?? [],
      logPatterns: data.logPatterns ?? [],
    }))
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
  const facetArgs = [
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    ...filters,
  ].join(", ")
  const patternsField = search.patterns
    ? `logPatterns(${facetArgs}, limit: 10000) { template count severityMixJson }`
    : ""
  return graphqlCached<LogsData>(`{
    services
    savedViews(page: "/logs") { id name page state updatedAtNanos }
    ${logsQuery}
    logCountSeries(${seriesArgs}) { tsNanos value }
    logFacets(${facetArgs}) { dimension values { value count } }
    ${patternsField}
  }`).then((data) => ({
    ...data,
    logFacets: data.logFacets ?? [],
    logPatterns: data.logPatterns ?? [],
  }))
}

export function LogsPage({ data, search }: { data: LogsData; search: LogsSearch }) {
  const navigate = useNavigate({ from: "/logs" })
  const routerLoading = useRouterState({
    select: (state) => state.status === "pending",
  })
  const delayedLoading = useDelayedLoading(routerLoading)
  const range = search.anchor ? contextWindow(search.anchor) : resolveRangeSearch(search)
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
  const [whereFocusKey, setWhereFocusKey] = useState(0)

  useEffect(() => {
    logsGeneration.current += 1
    setLogs(keyedDataLogs)
    setExhausted(keyedDataLogs.length < PAGE_SIZE)
  }, [keyedDataLogs])

  useEffect(() => setSavedViews(data.savedViews), [data.savedViews])

  useEffect(() => setPendingQuery(search.q ?? ""), [search.q])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "f" && event.key !== "F") return
      if (event.metaKey || event.ctrlKey || event.altKey) return
      const target = event.target
      if (
        target instanceof HTMLElement &&
        target.closest("input, textarea, select, [contenteditable]")
      ) {
        return
      }
      event.preventDefault()
      setWhereFocusKey((current) => current + 1)
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

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
    decoder: {
      safeParse(input) {
        const decoded = logStreamBatchDecoder.safeParse(input)
        if (!decoded.success) return decoded
        return { success: true, data: assignLogKeys(decoded.data) }
      },
    },
    onBatch: (incoming) => {
      setLogs((current) => mergeLiveLogs(current, incoming, PAGE_SIZE).items as LogDoc[])
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

  const whereFilters = useMemo(() => whereClauseFromSearch(search.where), [search.where])
  const applyWhereFilters = (filters: WhereFilter[]) =>
    update({ where: serializeWhereClause(filters) || undefined })
  const facetSelections = useMemo(() => {
    const selections: Record<string, string[]> = {}
    for (const filter of whereFilters) {
      if (filter.op !== "=") continue
      selections[filter.key] = [...(selections[filter.key] ?? []), filter.value]
    }
    return selections
  }, [whereFilters])
  const toggleFacet = (dimension: string, value: string) => {
    const existing = whereFilters.findIndex(
      (filter) => filter.key === dimension && filter.op === "=" && filter.value === value
    )
    const next =
      existing >= 0
        ? whereFilters.filter((_, index) => index !== existing)
        : [...whereFilters, { key: dimension, op: "=" as const, value }]
    applyWhereFilters(next)
  }
  const facets: Facet[] = (data.logFacets ?? []).map((facet) => ({
    dimension: facet.dimension,
    label: facet.dimension,
    values: facet.values.map((entry) => ({
      value: entry.value,
      count: Number(entry.count),
    })),
    serviceDots: facet.dimension === "service",
    searchable: true,
  }))
  const facetValueSuggestions = (key: string) =>
    (data.logFacets ?? [])
      .find((facet) => facet.dimension === key)
      ?.values.map((entry) => entry.value) ?? []

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
        logAttributeFilters(search.where) ?? "",
        `limit: ${PAGE_SIZE}`,
      ]
        .filter(Boolean)
        .join(", ")
      const more = await graphql<{ logs: LogDoc[] }>(`{ logs(${args}) {
        ${LOG_FIELDS}
      } }`)
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
              <SelectItem key={severity.value ?? 0} value={String(severity.value ?? 0)}>
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
        <WhereClauseEditor
          key={whereFocusKey}
          autoFocus={whereFocusKey > 0}
          className="min-w-72 flex-1"
          filters={whereFilters}
          onApply={applyWhereFilters}
          keySuggestions={
            facets.length > 0
              ? facets.map((facet) => facet.dimension)
              : ["service", "severity", "body"]
          }
          valueSuggestionsFor={facetValueSuggestions}
        />
        <ColumnMenu
          columns={columns}
          onChange={(next) => update({ cols: serializeLogColumns(next) })}
        />
        <Button
          type="button"
          variant={search.patterns ? "secondary" : "outline"}
          size="sm"
          onClick={() => update({ patterns: search.patterns ? undefined : true })}
        >
          Patterns
        </Button>
        <SavedViewsMenu
          views={savedViews}
          onSelect={selectSavedView}
          onDelete={(id) => void deleteSavedView(id)}
          onSave={() => {
            setSaveName("")
            setSaveOpen(true)
          }}
        />
        <Button type="button" variant="outline" size="sm" onClick={() => update({})}>
          <IconRefresh />
          Refresh
        </Button>
      </div>

      <WhereClauseChips
        filters={whereFilters}
        onRemove={(index) => applyWhereFilters(whereFilters.filter((_, i) => i !== index))}
      />

      {viewError ? <p className="text-sm text-destructive">{viewError}</p> : null}

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
            <Button type="button" variant="outline" onClick={() => setSaveOpen(false)}>
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
        onWindow={(fromNanos, toNanos) => update({ range: "custom", from: fromNanos, to: toNanos })}
        onReset={() => update({ from: undefined, to: undefined, range: undefined })}
        customWindow={Boolean(customWindow)}
        total={total}
      />

      <div className="flex items-start gap-4">
        {!live && facets.length > 0 ? (
          <FacetSidebar
            facets={facets}
            selections={facetSelections}
            onToggle={toggleFacet}
            onClear={() => update({ where: undefined })}
          />
        ) : null}
        <div className="min-w-0 flex-1">
          {delayedLoading ? (
            <TableSkeleton rows={8} />
          ) : search.patterns ? (
            (data.logPatterns ?? []).length === 0 ? (
              <EmptyState
                title="No patterns in this window"
                description="Widen the range or clear filters, then re-run Patterns."
                icon={IconArticleFilled}
                className="rounded-xl border border-dashed"
              />
            ) : (
              <div className="content-enter overflow-hidden rounded-xl border border-border/70">
                <table className="w-full text-sm">
                  <thead className="border-b border-border/70 bg-muted/30 text-left text-xs text-muted-foreground">
                    <tr>
                      <th className="px-3 py-2 font-medium">Template</th>
                      <th className="px-3 py-2 font-medium tabular-nums">Count</th>
                      <th className="px-3 py-2 font-medium">Severity mix</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(data.logPatterns ?? []).map((pattern) => (
                      <tr
                        key={pattern.template}
                        className="border-b border-border/40 last:border-0"
                      >
                        <td className="px-3 py-2 font-mono text-xs break-all">
                          {pattern.template}
                        </td>
                        <td className="px-3 py-2 tabular-nums">
                          {formatCount(Number(pattern.count))}
                        </td>
                        <td className="px-3 py-2 text-xs text-muted-foreground">
                          {pattern.severityMixJson}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )
          ) : logs.length === 0 ? (
            <EmptyState
              title={
                search.q || search.service || search.sev || search.where
                  ? "No matching logs"
                  : "No logs yet"
              }
              description={
                search.q || search.service || search.sev || search.where
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
                  ) : streamStatus === "reconnecting" || streamStatus === "error" ? (
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
      </div>
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
        <BarChart data={chartData} margin={{ left: 8, right: 8, top: 4 }} {...brush.chartHandlers}>
          <CartesianGrid vertical={false} />
          <XAxis dataKey="time" tickLine={false} axisLine={false} minTickGap={48} />
          <YAxis tickLine={false} axisLine={false} width={48} />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Bar dataKey="value" fill="var(--color-value)" radius={2} opacity={live ? 0.35 : 1} />
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
      <DropdownMenuTrigger render={<Button type="button" variant="outline" size="sm" />}>
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
      <DropdownMenuTrigger render={<Button type="button" variant="outline" size="sm" />}>
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

export { contextWindow, stepSecondsForRange } from "@/features/logs/model/logs-range"
export { parseSavedViewState, validateLogsSearch } from "@/features/logs/model/logs-search"
export type { LogsSearch } from "@/features/logs/model/logs-search"
export {
  LogsTable,
  OPTIONAL_LOG_COLUMNS,
  parseLogColumns,
  serializeLogColumns,
} from "@/features/logs/components/logs-table"
export type { LogDoc, OptionalLogColumn } from "@/features/logs/components/logs-table"
