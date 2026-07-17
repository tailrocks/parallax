// Compatibility surface for GraphQL transport + product wire DTOs.
// Transport lives at `@/platform/graphql/transport` (Plan 100).
// Product DTOs migrate with feature plans 134-142 / 149-150; Plan 152 hardens decode.
// Removal of this reexport path is owned by the last consumer feature plan + Plan 151.

export {
  clearGraphqlCache,
  gqlString,
  graphql,
  graphqlCached,
} from "@/platform/graphql/transport"

export interface Issue {
  fingerprint: string
  title: string
  errorType: string
  culprit: string | null
  service: string
  status: string
  firstSeenNanos: string
  lastSeenNanos: string
  eventCount: number
  lastTraceId: string | null
}

export interface ServiceCatalogRow {
  name: string
  serviceVersion: string | null
  serviceNamespace: string | null
  deploymentEnvironment: string | null
  telemetrySdkLanguage: string | null
  telemetrySdkName: string | null
  telemetrySdkVersion: string | null
  lastSeenNanos: string
  instanceCount: string
}

export interface ErrorEvent {
  tsNanos: string
  message: string
  stacktrace: string | null
  source: string
  traceId: string
  spanId: string
  attributes: string
}

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

/** Full GraphQL selection for a log row (`LogDoc` / `logs` connection). */
export const LOG_FIELDS =
  "tsNanos eventName observedTsNanos service severityNum severityText body traceId spanId invocationId scopeName attributes resource"

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

// Domain owners (Plan 149); re-exported for legacy route consumers until
// feature migrations switch to domain/feature facades.
export type { StoryBeat } from "@/domain/story/story-beat"

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

export type {
  MetricPoint,
  RuntimeMetric,
} from "@/domain/runtime-metrics/runtime-metric"

export interface EvidenceGap {
  kind: string
  subject: string
  detail: string
}

export interface LogRecord {
  tsNanos: string
  eventName: string
  observedTsNanos: string
  service: string
  severityText: string
  body: string
  traceId: string
}

export interface Invocation {
  invocationId: string
  registration: "cli" | "external"
  command: string | null
  appMode: string | null
  outcome: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
  endedAtNanos: string | null
  errorCount: number
  traceCount: number
  sessionCount: number
}

export interface ObservedInvocation {
  invocationId: string
  service: string
  lastCommand: string | null
  appMode: string | null
  firstNanos: string
  lastNanos: string
  spanCount: number
  logCount: number
}

export interface Session {
  sessionId: string
  previousSessionId: string | null
  startNanos: string
  endNanos: string | null
}

export interface ScreenVisit {
  screenId: string
  visitId: string
  sessionId: string | null
  navigationSequence: number | null
  transitionReason: string | null
  enteredNanos: string
  exitedNanos: string | null
}

export interface UiAction {
  name: string
  screenId: string | null
  widgetName: string | null
  sessionId: string | null
  traceId: string
  startNanos: string
  durationMs: number
  outcome: string | null
  hasError: boolean
}

export interface BackgroundCycle {
  name: string
  count: number
  errorCount: number
  p50Ms: number | null
  p95Ms: number | null
  lastNanos: string
  lastTraceId: string
}

export interface JobAttempt {
  startNanos: string
  durationMs: number
  outcome: string | null
  hasError: boolean
  traceId: string
}

export interface Job {
  jobId: string
  jobType: string | null
  producedNanos: string | null
  attempts: JobAttempt[]
  lastTraceId: string
}

export interface Conversation {
  conversationId: string
  agentName: string | null
  providerName: string | null
  firstNanos: string
  lastNanos: string
  spanCount: number
  inputTokens: number | null
  outputTokens: number | null
}

export interface Investigation {
  id: string
  name: string
  state: string
  createdAtNanos: string
  updatedAtNanos: string
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

export interface ServiceMapNode {
  name: string
  kind: "cli" | "browser" | "service"
  lastSeenNanos: string
  spanCount: string
  errorCount: string
  p95Ms: number | null
}

export interface ServiceMapEdge {
  source: string
  target: string
  callCount: string
  errorCount: string
  p50Ms: number
  p95Ms: number
}

export interface ServiceMap {
  nodes: ServiceMapNode[]
  edges: ServiceMapEdge[]
}
