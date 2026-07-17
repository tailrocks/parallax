// Decoded test-reporting GraphQL adapters (Plan 155).

import {
  TestCaseDetailDocument,
  TestCaseDetailQuerySchema,
  type TestCaseDetailQuery,
  type TestCaseDetailQueryVariables,
} from "@/features/tests/api/test-case-detail.generated"
import {
  TestsListDocument,
  TestsListQuerySchema,
  type TestsListQuery,
  type TestsListQueryVariables,
} from "@/features/tests/api/tests-list.generated"
import { mapTestCaseDetail, mapTestsList } from "@/features/tests/api/tests-mapper"
import type { TestCaseDetailData } from "@/features/tests/model/test-detail"
import type { TestsData } from "@/features/tests/model/test-summary"
import type { TestsSearch } from "@/features/tests/model/tests-search"
import { TestsError } from "@/features/tests/model/tests-error"
import {
  executeCachedGraphqlOperation,
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

function mapBoundary(error: unknown, code: TestsError["code"]): never {
  if (error instanceof TestsError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new TestsError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new TestsError(code, error instanceof Error ? error.message : String(error))
}

export async function loadTests(search: TestsSearch, range: ResolvedRange): Promise<TestsData> {
  try {
    const data = await executeCachedGraphqlOperation<TestsListQuery, TestsListQueryVariables>(
      brandDocument(TestsListDocument),
      brandSchema(TestsListQuerySchema),
      {
        query: search.q ?? null,
        suite: search.suite ?? null,
        service: search.service ?? null,
        serviceVersion: search.serviceVersion ?? null,
        status: search.status ?? null,
        flakyState: search.flakyState ?? null,
        fromNanos: range.fromNanos,
        toNanos: range.toNanos,
        sort: search.sort ?? "LAST_SEEN",
        limit: 100,
        offset: 0,
      }
    )
    return mapTestsList(data)
  } catch (error) {
    mapBoundary(error, "load")
  }
}

export async function loadTestCaseDetail(caseKey: string): Promise<TestCaseDetailData> {
  try {
    const data = await executeCachedGraphqlOperation<
      TestCaseDetailQuery,
      TestCaseDetailQueryVariables
    >(brandDocument(TestCaseDetailDocument), brandSchema(TestCaseDetailQuerySchema), {
      caseKey,
      variantLimit: 20,
      resultLimit: 50,
    })
    return mapTestCaseDetail(data)
  } catch (error) {
    mapBoundary(error, "load")
  }
}
