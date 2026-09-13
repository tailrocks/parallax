export type RumErrorCode = "load" | "invalid-response"

export class RumError extends Error {
  readonly code: RumErrorCode

  constructor(code: RumErrorCode, message: string) {
    super(message)
    this.name = "RumError"
    this.code = code
  }
}
