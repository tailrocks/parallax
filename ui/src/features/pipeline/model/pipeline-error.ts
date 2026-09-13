export type PipelineErrorCode = "transport" | "invalid-response" | "load"

export class PipelineError extends Error {
  readonly code: PipelineErrorCode
  constructor(code: PipelineErrorCode, message?: string) {
    super(message ?? `pipeline ${code}`)
    this.name = "PipelineError"
    this.code = code
  }
}

export function pipelineErrorMessage(error: unknown): string {
  if (error instanceof PipelineError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
