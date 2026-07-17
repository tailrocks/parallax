export type InvestigationErrorCode =
  | "transport"
  | "invalid-response"
  | "load"
  | "save"
  | "delete"

export class InvestigationError extends Error {
  readonly code: InvestigationErrorCode
  constructor(code: InvestigationErrorCode, message?: string) {
    super(message ?? `investigation ${code}`)
    this.name = "InvestigationError"
    this.code = code
  }
}

export function investigationErrorMessage(error: unknown): string {
  if (error instanceof InvestigationError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
