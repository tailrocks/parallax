// The UI's only data path: GraphQL against the Parallax API (same-origin —
// the vite dev proxy and the embedded prod build both serve /graphql).

// Loaders are isomorphic (run on server AND client): relative URLs only work
// in the browser, so SSR/loader calls target the API directly.
const BASE = typeof window === "undefined" ? "http://127.0.0.1:4000" : ""

export async function graphql<T>(
  query: string,
  init?: { signal?: AbortSignal }
): Promise<T> {
  const requestInit: RequestInit = {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query }),
  }
  if (init?.signal) requestInit.signal = init.signal
  const response = await fetch(`${BASE}/graphql`, {
    ...requestInit,
  })
  if (!response.ok) {
    throw new Error(`parallax api unreachable (${response.status})`)
  }
  const body = (await response.json()) as { data?: T; errors?: unknown[] }
  if (body.errors?.length) {
    throw new Error(`graphql error: ${JSON.stringify(body.errors)}`)
  }
  if (!body.data) {
    throw new Error("graphql response missing data")
  }
  return body.data
}

/** Escape a value for inclusion inside a GraphQL double-quoted literal. */
export function gqlString(value: string): string {
  // GraphQL string literals cannot contain raw newlines (multi-line SQL).
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "")
    .replace(/\t/g, "\\t")
}

export function relativeTime(nanosString: string): string {
  const nanos = Number(nanosString)
  if (!Number.isFinite(nanos) || nanos <= 0) return "-"
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - nanos / 1e9))
  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
  return `${Math.floor(seconds / 86400)}d ago`
}

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

export interface StoryBeat {
  tsNanos: string
  lane: string
  kind: string
  title: string
  traceId: string
  spanId: string | null
  severity: string | null
  durationNs: string | null
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

export interface EvidenceGap {
  kind: string
  subject: string
  detail: string
}

export interface LogRecord {
  tsNanos: string
  service: string
  severityText: string
  body: string
  traceId: string
}

export interface Run {
  runId: string
  command: string | null
  status: string
  exitCode: number | null
  startedAtNanos: string
}

export interface ObservedRun {
  runId: string
  service: string
  firstNanos: string
  lastNanos: string
  spanCount: number
  logCount: number
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
