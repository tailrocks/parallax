// Plan 152 — typed, secret-safe GraphQL boundary errors.

/** Stable codes for GraphQL transport / decode failures. */
export type GraphqlErrorCode =
  | "http"
  | "malformed-json"
  | "invalid-envelope"
  | "graphql-errors"
  | "invalid-operation-data"
  | "abort"
  | "transport"
  | "invalid-variables"
  | "invalid-document"

export interface GraphqlErrorDiagnostics {
  readonly code: GraphqlErrorCode
  readonly operationName: string | null
  readonly status: number | null
  readonly schemaIssueCount: number | null
  readonly schemaIssuePaths: readonly string[] | null
}

/**
 * Typed Error subclass with bounded, secret-safe diagnostics.
 * Never attach variables, query text, response bodies, headers, or tokens.
 */
export class GraphqlBoundaryError extends Error {
  readonly code: GraphqlErrorCode
  readonly operationName: string | null
  readonly status: number | null
  readonly schemaIssueCount: number | null
  readonly schemaIssuePaths: readonly string[] | null

  constructor(diagnostics: GraphqlErrorDiagnostics, message?: string) {
    const op = diagnostics.operationName ?? "unknown"
    super(
      message ??
        `graphql ${diagnostics.code} (operation=${op}${
          diagnostics.status != null ? ` status=${diagnostics.status}` : ""
        })`
    )
    this.name = "GraphqlBoundaryError"
    this.code = diagnostics.code
    this.operationName = diagnostics.operationName
    this.status = diagnostics.status
    this.schemaIssueCount = diagnostics.schemaIssueCount
    this.schemaIssuePaths = diagnostics.schemaIssuePaths
  }
}

export function graphqlError(
  code: GraphqlErrorCode,
  options: {
    operationName?: string | null
    status?: number | null
    schemaIssueCount?: number | null
    schemaIssuePaths?: readonly string[] | null
    message?: string
  } = {}
): GraphqlBoundaryError {
  return new GraphqlBoundaryError(
    {
      code,
      operationName: options.operationName ?? null,
      status: options.status ?? null,
      schemaIssueCount: options.schemaIssueCount ?? null,
      schemaIssuePaths: options.schemaIssuePaths ?? null,
    },
    options.message
  )
}
