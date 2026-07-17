/** Issue wire DTOs used outside issues feature loaders (plan 151 residual). */

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

export interface ErrorEvent {
  tsNanos: string
  message: string
  stacktrace: string | null
  source: string
  traceId: string
  spanId: string
  attributes: string
}
