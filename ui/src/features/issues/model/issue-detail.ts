import { formatDelta, type Delta } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"

import type { TrendPoint } from "@/features/issues/model/issue-summary"

export interface IssueEvent {
  readonly tsNanos: string
  readonly service: string
  readonly message: string
  readonly stacktrace: string | null
  readonly source: string
  readonly traceId: string
  readonly spanId: string
  readonly attributes: string
}

export interface IssueDetail {
  readonly fingerprint: string
  readonly title: string
  readonly errorType: string
  readonly culprit: string | null
  readonly service: string
  readonly status: string
  readonly firstSeenNanos: string
  readonly lastSeenNanos: string
  readonly eventCount: number
  readonly lastTraceId: string | null
  readonly tags: string
  readonly groupingExplanation: {
    readonly algorithmVersion: string
    readonly errorType: string
    readonly messageTemplate: string
    readonly anchorFrame: string
    readonly operation: string | null
    readonly inputsPresent: readonly string[]
  } | null
  readonly events: readonly IssueEvent[]
}

export interface BreadcrumbLog {
  readonly tsNanos: string
  readonly severityText: string
  readonly body: string
}

export interface IssueDetailData {
  readonly issue: IssueDetail | null
  readonly issueTrend: readonly TrendPoint[]
  readonly resource: Record<string, unknown>
  readonly breadcrumbs: readonly BreadcrumbLog[]
  readonly traceRunId: string | null
  readonly releaseVersion: string | null
}

export function rangeHours(range: ResolvedRange): number {
  const ns = BigInt(range.toNanos) - BigInt(range.fromNanos)
  return Math.max(1, Math.ceil(Number(ns / 3_600_000_000_000n)))
}

export function shortRunId(invocationId: string): string {
  return invocationId.length > 8 ? `${invocationId.slice(0, 8)}...` : invocationId
}

export function issueDelta(trend: readonly TrendPoint[]): Delta | null {
  if (trend.length < 2) return null
  const midpoint = Math.floor(trend.length / 2)
  const previous = trend.slice(0, midpoint).reduce((sum, point) => sum + point.count, 0)
  const current = trend.slice(midpoint).reduce((sum, point) => sum + point.count, 0)
  return formatDelta(current, previous)
}
