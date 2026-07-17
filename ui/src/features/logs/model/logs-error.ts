export type LogsErrorCode =
  | "transport"
  | "invalid-response"
  | "load"
  | "mutation"

export class LogsError extends Error {
  readonly code: LogsErrorCode
  constructor(code: LogsErrorCode, message?: string) {
    super(message ?? `logs ${code}`)
    this.name = "LogsError"
    this.code = code
  }
}
