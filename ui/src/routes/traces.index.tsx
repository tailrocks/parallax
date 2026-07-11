import {
  createFileRoute,
  useNavigate,
  useRouter,
  useRouterState,
} from "@tanstack/react-router"
import {
  IconAffiliateFilled,
  IconAlertTriangle,
  IconPlayerPlayFilled,
  IconPlayerStopFilled,
  IconRefresh,
} from "@tabler/icons-react"
import { useEffect, useMemo, useState } from "react"
import { z } from "zod"

import { AttributeComparePanel } from "@/components/console/attribute-compare"
import { PageHeader } from "@/components/page-header"
import { useLiveStream } from "@/hooks/use-live-stream"
import { FieldExplorer } from "@/components/console/field-explorer"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  ClearFiltersButton,
  FilterSelect,
  SearchInput,
  SortableHead,
  ToggleChip,
  pageWindow,
  parseSortParam,
} from "@/components/console/data-table"
import { EmptyState } from "@/components/console/empty-state"
import { HeatCell, buildHeatScale } from "@/components/console/heat-cell"
import { useDelayedLoading } from "@/components/console/hooks"
import { RangePicker } from "@/components/console/range-picker"
import { RelativeTime } from "@/components/console/relative-time"
import { TableSkeleton } from "@/components/console/skeletons"
import { formatCount, formatDurationNs, formatTimeInRange } from "@/lib/format"
import { gqlString, graphql } from "@/lib/api"
import type { AttributeCompareRow, LiveSpan, TraceSummary } from "@/lib/api"
import {
  rangeLinkSearch,
  resolveRangeSearch,
  updateRangeSearch,
} from "@/lib/range"
import type { ResolvedRange } from "@/lib/range"
import { cn } from "@/lib/utils"

/** Live feed span — derived from shared `Span` (+ `runId` on the wire). */
type SpanDoc = LiveSpan

type TraceSort =
  | "START_DESC"
  | "DURATION_DESC"
  | "DURATION_ASC"
  | "SPAN_COUNT_DESC"

interface TracePage {
  total: string
  items: TraceSummary[]
}

export interface TracesSearch {
  q?: string | undefined
  service?: string | undefined
  errors?: boolean | undefined
  minMs?: number | undefined
  maxMs?: number | undefined
  sort?: TraceSort | undefined
  page?: number | undefined
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  live?: boolean | undefined
}

const PAGE_SIZE = 25
const SORTS: TraceSort[] = [
  "START_DESC",
  "DURATION_DESC",
  "DURATION_ASC",
  "SPAN_COUNT_DESC",
]

const traceSearchSchema = z.object({
  q: z.unknown().optional(),
  service: z.unknown().optional(),
  errors: z.unknown().optional(),
  minMs: z.unknown().optional(),
  maxMs: z.unknown().optional(),
  sort: z.unknown().optional(),
  page: z.unknown().optional(),
  range: z.unknown().optional(),
  from: z.unknown().optional(),
  to: z.unknown().optional(),
  live: z.unknown().optional(),
})

export function validateTracesSearch(
  search: Record<string, unknown>
): TracesSearch {
  const parsed = traceSearchSchema.parse(search)
  const positiveNumber = (value: unknown) => {
    const number = Number(value)
    return Number.isFinite(number) && number > 0 ? number : undefined
  }
  const positiveInteger = (value: unknown) => {
    const number = Number(value)
    return Number.isInteger(number) && number > 0 ? number : undefined
  }
  const stringValue = (value: unknown) =>
    typeof value === "string" && value ? value : undefined
  return {
    q: stringValue(parsed.q),
    service: stringValue(parsed.service),
    errors: parsed.errors === "1" || parsed.errors === true ? true : undefined,
    minMs: positiveNumber(parsed.minMs),
    maxMs: positiveNumber(parsed.maxMs),
    sort: SORTS.includes(parsed.sort as TraceSort)
      ? (parsed.sort as TraceSort)
      : undefined,
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
  "range",
  "from",
  "to",
  "live",
])

export function patchTracesSearch(
  current: TracesSearch,
  patch: TraceSearchPatch
): TracesSearch {
  const next: TracesSearch = { ...current, ...patch }
  for (const key of Object.keys(next) as Array<keyof TracesSearch>) {
    if (next[key] === "" || next[key] == null || next[key] === false) {
      delete next[key]
    }
  }
  if (
    Object.keys(patch).some((key) => FILTER_KEYS.has(key as keyof TracesSearch))
  ) {
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
    default:
      return undefined
  }
}

export function traceDetailSearch(range: ResolvedRange) {
  return rangeLinkSearch(range)
}

export function paramToTraceSort(
  param: string | undefined
): TraceSort | undefined {
  const parsed = parseSortParam(param)
  if (!parsed) return undefined
  if (parsed.key === "duration") {
    return parsed.direction === "asc" ? "DURATION_ASC" : "DURATION_DESC"
  }
  if (parsed.key === "spans" && parsed.direction === "desc")
    return "SPAN_COUNT_DESC"
  if (parsed.key === "when" && parsed.direction === "desc") return "START_DESC"
  return undefined
}

function graphQlTraceArgs(search: TracesSearch, range: ResolvedRange): string {
  const page = search.page ?? 1
  return [
    search.service ? `service: "${gqlString(search.service)}"` : null,
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    search.minMs ? `minDurationMs: ${search.minMs}` : null,
    search.maxMs ? `maxDurationMs: ${search.maxMs}` : null,
    search.errors ? "errorOnly: true" : null,
    search.q ? `query: "${gqlString(search.q)}"` : null,
    search.sort ? `sort: ${search.sort}` : null,
    `limit: ${PAGE_SIZE}`,
    `offset: ${(page - 1) * PAGE_SIZE}`,
  ]
    .filter(Boolean)
    .join(", ")
}

function baselineRange(range: ResolvedRange): ResolvedRange {
  const from = BigInt(range.fromNanos)
  const to = BigInt(range.toNanos)
  const width = to > from ? to - from : 1n
  const baselineTo = from > 0n ? from - 1n : 0n
  const baselineFrom = baselineTo > width ? baselineTo - width : 0n
  return {
    key: "baseline",
    fromNanos: baselineFrom.toString(),
    toNanos: baselineTo.toString(),
  }
}

function graphQlAttributeCompareArgs(
  search: TracesSearch,
  range: ResolvedRange
): string {
  const baseline = baselineRange(range)
  return [
    `selectedFromNanos: "${range.fromNanos}"`,
    `selectedToNanos: "${range.toNanos}"`,
    `baselineFromNanos: "${baseline.fromNanos}"`,
    `baselineToNanos: "${baseline.toNanos}"`,
    search.service ? `service: "${gqlString(search.service)}"` : null,
    search.errors ? "errorOnly: true" : null,
    "topN: 8",
  ]
    .filter(Boolean)
    .join(", ")
}

export const Route = createFileRoute("/traces/")({
  validateSearch: validateTracesSearch,
  loaderDeps: ({ search }) => search,
  loader: ({ deps }) => {
    if (deps.live) {
      return graphql<{ services: string[] }>(`
        {
          services
        }
      `).then((data) => ({
        services: data.services,
        tracesPage: { total: "0", items: [] },
        attributeCompare: [],
      }))
    }
    const range = resolveRangeSearch(deps)
    const args = graphQlTraceArgs(deps, range)
    const compareArgs = graphQlAttributeCompareArgs(deps, range)
    return graphql<{
      services: string[]
      tracesPage: TracePage
      attributeCompare: AttributeCompareRow[]
    }>(`
      {
        services
        tracesPage(${args}) {
          total
          items {
            traceId rootName service startNanos durationNs spanCount hasError
          }
        }
        attributeCompare(${compareArgs}) {
          key value selectedCount selectedTotal baselineCount baselineTotal score
        }
      }
    `)
  },
  component: TracesPage,
})

function toNumber(value: string): number {
  const number = Number(value)
  return Number.isFinite(number) ? number : 0
}

function liveDurationMs(search: TracesSearch): number {
  return search.minMs && search.minMs > 0 ? search.minMs : NaN
}

function statusError(statusCode: string): boolean {
  return statusCode === "STATUS_CODE_ERROR"
}

function TracesPage() {
  const { services, tracesPage, attributeCompare } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const router = useRouter()
  const pending = useRouterState({
    select: (state) => state.status === "pending",
  })
  const showSkeleton = useDelayedLoading(pending)
  const [lookup, setLookup] = useState("")
  const [spans, setSpans] = useState<SpanDoc[]>([])
  const live = search.live === true
  const page = search.page ?? 1
  const range = useMemo(
    () => resolveRangeSearch(search),
    [search.range, search.from, search.to]
  )
  const total = toNumber(tracesPage.total)
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE))
  const pages = pageWindow(page, totalPages)
  const hasFilters = Boolean(
    search.q ||
    search.service ||
    search.errors ||
    search.minMs ||
    search.maxMs ||
    search.live
  )
  const durationValues = tracesPage.items.map((trace) =>
    Number(trace.durationNs)
  )
  const liveDurationValues = spans.map((span) => Number(span.durationNs))
  const serviceOptions = services.map((service) => ({
    value: service,
    label: service,
  }))
  const update = (patch: TraceSearchPatch, replace = true) => {
    void navigate({ search: patchTracesSearch(search, patch), replace })
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
  }, [live, search.service, search.errors, search.minMs, search.q])

  // Preserve prior behavior: clear the live buffer when the stream URL changes.
  useEffect(() => {
    if (!streamUrl) return
    setSpans([])
  }, [streamUrl])

  const streamStatus = useLiveStream<SpanDoc>({
    url: streamUrl,
    parse: (data) => {
      const batch: unknown = JSON.parse(data)
      return Array.isArray(batch) ? (batch as SpanDoc[]) : []
    },
    onBatch: (incoming) => {
      setSpans((current) => [...incoming.reverse(), ...current].slice(0, 100))
    },
  })

  const pageStart = total === 0 ? 0 : (page - 1) * PAGE_SIZE + 1
  const pageEnd = Math.min(
    total,
    (page - 1) * PAGE_SIZE + tracesPage.items.length
  )

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
            <RangePicker
              value={range}
              onChange={(next) => update(updateRangeSearch(next))}
            />
          </>
        }
      />

      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(value) => update({ q: value || undefined })}
            placeholder="Search root span..."
          />
          <FilterSelect
            onChange={(value) => update({ service: value })}
            options={serviceOptions}
            placeholder="All services"
            {...(search.service ? { value: search.service } : {})}
          />
          <Input
            value={search.minMs?.toString() ?? ""}
            inputMode="numeric"
            onChange={(event) =>
              update({
                minMs: event.target.value
                  ? Number(event.target.value)
                  : undefined,
              })
            }
            placeholder="Min ms"
            className="h-8 w-24"
          />
          <Input
            value={search.maxMs?.toString() ?? ""}
            inputMode="numeric"
            onChange={(event) =>
              update({
                maxMs: event.target.value
                  ? Number(event.target.value)
                  : undefined,
              })
            }
            placeholder="Max ms"
            className="h-8 w-24"
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
          {hasFilters ? (
            <ClearFiltersButton
              onClick={() =>
                update({
                  q: undefined,
                  service: undefined,
                  errors: undefined,
                  minMs: undefined,
                  maxMs: undefined,
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
              ) : streamStatus === "error" ? (
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
        </div>

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
                <CardDescription>
                  Selected window vs previous window
                </CardDescription>
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
  )
}

export function TraceTable({
  rows,
  durationValues,
  range,
  sort,
  onSort,
  onOpen,
}: {
  rows: TraceSummary[]
  durationValues: number[]
  range: ResolvedRange
  sort: string | undefined
  onSort: (next: string | undefined) => void
  onOpen: (traceId: string) => void
}) {
  const durationScale = useMemo(
    () => buildHeatScale(durationValues),
    [durationValues]
  )
  return (
    <Table density="compact">
      <TableHeader>
        <TableRow>
          <TableHead>Trace</TableHead>
          <TableHead className="w-28 text-right">
            <SortableHead sort={sort ?? ""} sortKey="spans" onSort={onSort}>
              Spans
            </SortableHead>
          </TableHead>
          <TableHead className="w-32 text-right">
            <SortableHead sort={sort ?? ""} sortKey="duration" onSort={onSort}>
              Duration
            </SortableHead>
          </TableHead>
          <TableHead className="w-32 text-right">
            <SortableHead sort={sort ?? ""} sortKey="when" onSort={onSort}>
              When
            </SortableHead>
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {rows.map((trace) => (
          <TableRow
            key={`${trace.traceId}-${trace.startNanos}`}
            interactive
            onClick={() => onOpen(trace.traceId)}
            className={cn(
              trace.hasError && "shadow-[inset_1px_0_0_0_var(--color-rose-500)]"
            )}
          >
            <TableCell>
              <div className="flex min-w-0 flex-col gap-1">
                <div className="flex min-w-0 items-center gap-2">
                  <span className="truncate font-medium">{trace.rootName}</span>
                  {trace.hasError ? <Badge variant="rose">errors</Badge> : null}
                </div>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Badge variant="outline">{trace.service || "unknown"}</Badge>
                  <span className="font-mono">
                    {trace.traceId.slice(0, 16)}
                  </span>
                </div>
              </div>
            </TableCell>
            <TableCell className="text-right tabular-nums">
              {trace.spanCount}
            </TableCell>
            <TableCell className="text-right">
              <HeatCell value={Number(trace.durationNs)} scale={durationScale}>
                {formatDurationNs(trace.durationNs)}
              </HeatCell>
            </TableCell>
            <TableCell className="text-right text-muted-foreground tabular-nums">
              <span title={formatTimeInRange(trace.startNanos, range)}>
                <RelativeTime nanos={trace.startNanos} />
              </span>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
