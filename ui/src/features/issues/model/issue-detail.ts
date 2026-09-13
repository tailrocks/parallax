import { formatDelta, type Delta } from "@/shared/format"
import type { ResolvedRange } from "@/domain/time-range/range"

import type { TrendPoint } from "@/features/issues/model/issue-summary"

export interface MappedFrame {
  readonly raw: string
  readonly file: string
  readonly line: number
  readonly column: number
  readonly resolved: boolean
  readonly source: string | null
  readonly sourceLine: number | null
  readonly sourceColumn: number | null
  readonly name: string | null
}

export interface IssueEvent {
  readonly tsNanos: string
  readonly service: string
  readonly serviceVersion: string | null
  readonly message: string
  readonly stacktrace: string | null
  readonly source: string
  readonly traceId: string
  readonly spanId: string
  readonly attributes: string
  readonly mappedFrames: readonly MappedFrame[]
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

export interface IssueCorrelationLog {
  readonly tsNanos: string
  readonly severityText: string
  readonly body: string
}

export interface IssueCorrelation {
  readonly invocationId: string | null
  readonly resource: Record<string, unknown>
  readonly releaseVersion: string | null
  readonly logs: readonly IssueCorrelationLog[]
}

export type IssueCorrelationResult =
  | { readonly status: "ready"; readonly correlation: IssueCorrelation }
  | { readonly status: "trace-unavailable" }

export interface IssueDetailData {
  readonly issue: IssueDetail | null
  readonly issueTrend: readonly TrendPoint[]
}

export type ParsedIssueAttributes =
  | { readonly kind: "empty" }
  | { readonly kind: "entries"; readonly entries: readonly IssueAttributeEntry[] }
  | { readonly kind: "raw"; readonly raw: string }

export interface IssueAttributeEntry {
  readonly key: string
  readonly value: string
}

/** Event attributes arrive as opaque JSON. Invalid JSON remains visible as
 * raw text instead of being silently discarded. */
export function parseIssueAttributes(raw: string): ParsedIssueAttributes {
  if (!raw.trim()) return { kind: "empty" }
  try {
    const parsed: unknown = JSON.parse(raw)
    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
      return { kind: "raw", raw }
    }
    const entries = Object.entries(parsed).map(([key, value]) => ({
      key,
      value: typeof value === "string" ? value : JSON.stringify(value),
    }))
    return entries.length === 0 ? { kind: "empty" } : { kind: "entries", entries }
  } catch {
    return { kind: "raw", raw }
  }
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
