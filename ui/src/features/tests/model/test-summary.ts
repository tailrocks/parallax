export type TestRollup = "PASSED" | "FLAKY_PASS" | "FAILED" | "BROKEN" | "SKIPPED" | "UNKNOWN"

export type TestResultStatus = "PASSED" | "FAILED" | "BROKEN" | "SKIPPED" | "UNKNOWN"

export type TestFlakyState = "HEALTHY" | "FLAKY" | "FIXED" | "BROKEN"

export interface TestParameter {
  readonly name: string
  readonly value: string
  readonly excluded: boolean
}

export interface TestDimension {
  readonly key: string
  readonly value: string
}

export interface TestFlaky {
  readonly state: TestFlakyState
  readonly sameCommitDivergence: boolean
  readonly intraInvocationMix: boolean
  readonly transitionCount: number
  readonly consecutivePasses: number
  readonly updatedAtNanos: string
}

export interface TestResultRef {
  readonly invocationId: string
  readonly attempt: number
  readonly status: TestResultStatus
  readonly traceId: string
  readonly spanId: string
  readonly startedAtNanos: string
  readonly endedAtNanos: string
  readonly service: string
  readonly serviceVersion: string | null
  readonly vcsHeadRevision: string | null
  readonly failureFingerprint: string | null
  readonly configuration: readonly TestDimension[]
}

export interface TestExplorerRow {
  readonly caseKey: string
  readonly variantKey: string
  readonly name: string
  readonly suitePath: readonly string[]
  readonly codeReference: string | null
  readonly explicitId: string | null
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly parameters: readonly TestParameter[]
  readonly invocationId: string
  readonly rollup: TestRollup
  readonly attemptCount: number
  readonly lastResult: TestResultRef
  readonly flaky: TestFlaky | null
}

export interface TestsData {
  readonly items: readonly TestExplorerRow[]
  readonly hasMore: boolean
  readonly services: readonly string[]
}

export function suiteLabel(path: readonly string[]): string {
  return path.length === 0 ? "—" : path.join(" / ")
}

export function rollupLabel(rollup: TestRollup): string {
  switch (rollup) {
    case "PASSED":
      return "passed"
    case "FLAKY_PASS":
      return "flaky pass"
    case "FAILED":
      return "failed"
    case "BROKEN":
      return "broken"
    case "SKIPPED":
      return "skipped"
    case "UNKNOWN":
      return "unknown"
  }
}

export function flakyLabel(state: TestFlakyState): string {
  switch (state) {
    case "HEALTHY":
      return "healthy"
    case "FLAKY":
      return "flaky"
    case "FIXED":
      return "fixed"
    case "BROKEN":
      return "broken"
  }
}
