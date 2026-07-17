import { useEffect, useState } from "react"
import { useNavigate, useRouter } from "@tanstack/react-router"
import { IconRefresh, IconTerminal2 } from "@tabler/icons-react"
import { z } from "zod"
import { CopyButton } from "@/shared/console/copy-button"
import { EmptyState } from "@/shared/console/empty-state"
import { FacetSidebar, type Facet } from "@/shared/console/facet-sidebar"
import { FilterSelect, SearchInput, Toolbar } from "@/shared/console/data-table"
import { InvocationsTable } from "@/features/invocations/components/invocations-table"
import { RangePicker } from "@/features/time-range"
import { PageHeader } from "@/shared/components/page-header"
import { Button } from "@/components/ui/button"
import { graphql, graphqlCached } from "../api/gql"
import type { Invocation, ObservedInvocation } from "../model/wire"
import { formatCount } from "@/shared/format"
import {
  invocationStatus,
  mergeInvocations,
  type InvocationRow,
  type InvocationStatus,
} from "../model/invocation"
import {
  rangeLinkSearch,
  rangeSearchSchema,
  resolveRangeSearch,
  updateRangeSearch,
  type ResolvedRange,
} from "@/domain/range"
import { usePageVisible } from "@/platform/visibility/use-page-visible"

const MODES = ["one_shot", "interactive", "daemon", "capsule"] as const
const STATUSES = ["running", "finished", "failed", "stale"] as const
const OUTCOMES = ["success", "failure", "error", "timeout", "skip", "cancellation"] as const

export interface InvocationsSearch {
  q?: string
  mode?: (typeof MODES)[number]
  status?: InvocationStatus
  outcome?: (typeof OUTCOMES)[number]
  live?: boolean
  range?: string
  from?: string
  to?: string
}

type InvocationsSearchPatch = {
  [K in keyof InvocationsSearch]?: InvocationsSearch[K] | undefined
}

function invocationFacetsQuery(range: ResolvedRange) {
  return `invocationFacets(fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}") { dimension values { value count } }`
}

const INVOCATIONS_QUERY = `
  {
    invocations {
      invocationId
      registration
      command
      appMode
      outcome
      status
      exitCode
      startedAtNanos
      endedAtNanos
      errorCount
      traceCount
      sessionCount
    }
    observedInvocations {
      invocationId
      service
      lastCommand
      appMode
      firstNanos
      lastNanos
      spanCount
      logCount
    }
  }
`

interface InvocationsQueryData {
  invocations: Invocation[]
  observedInvocations: ObservedInvocation[]
}

interface InvocationFacet {
  dimension: string
  values: Array<{ value: string; count: string }>
}

export function filterInvocationsByRange(
  rows: InvocationRow[],
  range: ResolvedRange,
  nowNanos = (BigInt(Date.now()) * 1_000_000n).toString()
): InvocationRow[] {
  const windowStart = BigInt(range.fromNanos)
  const windowEnd = BigInt(range.toNanos)
  const openEnd = BigInt(nowNanos)
  return rows.filter((row) => {
    const start = BigInt(row.startedAtNanos)
    const end = row.endedAtNanos != null ? BigInt(row.endedAtNanos) : openEnd
    return start <= windowEnd && end >= windowStart
  })
}

const invocationsSearchSchema = rangeSearchSchema.extend({
  q: z.unknown().optional(),
  mode: z.unknown().optional(),
  status: z.unknown().optional(),
  outcome: z.unknown().optional(),
  live: z.unknown().optional(),
})

export function validateInvocationsSearch(search: Record<string, unknown>): InvocationsSearch {
  const parsed = invocationsSearchSchema.parse(search)
  const result: InvocationsSearch = {}
  if (typeof parsed.q === "string" && parsed.q) result.q = parsed.q
  const mode = MODES.find((value) => value === parsed.mode)
  if (mode) result.mode = mode
  const status = STATUSES.find((value) => value === parsed.status)
  if (status) result.status = status
  const outcome = OUTCOMES.find((value) => value === parsed.outcome)
  if (outcome) result.outcome = outcome
  if (parsed.live === true || parsed.live === "true") result.live = true
  if (parsed.range) result.range = parsed.range
  if (parsed.from) result.from = parsed.from
  if (parsed.to) result.to = parsed.to
  return result
}

export type InvocationsListData = {
  rows: InvocationRow[]
  facets: InvocationFacet[]
}

export async function loadInvocations(search: InvocationsSearch): Promise<InvocationsListData> {
  const range = resolveRangeSearch(search)
  const facetsQuery = `{ ${invocationFacetsQuery(range)} }`
  const [data, facetsData] = await Promise.all([
    graphqlCached<InvocationsQueryData>(INVOCATIONS_QUERY),
    graphqlCached<{ invocationFacets: InvocationFacet[] }>(facetsQuery),
  ])
  return {
    rows: mergeInvocations(data.invocations, data.observedInvocations),
    facets: facetsData.invocationFacets,
  }
}

export function InvocationsPage({
  data,
  search,
}: {
  data: InvocationsListData
  search: InvocationsSearch
}) {
  const navigate = useNavigate({ from: "/invocations/" })
  const router = useRouter()
  const range = resolveRangeSearch(search)
  const pageVisible = usePageVisible()
  const live = search.live === true
  const [polledRows, setPolledRows] = useState<InvocationRow[] | null>(null)

  useEffect(() => {
    if (!live || !pageVisible) return
    const timer = setInterval(() => {
      void graphql<InvocationsQueryData>(INVOCATIONS_QUERY)
        .then((next) => setPolledRows(mergeInvocations(next.invocations, next.observedInvocations)))
        .catch(() => {})
    }, 5_000)
    return () => clearInterval(timer)
  }, [live, pageVisible])

  const setSearch = (patch: InvocationsSearchPatch) => {
    const raw = { ...search, ...patch }
    const next: InvocationsSearch = {}
    for (const key of Object.keys(raw) as Array<keyof InvocationsSearch>) {
      const value = raw[key]
      if (value != null && value !== "" && value !== false) {
        Object.assign(next, { [key]: value })
      }
    }
    void navigate({ search: next })
  }

  return (
    <InvocationsContent
      rows={polledRows ?? data.rows}
      facets={data.facets}
      search={search}
      range={range}
      live={live}
      onSearch={setSearch}
      onRefresh={() => void router.invalidate()}
      onOpen={(invocationId) =>
        void navigate({
          to: "/invocations/$invocationId",
          params: { invocationId },
          search: rangeLinkSearch(range),
        })
      }
    />
  )
}

export function InvocationsContent({
  rows: allRows,
  facets = [],
  search,
  range,
  live,
  onSearch,
  onRefresh,
  onOpen,
}: {
  rows: InvocationRow[]
  facets?: InvocationFacet[]
  search: InvocationsSearch
  range: ResolvedRange
  live: boolean
  onSearch: (patch: InvocationsSearchPatch) => void
  onRefresh: () => void
  onOpen: (invocationId: string) => void
}) {
  const facetSelections: Record<string, string[]> = {
    ...(search.mode ? { "app.mode": [search.mode] } : {}),
    ...(search.outcome ? { outcome: [search.outcome] } : {}),
    ...(search.q ? { service: [search.q], "cli.command.name": [search.q] } : {}),
  }
  const toggleFacet = (dimension: string, value: string) => {
    if (dimension === "app.mode") {
      const mode = MODES.find((candidate) => candidate === value)
      onSearch({ mode: search.mode === mode ? undefined : mode })
    } else if (dimension === "outcome") {
      const outcome = OUTCOMES.find((candidate) => candidate === value)
      onSearch({ outcome: search.outcome === outcome ? undefined : outcome })
    } else {
      onSearch({ q: search.q === value ? undefined : value })
    }
  }
  const sidebarFacets: Facet[] = facets.map((facet) => ({
    dimension: facet.dimension,
    label: facet.dimension,
    values: facet.values.map((entry) => ({
      value: entry.value,
      count: Number(entry.count),
    })),
    serviceDots: facet.dimension === "service",
    searchable: true,
  }))
  const query = search.q?.toLowerCase() ?? ""
  const rowsInWindow = filterInvocationsByRange(allRows, range)
  const rows = rowsInWindow.filter((row) => {
    if (search.mode && row.appMode !== search.mode) return false
    if (search.status && invocationStatus(row) !== search.status) return false
    if (search.outcome && row.outcome !== search.outcome) return false
    const haystack = `${row.invocationId} ${row.command ?? ""} ${row.service ?? ""}`.toLowerCase()
    return haystack.includes(query)
  })

  return (
    <div className="space-y-4">
      <PageHeader
        icon={IconTerminal2}
        iconClassName="text-violet-500"
        title="CLI Apps"
        description="Every observed CLI invocation — command, mode, sessions, errors, and live activity."
        actions={
          <>
            <Button
              size="sm"
              variant={live ? "secondary" : "outline"}
              onClick={() => onSearch({ live: live ? undefined : true })}
            >
              {live ? (
                <span className="size-1.5 animate-pulse rounded-full bg-emerald-500" />
              ) : null}
              {live ? "Live" : "Go live"}
            </Button>
            {!live ? (
              <Button size="sm" variant="outline" onClick={onRefresh}>
                <IconRefresh />
                Refresh
              </Button>
            ) : null}
            <RangePicker value={range} onChange={(next) => onSearch(updateRangeSearch(next))} />
          </>
        }
      />

      <Toolbar className="justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <SearchInput
            value={search.q ?? ""}
            onChange={(q) => onSearch({ q })}
            placeholder="Search invocations"
          />
          <FilterSelect
            {...(search.mode ? { value: search.mode } : {})}
            onChange={(mode) => onSearch({ mode: MODES.find((value) => value === mode) })}
            placeholder="Any mode"
            options={MODES.map((value) => ({
              value,
              label: value.replace("_", "-"),
            }))}
          />
          <FilterSelect
            {...(search.status ? { value: search.status } : {})}
            onChange={(status) => onSearch({ status: STATUSES.find((value) => value === status) })}
            placeholder="Any status"
            options={STATUSES.map((value) => ({ value, label: value }))}
          />
          <FilterSelect
            {...(search.outcome ? { value: search.outcome } : {})}
            onChange={(outcome) =>
              onSearch({
                outcome: OUTCOMES.find((value) => value === outcome),
              })
            }
            placeholder="Any outcome"
            options={OUTCOMES.map((value) => ({ value, label: value }))}
          />
        </div>
        <span className="text-xs text-muted-foreground">
          {formatCount(rows.length)} of {formatCount(rowsInWindow.length)} in window
        </span>
      </Toolbar>

      <div className="flex items-start gap-4">
        {sidebarFacets.length > 0 ? (
          <FacetSidebar
            facets={sidebarFacets}
            selections={facetSelections}
            onToggle={toggleFacet}
            onClear={() => onSearch({ q: undefined, mode: undefined, outcome: undefined })}
          />
        ) : null}
        <div className="min-w-0 flex-1">
          {rows.length === 0 ? (
            <EmptyState
              icon={IconTerminal2}
              title="No CLI invocations yet"
              description={
                <span className="inline-flex items-center gap-2">
                  <code>parallax invocation start -- &lt;your command&gt;</code>
                  <CopyButton value="parallax invocation start -- <your command>" />
                </span>
              }
            />
          ) : (
            <InvocationsTable rows={rows} detailSearch={rangeLinkSearch(range)} onOpen={onOpen} />
          )}
        </div>
      </div>
    </div>
  )
}
