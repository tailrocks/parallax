export type DashboardErrorCode =
  | "transport"
  | "invalid-response"
  | "load"
  | "save"
  | "delete"

export class DashboardError extends Error {
  readonly code: DashboardErrorCode
  constructor(code: DashboardErrorCode, message?: string) {
    super(message ?? `dashboard ${code}`)
    this.name = "DashboardError"
    this.code = code
  }
}

export function dashboardErrorMessage(error: unknown): string {
  if (error instanceof DashboardError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
