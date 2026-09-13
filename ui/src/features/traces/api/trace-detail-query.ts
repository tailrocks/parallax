import { gqlString } from "@/platform/graphql/transport"

export type DominantDbQueryRow = {
  readonly normalized: string
  readonly example: string
  readonly count: number
  readonly totalNs: string
  readonly maxNs: string
  readonly exampleSpanId: string
  readonly service: string
}

/** GraphQL document for trace detail. Must select `dominantDbQueries` from
 * the shipped `Trace` type — the page must not re-rank spans in the client. */
export function traceDetailQuery(traceId: string): string {
  const safeId = gqlString(traceId)
  return `{ trace(traceId: "${safeId}") {
         spans { tsNanos service traceId name kind statusCode statusMessage durationNs
                 spanId parentSpanId invocationId links typedLinks { traceId spanId attributes }
                 events attributes resource }
         dominantDbQueries { normalized example count totalNs maxNs exampleSpanId service }
       }
       linkedTraces(traceId: "${safeId}") {
         traceId rootName service startNanos durationNs spanCount hasError
       }
       story(traceId: "${safeId}") {
         tsNanos lane kind title traceId spanId severity durationNs
       }
       evidenceGaps(traceId: "${safeId}") {
         kind subject detail
       }
       rpcTraceEvents: traceEvents(traceId: "${safeId}", namePrefix: "rpc", limit: 500) {
         truncated skippedSpans
         events { spanId spanName service name timeUnixNano attributes }
       }
       logsByTrace(traceId: "${safeId}") { tsNanos service severityText body spanId } }`
}
