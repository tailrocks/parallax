import type { TestCaseDetailQuery } from "@/features/tests/api/test-case-detail.generated"
import type { TestsListQuery } from "@/features/tests/api/tests-list.generated"
import type {
  TestCaseDetail,
  TestCaseDetailData,
  TestVariantDetail,
} from "@/features/tests/model/test-detail"
import type {
  TestExplorerRow,
  TestFlaky,
  TestParameter,
  TestResultRef,
  TestsData,
} from "@/features/tests/model/test-summary"

function mapParameters(
  parameters: ReadonlyArray<{
    readonly name: string
    readonly value: string
    readonly excluded: boolean
  }>
): TestParameter[] {
  return parameters.map((parameter) => ({
    name: parameter.name,
    value: parameter.value,
    excluded: parameter.excluded,
  }))
}

function mapFlaky(
  flaky:
    | {
        readonly state: TestFlaky["state"]
        readonly sameCommitDivergence: boolean
        readonly intraInvocationMix: boolean
        readonly transitionCount: number
        readonly consecutivePasses: number
        readonly updatedAtNanos: string
      }
    | null
    | undefined
): TestFlaky | null {
  if (!flaky) return null
  return {
    state: flaky.state,
    sameCommitDivergence: flaky.sameCommitDivergence,
    intraInvocationMix: flaky.intraInvocationMix,
    transitionCount: flaky.transitionCount,
    consecutivePasses: flaky.consecutivePasses,
    updatedAtNanos: flaky.updatedAtNanos,
  }
}

function mapResult(result: {
  readonly invocationId: string
  readonly attempt: number
  readonly status: TestResultRef["status"]
  readonly traceId: string
  readonly spanId: string
  readonly startedAtNanos: string
  readonly endedAtNanos: string
  readonly service: string
  readonly serviceVersion: string | null
  readonly vcsHeadRevision: string | null
  readonly failureFingerprint: string | null
  readonly configuration: ReadonlyArray<{ readonly key: string; readonly value: string }>
}): TestResultRef {
  return {
    invocationId: result.invocationId,
    attempt: result.attempt,
    status: result.status,
    traceId: result.traceId,
    spanId: result.spanId,
    startedAtNanos: result.startedAtNanos,
    endedAtNanos: result.endedAtNanos,
    service: result.service,
    serviceVersion: result.serviceVersion,
    vcsHeadRevision: result.vcsHeadRevision,
    failureFingerprint: result.failureFingerprint,
    configuration: result.configuration.map((dimension) => ({
      key: dimension.key,
      value: dimension.value,
    })),
  }
}

function mapRow(row: TestsListQuery["testCases"]["items"][number]): TestExplorerRow {
  return {
    caseKey: row.caseKey,
    variantKey: row.variantKey,
    name: row.name,
    suitePath: [...row.suitePath],
    codeReference: row.codeReference,
    explicitId: row.explicitId,
    firstSeenNanos: row.firstSeenNanos,
    lastSeenNanos: row.lastSeenNanos,
    parameters: mapParameters(row.parameters),
    invocationId: row.invocationId,
    rollup: row.rollup,
    attemptCount: row.attemptCount,
    lastResult: mapResult(row.lastResult),
    flaky: mapFlaky(row.flaky),
  }
}

export function mapTestsList(data: TestsListQuery): TestsData {
  return {
    items: data.testCases.items.map(mapRow),
    hasMore: data.testCases.hasMore,
    services: [...data.services],
  }
}

function mapVariant(
  variant: NonNullable<TestCaseDetailQuery["testCase"]>["variants"][number]
): TestVariantDetail {
  return {
    variantKey: variant.variantKey,
    parameters: mapParameters(variant.parameters),
    firstSeenNanos: variant.firstSeenNanos,
    lastSeenNanos: variant.lastSeenNanos,
    history: variant.history.map(mapResult),
    flaky: mapFlaky(variant.flaky),
  }
}

export function mapTestCaseDetail(data: TestCaseDetailQuery): TestCaseDetailData {
  const raw = data.testCase
  if (!raw) return { case: null }
  const detail: TestCaseDetail = {
    caseKey: raw.caseKey,
    name: raw.name,
    identitySource: raw.identitySource,
    suitePath: [...raw.suitePath],
    codeReference: raw.codeReference,
    explicitId: raw.explicitId,
    firstSeenNanos: raw.firstSeenNanos,
    lastSeenNanos: raw.lastSeenNanos,
    variants: raw.variants.map(mapVariant),
  }
  return { case: detail }
}
