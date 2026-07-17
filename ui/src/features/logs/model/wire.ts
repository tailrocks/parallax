/** Log wire types and GraphQL field selections (plan 151 residual claim). */

/** Full GraphQL selection for a log row (`LogDoc` / `logs` connection). */
export const LOG_FIELDS =
  "tsNanos eventName observedTsNanos service severityNum severityText body traceId spanId invocationId scopeName attributes resource"

export interface LogRecord {
  tsNanos: string
  eventName: string
  observedTsNanos: string
  service: string
  severityText: string
  body: string
  traceId: string
}
