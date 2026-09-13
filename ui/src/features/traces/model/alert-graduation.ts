import {
  encodeAlertGraduationSearch,
  type AlertGraduationSearch,
} from "@/features/alerts/model/alert-rule-form"
import type { TracesSearch } from "@/features/traces/components/traces-query"

/** Graduate the current traces query: errors-only → error_rate, else p95_latency. */
export function encodeTracesAlertGraduation(
  search: Pick<TracesSearch, "service" | "errors">
): AlertGraduationSearch {
  return encodeAlertGraduationSearch({
    signalType: search.errors ? "error_rate" : "p95_latency",
    service: search.service,
  })
}
