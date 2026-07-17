export type EcosystemErrorCode = "transport" | "invalid-response" | "load"

export class EcosystemError extends Error {
  readonly code: EcosystemErrorCode
  constructor(code: EcosystemErrorCode, message?: string) {
    super(message ?? `ecosystem ${code}`)
    this.name = "EcosystemError"
    this.code = code
  }
}

export function ecosystemErrorMessage(error: unknown): string {
  if (error instanceof EcosystemError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
