import type { VitalId, VitalRating } from "@/features/rum/model/rum-vitals"

export interface RumPoint {
  readonly tsNanos: string
  readonly value: number
}

export interface RumVitalRow {
  readonly name: string
  readonly vital: VitalId
  readonly kind: string
  readonly unit: string | null
  readonly services: readonly string[]
  readonly lastDatapointNanos: string
  readonly pointCount: string
  readonly p75: number | null
  readonly rating: VitalRating | null
}

export interface RumTraceRow {
  readonly traceId: string
  readonly rootName: string
  readonly service: string
  readonly startNanos: string
  readonly durationNs: string
  readonly spanCount: number
  readonly hasError: boolean
}

export interface RumIssueRow {
  readonly fingerprint: string
  readonly title: string
  readonly errorType: string
  readonly culprit: string | null
  readonly service: string
  readonly status: string
  readonly lastSeenNanos: string
  readonly eventCount: number
  readonly lastTraceId: string | null
}

export interface RumExemplar {
  readonly tsNanos: string
  readonly service: string
  readonly name: string
  readonly value: number
  readonly traceId: string
  readonly spanId: string
  readonly attributes: string
}

export interface RumLogRow {
  readonly tsNanos: string
  readonly severityText: string
  readonly body: string
  readonly service: string
  readonly spanId: string
}

export interface RumData {
  readonly services: readonly string[]
  readonly service: string | null
  readonly vitals: readonly RumVitalRow[]
  readonly issues: readonly RumIssueRow[]
  readonly issueTotal: number
  readonly errorTraces: readonly RumTraceRow[]
  readonly errorTraceTotal: string
  readonly journeys: readonly RumTraceRow[]
  readonly journeyTotal: string
}

export interface RumVitalData {
  readonly row: RumVitalRow
  readonly trend: readonly RumPoint[]
  readonly exemplars: readonly RumExemplar[]
}

export interface RumTraceData {
  readonly traceId: string
  readonly linked: readonly RumTraceRow[]
  readonly logs: readonly RumLogRow[]
}

export interface RumSessionRow {
  readonly sessionId: string
  readonly service: string
  readonly startNanos: string
  readonly endNanos: string
  readonly spanCount: number
  readonly traceCount: number
  readonly viewCount: number
  readonly vitalCount: number
  readonly errorCount: number
  readonly hasError: boolean
}

export interface RumSessionViewRow {
  readonly tsNanos: string
  readonly screen: string
  readonly path: string | null
  readonly traceId: string
  readonly spanId: string
}

export interface RumSessionVitalRow {
  readonly tsNanos: string
  readonly name: string
  readonly value: number
  readonly rating: string | null
  readonly traceId: string
  readonly spanId: string
}

export interface RumSessionErrorRow {
  readonly tsNanos: string
  readonly name: string
  readonly errorType: string | null
  readonly message: string
  readonly traceId: string
  readonly spanId: string
}

export interface RumSessionDetailData {
  readonly session: RumSessionRow
  readonly views: readonly RumSessionViewRow[]
  readonly vitals: readonly RumSessionVitalRow[]
  readonly errors: readonly RumSessionErrorRow[]
}
