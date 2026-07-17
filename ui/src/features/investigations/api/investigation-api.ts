// Decoded investigations GraphQL adapters (Plan 134).

import {
  InvestigationDeleteDocument,
  InvestigationDeleteMutationSchema,
  type InvestigationDeleteMutation,
  type InvestigationDeleteMutationVariables,
} from "@/features/investigations/api/investigation-delete.generated"
import {
  InvestigationDetailDocument,
  InvestigationDetailQuerySchema,
  type InvestigationDetailQuery,
  type InvestigationDetailQueryVariables,
} from "@/features/investigations/api/investigation-detail.generated"
import {
  InvestigationPinOptionsDocument,
  InvestigationPinOptionsQuerySchema,
  type InvestigationPinOptionsQuery,
  type InvestigationPinOptionsQueryVariables,
} from "@/features/investigations/api/investigation-pin-options.generated"
import {
  InvestigationSaveDocument,
  InvestigationSaveMutationSchema,
  type InvestigationSaveMutation,
  type InvestigationSaveMutationVariables,
} from "@/features/investigations/api/investigation-save.generated"
import {
  InvestigationsListDocument,
  InvestigationsListQuerySchema,
  type InvestigationsListQuery,
  type InvestigationsListQueryVariables,
} from "@/features/investigations/api/investigations-list.generated"
import { mapInvestigation, type Investigation } from "@/features/investigations/model/investigation"
import { InvestigationError } from "@/features/investigations/model/investigation-error"
import {
  executeCachedGraphqlOperation,
  executeGraphqlOperation,
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

function mapBoundary(error: unknown, code: InvestigationError["code"]): never {
  if (error instanceof InvestigationError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new InvestigationError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new InvestigationError(code, error instanceof Error ? error.message : String(error))
}

export async function loadInvestigationsList(): Promise<Investigation[]> {
  try {
    const data = await executeCachedGraphqlOperation<
      InvestigationsListQuery,
      InvestigationsListQueryVariables
    >(brandDocument(InvestigationsListDocument), brandSchema(InvestigationsListQuerySchema), {})
    return data.investigations.map(mapInvestigation)
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadInvestigationDetail(id: string): Promise<Investigation | null> {
  try {
    const data = await executeCachedGraphqlOperation<
      InvestigationDetailQuery,
      InvestigationDetailQueryVariables
    >(brandDocument(InvestigationDetailDocument), brandSchema(InvestigationDetailQuerySchema), {
      id,
    })
    return data.investigation ? mapInvestigation(data.investigation) : null
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadInvestigationPinOptions(): Promise<Investigation[]> {
  try {
    const data = await executeGraphqlOperation<
      InvestigationPinOptionsQuery,
      InvestigationPinOptionsQueryVariables
    >(
      brandDocument(InvestigationPinOptionsDocument),
      brandSchema(InvestigationPinOptionsQuerySchema),
      {}
    )
    return data.investigations.map((row) =>
      mapInvestigation({
        ...row,
        createdAtNanos: row.updatedAtNanos,
      })
    )
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function saveInvestigation(input: {
  readonly name: string
  readonly state: string
  readonly id?: string | undefined
}): Promise<Investigation> {
  try {
    const data = await executeGraphqlOperation<
      InvestigationSaveMutation,
      InvestigationSaveMutationVariables
    >(brandDocument(InvestigationSaveDocument), brandSchema(InvestigationSaveMutationSchema), {
      name: input.name,
      state: input.state,
      id: input.id ?? null,
    })
    return mapInvestigation(data.investigationSave)
  } catch (error) {
    mapBoundary(error, "save")
  }
}

export async function deleteInvestigation(id: string): Promise<void> {
  try {
    await executeGraphqlOperation<
      InvestigationDeleteMutation,
      InvestigationDeleteMutationVariables
    >(brandDocument(InvestigationDeleteDocument), brandSchema(InvestigationDeleteMutationSchema), {
      id,
    })
  } catch (error) {
    mapBoundary(error, "delete")
  }
}
