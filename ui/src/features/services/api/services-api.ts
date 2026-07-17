// Decoded services GraphQL adapters (Plan 138). Cached transport preserves
// previous graphqlCached TTL/dedup until plan 133 owns cache.

import {
  ServiceDetailDocument,
  ServiceDetailQuerySchema,
  type ServiceDetailQuery,
  type ServiceDetailQueryVariables,
} from "@/features/services/api/service-detail.generated"
import {
  ServicesListDocument,
  ServicesListQuerySchema,
  type ServicesListQuery,
  type ServicesListQueryVariables,
} from "@/features/services/api/services-list.generated"
import {
  mapServiceDetail,
  mapServicesList,
} from "@/features/services/api/services-mapper"
import type { ServiceDetailData } from "@/features/services/model/service-detail"
import { stepSecondsForRange } from "@/features/services/model/service-detail"
import type { ServicesData } from "@/features/services/model/service-summary"
import { ServicesError } from "@/features/services/model/services-error"
import {
  executeCachedGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"
import type { ResolvedRange } from "@/lib/range"
import * as Semconv from "@/shared/semconv"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown, code: ServicesError["code"]): never {
  if (error instanceof ServicesError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new ServicesError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new ServicesError(
    code,
    error instanceof Error ? error.message : String(error)
  )
}

export async function loadServices(
  range: ResolvedRange
): Promise<ServicesData> {
  try {
    const data = await executeCachedGraphqlOperation<
      ServicesListQuery,
      ServicesListQueryVariables
    >(
      brandDocument(ServicesListDocument),
      brandSchema(ServicesListQuerySchema),
      {
        fromNanos: range.fromNanos,
        toNanos: range.toNanos,
      }
    )
    return mapServicesList(data)
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadServiceDetail(
  service: string,
  range: ResolvedRange
): Promise<ServiceDetailData> {
  try {
    const data = await executeCachedGraphqlOperation<
      ServiceDetailQuery,
      ServiceDetailQueryVariables
    >(
      brandDocument(ServiceDetailDocument),
      brandSchema(ServiceDetailQuerySchema),
      {
        service,
        fromNanos: range.fromNanos,
        toNanos: range.toNanos,
        stepSeconds: stepSecondsForRange(range),
        httpDurationMetric: Semconv.HTTP_SERVER_REQUEST_DURATION,
        rpcDurationMetric: Semconv.REQUEST_DURATION_METRICS[1],
      }
    )
    return mapServiceDetail(data)
  } catch (error) {
    mapBoundary(error, "load")
  }
}
