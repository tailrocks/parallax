/** Trace/span wire types and GraphQL field selections (plan 151 residual claim). */

export interface Span {
  tsNanos: string
  service: string
  traceId: string
  spanId: string
  parentSpanId: string | null
  name: string
  kind: string
  statusCode: string
  durationNs: string
}

/** Full GraphQL selection for a stored span (`Span`). */
export const SPAN_FIELDS =
  "tsNanos service traceId spanId parentSpanId name kind statusCode durationNs"

/**
 * Live SSE span from `/v1/traces/stream`.
 *
 * Matches `Span` and adds `invocationId` (the live serializer always emits it;
 * `parentSpanId` is present on the wire — unlike the former inline `SpanDoc`).
 */
export type LiveSpan = Span & {
  invocationId: string | null
  sessionId: string | null
}

export interface SpanLink {
  traceId: string
  spanId: string
  attributes: string
}

export interface CriticalHop {
  spanId: string
  selfTimeNs: string
  gatedByChild: string | null
  clockSuspect: boolean
}

export interface CriticalPath {
  hops: CriticalHop[]
  totalGatedNs: string
  unattached: string[]
}

export interface TraceDiffSpan {
  spanId: string
  service: string
  name: string
  kind: string
  statusCode: string
  durationNs: string
  depth: number
  matchKey: string
}

export interface TraceDiffChange {
  before: TraceDiffSpan
  after: TraceDiffSpan
  durationDeltaNs: string
  durationDeltaPct: number
  statusChanged: boolean
}

export interface TraceDiff {
  added: TraceDiffSpan[]
  removed: TraceDiffSpan[]
  changed: TraceDiffChange[]
}

export interface AttributeCompareRow {
  key: string
  value: string
  selectedCount: string
  selectedTotal: string
  baselineCount: string
  baselineTotal: string
  score: number
}

export interface FieldKey {
  key: string
  namespace: string
  source: "SPAN" | "RESOURCE"
  rowCount: string
  nonNullCount: string
  coverage: number
  isIdentifier: boolean
}

export interface FieldValueCount {
  value: string
  count: string
}

export interface FieldStats {
  key: string
  namespace: string
  source: "SPAN" | "RESOURCE"
  rowCount: string
  nonNullCount: string
  distinctCount: string
  coverage: number
  capped: boolean
  isIdentifier: boolean
  topValues: FieldValueCount[]
}

export interface EvidenceGap {
  kind: string
  subject: string
  detail: string
}

export interface TraceSummary {
  traceId: string
  rootName: string
  service: string
  startNanos: string
  durationNs: string
  spanCount: number
  hasError: boolean
}
