import type { TestFlaky, TestParameter, TestResultRef } from "@/features/tests/model/test-summary"

export type TestIdentitySource = "EXPLICIT" | "CODE_REFERENCE" | "NAME_PATH"

export interface TestVariantDetail {
  readonly variantKey: string
  readonly parameters: readonly TestParameter[]
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly history: readonly TestResultRef[]
  readonly flaky: TestFlaky | null
}

export interface TestCaseDetail {
  readonly caseKey: string
  readonly name: string
  readonly identitySource: TestIdentitySource
  readonly suitePath: readonly string[]
  readonly codeReference: string | null
  readonly explicitId: string | null
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly variants: readonly TestVariantDetail[]
}

export interface TestCaseDetailData {
  readonly case: TestCaseDetail | null
}

export function identitySourceLabel(source: TestIdentitySource): string {
  switch (source) {
    case "EXPLICIT":
      return "explicit id"
    case "CODE_REFERENCE":
      return "code reference"
    case "NAME_PATH":
      return "name path"
  }
}
