export type ServicesErrorCode =
  | "transport"
  | "invalid-response"
  | "load"
  | "not-found"

export class ServicesError extends Error {
  readonly code: ServicesErrorCode
  constructor(code: ServicesErrorCode, message?: string) {
    super(message ?? `services ${code}`)
    this.name = "ServicesError"
    this.code = code
  }
}

export function servicesErrorMessage(error: unknown): string {
  if (error instanceof ServicesError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
