export type DiagnosticLevel = "error" | "warn"

export type Diagnostic = Readonly<{
  level: DiagnosticLevel
  message: string
}>

const expected: Diagnostic[] = []
const observed: Diagnostic[] = []

export function expectDiagnostic(level: DiagnosticLevel, message: string) {
  if (message.length === 0) {
    throw new Error("expected diagnostic message cannot be empty")
  }
  expected.push({ level, message })
}

export function recordDiagnostic(level: DiagnosticLevel, values: unknown[]) {
  observed.push({ level, message: formatDiagnostic(values) })
}

export function assertDiagnostics() {
  const mismatch = diagnosticMismatch(expected, observed)
  resetDiagnostics()
  if (mismatch !== null) {
    throw new Error(mismatch)
  }
}

export function resetDiagnostics() {
  expected.length = 0
  observed.length = 0
}

export function diagnosticMismatch(
  expectedDiagnostics: readonly Diagnostic[],
  observedDiagnostics: readonly Diagnostic[]
): string | null {
  if (
    expectedDiagnostics.length === observedDiagnostics.length &&
    expectedDiagnostics.every(
      (value, index) =>
        value.level === observedDiagnostics[index]?.level &&
        value.message === observedDiagnostics[index]?.message
    )
  ) {
    return null
  }
  return `runtime diagnostics differ\nexpected: ${JSON.stringify(expectedDiagnostics)}\nobserved: ${JSON.stringify(observedDiagnostics)}`
}

function formatDiagnostic(values: unknown[]) {
  return values
    .map((value) => {
      if (value instanceof Error) {
        return `${value.name}: ${value.message}`
      }
      if (typeof value === "string") {
        return value
      }
      try {
        return JSON.stringify(value)
      } catch {
        return String(value)
      }
    })
    .join(" ")
}
