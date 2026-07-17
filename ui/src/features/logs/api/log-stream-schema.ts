// Plan 147 — unknown-first runtime schema for /v1/logs/stream frames.

import type { RuntimeDecoder } from "@/platform/external-values/runtime-decoder"
import type { LogDoc } from "@/features/logs/components/logs-table"

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

function readNumber(row: Record<string, unknown>, key: string): number | null {
  const value = row[key]
  return typeof value === "number" && Number.isFinite(value) ? value : null
}

/** Decode one log stream row; reject incomplete wire objects. */
export function decodeLogStreamRow(input: unknown): LogDoc | null {
  if (!isRecord(input)) return null
  const tsNanos = readString(input, "tsNanos")
  const body = readString(input, "body")
  if (tsNanos === null || body === null) return null

  const eventName = readString(input, "eventName") ?? ""
  const observedTsNanos = readString(input, "observedTsNanos") ?? tsNanos
  const service = readString(input, "service") ?? ""
  const severityNum = readNumber(input, "severityNum") ?? 0
  const severityText = readString(input, "severityText") ?? ""
  const traceId = readString(input, "traceId") ?? ""
  const spanId = readString(input, "spanId") ?? ""
  const invocationId = readNullableString(input, "invocationId")
  const scopeName = readString(input, "scopeName") ?? ""
  const attributes = readString(input, "attributes") ?? "{}"
  const resource = readString(input, "resource") ?? "{}"

  return {
    tsNanos,
    eventName,
    observedTsNanos,
    service,
    severityNum,
    severityText,
    body,
    traceId,
    spanId,
    invocationId,
    scopeName,
    attributes,
    resource,
  }
}

/**
 * RuntimeDecoder for an SSE log frame: JSON array of log rows.
 * Invalid elements are dropped; a non-array frame fails closed.
 */
export const logStreamBatchDecoder: RuntimeDecoder<LogDoc[]> = {
  safeParse(input) {
    if (!Array.isArray(input)) {
      return { success: false, error: "log-stream-frame-not-array" }
    }
    const items: LogDoc[] = []
    for (const row of input) {
      const decoded = decodeLogStreamRow(row)
      if (decoded) items.push(decoded)
    }
    return { success: true, data: items }
  },
}
