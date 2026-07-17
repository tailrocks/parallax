export type IssuesErrorCode = "transport" | "invalid-response" | "load" | "mutation" | "not-found"

export class IssuesError extends Error {
  readonly code: IssuesErrorCode
  constructor(code: IssuesErrorCode, message?: string) {
    super(message ?? `issues ${code}`)
    this.name = "IssuesError"
    this.code = code
  }
}

export function issuesErrorMessage(error: unknown): string {
  if (error instanceof IssuesError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
