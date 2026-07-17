// Decoded dashboard GraphQL adapters (Plan 137).

import {
  DashboardDeleteDocument,
  DashboardDeleteMutationSchema,
  type DashboardDeleteMutation,
  type DashboardDeleteMutationVariables,
} from "@/features/dashboards/api/dashboard-delete.generated"
import {
  DashboardDetailDocument,
  DashboardDetailQuerySchema,
  type DashboardDetailQuery,
  type DashboardDetailQueryVariables,
} from "@/features/dashboards/api/dashboard-detail.generated"
import {
  DashboardSaveDocument,
  DashboardSaveMutationSchema,
  type DashboardSaveMutation,
  type DashboardSaveMutationVariables,
} from "@/features/dashboards/api/dashboard-save.generated"
import {
  DashboardsListDocument,
  DashboardsListQuerySchema,
  type DashboardsListQuery,
  type DashboardsListQueryVariables,
} from "@/features/dashboards/api/dashboards-list.generated"
import { DashboardError } from "@/features/dashboards/model/dashboard-error"
import {
  mapDashboard,
  type Dashboard,
  type DashboardSummary,
} from "@/features/dashboards/model/dashboard"
import {
  executeCachedGraphqlOperation,
  executeGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"
import { graphql } from "@/lib/api"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown, code: DashboardError["code"]): never {
  if (error instanceof DashboardError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new DashboardError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new DashboardError(
    code,
    error instanceof Error ? error.message : String(error)
  )
}

export async function loadDashboardsList(): Promise<{
  dashboards: Dashboard[]
  metricNames: string[]
}> {
  try {
    const data = await executeCachedGraphqlOperation<
      DashboardsListQuery,
      DashboardsListQueryVariables
    >(
      brandDocument(DashboardsListDocument),
      brandSchema(DashboardsListQuerySchema),
      {}
    )
    return {
      dashboards: data.dashboards.map(mapDashboard),
      metricNames: [...data.metricNames],
    }
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadDashboardDetail(id: string): Promise<{
  dashboard: DashboardSummary | null
  metricNames: string[]
}> {
  try {
    const data = await executeCachedGraphqlOperation<
      DashboardDetailQuery,
      DashboardDetailQueryVariables
    >(
      brandDocument(DashboardDetailDocument),
      brandSchema(DashboardDetailQuerySchema),
      { id }
    )
    return {
      dashboard: data.dashboard
        ? {
            id: data.dashboard.id,
            name: data.dashboard.name,
            layout: data.dashboard.layout,
          }
        : null,
      metricNames: [...data.metricNames],
    }
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function saveDashboard(input: {
  readonly name: string
  readonly layout: string
  readonly id?: string | undefined
}): Promise<Dashboard> {
  try {
    const data = await executeGraphqlOperation<
      DashboardSaveMutation,
      DashboardSaveMutationVariables
    >(
      brandDocument(DashboardSaveDocument),
      brandSchema(DashboardSaveMutationSchema),
      {
        name: input.name,
        layout: input.layout,
        id: input.id ?? null,
      }
    )
    return mapDashboard(data.dashboardSave)
  } catch (error) {
    mapBoundary(error, "save")
  }
}

export async function deleteDashboard(id: string): Promise<void> {
  try {
    await executeGraphqlOperation<
      DashboardDeleteMutation,
      DashboardDeleteMutationVariables
    >(
      brandDocument(DashboardDeleteDocument),
      brandSchema(DashboardDeleteMutationSchema),
      { id }
    )
  } catch (error) {
    mapBoundary(error, "delete")
  }
}

/** Minimal raw dashboard id/name list for shell navigation (Plan 143). */
export type DashboardNavigationItem = {
  id: string
  name: string
}

export async function loadDashboardNavigation({
  signal,
}: {
  signal?: AbortSignal
} = {}): Promise<DashboardNavigationItem[]> {
  const options = signal ? { signal } : {}
  const data = await graphql<{ dashboards: DashboardNavigationItem[] }>(
    `
      {
        dashboards {
          id
          name
        }
      }
    `,
    options
  )
  return data.dashboards
}
