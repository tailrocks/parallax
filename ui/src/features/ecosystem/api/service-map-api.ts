// Decoded service-map GraphQL adapter (Plan 136). Cached transport preserves
// the previous graphqlCached TTL/dedup behavior until plan 133 owns cache.

import {
  ServiceMapDocument,
  ServiceMapQuerySchema,
  type ServiceMapQuery,
  type ServiceMapQueryVariables,
} from "@/features/ecosystem/api/service-map.generated"
import { EcosystemError } from "@/features/ecosystem/model/ecosystem-error"
import {
  mapServiceMap,
  type ServiceMap,
} from "@/features/ecosystem/model/service-map"
import {
  executeCachedGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown): never {
  if (error instanceof EcosystemError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new EcosystemError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : "transport",
      error.message
    )
  }
  throw new EcosystemError(
    "load",
    error instanceof Error ? error.message : String(error)
  )
}

export async function loadServiceMap(input: {
  readonly fromNanos: string
  readonly toNanos: string
  readonly maxTraces?: number
}): Promise<ServiceMap> {
  try {
    const data = await executeCachedGraphqlOperation<
      ServiceMapQuery,
      ServiceMapQueryVariables
    >(brandDocument(ServiceMapDocument), brandSchema(ServiceMapQuerySchema), {
      fromNanos: input.fromNanos,
      toNanos: input.toNanos,
      maxTraces: input.maxTraces ?? 100,
    })
    return mapServiceMap(data.serviceMap)
  } catch (error) {
    mapBoundary(error)
  }
}
