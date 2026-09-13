/** One log row, with every field the doc viewer needs. Shared by the Logs page
 * and the run detail page so both render logs identically. */
export interface LogDoc {
  _key?: string
  tsNanos: string
  eventName: string
  observedTsNanos: string
  service: string
  severityNum: number
  severityText: string
  body: string
  traceId: string
  spanId: string
  invocationId: string | null
  scopeName: string
  attributes: string
  resource: string
}

import { formatDateTime, stripAnsi } from "@/shared/format"

export function severityVariant(num: number): "rose" | "amber" | "secondary" | "outline" {
  if (num >= 17) return "rose"
  if (num >= 13) return "amber"
  if (num >= 9) return "secondary"
  return "outline"
}

export function severityLabel(log: LogDoc) {
  return log.severityText || (log.severityNum >= 17 ? "ERROR" : "LOG")
}

function observedSkewField(log: LogDoc): [string, string] | null {
  try {
    const observed = BigInt(log.observedTsNanos || "0")
    if (observed === 0n) return null
    const emitted = BigInt(log.tsNanos || "0")
    const delta = observed > emitted ? observed - emitted : emitted - observed
    if (delta <= 1_000_000_000n) return null
    return ["@observed", formatDateTime(log.observedTsNanos)]
  } catch {
    return null
  }
}

/** Flatten one log into ordered field/value rows for the doc viewer. */
export function docFields(log: LogDoc): Array<[string, string]> {
  const rows: Array<[string, string]> = [
    ["@timestamp", formatDateTime(log.tsNanos)],
    ["severity", `${severityLabel(log)} (${log.severityNum})`],
    ["service.name", log.service],
    ["body", stripAnsi(log.body)],
  ]
  if (log.eventName) rows.splice(2, 0, ["event.name", log.eventName])
  const observed = observedSkewField(log)
  if (observed) rows.splice(log.eventName ? 3 : 2, 0, observed)
  if (log.traceId) rows.push(["trace_id", log.traceId])
  if (log.spanId) rows.push(["span_id", log.spanId])
  if (log.invocationId) rows.push(["run_id", log.invocationId])
  if (log.scopeName) rows.push(["scope.name", log.scopeName])
  for (const [prefix, json] of [
    ["", log.attributes],
    ["resource.", log.resource],
  ] as const) {
    try {
      const parsed: unknown = JSON.parse(json)
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        for (const [key, value] of Object.entries(parsed)) {
          rows.push([`${prefix}${key}`, typeof value === "string" ? value : JSON.stringify(value)])
        }
      }
    } catch {
      // non-object payloads stay out of the table
    }
  }
  return rows
}

export function rawDocument(log: LogDoc) {
  return JSON.stringify(log, null, 2)
}
