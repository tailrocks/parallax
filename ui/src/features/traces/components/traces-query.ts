import type { SavedView } from "@/features/logs"
import { gqlString, graphqlCached } from "@/platform/graphql/transport"
import { whereClauseFromSearch } from "@/shared/where-clause"
import { resolveRangeSearch, type ResolvedRange } from "@/domain/time-range/range"
import type { AttributeCompareRow, TraceSummary } from "@/features/traces/model/wire"

export interface TracePage {
  total: string
  items: TraceSummary[]
}

export interface TraceFacet {
  dimension: string
  values: Array<{ value: string; count: string }>
}

export type TraceSort = "START_DESC" | "DURATION_DESC" | "DURATION_ASC" | "SPAN_COUNT_DESC"

export interface TracesSearch {
  q?: string | undefined
  service?: string | undefined
  errors?: boolean | undefined
  minMs?: number | undefined
  maxMs?: number | undefined
  where?: string | undefined
  sort?: TraceSort | undefined
  page?: number | undefined
  range?: string | undefined
  from?: string | undefined
  to?: string | undefined
  live?: boolean | undefined
}

export const PAGE_SIZE = 25

function graphQlAttributeFilters(search: TracesSearch): string | null {
  const filters = whereClauseFromSearch(search.where)
  if (filters.length === 0) return null
  const items = filters
    .map(
      (filter) =>
        `{key: "${gqlString(filter.key)}", op: "${gqlString(filter.op)}", value: "${gqlString(filter.value)}"}`
    )
    .join(", ")
  return `attributeFilters: [${items}]`
}

function graphQlTraceBaseArgs(search: TracesSearch, range: ResolvedRange): string {
  return [
    search.service ? `service: "${gqlString(search.service)}"` : null,
    `fromNanos: "${range.fromNanos}"`,
    `toNanos: "${range.toNanos}"`,
    search.errors ? "errorOnly: true" : null,
    search.q ? `query: "${gqlString(search.q)}"` : null,
    graphQlAttributeFilters(search),
  ]
    .filter(Boolean)
    .join(", ")
}

function graphQlTraceArgs(search: TracesSearch, range: ResolvedRange): string {
  const page = search.page ?? 1
  return [
    graphQlTraceBaseArgs(search, range),
    search.minMs ? `minDurationMs: ${search.minMs}` : null,
    search.maxMs ? `maxDurationMs: ${search.maxMs}` : null,
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

function graphQlAttributeCompareArgs(search: TracesSearch, range: ResolvedRange): string {
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

export type TracesLoaderData = {
  services: string[]
  tracesPage: TracePage
  attributeCompare: AttributeCompareRow[]
  traceFacets: TraceFacet[]
  traceDurationStats: { p50Ms: number | null; p95Ms: number | null }
  savedViews: SavedView[]
}

export async function loadTraces(search: TracesSearch): Promise<TracesLoaderData> {
  if (search.live) {
    return graphqlCached<{ services: string[]; savedViews: SavedView[] }>(`
      {
        services
        savedViews(page: "/traces") { id name page state updatedAtNanos }
      }
    `).then((data) => ({
      services: data.services,
      tracesPage: { total: "0", items: [] },
      attributeCompare: [],
      traceFacets: [],
      traceDurationStats: { p50Ms: null, p95Ms: null },
      savedViews: data.savedViews,
    }))
  }
  const range = resolveRangeSearch(search)
  const args = graphQlTraceArgs(search, range)
  const baseArgs = graphQlTraceBaseArgs(search, range)
  const compareArgs = graphQlAttributeCompareArgs(search, range)
  return graphqlCached<{
    services: string[]
    tracesPage: TracePage
    attributeCompare: AttributeCompareRow[]
    traceFacets: TraceFacet[]
    traceDurationStats: { p50Ms: number | null; p95Ms: number | null }
    savedViews: SavedView[]
  }>(`
    {
      services
      savedViews(page: "/traces") { id name page state updatedAtNanos }
      tracesPage(${args}) {
        total
        items {
          traceId rootName service startNanos durationNs spanCount hasError
        }
      }
      attributeCompare(${compareArgs}) {
        key value selectedCount selectedTotal baselineCount baselineTotal score
      }
      traceFacets(${baseArgs}) {
        dimension
        values { value count }
      }
      traceDurationStats(${baseArgs}) { p50Ms p95Ms }
    }
  `)
}

export function toNumber(value: string): number {
  const number = Number(value)
  return Number.isFinite(number) ? number : 0
}
