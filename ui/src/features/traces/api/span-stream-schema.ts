// Plan 147 — unknown-first runtime schema for /v1/traces/stream frames.

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import type { LiveSpan } from "@/features/traces/model/wire"

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function readString(row: Record<string, unknown>, key: string): string | null {
  const value = row[key]
  return typeof value === "string" ? value : null
}

function readNullableString(row: Record<string, unknown>, key: string): string | null {
  const value = row[key]
  if (value === null || value === undefined) return null
  return typeof value === "string" ? value : null
}

/** Decode one live span stream row. Identity requires spanId. */
export function decodeSpanStreamRow(input: unknown): LiveSpan | null {
  if (!isRecord(input)) return null
  const spanId = readString(input, "spanId")
  if (spanId === null || spanId.length === 0) return null

  const tsNanos = readString(input, "tsNanos") ?? "0"
  const service = readString(input, "service") ?? ""
  const traceId = readString(input, "traceId") ?? ""
  const parentSpanId = readNullableString(input, "parentSpanId")
  const name = readString(input, "name") ?? ""
  const kind = readString(input, "kind") ?? ""
  const statusCode = readString(input, "statusCode") ?? ""
  const durationNs = readString(input, "durationNs") ?? "0"
  const invocationId = readNullableString(input, "invocationId")
  const sessionId = readNullableString(input, "sessionId")

  return {
    tsNanos,
    service,
    traceId,
    spanId,
    parentSpanId,
    name,
    kind,
    statusCode,
    durationNs,
    invocationId,
    sessionId,
  }
}

/**
 * RuntimeDecoder for an SSE span frame: JSON array of live spans.
 * Invalid elements are dropped; a non-array frame fails closed.
 */
export const spanStreamBatchDecoder: RuntimeDecoder<LiveSpan[]> = {
  safeParse(input) {
    if (!Array.isArray(input)) {
      return { success: false, error: "span-stream-frame-not-array" }
    }
    const items: LiveSpan[] = []
    for (const row of input) {
      const decoded = decodeSpanStreamRow(row)
      if (decoded) items.push(decoded)
    }
    return { success: true, data: items }
  },
}
