import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router"
import {
  IconAffiliateFilled,
  IconAlertTriangle,
  IconDeviceFloppy,
  IconPlayerPlayFilled,
  IconPlayerStopFilled,
  IconRefresh,
} from "@tabler/icons-react"
import { useEffect, useMemo, useState } from "react"
import { z } from "zod"
import { SavedViewsMenu, type SavedView } from "@/features/logs"
import {
  ClearFiltersButton,
  FilterSelect,
  SearchInput,
  ToggleChip,
  pageWindow,
  parseSortParam,
} from "@/shared/console/data-table"
import { formatCount } from "@/shared/format"
import { QueryBar, QueryBarRow } from "@/shared/console/query-bar"
import { SectionError } from "@/shared/console/error-state"
import { useFilterFocusShortcut } from "@/shared/keyboard"
import { AttributeComparePanel } from "@/features/traces/components/trace-attribute-compare"
import {
  PAGE_SIZE,
  toNumber,
  type TraceSort,
  type TracesLoaderData,
  type TracesSearch,
} from "@/features/traces/components/traces-query"
import { TraceTable } from "@/features/traces/components/trace-table"
import { PageHeader } from "@/shared/components/page-header"
import { useLiveStream } from "@/platform/sse/use-live-stream"
import { spanStreamBatchDecoder } from "@/features/traces/api/span-stream-schema"
import { FieldExplorer } from "@/features/traces/components/trace-field-explorer"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { EmptyState } from "@/shared/console/empty-state"
import { DurationFilter } from "@/shared/console/duration-filter"
import { FacetSidebar, type Facet } from "@/shared/console/facet-sidebar"
import { WhereClauseChips, WhereClauseEditor } from "@/shared/console/where-clause-editor"
import {
  serializeWhereClause,
  whereClauseFromSearch,
  type WhereFilter,
} from "@/shared/where-clause"
import { useDelayedLoading } from "@/shared/console/hooks"
import { RangePicker } from "@/features/time-range"
import { TableSkeleton } from "@/shared/console/skeletons"

import { gqlString, graphql } from "@/platform/graphql/transport"
import { mergeLiveSpans } from "@/features/traces/model/merge-live-spans"
import type { LiveSpan } from "@/features/traces/model/wire"
import { rangeLinkSearch, resolveRangeSearch, updateRangeSearch } from "@/domain/time-range/range"
import type { ResolvedRange } from "@/domain/time-range/range"

type SpanDoc = LiveSpan

const SORTS: TraceSort[] = ["START_DESC", "DURATION_DESC", "DURATION_ASC", "SPAN_COUNT_DESC"]

const traceSearchSchema = z.object({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  errors: z.unknown().optional(),
  minMs: z.unknown().optional(),
  maxMs: z.unknown().optional(),
  where: z.unknown().optional(),
  sort: z.unknown().optional(),
  page: z.unknown().optional(),
  range: z.unknown().optional(),
  from: z.unknown().optional(),
  to: z.unknown().optional(),
  live: z.unknown().optional(),
})

export function validateTracesSearch(search: Record<string, unknown>): TracesSearch {
  const parsed = traceSearchSchema.parse(search)
  const positiveNumber = (value: unknown) => {
    const number = Number(value)
    return Number.isFinite(number) && number > 0 ? number : undefined
  }
  const positiveInteger = (value: unknown) => {
    const number = Number(value)
    return Number.isInteger(number) && number > 0 ? number : undefined
  }
  const stringValue = (value: unknown) => (typeof value === "string" && value ? value : undefined)
  return {
    q: stringValue(parsed.q),
    service: stringValue(parsed.service),
    errors: parsed.errors === "1" || parsed.errors === true ? true : undefined,
    minMs: positiveNumber(parsed.minMs),
    maxMs: positiveNumber(parsed.maxMs),
    where: stringValue(parsed.where),
    sort: SORTS.includes(parsed.sort as TraceSort) ? (parsed.sort as TraceSort) : undefined,
    page: positiveInteger(parsed.page),
    range: stringValue(parsed.range),
    from: stringValue(parsed.from),
    to: stringValue(parsed.to),
    live: parsed.live === "1" || parsed.live === true ? true : undefined,
  }
}

type TraceSearchPatch = Partial<TracesSearch>

const FILTER_KEYS = new Set<keyof TracesSearch>([
  "q",
  "service",
  "errors",
  "minMs",
  "maxMs",
  "where",
  "range",
  "from",
  "to",
  "live",
])

export function patchTracesSearch(current: TracesSearch, patch: TraceSearchPatch): TracesSearch {
  const next: TracesSearch = { ...current, ...patch }
  for (const key of Object.keys(next) as Array<keyof TracesSearch>) {
    if (next[key] === "" || next[key] == null || next[key] === false) {
      delete next[key]
    }
  }
  if (Object.keys(patch).some((key) => FILTER_KEYS.has(key as keyof TracesSearch))) {
    delete next.page
  }
  return next
}

export function traceSortToParam(sort?: TraceSort): string | undefined {
  switch (sort) {
    case "DURATION_DESC":
      return "duration:desc"
    case "DURATION_ASC":
      return "duration:asc"
    case "SPAN_COUNT_DESC":
      return "spans:desc"
    case "START_DESC":
      return "when:desc"
    case undefined:
      return undefined
  }
}

export function traceDetailSearch(range: ResolvedRange) {
  return rangeLinkSearch(range)
}

export function paramToTraceSort(param: string | undefined): TraceSort | undefined {
  const parsed = parseSortParam(param)
  if (!parsed) return undefined
  if (parsed.key === "duration") {
    return parsed.direction === "asc" ? "DURATION_ASC" : "DURATION_DESC"
  }
  if (parsed.key === "spans" && parsed.direction === "desc") return "SPAN_COUNT_DESC"
  if (parsed.key === "when" && parsed.direction === "desc") return "START_DESC"
  return undefined
}

function liveDurationMs(search: TracesSearch): number {
  return search.minMs && search.minMs > 0 ? search.minMs : NaN
}

function statusError(statusCode: string): boolean {
  return statusCode === "STATUS_CODE_ERROR"
}

export function serializeTracesSearch(search: TracesSearch): string {
  const params = new URLSearchParams()
  if (search.q) params.set("q", search.q)
  if (search.service) params.set("service", search.service)
  if (search.errors) params.set("errors", "1")
  if (search.minMs !== undefined) params.set("minMs", String(search.minMs))
  if (search.maxMs !== undefined) params.set("maxMs", String(search.maxMs))
  if (search.where) params.set("where", search.where)
  if (search.sort) params.set("sort", search.sort)
  if (search.range) params.set("range", search.range)
  if (search.from) params.set("from", search.from)
  if (search.to) params.set("to", search.to)
  const value = params.toString()
  return value ? `?${value}` : ""
}

export function parseTracesViewState(state: string): TracesSearch {
  const params = new URLSearchParams(state.startsWith("?") ? state.slice(1) : state)
  const raw: Record<string, unknown> = {}
  params.forEach((value, key) => {
    raw[key] = value
  })
  return validateTracesSearch(raw)
}

export function TracesPage({ data, search }: { data: TracesLoaderData; search: TracesSearch }) {
  const { services, tracesPage, attributeCompare, traceFacets, traceDurationStats } = data
  const navigate = useNavigate({ from: "/traces/" })
  const router = useRouter()
  const pending = useRouterState({
    select: (state) => state.status === "pending",
  })
  const showSkeleton = useDelayedLoading(pending)
  const [lookup, setLookup] = useState("")
  const [whereFocusKey, setWhereFocusKey] = useState(0)
  const [savedViews, setSavedViews] = useState(data.savedViews)
  const [viewError, setViewError] = useState<string | null>(null)
  const [saveOpen, setSaveOpen] = useState(false)
  const [saveName, setSaveName] = useState("")
  const [savingView, setSavingView] = useState(false)

  useEffect(() => setSavedViews(data.savedViews), [data.savedViews])
  useFilterFocusShortcut(() => setWhereFocusKey((current) => current + 1))
  const [spans, setSpans] = useState<SpanDoc[]>([])
  const live = search.live === true
  const page = search.page ?? 1
  const range = useMemo(() => resolveRangeSearch(search), [search])
  const total = toNumber(tracesPage.total)
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))
  const pages = pageWindow(page, totalPages)
  const hasFilters = Boolean(
    search.q ||
    search.service ||
    search.errors ||
    search.minMs ||
    search.maxMs ||
    search.where ||
    search.live
  )
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
  const facets: Facet[] = traceFacets.map((facet) => ({
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
    traceFacets.find((facet) => facet.dimension === key)?.values.map((entry) => entry.value) ?? []
  const durationValues = tracesPage.items.map((trace) => Number(trace.durationNs))
  const liveDurationValues = spans.map((span) => Number(span.durationNs))
  const serviceOptions = services.map((service) => ({
    value: service,
    label: service,
  }))
  const update = (patch: TraceSearchPatch, replace = true) => {
    void navigate({ search: patchTracesSearch(search, patch), replace })
  }
  const selectSavedView = (view: SavedView) => {
    setViewError(null)
    try {
      const next = parseTracesViewState(view.state)
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
      const state = serializeTracesSearch(search)
      const result = await graphql<{
        savedViewSave: SavedView
      }>(`mutation { savedViewSave(name: "${gqlString(name)}", page: "/traces", state: "${gqlString(state)}") { id name page state updatedAtNanos } }`)
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
  const sortParam = traceSortToParam(search.sort)
  const setSortParam = (next: string | undefined) =>
    update({ sort: paramToTraceSort(next), page: undefined })

  const streamUrl = useMemo(() => {
    if (!live) return null
    const params = new URLSearchParams()
    if (search.service) params.set("service", search.service)
    const minMs = liveDurationMs(search)
    if (Number.isFinite(minMs) && minMs > 0) {
      params.set("min_duration_ms", String(minMs))
    }
    if (search.errors) params.set("errors_only", "true")
    if (search.q?.trim()) params.set("q", search.q.trim())
    return `/v1/traces/stream?${params}`
  }, [live, search])

  useEffect(() => {
    if (!streamUrl) return
    setSpans([])
  }, [streamUrl])

  const streamStatus = useLiveStream<SpanDoc>({
    url: streamUrl,
    decoder: spanStreamBatchDecoder,
    onBatch: (incoming) => {
      setSpans((current) => mergeLiveSpans(current, incoming, 100).items as SpanDoc[])
    },
  })

  const pageStart = total === 0 ? 0 : (page - 1) * PAGE_SIZE + 1
  const pageEnd = Math.min(total, (page - 1) * PAGE_SIZE + tracesPage.items.length)

  return (
    <div className="flex flex-col gap-4">
      <PageHeader
        icon={IconAffiliateFilled}
        title="Traces"
        description="Investigate distributed requests by latency, service, errors, and time."
        actions={
          <>
            <form
              className="flex items-center gap-2"
              onSubmit={(event) => {
                event.preventDefault()
                const id = lookup.trim()
                if (id) {
                  void navigate({
                    to: "/traces/$traceId",
                    params: { traceId: id },
                    search: traceDetailSearch(range),
                  })
                }
              }}
            >
              <Input
                value={lookup}
                onChange={(event) => setLookup(event.target.value)}
                placeholder="Open trace id"
                className="h-8 w-64 font-mono text-xs"
              />
              <Button type="submit" variant="outline" size="sm">
                Open
              </Button>
            </form>
            <div className="flex items-center gap-1">
              <Button
                type="button"
                variant={!live ? "secondary" : "outline"}
                size="sm"
                onClick={() => update({ live: undefined })}
              >
                <IconPlayerStopFilled />
                Query
              </Button>
              <Button
                type="button"
                variant={live ? "secondary" : "outline"}
                size="sm"
                onClick={() => update({ live: true })}
              >
                <IconPlayerPlayFilled />
                Live
              </Button>
              {!live ? (
                <Button
                  type="button"
                  variant="outline"
                  size="icon-sm"
                  aria-label="Refresh traces"
                  onClick={() => void router.invalidate()}
                >
                  <IconRefresh />
                </Button>
              ) : null}
            </div>
            <RangePicker value={range} onChange={(next) => update(updateRangeSearch(next))} />
          </>
        }
      />

      <QueryBar>
        <QueryBarRow>
          <SearchInput
            value={search.q ?? ""}
            onChange={(value) => update({ q: value || undefined })}
            placeholder="Search root span..."
          />
          {!live ? (
            <WhereClauseEditor
              key={whereFocusKey}
              autoFocus={whereFocusKey > 0}
              filters={whereFilters}
              onApply={applyWhereFilters}
              keySuggestions={facets.map((facet) => facet.dimension)}
              valueSuggestionsFor={facetValueSuggestions}
            />
          ) : null}
        </QueryBarRow>
        <QueryBarRow>
          <FilterSelect
            onChange={(value) => update({ service: value })}
            options={serviceOptions}
            placeholder="All services"
            {...(search.service ? { value: search.service } : {})}
          />
          <DurationFilter
            range={{
              ...(search.minMs === undefined ? {} : { minMs: search.minMs }),
              ...(search.maxMs === undefined ? {} : { maxMs: search.maxMs }),
            }}
            {...(traceDurationStats.p50Ms != null && traceDurationStats.p95Ms != null
              ? {
                  stats: {
                    p50Ms: traceDurationStats.p50Ms,
                    p95Ms: traceDurationStats.p95Ms,
                  },
                }
              : {})}
            onChange={(next) =>
              update({
                minMs: next.minMs ?? undefined,
                maxMs: next.maxMs ?? undefined,
              })
            }
          />
          <ToggleChip
            active={Boolean(search.errors)}
            onClick={() => update({ errors: search.errors ? undefined : true })}
          >
            <IconAlertTriangle />
            Errors only
          </ToggleChip>
          {!live ? (
            <FieldExplorer
              range={range}
              service={search.service}
              onApplyService={(service) => update({ service })}
            />
          ) : null}
          <SavedViewsMenu
            views={savedViews}
            onSelect={selectSavedView}
            onDelete={(id) => void deleteSavedView(id)}
            onSave={() => {
              setSaveName("")
              setSaveOpen(true)
            }}
          />
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                update({
                  q: undefined,
                  service: undefined,
                  errors: undefined,
                  minMs: undefined,
                  maxMs: undefined,
                  where: undefined,
                  live: undefined,
                })
              }
            />
          ) : null}
          <div className="ml-auto flex items-center gap-3">
            {live ? (
              streamStatus === "open" ? (
                <Badge variant="emerald">
                  <span className="size-1.5 rounded-full bg-current motion-safe:animate-pulse" />
                  Live
                </Badge>
              ) : streamStatus === "reconnecting" || streamStatus === "error" ? (
                <Badge variant="amber">reconnecting…</Badge>
              ) : (
                <Badge variant="secondary">connecting…</Badge>
              )
            ) : (
              <span className="text-sm text-muted-foreground tabular-nums">
                {formatCount(total)} {total === 1 ? "trace" : "traces"}
              </span>
            )}
          </div>
        </QueryBarRow>

        {viewError ? <SectionError message={viewError} /> : null}

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

        {!live ? (
          <WhereClauseChips
            filters={whereFilters}
            onRemove={(index) => applyWhereFilters(whereFilters.filter((_, i) => i !== index))}
          />
        ) : null}

        <div className="flex items-start gap-4">
          {!live && facets.length > 0 ? (
            <FacetSidebar
              facets={facets}
              selections={facetSelections}
              onToggle={toggleFacet}
              onClear={() => update({ where: undefined })}
            />
          ) : null}
          <div className="flex min-w-0 flex-1 flex-col gap-4">
            {showSkeleton ? (
              <TableSkeleton rows={PAGE_SIZE} />
            ) : live ? (
              <TraceTable
                rows={spans.map((span) => ({
                  traceId: span.traceId,
                  rootName: span.name,
                  service: span.service,
                  startNanos: span.tsNanos,
                  durationNs: span.durationNs,
                  spanCount: 1,
                  hasError: statusError(span.statusCode),
                }))}
                durationValues={liveDurationValues}
                range={range}
                sort={undefined}
                onSort={() => undefined}
                onOpen={(traceId) =>
                  void navigate({
                    to: "/traces/$traceId",
                    params: { traceId },
                    search: traceDetailSearch(range),
                  })
                }
              />
            ) : tracesPage.items.length > 0 ? (
              <>
                <Card>
                  <CardHeader>
                    <CardTitle className="text-sm">Attribute compare</CardTitle>
                    <CardDescription>Selected window vs previous window</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <AttributeComparePanel rows={attributeCompare} />
                  </CardContent>
                </Card>
                <TraceTable
                  rows={tracesPage.items}
                  durationValues={durationValues}
                  range={range}
                  sort={sortParam}
                  onSort={setSortParam}
                  onOpen={(traceId) =>
                    void navigate({
                      to: "/traces/$traceId",
                      params: { traceId },
                      search: traceDetailSearch(range),
                    })
                  }
                />
                <div className="flex flex-wrap items-center justify-between gap-3 px-1">
                  <span className="text-sm text-muted-foreground tabular-nums">
                    Showing {pageStart}-{pageEnd} of {formatCount(total)}
                  </span>
                  <div className="flex items-center gap-1">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={page <= 1 || pending}
                      onClick={() => update({ page: page - 1 })}
                    >
                      Previous
                    </Button>
                    {pages.map((item, index) =>
                      item === "..." ? (
                        <span
                          key={`ellipsis-${index}`}
                          className="px-2 text-sm text-muted-foreground"
                        >
                          ...
                        </span>
                      ) : (
                        <Button
                          key={item}
                          type="button"
                          variant={item === page ? "secondary" : "ghost"}
                          size="sm"
                          disabled={pending}
                          onClick={() => update({ page: item })}
                        >
                          {item}
                        </Button>
                      )
                    )}
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={page >= totalPages || pending}
                      onClick={() => update({ page: page + 1 })}
                    >
                      Next
                    </Button>
                  </div>
                </div>
              </>
            ) : (
              <EmptyState
                icon={IconAffiliateFilled}
                title={hasFilters ? "No matching traces" : "No traces yet"}
                description={
                  hasFilters ? (
                    "Try a different search or clear filters."
                  ) : (
                    <span className="font-mono text-xs">
                      OTLP/gRPC: localhost:4317 · OTLP/HTTP: localhost:4318
                    </span>
                  )
                }
              />
            )}
          </div>
        </div>
      </QueryBar>
    </div>
  )
}
