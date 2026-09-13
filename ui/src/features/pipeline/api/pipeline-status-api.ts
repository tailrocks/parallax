// Decoded pipeline-status GraphQL adapter (R2). Uncached transport: drop
// counters and queue watermarks must read live on every page load.

import {
  PipelineStatusDocument,
  PipelineStatusQuerySchema,
  type PipelineStatusQuery,
  type PipelineStatusQueryVariables,
} from "@/features/pipeline/api/pipeline-status.generated"
import { PipelineError } from "@/features/pipeline/model/pipeline-error"
import { mapPipelineStatus, type PipelineStatus } from "@/features/pipeline/model/pipeline-status"
import { executeGraphqlOperation, type OperationResultSchema } from "@/platform/graphql/client"
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
  if (error instanceof PipelineError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new PipelineError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : "transport",
      error.message
    )
  }
  throw new PipelineError("load", error instanceof Error ? error.message : String(error))
}

export async function loadPipelineStatus(): Promise<PipelineStatus> {
  try {
    const data = await executeGraphqlOperation<PipelineStatusQuery, PipelineStatusQueryVariables>(
      brandDocument(PipelineStatusDocument),
      brandSchema(PipelineStatusQuerySchema),
      {}
    )
    return mapPipelineStatus(data)
  } catch (error) {
    mapBoundary(error)
  }
}
