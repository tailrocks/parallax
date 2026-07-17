// Decoded SQL GraphQL adapters (Plan 135). Raw uncached transport — plan 133 owns cache.

import {
  SqlExecuteDocument,
  SqlExecuteQuerySchema,
  type SqlExecuteQuery,
  type SqlExecuteQueryVariables,
} from "@/features/sql/api/sql-execute.generated"
import {
  SqlSchemaDocument,
  SqlSchemaQuerySchema,
  type SqlSchemaQuery,
  type SqlSchemaQueryVariables,
} from "@/features/sql/api/sql-schema.generated"
import {
  SqlSnippetDeleteDocument,
  SqlSnippetDeleteMutationSchema,
  type SqlSnippetDeleteMutation,
  type SqlSnippetDeleteMutationVariables,
} from "@/features/sql/api/sql-snippet-delete.generated"
import {
  SqlSnippetSaveDocument,
  SqlSnippetSaveMutationSchema,
  type SqlSnippetSaveMutation,
  type SqlSnippetSaveMutationVariables,
} from "@/features/sql/api/sql-snippet-save.generated"
import {
  SqlSnippetsListDocument,
  SqlSnippetsListQuerySchema,
  type SqlSnippetsListQuery,
  type SqlSnippetsListQueryVariables,
} from "@/features/sql/api/sql-snippets-list.generated"
import { SqlError, type SqlErrorCode } from "@/features/sql/model/sql-error"
import { mapSqlResult, type SqlResult } from "@/features/sql/model/sql-result"
import {
  groupSchemaRows,
  type SchemaColumn,
} from "@/features/sql/model/sql-row"
import {
  mapSqlSnippet,
  SQL_SNIPPET_PAGE,
  type SqlSnippet,
} from "@/features/sql/model/sql-snippet"
import {
  executeGraphqlOperation,
  type OperationResultSchema,
} from "@/platform/graphql/client"
import { GraphqlBoundaryError } from "@/platform/graphql/error"
import type { TypedDocumentNode } from "@/platform/graphql/typed-document"

const SCHEMA_DISCOVERY_SQL =
  "SELECT table_name, column_name, data_type FROM information_schema.columns WHERE table_schema = 'public' ORDER BY table_name, column_name"

function brandDocument<TResult, TVariables>(
  document: unknown
): TypedDocumentNode<TResult, TVariables> {
  return document as unknown as TypedDocumentNode<TResult, TVariables>
}

function brandSchema<T>(schema: unknown): OperationResultSchema<T> {
  return schema as OperationResultSchema<T>
}

function mapBoundary(error: unknown, code: SqlErrorCode): never {
  if (error instanceof SqlError) throw error
  if (error instanceof GraphqlBoundaryError) {
    throw new SqlError(
      error.code === "invalid-operation-data" ||
        error.code === "invalid-envelope" ||
        error.code === "graphql-errors"
        ? "invalid-response"
        : code,
      error.message
    )
  }
  throw new SqlError(
    code,
    error instanceof Error ? error.message : String(error)
  )
}

export async function loadSqlSchema(): Promise<Map<string, SchemaColumn[]>> {
  try {
    const data = await executeGraphqlOperation<
      SqlSchemaQuery,
      SqlSchemaQueryVariables
    >(brandDocument(SqlSchemaDocument), brandSchema(SqlSchemaQuerySchema), {
      query: SCHEMA_DISCOVERY_SQL,
    })
    return groupSchemaRows(data.sql.rows)
  } catch (error) {
    mapBoundary(error, "schema-discovery")
  }
}

export async function runSql(query: string): Promise<SqlResult> {
  try {
    const data = await executeGraphqlOperation<
      SqlExecuteQuery,
      SqlExecuteQueryVariables
    >(brandDocument(SqlExecuteDocument), brandSchema(SqlExecuteQuerySchema), {
      query,
    })
    return mapSqlResult(data.sql)
  } catch (error) {
    mapBoundary(error, "query-execution")
  }
}

export async function loadSqlSnippets(): Promise<SqlSnippet[]> {
  try {
    const data = await executeGraphqlOperation<
      SqlSnippetsListQuery,
      SqlSnippetsListQueryVariables
    >(
      brandDocument(SqlSnippetsListDocument),
      brandSchema(SqlSnippetsListQuerySchema),
      { page: SQL_SNIPPET_PAGE }
    )
    return data.savedViews.map(mapSqlSnippet)
  } catch (error) {
    mapBoundary(error, "snippet-list")
  }
}

export async function saveSqlSnippet(input: {
  readonly name: string
  readonly state: string
}): Promise<SqlSnippet> {
  try {
    const data = await executeGraphqlOperation<
      SqlSnippetSaveMutation,
      SqlSnippetSaveMutationVariables
    >(
      brandDocument(SqlSnippetSaveDocument),
      brandSchema(SqlSnippetSaveMutationSchema),
      {
        name: input.name,
        page: SQL_SNIPPET_PAGE,
        state: input.state,
      }
    )
    return mapSqlSnippet(data.savedViewSave)
  } catch (error) {
    mapBoundary(error, "snippet-save")
  }
}

export async function deleteSqlSnippet(id: string): Promise<void> {
  try {
    await executeGraphqlOperation<
      SqlSnippetDeleteMutation,
      SqlSnippetDeleteMutationVariables
    >(
      brandDocument(SqlSnippetDeleteDocument),
      brandSchema(SqlSnippetDeleteMutationSchema),
      { id }
    )
  } catch (error) {
    mapBoundary(error, "snippet-delete")
  }
}
