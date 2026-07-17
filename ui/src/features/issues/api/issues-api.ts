// Decoded issues GraphQL adapters (Plan 139).

import {
  IssueCorrelationDocument,
  IssueCorrelationQuerySchema,
  type IssueCorrelationQuery,
  type IssueCorrelationQueryVariables,
} from "@/features/issues/api/issue-correlation.generated"
import {
  IssueDetailDocument,
  IssueDetailQuerySchema,
  type IssueDetailQuery,
  type IssueDetailQueryVariables,
} from "@/features/issues/api/issue-detail.generated"
import {
  IssueOccurrencesDocument,
  IssueOccurrencesQuerySchema,
  type IssueOccurrencesQuery,
  type IssueOccurrencesQueryVariables,
} from "@/features/issues/api/issue-occurrences.generated"
import {
  IssueSetStatusDocument,
  IssueSetStatusMutationSchema,
  type IssueSetStatusMutation,
  type IssueSetStatusMutationVariables,
} from "@/features/issues/api/issue-status.generated"
import {
  IssuesListDocument,
  IssuesListQuerySchema,
  type IssuesListQuery,
  type IssuesListQueryVariables,
} from "@/features/issues/api/issues-list.generated"
import { mapIssueDetail, mapIssueEvents, mapIssuesList } from "@/features/issues/api/issues-mapper"
import { rangeHours } from "@/features/issues/model/issue-detail"
import type { IssueDetailData, IssueEvent } from "@/features/issues/model/issue-detail"
import type { IssuesData } from "@/features/issues/model/issue-summary"
import type { IssuesSearch } from "@/features/issues/model/issues-search"
import { IssuesError } from "@/features/issues/model/issues-error"
import {
  executeCachedGraphqlOperation,
  executeGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"
import type { ResolvedRange } from "@/domain/time-range/range"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown, code: IssuesError["code"]): never {
  if (error instanceof IssuesError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new IssuesError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new IssuesError(code, error instanceof Error ? error.message : String(error))
}

export async function loadIssues(search: IssuesSearch, range: ResolvedRange): Promise<IssuesData> {
  try {
    const data = await executeCachedGraphqlOperation<IssuesListQuery, IssuesListQueryVariables>(
      brandDocument(IssuesListDocument),
      brandSchema(IssuesListQuerySchema),
      {
        service: search.service ?? null,
        status: search.status ?? null,
        query: search.q ?? null,
        fromNanos: range.fromNanos,
        toNanos: range.toNanos,
        sort: search.sort ?? "LAST_SEEN",
        limit: 100,
      }
    )
    return mapIssuesList(data)
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadIssueDetail(
  fingerprint: string,
  range: ResolvedRange
): Promise<IssueDetailData> {
  try {
    const data = await executeCachedGraphqlOperation<IssueDetailQuery, IssueDetailQueryVariables>(
      brandDocument(IssueDetailDocument),
      brandSchema(IssueDetailQuerySchema),
      {
        fingerprint,
        fromNanos: range.fromNanos,
        toNanos: range.toNanos,
        hours: rangeHours(range),
      }
    )

    let resource: Record<string, unknown> = {}
    let breadcrumbs: IssueDetailData["breadcrumbs"] = []
    let traceRunId: string | null = null
    let releaseVersion: string | null = null
    const traceId = data.issue?.lastTraceId
    if (traceId) {
      try {
        const correlated = await executeCachedGraphqlOperation<
          IssueCorrelationQuery,
          IssueCorrelationQueryVariables
        >(brandDocument(IssueCorrelationDocument), brandSchema(IssueCorrelationQuerySchema), {
          traceId,
        })
        const resourceRaw = correlated.trace?.spans[0]?.resource ?? "{}"
        try {
          resource = JSON.parse(resourceRaw) as Record<string, unknown>
        } catch {
          resource = {}
        }
        const version = resource["service.version"]
        releaseVersion = typeof version === "string" && version.trim() ? version.trim() : null
        breadcrumbs = correlated.logsByTrace.slice(-12).map((log) => ({
          tsNanos: log.tsNanos,
          severityText: log.severityText,
          body: log.body,
        }))
        traceRunId = correlated.trace?.spans.find((s) => s.invocationId)?.invocationId ?? null
      } catch {
        // Trace may have aged out; issue detail still renders.
      }
    }

    return mapIssueDetail(data, {
      resource,
      breadcrumbs,
      traceRunId,
      releaseVersion,
    })
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function setIssueStatus(
  fingerprint: string,
  status: "open" | "resolved"
): Promise<void> {
  try {
    await executeGraphqlOperation<IssueSetStatusMutation, IssueSetStatusMutationVariables>(
      brandDocument(IssueSetStatusDocument),
      brandSchema(IssueSetStatusMutationSchema),
      { fingerprint, status }
    )
  } catch (error) {
    mapBoundary(error, "mutation")
  }
}

export async function loadIssueOccurrences(
  fingerprint: string,
  fromNanos: string,
  toNanos: string
): Promise<readonly IssueEvent[]> {
  try {
    const data = await executeGraphqlOperation<
      IssueOccurrencesQuery,
      IssueOccurrencesQueryVariables
    >(brandDocument(IssueOccurrencesDocument), brandSchema(IssueOccurrencesQuerySchema), {
      fingerprint,
      fromNanos,
      toNanos,
    })
    return mapIssueEvents(data.issue?.events ?? [])
  } catch (error) {
    mapBoundary(error, "load")
  }
}
