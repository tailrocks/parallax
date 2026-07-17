export type TestsErrorCode = "transport" | "invalid-response" | "load" | "not-found"

export class TestsError extends Error {
  readonly code: TestsErrorCode
  constructor(code: TestsErrorCode, message?: string) {
    super(message ?? `tests ${code}`)
    this.name = "TestsError"
    this.code = code
  }
}

export function testsErrorMessage(error: unknown): string {
  if (error instanceof TestsError) return error.message
  if (error instanceof Error) return error.message
  return String(error)
}
