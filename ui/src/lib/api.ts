// The UI's only data path: GraphQL against the Parallax API (same-origin —
// the vite dev proxy and the embedded prod build both serve /graphql).

// Loaders are isomorphic (run on server AND client): relative URLs only work
// in the browser, so SSR/loader calls target the API directly.
const BASE = typeof window === "undefined" ? "http://127.0.0.1:4000" : ""

const CACHE_TTL_MS = 15_000
const CACHE_MAX = 100

/** In-flight dedup of identical query strings (client-side only). */
const inflight = new Map<string, Promise<unknown>>()
/** Short-lived result cache keyed by query string (client-side only). */
const cache = new Map<string, { at: number; data: unknown }>()

/** Test-only: clear the client GraphQL cache and inflight map. */
export function clearGraphqlCache(): void {
  cache.clear()
  inflight.clear()
}

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

/**
 * Client-side query cache + in-flight dedup for route loaders and preload.
 *
 * Key = full query string (variables are embedded today). SSR always bypasses
 * the cache so a shared module never leaks data across requests. Pollers and
 * explicit Refresh paths should keep using raw `graphql`.
 */
export async function graphqlCached<T>(
  query: string,
  init?: { signal?: AbortSignal }
): Promise<T> {
  // Cache is client-only — Bun/SSR may share the module across requests.
  if (typeof window === "undefined") {
    return graphql<T>(query, init)
  }

  const hit = cache.get(query)
  if (hit && Date.now() - hit.at < CACHE_TTL_MS) {
    return hit.data as T
  }

  const pending = inflight.get(query)
  if (pending) return pending as Promise<T>

  const p = graphql<T>(query, init).then(
    (data) => {
      // Insertion-order LRU-ish: drop oldest when over cap.
      if (cache.size >= CACHE_MAX && !cache.has(query)) {
        const oldest = cache.keys().next().value
        if (oldest !== undefined) cache.delete(oldest)
      }
      cache.set(query, { at: Date.now(), data })
      inflight.delete(query)
      return data
    },
    (error: unknown) => {
      inflight.delete(query)
      throw error
    }
  )
  inflight.set(query, p)
  return p
}

/** Escape a value for inclusion inside a GraphQL double-quoted literal. */
export function gqlString(value: string): string {
  // GraphQL string literals cannot contain raw newlines or other control chars.
  // oxlint-disable-next-line no-control-regex -- GraphQL forbids raw C0 controls
  const rawControlCharacters = /[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "")
    .replace(/\t/g, "\\t")
    .replace(
      rawControlCharacters,
      (c) => "\\u" + c.charCodeAt(0).toString(16).padStart(4, "0")
    )
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

export interface MetricPoint {
  tsNanos: string
  value: number
}

export interface RuntimeMetric {
  family: string
  metric: string
  unit: string | null
  points: MetricPoint[]
}

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
