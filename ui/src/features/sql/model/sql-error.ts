export type SqlErrorCode =
  | "transport"
  | "invalid-response"
  | "schema-discovery"
  | "query-execution"
  | "history-persistence"
  | "snippet-list"
  | "snippet-save"
  | "snippet-delete"

export class SqlError extends Error {
  readonly code: SqlErrorCode
  constructor(code: SqlErrorCode, message?: string) {
    super(message ?? `sql ${code}`)
    this.name = "SqlError"
    this.code = code
  }
}

export function sqlErrorMessage(error: unknown): string {
  if (error instanceof SqlError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
