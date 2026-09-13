// Decoded RUM GraphQL adapters. Every loader hits live Parallax APIs; no
// mocks, no fixtures. Vital p75 uses histogramQuantile (q=0.75).

import {
  RumCatalogDocument,
  RumCatalogQuerySchema,
  type RumCatalogQuery,
  type RumCatalogQueryVariables,
} from "@/features/rum/api/rum-catalog.generated"
import {
  RumErrorsDocument,
  RumErrorsQuerySchema,
  type RumErrorsQuery,
  type RumErrorsQueryVariables,
} from "@/features/rum/api/rum-errors.generated"
import {
  RumJourneysDocument,
  RumJourneysQuerySchema,
  type RumJourneysQuery,
  type RumJourneysQueryVariables,
} from "@/features/rum/api/rum-journeys.generated"
import {
  RumTraceDocument,
  RumTraceQuerySchema,
  type RumTraceQuery,
  type RumTraceQueryVariables,
} from "@/features/rum/api/rum-trace.generated"
import {
  RumVitalDetailDocument,
  RumVitalDetailQuerySchema,
  type RumVitalDetailQuery,
  type RumVitalDetailQueryVariables,
} from "@/features/rum/api/rum-vital-detail.generated"
import {
  RumVitalStatsDocument,
  RumVitalStatsQuerySchema,
  type RumVitalStatsQuery,
  type RumVitalStatsQueryVariables,
} from "@/features/rum/api/rum-vital-stats.generated"
import {
  mapCatalogVitals,
  mapErrorTraces,
  mapExemplars,
  mapIssues,
  mapJourneys,
  mapLinkedTraces,
  mapTraceLogs,
  mapVitalRow,
  mapVitalTrend,
  type CatalogVital,
} from "@/features/rum/api/rum-mapper"
import { RumError } from "@/features/rum/model/rum-error"
import type { RumData, RumTraceData, RumVitalData } from "@/features/rum/model/rum-overview"
import type { RumSearch } from "@/features/rum/model/rum-search"
import {
  executeCachedGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"
import type { ResolvedRange } from "@/domain/time-range/range"
import type { WhereFilter } from "@/shared/where-clause"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown, code: RumError["code"]): never {
  if (error instanceof RumError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new RumError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new RumError(code, error instanceof Error ? error.message : String(error))
}

async function loadCatalog(range: ResolvedRange): Promise<RumCatalogQuery> {
  return executeCachedGraphqlOperation<RumCatalogQuery, RumCatalogQueryVariables>(
    brandDocument(RumCatalogDocument),
    brandSchema(RumCatalogQuerySchema),
    { fromNanos: range.fromNanos, toNanos: range.toNanos, limit: 500 }
  )
}

async function loadVitalStats(
  name: string,
  service: string | null,
  range: ResolvedRange
): Promise<RumVitalStatsQuery> {
  return executeCachedGraphqlOperation<RumVitalStatsQuery, RumVitalStatsQueryVariables>(
    brandDocument(RumVitalStatsDocument),
    brandSchema(RumVitalStatsQuerySchema),
    {
      name,
      fromNanos: range.fromNanos,
      toNanos: range.toNanos,
      q: 0.75,
      service,
      stepSeconds: 300,
    }
  )
}

function resolveService(
  search: RumSearch,
  catalog: RumCatalogQuery,
  vitals: readonly CatalogVital[]
): string | null {
  if (search.service) return search.service
  for (const vital of vitals) {
    if (vital.services.length > 0) return vital.services[0] ?? null
  }
  return catalog.services[0] ?? null
}

function toAttributeFilters(filters: readonly WhereFilter[]) {
  return filters.map((filter) => ({
    key: filter.key,
    op: filter.op,
    value: filter.value,
  }))
}

export async function loadRum(
  search: RumSearch,
  range: ResolvedRange,
  whereFilters: readonly WhereFilter[]
): Promise<RumData> {
  try {
    const catalog = await loadCatalog(range)
    const catalogVitals = mapCatalogVitals(catalog)
    const service = resolveService(search, catalog, catalogVitals)

    const [stats, errors, journeys] = await Promise.all([
      Promise.all(
        catalogVitals.map(async (vital) => {
          try {
            return await loadVitalStats(vital.name, service, range)
          } catch {
            // One missing histogram must not take down the vitals table.
            return null
          }
        })
      ),
      executeCachedGraphqlOperation<RumErrorsQuery, RumErrorsQueryVariables>(
        brandDocument(RumErrorsDocument),
        brandSchema(RumErrorsQuerySchema),
        {
          service,
          fromNanos: range.fromNanos,
          toNanos: range.toNanos,
          limit: 20,
        }
      ),
      executeCachedGraphqlOperation<RumJourneysQuery, RumJourneysQueryVariables>(
        brandDocument(RumJourneysDocument),
        brandSchema(RumJourneysQuerySchema),
        {
          service,
          fromNanos: range.fromNanos,
          toNanos: range.toNanos,
          attributeFilters: whereFilters.length > 0 ? toAttributeFilters(whereFilters) : null,
          limit: 20,
        }
      ),
    ])

    return {
      services: [...catalog.services],
      service,
      vitals: catalogVitals.map((vital, index) => mapVitalRow(vital, stats[index] ?? null)),
      issues: mapIssues(errors),
      issueTotal: errors.issues.total,
      errorTraces: mapErrorTraces(errors),
      errorTraceTotal: errors.tracesPage.total,
      journeys: mapJourneys(journeys),
      journeyTotal: journeys.tracesPage.total,
    }
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadRumVital(
  search: RumSearch,
  range: ResolvedRange
): Promise<RumVitalData | null> {
  if (!search.vital) return null
  try {
    const catalog = await loadCatalog(range)
    const found = mapCatalogVitals(catalog).find((row) => row.name === search.vital)
    if (!found) return null
    const service = search.service ?? null
    const [stats, detail] = await Promise.all([
      loadVitalStats(found.name, service, range),
      executeCachedGraphqlOperation<RumVitalDetailQuery, RumVitalDetailQueryVariables>(
        brandDocument(RumVitalDetailDocument),
        brandSchema(RumVitalDetailQuerySchema),
        {
          name: found.name,
          fromNanos: range.fromNanos,
          toNanos: range.toNanos,
          service,
          limit: 20,
        }
      ),
    ])
    return {
      row: mapVitalRow(found, stats),
      trend: mapVitalTrend(stats),
      exemplars: mapExemplars(detail),
    }
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadRumTrace(traceId: string): Promise<RumTraceData> {
  try {
    const data = await executeCachedGraphqlOperation<RumTraceQuery, RumTraceQueryVariables>(
      brandDocument(RumTraceDocument),
      brandSchema(RumTraceQuerySchema),
      { traceId }
    )
    return {
      traceId,
      linked: mapLinkedTraces(data),
      logs: mapTraceLogs(data),
    }
  } catch (error) {
    mapBoundary(error, "load")
  }
}
